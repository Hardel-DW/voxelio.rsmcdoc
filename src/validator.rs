//! Validateur MCDOC principal
//! 
//! Orchestre toute la pipeline: Lexer → Parser → Resolver → Validation

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::resolver::ImportResolver;
use crate::registry::RegistryManager;
use crate::types::{ValidationResult, DatapackResult, McDocError, McDocDependency, MinecraftVersion};
use crate::error::McDocParserError;
use crate::{ResourceId, RegistryDependency};
use rustc_hash::FxHashMap;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::time::Instant;

/// Validateur principal MCDOC
pub struct McDocValidator<'input> {
    resolver: ImportResolver<'input>,
    pub registry_manager: RegistryManager,
    current_version: Option<MinecraftVersion>,
}

impl<'input> McDocValidator<'input> {
    /// Créer un nouveau validateur
    pub fn new() -> Self {
        Self {
            resolver: ImportResolver::new(),
            registry_manager: RegistryManager::new(),
            current_version: None,
        }
    }
    
    /// Initialiser avec des fichiers MCDOC
    pub fn init(mcdoc_files: FxHashMap<String, &'input str>) -> Result<Self, Vec<McDocParserError>> {
        let mut validator = Self::new();
        validator.load_mcdoc_files(mcdoc_files)?;
        Ok(validator)
    }
    
    /// Charger des fichiers MCDOC
    pub fn load_mcdoc_files(&mut self, files: FxHashMap<String, &'input str>) -> Result<(), Vec<McDocParserError>> {
        let mut errors = Vec::new();
        
        // 1. Parser tous les fichiers MCDOC
        for (file_path, content) in files {
            match self.parse_mcdoc_file(content) {
                Ok(parsed_file) => {
                    self.resolver.add_module(file_path, parsed_file);
                }
                Err(parse_errors) => {
                    errors.extend(parse_errors);
                }
            }
        }
        
        if !errors.is_empty() {
            return Err(errors);
        }
        
        // 2. Résoudre les imports
        if let Err(resolve_error) = self.resolver.resolve_all() {
            errors.push(resolve_error);
            return Err(errors);
        }
        
        Ok(())
    }
    
