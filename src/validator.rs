//! Main MCDOC validator

use crate::registry::RegistryManager;
use crate::types::{ValidationResult, McDocError, McDocDependency};
use crate::error::{McDocParserError, ErrorType};
use crate::ResourceId;
use crate::parser::{McDocFile, Declaration, TypeExpression};
use rustc_hash::FxHashMap;

/// Context for a single validation run.
struct ValidationContext<'a> {
    errors: Vec<McDocError>,
    dependencies: Vec<McDocDependency>,
    version: Option<&'a str>,
    resource_type: &'a str,
}

impl<'a> ValidationContext<'a> {
    fn new(version: Option<&'a str>, resource_type: &'a str) -> Self {
        Self {
            errors: Vec::new(),
            dependencies: Vec::new(),
            version,
            resource_type,
        }
    }

    fn add_error(&mut self, path: &str, message: String) {
        self.errors.push(McDocError {
            file: self.resource_type.to_string(),
            path: path.to_string(),
            message,
            error_type: ErrorType::Validation,
            line: None,
            column: None,
        });
    }
}

/// Main MCDOC validator
pub struct DatapackValidator<'input> {
    pub registry_manager: RegistryManager,
    pub mcdoc_schemas: FxHashMap<String, McDocFile<'input>>,
    _phantom: std::marker::PhantomData<&'input ()>,
}

