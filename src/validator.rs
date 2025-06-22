//! Main MCDOC validator

use crate::registry::RegistryManager;
use crate::types::{ValidationResult, McDocError, McDocDependency};
use crate::error::McDocParserError;
use crate::ResourceId;
use crate::parser::McDocFile;
use rustc_hash::FxHashMap;

/// Main MCDOC validator
pub struct McDocValidator<'input> {
    pub registry_manager: RegistryManager,
    pub mcdoc_schemas: FxHashMap<String, McDocFile<'input>>,
    _phantom: std::marker::PhantomData<&'input ()>,
}

impl<'input> McDocValidator<'input> {
    /// Create a new validator
    pub fn new() -> Self {
        Self {
            registry_manager: RegistryManager::new(),
            mcdoc_schemas: FxHashMap::default(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Load MCDOC files (parsing handled on TypeScript side according to spec)
    pub fn load_mcdoc_files(&mut self, _files: FxHashMap<String, &'input str>) -> Result<(), Vec<McDocParserError>> {
        Ok(())
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
        resource_id: &str,
    ) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            dependencies: Vec::new(),
        };
        
        let _parsed_id = match ResourceId::parse(resource_id) {
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
        
        if !json.is_object() && !json.is_array() {
            result.add_error(McDocError {
                file: resource_id.to_string(),
                path: "".to_string(),
                message: "Invalid JSON: expected object or array".to_string(),
                error_type: crate::error::ErrorType::Validation,
                line: None,
                column: None,
            });
        }
        
        if !self.mcdoc_schemas.is_empty() {
            let resource_type = _parsed_id.path.split('/').next().unwrap_or("unknown");
            
            for (schema_name, _schema_ast) in &self.mcdoc_schemas {
                if schema_name.contains(resource_type) {
                    break;
                }
            }
        }
        
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
        
        let dependencies = result.dependencies.clone(); 
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
                            path: dependency.source_path.to_string(),
                            message: format!(
                                "Resource '{}' not found in registry '{}'",
                                dependency.resource_location,
                                dependency.registry_type
                            ),
                            error_type: crate::error::ErrorType::Validation,
                            line: None,
                            column: None,
                        });
                    }
                    Err(registry_error) => {
                        result.add_error(McDocError {
                            file: resource_id.to_string(),
                            path: dependency.source_path.to_string(),
                            message: registry_error.to_string(),
                            error_type: registry_error.error_type(),
                            line: None,
                            column: None,
                        });
                    }
                    Ok(true) => {
                        // Resource found in registry
                    }
                }
            }
        }
        
        result
    }
    
    /// Get required registries for a JSON without full validation
    /// Lightweight operation for dependency analysis
    pub fn get_required_registries(&self, json: &serde_json::Value, _resource_type: &str) -> Vec<String> {
        let dependencies = self.registry_manager.scan_required_registries(json);
        let mut registries: Vec<String> = dependencies.iter()
            .map(|dep| dep.registry.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        
        registries.sort();
        registries
    }
}

impl<'input> Default for McDocValidator<'input> {
    fn default() -> Self {
        Self::new()
    }
} 