    /// Parser un fichier MCDOC
    fn parse_mcdoc_file(&self, content: &'input str) -> Result<crate::parser::McDocFile<'input>, Vec<McDocParserError>> {
        // Lexing
        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize().map_err(|e| vec![e])?;
        
        // Parsing
        let mut parser = Parser::new(tokens);
        parser.parse()
    }
    
    /// Définir la version Minecraft courante
    pub fn set_minecraft_version(&mut self, version: &str) -> Result<(), McDocParserError> {
        self.current_version = MinecraftVersion::parse(version)
            .ok_or_else(|| McDocParserError::InvalidResourceId(format!("Invalid version: {}", version)))?
            .into();
        Ok(())
    }
    
    /// Charger un registre depuis JSON
    pub fn load_registry(&mut self, name: String, version: String, json: &serde_json::Value) -> Result<(), McDocParserError> {
        self.registry_manager.load_registry_from_json(name, version, json)
    }
    
    /// Charger plusieurs registres (en parallèle)
    pub fn load_registries(&mut self, registries: Vec<(String, String, serde_json::Value)>) -> Result<(), Vec<McDocParserError>> {
        #[cfg(feature = "rayon")]
        let errors: Vec<_> = registries
            .into_par_iter()
            .map(|(name, version, json)| {
                // NOTE: Cannot directly mutate across threads, would need Arc<Mutex<_>>
                // For now, sequential loading
                (name, version, json)
            })
            .collect();
        
        #[cfg(not(feature = "rayon"))]
        let errors: Vec<_> = registries;
        
        // Sequential loading for now (parallel would require Arc<Mutex<RegistryManager>>)
        let mut load_errors = Vec::new();
        for (name, version, json) in errors {
            if let Err(e) = self.load_registry(name, version, &json) {
                load_errors.push(e);
            }
        }
        
        if load_errors.is_empty() {
            Ok(())
        } else {
            Err(load_errors)
        }
    }
    
    /// Obtenir les dépendances registres nécessaires pour un JSON (sans validation complète)
    pub fn get_required_registries(&self, json: &serde_json::Value, _resource_id: &str) -> Vec<RegistryDependency> {
        self.registry_manager.scan_required_registries(json)
    }
    
    /// Valider un JSON contre les schemas MCDOC
    pub fn validate_json(
        &self,
        json: &serde_json::Value,
        resource_id: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            dependencies: Vec::new(),
        };
        
        // 1. Parser le resource ID
        let parsed_id = match ResourceId::parse(resource_id) {
            Ok(id) => id,
            Err(e) => {
                result.add_error(McDocError {
                    file: resource_id.to_string(),
                    path: "".to_string(),
                    message: e.to_string(),
                    error_type: e.error_type(),
                    line: None,
                    column: None,
                });
                return result;
            }
        };
        
        // 2. Extraire les dépendances registres (heuristiques)
        let dependencies = self.registry_manager.scan_required_registries(json);
        for dep in dependencies {
            result.add_dependency(McDocDependency {
                resource_location: dep.identifier,
                registry_type: dep.registry,
                source_path: "auto-detected".to_string(),
                source_file: Some(resource_id.to_string()),
                is_tag: dep.is_tag,
            });
        }
        
        // 3. Validation basique JSON structure
        if let Err(validation_errors) = self.validate_json_structure(json, &parsed_id) {
            for error in validation_errors {
                result.add_error(McDocError {
                    file: resource_id.to_string(),
                    path: "".to_string(),
                    message: error.to_string(),
                    error_type: error.error_type(),
                    line: error.position().map(|(l, _)| l),
                    column: error.position().map(|(_, c)| c),
                });
            }
        }
        
        // 4. Validation des registres (si chargés)
        let dependencies = result.dependencies.clone(); // Clone to avoid borrow checker issues
        for dependency in &dependencies {
            if self.registry_manager.has_registry(&dependency.registry_type) {
                match self.registry_manager.validate_resource_location(
                    &dependency.registry_type,
                    &dependency.resource_location,
                    dependency.is_tag,
                ) {
                    Ok(false) => {
                        result.add_error(McDocError {
                            file: resource_id.to_string(),
                            path: dependency.source_path.clone(),
                            message: format!(
                                "Resource '{}' not found in registry '{}'",
                                dependency.resource_location,
                                dependency.registry_type
                            ),
                            error_type: crate::error::ErrorType::ValidationError,
                            line: None,
                            column: None,
                        });
                    }
                    Err(e) => {
                        result.add_error(McDocError::from(e));
                    }
                    Ok(true) => {} // Valid
                }
            }
        }
        
        result
    }
    
    /// Validation structure JSON basique
    fn validate_json_structure(&self, json: &serde_json::Value, resource_id: &ResourceId) -> Result<(), Vec<McDocParserError>> {
        // Extract namespace and resource type from resource_id
        let (namespace, resource_type) = self.parse_resource_identifier(&resource_id.to_string())?;
        
        // Find the appropriate MCDOC dispatch for this resource type
        let dispatch_target = self.resolver.find_dispatch_target(&namespace, &resource_type)
            .ok_or_else(|| vec![McDocParserError::ValidationError {
                message: format!("No MCDOC dispatch found for resource type: {}", resource_type),
                line: 0,
                column: 0,
            }])?;
        
        // Validate JSON against the target structure
        self.validate_json_against_structure(json, &dispatch_target.target_type, &[], &resource_id.to_string())
    }

