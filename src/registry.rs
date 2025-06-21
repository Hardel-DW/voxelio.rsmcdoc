//! Gestion des registres Minecraft

use crate::RegistryDependency;
use crate::error::ParseError;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Registre Minecraft avec ses entrées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub name: String,
    pub entries: HashSet<String>,
    pub tags: HashMap<String, Vec<String>>, // tag -> list of resource locations
    pub version: String,
}

impl Registry {
    /// Créer un nouveau registre
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            entries: HashSet::default(),
            tags: HashMap::default(),
            version,
        }
    }
    
    /// Vérifier si une resource location existe
    pub fn contains(&self, resource_location: &str) -> bool {
        self.entries.contains(resource_location)
    }
    
    /// Vérifier si un tag existe
    pub fn contains_tag(&self, tag_name: &str) -> bool {
        self.tags.contains_key(tag_name)
    }
    
    /// Charger depuis JSON (format vanilla registries)
    pub fn from_json(name: String, version: String, json: &serde_json::Value) -> Result<Self, ParseError> {
        let mut registry = Registry::new(name, version);
        
        // Format: { "entries": { "minecraft:item": {...}, ... } }
        if let Some(entries) = json.get("entries").and_then(|e| e.as_object()) {
            for key in entries.keys() {
                registry.entries.insert(key.clone());
            }
        }
        
        // Format: { "tags": { "minecraft:swords": ["minecraft:diamond_sword", ...], ... } }
        if let Some(tags) = json.get("tags").and_then(|t| t.as_object()) {
            for (tag_name, tag_entries) in tags {
                if let Some(entries_array) = tag_entries.as_array() {
                    let entries: Vec<String> = entries_array
                        .iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect();
                    registry.tags.insert(tag_name.clone(), entries);
                }
            }
        }
        
        Ok(registry)
    }
}

/// Manager pour tous les registres - SIMPLIFIÉ
pub struct RegistryManager {
    registries: FxHashMap<String, Registry>,
}

impl RegistryManager {
    /// Créer un nouveau manager
    pub fn new() -> Self {
        Self {
            registries: FxHashMap::default(),
        }
    }
    
    /// Charger un registre depuis JSON
    pub fn load_registry_from_json(
        &mut self,
        name: String,
        version: String,
        json: &serde_json::Value,
    ) -> Result<(), ParseError> {
        let registry = Registry::from_json(name, version, json)?;
        self.registries.insert(registry.name.clone(), registry);
        Ok(())
    }
    
    /// Valider une resource location dans un registre
    pub fn validate_resource_location(
        &self,
        registry_name: &str,
        resource_location: &str,
        is_tag: bool,
    ) -> Result<bool, ParseError> {
        self.validate_resource_location_with_namespace(registry_name, resource_location, is_tag, None)
    }
    
    /// Valider une resource location avec namespace configurable (no hardcoding)
    pub fn validate_resource_location_with_namespace(
        &self,
        registry_name: &str,
        resource_location: &str,
        is_tag: bool,
        default_namespace: Option<&str>,
    ) -> Result<bool, ParseError> {
        let registry = self.registries.get(registry_name)
            .ok_or_else(|| ParseError::validation(
                format!("Unknown registry '{}'", registry_name),
                format!("Resource location: {}", resource_location)
            ))?;
        
        if is_tag {
            // Tag validation (starts with #)
            let tag_name = if let Some(stripped) = resource_location.strip_prefix('#') {
                stripped // Remove #
            } else {
                resource_location
            };
            
            Ok(registry.contains_tag(tag_name))
        } else {
            // For resource locations, try both with and without namespace prefix (configurable)
            let found = registry.contains(resource_location);
            
            if let Some(namespace) = default_namespace {
                let namespace_prefix = format!("{}:", namespace);
                
                if !found && resource_location.starts_with(&namespace_prefix) {
                    // Try without the namespace prefix
                    let bare_name = &resource_location[namespace_prefix.len()..];
                    Ok(registry.contains(bare_name))
                } else if !found && !resource_location.contains(':') {
                    // Try with namespace prefix
                    let prefixed_name = format!("{}{}", namespace_prefix, resource_location);
                    Ok(registry.contains(&prefixed_name))
                } else {
                    Ok(found)
                }
            } else {
                // No namespace handling if not provided
                Ok(found)
            }
        }
    }
    
    /// Pre-scan d'un JSON pour détecter les types de registres nécessaires
    pub fn scan_required_registries(&self, json: &serde_json::Value) -> Vec<RegistryDependency> {
        let mut registries = Vec::new();
        // Use empty mapping if none provided (no default hardcoding)
        let empty_mapping = HashMap::new();
        self.scan_json_simple(json, "", &mut registries, &empty_mapping);
        registries
    }
    
    /// Pre-scan with custom registry mapping (no hardcoding)
    pub fn scan_required_registries_with_mapping(
        &self, 
        json: &serde_json::Value, 
        registry_mapping: &HashMap<String, String>
    ) -> Vec<RegistryDependency> {
        let mut registries = Vec::new();
        self.scan_json_simple(json, "", &mut registries, registry_mapping);
        registries
    }
    
    /// Scan JSON simplifié (remplace scan_json_recursive complexe)
    fn scan_json_simple(&self, value: &serde_json::Value, path: &str, registries: &mut Vec<RegistryDependency>, registry_mapping: &HashMap<String, String>) {
        match value {
            serde_json::Value::String(s) => {
                // Pattern simple: namespace:path ou #namespace:path
                if s.contains(':') && (s.starts_with('#') || s.chars().all(|c| c.is_alphanumeric() || c == ':' || c == '_' || c == '/')) {
                    let is_tag = s.starts_with('#');
                    let registry_type = self.infer_registry_with_mapping(path, registry_mapping);
                    
                    registries.push(RegistryDependency {
                        registry: registry_type,
                        identifier: s.clone(),
                        is_tag,
                    });
                }
            }
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let new_path = if path.is_empty() { key.clone() } else { format!("{}.{}", path, key) };
                    self.scan_json_simple(val, &new_path, registries, registry_mapping);
                }
            }
            serde_json::Value::Array(arr) => {
                for val in arr {
                    self.scan_json_simple(val, path, registries, registry_mapping);
                }
            }
            _ => {}
        }
    }
    
    /// Inférer le type de registre avec mapping configurable (no hardcoding)
    fn infer_registry_with_mapping(&self, path: &str, registry_mapping: &HashMap<String, String>) -> String {
        // Use configurable mapping instead of hardcoded logic
        for (pattern, registry_type) in registry_mapping {
            if path.contains(pattern) {
                return registry_type.clone();
            }
        }
        // Return empty string if no mapping found (no hardcoding)
        String::new()
    }

    /// Vérifier si un registre est chargé
    pub fn has_registry(&self, name: &str) -> bool {
        self.registries.contains_key(name)
    }

    /// Create registry mapping from configuration (completely configurable, no hardcoding)
    pub fn create_registry_mapping_from_config(config: Vec<(String, String)>) -> HashMap<String, String> {
        config.into_iter().collect()
    }
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
} 