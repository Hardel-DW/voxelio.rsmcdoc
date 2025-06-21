//! Validateur MCDOC principal

use crate::registry::RegistryManager;
use crate::types::{ValidationResult, McDocError, McDocDependency};
use crate::error::McDocParserError;
use crate::ResourceId;
use rustc_hash::FxHashMap;

/// Validateur principal MCDOC - SIMPLIFIÉ
pub struct McDocValidator<'input> {
    pub registry_manager: RegistryManager,
    _phantom: std::marker::PhantomData<&'input ()>,
}

impl<'input> McDocValidator<'input> {
    /// Créer un nouveau validateur
    pub fn new() -> Self {
        Self {
            registry_manager: RegistryManager::new(),
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Charger des fichiers MCDOC (SIMPLIFIÉ - parsing côté TypeScript)
    pub fn load_mcdoc_files(&mut self, _files: FxHashMap<String, &'input str>) -> Result<(), Vec<McDocParserError>> {
        // Parsing et validation réels côté TypeScript selon spec
        Ok(())
    }
    
    /// Charger un registre depuis JSON
    pub fn load_registry(&mut self, name: String, version: String, json: &serde_json::Value) -> Result<(), McDocParserError> {
        self.registry_manager.load_registry_from_json(name, version, json)
    }
    
    /// Valider un JSON contre les schemas MCDOC - VERSION SIMPLIFIÉE
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
        
        // 2. Validation basique JSON (pas de schéma MCDOC complexe)
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
        
        // 3. Extraire les dépendances registres (heuristiques simples)
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
        
        // 4. Validation des registres (si chargés) 
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
                        // Resource found in registry - OK
                    }
                }
            }
        }
        
        result
    }
}

impl<'input> Default for McDocValidator<'input> {
    fn default() -> Self {
        Self::new()
    }
} 