    /// Parse resource identifier (e.g., "minecraft:recipe/diamond_sword" -> ("minecraft", "recipe"))
    fn parse_resource_identifier(&self, resource_id: &str) -> Result<(String, String), Vec<McDocParserError>> {
        let parts: Vec<&str> = resource_id.split(':').collect();
        if parts.len() != 2 {
            return Err(vec![McDocParserError::ValidationError {
                message: format!("Invalid resource identifier format: {}", resource_id),
                line: 0,
                column: 0,
            }]);
        }
        
        let namespace = parts[0].to_string();
        let resource_path = parts[1];
        
        // Extract resource type from path (e.g., "recipe/diamond_sword" -> "recipe")
        let resource_type = resource_path.split('/').next().unwrap_or(resource_path);
        
        Ok((namespace, resource_type.to_string()))
    }

    /// Validate JSON value against MCDOC type expression
    fn validate_json_against_structure(
        &self,
        json: &serde_json::Value,
        type_expr: &crate::parser::TypeExpression,
        path: &[String],
        context: &str,
    ) -> Result<(), Vec<McDocParserError>> {
        use crate::parser::TypeExpression;
        
        match type_expr {
            TypeExpression::Simple(type_name) => {
                self.validate_simple_type(json, type_name, path, context)
            }
            
            TypeExpression::Array { element_type, constraints } => {
                self.validate_array_type(json, element_type, constraints.as_ref(), path, context)
            }
            
            TypeExpression::Union(variants) => {
                // Try each variant until one succeeds
                let mut all_errors = Vec::new();
                
                for variant in variants {
                    match self.validate_json_against_structure(json, variant, path, context) {
                        Ok(()) => return Ok(()), // Success with this variant
                        Err(mut errors) => all_errors.append(&mut errors),
                    }
                }
                
                // All variants failed
                Err(vec![McDocParserError::ValidationError {
                    message: format!("Value does not match any variant in union type at path: {}", path.join(".")),
                    line: 0,
                    column: 0,
                }])
            }
            
            TypeExpression::Struct(fields) => {
                self.validate_struct_type(json, fields, path, context)
            }
            
            TypeExpression::Generic { name, type_args: _ } => {
                // For now, treat generic types as their base type
                self.validate_simple_type(json, name, path, context)
            }
            
            TypeExpression::Reference(import_path) => {
                // Resolve the reference and validate against the target type
                match self.resolver.resolve_type_reference(import_path) {
                    Some(resolved_type) => {
                        self.validate_json_against_structure(json, resolved_type, path, context)
                    }
                    None => Err(vec![McDocParserError::ValidationError {
                        message: format!("Unresolved type reference: {:?}", import_path),
                        line: 0,
                        column: 0,
                    }])
                }
            }
            
            TypeExpression::Spread(spread) => {
                // Handle spread expressions by resolving them dynamically
                self.validate_spread_expression(json, spread, path, context)
            }
        }
    }

    /// Validate simple types (string, int, float, boolean)
    fn validate_simple_type(
        &self,
        json: &serde_json::Value,
        type_name: &str,
        path: &[String],
        context: &str,
    ) -> Result<(), Vec<McDocParserError>> {
        let is_valid = match type_name {
            "string" => json.is_string(),
            "int" | "integer" => json.is_i64(),
            "float" | "number" => json.is_f64() || json.is_i64(),
            "boolean" | "bool" => json.is_boolean(),
            "null" => json.is_null(),
            _ => {
                // Custom type - look up in resolver for struct/enum types
                if let Some(resolved_type) = self.resolver.resolve_type_reference(&crate::parser::ImportPath::Absolute(vec![type_name])) {
                    // Recursively validate against the resolved type
                    return self.validate_json_against_structure(json, resolved_type, path, context);
                } else {
                    // If type not found in resolver, check if it's a valid identifier pattern
                    // This allows for external types not yet loaded
                    if type_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                        true // Assume valid for external types
                    } else {
                        false // Invalid type name format
                    }
                }
            }
        };
        