impl<'input> DatapackValidator<'input> {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            registry_manager: RegistryManager::new(),
            mcdoc_schemas: FxHashMap::default(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Load a previously parsed MCDOC schema
    pub fn load_parsed_mcdoc(&mut self, filename: String, ast: McDocFile<'input>) -> Result<(), McDocParserError> {
        self.mcdoc_schemas.insert(filename, ast);
        Ok(())
    }
    
    /// Load a registry from JSON
    pub fn load_registry(&mut self, name: String, version: String, json: &serde_json::Value) -> Result<(), McDocParserError> {
        self.registry_manager.load_registry_from_json(name, version, json)
    }
    
    /// Validate JSON against MCDOC schemas
    pub fn validate_json(
        &self,
        json: &serde_json::Value,
        resource_type: &str,
        version: Option<&str>,
    ) -> ValidationResult {
        let mut context = ValidationContext::new(version, resource_type);

        if let Some(type_expr) = self.find_type_for_resource(resource_type) {
            Self::validate_node(json, type_expr, "", &mut context, None);
        } else {
            context.add_error("", format!("No MCDOC schema found for resource type '{}'", resource_type));
        }

        // 4. Valider les dépendances contre le registre
        let dependencies = context.dependencies.clone(); 
        for dependency in &dependencies {
            if self.registry_manager.has_registry(&dependency.registry_type) {
                match self.registry_manager.validate_resource_location(
                    &dependency.registry_type,
                    &dependency.resource_location,
                    dependency.is_tag,
                ) {
                    Ok(false) => {
                        context.add_error(&dependency.source_path, format!(
                            "Resource '{}' not found in registry '{}'",
                            dependency.resource_location,
                            dependency.registry_type
                        ));
                    }
                    Err(e) => {
                        context.add_error(&dependency.source_path, e.to_string());
                    }
                    Ok(true) => {} // Valid
                }
            } else if dependency.registry_type != "unknown" {
                context.add_error(&dependency.source_path, format!("Unknown registry '{}'", dependency.registry_type));
            }
        }
        
        ValidationResult {
            is_valid: context.errors.is_empty(),
            errors: context.errors,
            dependencies: context.dependencies,
        }
    }

    /// Recursive validation function
    fn validate_node(
        json_node: &serde_json::Value,
        mcdoc_node: &TypeExpression<'input>,
        path: &str,
        context: &mut ValidationContext,
        annotations: Option<&Vec<crate::parser::Annotation<'input>>>,
    ) {
        if let Some(annotations) = annotations {
            if let Some(id_annotation) = annotations.iter().find(|a| a.name == "id") {
                if let Some(s) = json_node.as_str() {
                    let registry_type = match &id_annotation.data {
                        crate::parser::AnnotationData::Simple(registry) => registry.to_string(),
                        crate::parser::AnnotationData::Complex(map) => {
                            map.get("registry").unwrap_or(&"unknown").to_string()
                        }
                        _ => "unknown".to_string()
                    };
                    context.dependencies.push(McDocDependency {
                        resource_location: s.to_string(),
                        registry_type,
                        source_path: path.to_string(),
                        source_file: Some(context.resource_type.to_string()),
                        is_tag: s.starts_with('#'),
                    });
                }
            }
        }

        match mcdoc_node {
            TypeExpression::Simple(type_name) => {
                let type_str = match json_node {
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::Object(_) => "object",
                    serde_json::Value::Null => "null",
                };

                match *type_name {
                    "string" => if !json_node.is_string() {
                        context.add_error(path, format!("Expected string, found {}", type_str));
                    },
                    "int" | "float" => if !json_node.is_number() {
                        context.add_error(path, format!("Expected number, found {}", type_str));
                    },
                    "boolean" => if !json_node.is_boolean() {
                        context.add_error(path, format!("Expected boolean, found {}", type_str));
                    },
                    _ => { /* It could be a reference to another type, needs resolver */ }
                }
            }
            TypeExpression::Struct(members) => {
                if let Some(obj) = json_node.as_object() {
                    for member in members {
                        match member {
                            crate::parser::StructMember::Field(field) => {
                                let field_name = field.name;
                                let new_path = if path.is_empty() { field_name.to_string() } else { format!("{}.{}", path, field_name) };
                                
                                if let Some(value) = obj.get(field_name) {
                                    Self::validate_node(value, &field.field_type, &new_path, context, Some(&field.annotations));
                                } else if !field.optional {
                                    context.add_error(&new_path, format!("Missing required field '{}'", field_name));
                                }
                            }
                            crate::parser::StructMember::DynamicField(dynamic_field) => {
                                // For dynamic fields like [#[id="mob_effect"] string]: MobEffectPredicate
                                // We need to validate each key-value pair in the object
                                for (key, value) in obj.iter() {
                                    let key_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                                    
                                    // Validate the key against key_type
                                    // For now, we assume the key is a string and skip key validation
                                    // TODO: Implement proper key validation
                                    
                                    // Validate the value against value_type
                                    Self::validate_node(value, &dynamic_field.value_type, &key_path, context, Some(&dynamic_field.annotations));
                                }
                            }
                            crate::parser::StructMember::Spread(_spread) => {
                                // TODO: Handle spread expressions - for now skip them
                                // In a real implementation, we would need to resolve the spread target
                                // and validate against its fields
                            }
                        }
                    }
                } else {
                    context.add_error(path, "Expected object".to_string());
                }
            }
            TypeExpression::Array { element_type, constraints } => {
                if let Some(arr) = json_node.as_array() {
                    if let Some(constraints) = constraints {
                        if let Some(min) = constraints.min {
                            if arr.len() < min as usize {
                                context.add_error(path, format!("Expected at least {} elements, found {}", min, arr.len()));
                            }
                        }
                        if let Some(max) = constraints.max {
                            if arr.len() > max as usize {
                                context.add_error(path, format!("Expected at most {} elements, found {}", max, arr.len()));
                            }
                        }
                    }

                    for (i, elem) in arr.iter().enumerate() {
                        let new_path = format!("{}[{}]", path, i);
                        Self::validate_node(elem, element_type, &new_path, context, None);
                    }
                } else {
                    context.add_error(path, "Expected array".to_string());
                }
            }
            TypeExpression::Union(types) => {
                let mut local_errors = Vec::new();
                for mcdoc_type in types {
                    let mut temp_context = ValidationContext::new(context.version, context.resource_type);
                    Self::validate_node(json_node, mcdoc_type, path, &mut temp_context, None);
                    if temp_context.errors.is_empty() {
                        // It matched one of the types in the union, so it's valid.
                        // We also need to merge the dependencies found.
                        context.dependencies.extend(temp_context.dependencies);
                        return;
                    }
                    local_errors.extend(temp_context.errors);
                }
                
                context.add_error(path, "JSON does not match any of the expected types".to_string());
            }
            TypeExpression::Literal(literal_value) => {
                // Validate that the JSON value exactly matches the literal constraint
                match literal_value {
                    crate::parser::LiteralValue::String(expected) => {
                        if let Some(actual) = json_node.as_str() {
                            if actual != *expected {
                                context.add_error(path, format!("Expected '{}', found '{}'", expected, actual));
                            }
                        } else {
                            context.add_error(path, format!("Expected string '{}', found non-string", expected));
                        }
                    }
                    crate::parser::LiteralValue::Number(expected) => {
                        if let Some(actual) = json_node.as_f64() {
                            if (actual - expected).abs() > f64::EPSILON {
                                context.add_error(path, format!("Expected {}, found {}", expected, actual));
                            }
                        } else {
                            context.add_error(path, format!("Expected number {}, found non-number", expected));
                        }
                    }
                    crate::parser::LiteralValue::Boolean(expected) => {
                        if let Some(actual) = json_node.as_bool() {
                            if actual != *expected {
                                context.add_error(path, format!("Expected {}, found {}", expected, actual));
                            }
                        } else {
                            context.add_error(path, format!("Expected boolean {}, found non-boolean", expected));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// Finds the corresponding TypeExpression for a given resource type string.
    fn find_type_for_resource(&self, resource_type: &str) -> Option<&TypeExpression<'input>> {
        let parsed_id = ResourceId::parse(resource_type).ok()?;
        
        for schema in self.mcdoc_schemas.values() {
            for decl in &schema.declarations {
                if let Declaration::Dispatch(dispatch) = decl {
                    if dispatch.source.key == Some(parsed_id.path.as_str()) {
                         return Some(&dispatch.target_type);
                    }
                }
            }
        }
        None
    }
}

impl<'input> Default for DatapackValidator<'input> {
    fn default() -> Self {
        Self::new()
    }
} 