        if is_valid {
            Ok(())
        } else {
            Err(vec![McDocParserError::ValidationError {
                message: format!("Expected type '{}' but found {} at path: {}", 
                    type_name, 
                    self.json_type_name(json),
                    path.join(".")
                ),
                line: 0,
                column: 0,
            }])
        }
    }

    /// Validate array type with constraints
    fn validate_array_type(
        &self,
        json: &serde_json::Value,
        element_type: &crate::parser::TypeExpression,
        constraints: Option<&crate::parser::ArrayConstraints>,
        path: &[String],
        context: &str,
    ) -> Result<(), Vec<McDocParserError>> {
        if !json.is_array() {
            return Err(vec![McDocParserError::ValidationError {
                message: format!("Expected array but found {} at path: {}", 
                    self.json_type_name(json),
                    path.join(".")
                ),
                line: 0,
                column: 0,
            }]);
        }
        
        let array = json.as_array().unwrap();
        let array_len = array.len();
        
        // Validate array size constraints
        if let Some(constraints) = constraints {
            if let Some(min) = constraints.min {
                if array_len < min as usize {
                    return Err(vec![McDocParserError::ValidationError {
                        message: format!("Array too short: expected at least {} elements, found {} at path: {}", 
                            min, array_len, path.join(".")
                        ),
                        line: 0,
                        column: 0,
                    }]);
                }
            }
            
            if let Some(max) = constraints.max {
                if array_len > max as usize {
                    return Err(vec![McDocParserError::ValidationError {
                        message: format!("Array too long: expected at most {} elements, found {} at path: {}", 
                            max, array_len, path.join(".")
                        ),
                        line: 0,
                        column: 0,
                    }]);
                }
            }
        }
        
        // Validate each element
        let mut all_errors = Vec::new();
        
        for (index, element) in array.iter().enumerate() {
            let mut element_path = path.to_vec();
            element_path.push(format!("[{}]", index));
            
            if let Err(mut errors) = self.validate_json_against_structure(element, element_type, &element_path, context) {
                all_errors.append(&mut errors);
            }
        }
        
        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Validate struct type
    fn validate_struct_type(
        &self,
        json: &serde_json::Value,
        fields: &[crate::parser::FieldDeclaration],
        path: &[String],
        context: &str,
    ) -> Result<(), Vec<McDocParserError>> {
        if !json.is_object() {
            return Err(vec![McDocParserError::ValidationError {
                message: format!("Expected object but found {} at path: {}", 
                    self.json_type_name(json),
                    path.join(".")
                ),
                line: 0,
                column: 0,
            }]);
        }
        
        let obj = json.as_object().unwrap();
        let mut all_errors = Vec::new();
        
        // Validate each field
        for field in fields {
            let field_value = obj.get(field.name);
            let mut field_path = path.to_vec();
            field_path.push(field.name.to_string());
            
            match (field_value, field.optional) {
                (None, false) => {
                    // Required field is missing
                    all_errors.push(McDocParserError::ValidationError {
                        message: format!("Required field '{}' is missing at path: {}", 
                            field.name, path.join(".")
                        ),
                        line: 0,
                        column: 0,
                    });
                }
                (Some(value), _) => {
                    // Field exists, validate it
                    if let Err(mut errors) = self.validate_json_against_structure(value, &field.field_type, &field_path, context) {
                        all_errors.append(&mut errors);
                    }
                }
                (None, true) => {
                    // Optional field is missing - OK
                }
            }
        }
        
        if all_errors.is_empty() {
            Ok(())
        } else {
            Err(all_errors)
        }
    }

    /// Validate spread expression (dynamic inheritance)
    fn validate_spread_expression(
        &self,
        json: &serde_json::Value,
        spread: &crate::parser::SpreadExpression,
        path: &[String],
        context: &str,
    ) -> Result<(), Vec<McDocParserError>> {
        // For spread expressions like ...minecraft:recipe_serializer[[type]],
        // we need to dynamically determine the target type based on the JSON content
        
        if let Some(dynamic_ref) = &spread.dynamic_key {
            // Extract the dynamic key value from the JSON
            let dynamic_value = self.extract_dynamic_value(json, dynamic_ref, context)?;
            
            // Resolve the spread target using the dynamic value
            let target_type = self.resolver.resolve_spread_target(spread.base_path, &dynamic_value)
                .ok_or_else(|| vec![McDocParserError::ValidationError {
                    message: format!("Cannot resolve spread target for '{}' with dynamic value '{}'", 
                        spread.base_path, dynamic_value
                    ),
                    line: 0,
                    column: 0,
                }])?;
            
            // Validate against the resolved target
            self.validate_json_against_structure(json, target_type, path, context)
        } else {
            // Static spread - resolve directly
            let target_type = self.resolver.resolve_spread_target(spread.base_path, "")
                .ok_or_else(|| vec![McDocParserError::ValidationError {
                    message: format!("Cannot resolve static spread target for '{}'", spread.base_path),
                    line: 0,
                    column: 0,
                }])?;
            
            self.validate_json_against_structure(json, target_type, path, context)
        }
    }

    /// Extract dynamic value from JSON for spread resolution
    fn extract_dynamic_value(
        &self,
        json: &serde_json::Value,
        dynamic_ref: &crate::parser::DynamicReference,
        _context: &str,
    ) -> Result<String, Vec<McDocParserError>> {
        use crate::parser::DynamicReferenceType;
        
        match &dynamic_ref.reference {
            DynamicReferenceType::Field(field_name) => {
                if let Some(obj) = json.as_object() {
                    if let Some(field_value) = obj.get(*field_name) {
                        if let Some(string_value) = field_value.as_str() {
                            Ok(string_value.to_string())
                        } else {
                            Err(vec![McDocParserError::ValidationError {
                                message: format!("Dynamic reference field '{}' must be a string", field_name),
                                line: 0,
                                column: 0,
                            }])
                        }
                    } else {
                        Err(vec![McDocParserError::ValidationError {
                            message: format!("Dynamic reference field '{}' not found", field_name),
                            line: 0,
                            column: 0,
                        }])
                    }
                } else {
                    Err(vec![McDocParserError::ValidationError {
                        message: "Cannot extract dynamic field from non-object".to_string(),
                        line: 0,
                        column: 0,
                    }])
                }
            }
            DynamicReferenceType::SpecialKey(key) => {
                // Special keys like %key might refer to object keys themselves
                // For now, return the key as-is
                Ok(key.to_string())
            }
        }
    }

    /// Get human-readable name for JSON value type
    fn json_type_name(&self, json: &serde_json::Value) -> &'static str {
        match json {
            serde_json::Value::Null => "null",
            serde_json::Value::Bool(_) => "boolean",
            serde_json::Value::Number(_) => "number",
            serde_json::Value::String(_) => "string",
            serde_json::Value::Array(_) => "array",
            serde_json::Value::Object(_) => "object",
        }
    }
    
    /// Analyser un datapack complet (parallèle)
    pub fn analyze_datapack(&self, files: FxHashMap<String, Vec<u8>>) -> DatapackResult {
        let start_time = Instant::now();
        let mut result = DatapackResult::new();
        
        // Filtrer les fichiers JSON
        let json_files: Vec<_> = files
            .into_iter()
            .filter(|(path, _)| path.ends_with(".json"))
            .collect();
        
        // Traitement en parallèle avec Rayon (si disponible)
        #[cfg(feature = "rayon")]
        let file_results: Vec<_> = json_files
            .into_par_iter()
            .map(|(file_path, content)| {
                let json_result = serde_json::from_slice::<serde_json::Value>(&content);
                
                match json_result {
                    Ok(json) => {
                        let resource_id = self.extract_resource_id_from_path(&file_path);
                        let validation_result = self.validate_json(&json, &resource_id);
                        (file_path, Ok::<ValidationResult, Vec<McDocParserError>>(validation_result))
                    }
                    Err(e) => {
                        let error_result = ValidationResult::failure(vec![McDocError {
                            file: file_path.clone(),
                            path: "".to_string(),
                            message: format!("JSON parse error: {}", e),
                            error_type: crate::error::ErrorType::JsonError,
                            line: None,
                            column: None,
                        }]);
                        (file_path, Ok(error_result))
                    }
                }
            })
            .collect();
        
        #[cfg(not(feature = "rayon"))]
        let file_results: Vec<_> = json_files
            .into_iter()
            .map(|(file_path, content)| {
                let json_result = serde_json::from_slice::<serde_json::Value>(&content);
                
                match json_result {
                    Ok(json) => {
                        let resource_id = self.extract_resource_id_from_path(&file_path);
                        let validation_result = self.validate_json(&json, &resource_id);
                        (file_path, Ok::<ValidationResult, Vec<McDocParserError>>(validation_result))
                    }
                    Err(e) => {
                        let error_result = ValidationResult::failure(vec![McDocError {
                            file: file_path.clone(),
                            path: "".to_string(),
                            message: format!("JSON parse error: {}", e),
                            error_type: crate::error::ErrorType::JsonError,
                            line: None,
                            column: None,
                        }]);
                        (file_path, Ok(error_result))
                    }
                }
            })
            .collect();
        
        // Agrégation des résultats
        for (file_path, file_result) in file_results {
            match file_result {
                Ok(validation_result) => {
                    result.add_file_result(file_path, validation_result);
                }
                Err(errors) => {
                    let mcdoc_errors: Vec<McDocError> = errors.into_iter().map(McDocError::from).collect();
                    let error_result = ValidationResult::failure(mcdoc_errors);
                    result.add_file_result(file_path, error_result);
                }
            }
        }
        
        let elapsed = start_time.elapsed();
        result.set_analysis_time(elapsed.as_millis() as u32);
        
        result
    }
    
    /// Extraire un resource ID depuis un chemin de fichier
    pub fn extract_resource_id_from_path(&self, file_path: &str) -> String {
        // Exemple: "data/minecraft/recipes/diamond_sword.json" -> "minecraft:diamond_sword"
        let path_parts: Vec<&str> = file_path.split('/').collect();
        
        if path_parts.len() >= 4 && path_parts[0] == "data" {
            let namespace = path_parts[1];
            let file_name = path_parts.last()
                .and_then(|name| name.strip_suffix(".json"))
                .unwrap_or("unknown");
            
            format!("{}:{}", namespace, file_name)
        } else {
            file_path.to_string()
        }
    }
    
    /// Obtenir les statistiques des registres chargés
    pub fn get_registry_stats(&self) -> FxHashMap<String, (usize, usize)> {
        let mut stats = FxHashMap::default();
        
        for registry_name in self.registry_manager.get_loaded_registries() {
            if let Some(stat) = self.registry_manager.get_registry_stats(registry_name) {
                stats.insert(registry_name.to_string(), stat);
            }
        }
        
        stats
    }
    
    /// Vérifier si tous les registres nécessaires sont chargés
    pub fn check_missing_registries(&self, dependencies: &[RegistryDependency]) -> Vec<String> {
        dependencies
            .iter()
            .map(|dep| &dep.registry)
            .filter(|registry| !self.registry_manager.has_registry(registry))
            .cloned()
            .collect()
    }
    
    /// Charger les registres de test depuis un fichier JSON utilisateur (pour development/tests)
    pub fn load_test_registries(&mut self, data_file_path: &str) -> Result<(), McDocParserError> {
        self.registry_manager.load_test_data(data_file_path)
    }
    
    /// Charger les registres depuis des données JSON utilisateur
    pub fn load_registries_from_data(&mut self, data_json: &serde_json::Value, version: &str) -> Result<(), McDocParserError> {
        self.registry_manager.load_minecraft_data(data_json, version)
    }
}

impl<'input> Default for McDocValidator<'input> {
    fn default() -> Self {
        Self::new()
    }
} 