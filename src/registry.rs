//! Gestion des registres Minecraft
//! 
//! Cache optimisé avec FxHashMap pour la validation des resource locations.

use crate::RegistryDependency;
use crate::error::McDocParserError;
use crate::types::McDocDependency;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

/// Registre Minecraft avec ses entrées
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub name: String,
    pub entries: FxHashSet<String>,
    pub tags: FxHashMap<String, Vec<String>>, // tag -> list of resource locations
    pub version: String,
}

impl Registry {
    /// Créer un nouveau registre
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            entries: FxHashSet::default(),
            tags: FxHashMap::default(),
            version,
        }
    }
    
    /// Ajouter une entrée au registre
    pub fn add_entry(&mut self, resource_location: String) {
        self.entries.insert(resource_location);
    }
    
    /// Ajouter un tag avec ses entrées
    pub fn add_tag(&mut self, tag_name: String, entries: Vec<String>) {
        self.tags.insert(tag_name, entries);
    }
    
    /// Vérifier si une resource location existe
    pub fn contains(&self, resource_location: &str) -> bool {
        self.entries.contains(resource_location)
    }
    
    /// Vérifier si un tag existe
    pub fn contains_tag(&self, tag_name: &str) -> bool {
        self.tags.contains_key(tag_name)
    }
    
    /// Obtenir les entrées d'un tag
    pub fn get_tag_entries(&self, tag_name: &str) -> Option<&Vec<String>> {
        self.tags.get(tag_name)
    }
    
    /// Charger depuis JSON (format vanilla registries)
    pub fn from_json(name: String, version: String, json: &serde_json::Value) -> Result<Self, McDocParserError> {
        let mut registry = Registry::new(name, version);
        
        // Format: { "entries": { "minecraft:item": {...}, ... } }
        if let Some(entries) = json.get("entries").and_then(|e| e.as_object()) {
            for key in entries.keys() {
                registry.add_entry(key.clone());
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
                    registry.add_tag(tag_name.clone(), entries);
                }
            }
        }
        
        Ok(registry)
    }
}

/// Manager pour tous les registres avec cache optimisé
pub struct RegistryManager {
    registries: FxHashMap<String, Registry>,
    // Future: version_compatibility: FxHashMap<String, Vec<String>>, // version -> compatible registry versions
}

impl RegistryManager {
    /// Créer un nouveau manager
    pub fn new() -> Self {
        Self {
            registries: FxHashMap::default(),
            // Future: version_compatibility: FxHashMap::default(),
        }
    }
    
    /// Ajouter un registre
    pub fn add_registry(&mut self, registry: Registry) {
        self.registries.insert(registry.name.clone(), registry);
    }
    
    /// Charger un registre depuis JSON
    pub fn load_registry_from_json(
        &mut self,
        name: String,
        version: String,
        json: &serde_json::Value,
    ) -> Result<(), McDocParserError> {
        let registry = Registry::from_json(name, version, json)?;
        self.add_registry(registry);
        Ok(())
    }
    
    /// Valider une resource location dans un registre
    pub fn validate_resource_location(
        &self,
        registry_name: &str,
        resource_location: &str,
        is_tag: bool,
    ) -> Result<bool, McDocParserError> {
        let registry = self.registries.get(registry_name)
            .ok_or_else(|| McDocParserError::InvalidRegistry {
                path: "unknown".to_string(),
                value: resource_location.to_string(),
                registry: registry_name.to_string(),
            })?;
        
        if is_tag {
            // Tag validation (starts with #)
            let tag_name = if let Some(stripped) = resource_location.strip_prefix('#') {
                stripped // Remove #
            } else {
                resource_location
            };
            
            Ok(registry.contains_tag(tag_name))
        } else {
            // For resource locations, try both with and without minecraft: prefix
            let found = registry.contains(resource_location);
            
            if !found && resource_location.starts_with("minecraft:") {
                // Try without the minecraft: prefix
                let bare_name = &resource_location[10..]; // Remove "minecraft:"
                Ok(registry.contains(bare_name))
            } else if !found && !resource_location.contains(':') {
                // Try with minecraft: prefix
                let prefixed_name = format!("minecraft:{}", resource_location);
                Ok(registry.contains(&prefixed_name))
            } else {
                Ok(found)
            }
        }
    }
    
    /// Extraire les dépendances registres d'un JSON selon les annotations MCDOC
    pub fn extract_dependencies_from_json(
        &self,
        json: &serde_json::Value,
        annotations: &[(&str, &str)], // (json_path, registry_type)
    ) -> Vec<McDocDependency> {
        let mut dependencies = Vec::new();
        
        for (json_path, registry_type) in annotations {
            if let Some(value) = self.get_json_value_at_path(json, json_path) {
                let is_tag = value.starts_with('#');
                
                dependencies.push(McDocDependency {
                    resource_location: value.to_string(),
                    registry_type: registry_type.to_string(),
                    source_path: json_path.to_string(),
                    source_file: None,
                    is_tag,
                });
            }
        }
        
        dependencies
    }
    
    /// Obtenir une valeur JSON à un chemin donné (e.g., "result.item")
    pub fn get_json_value_at_path<'a>(&self, json: &'a serde_json::Value, path: &str) -> Option<&'a str> {
        let mut current = json;
        let mut i = 0;
        let path_chars: Vec<char> = path.chars().collect();
        
        while i < path_chars.len() {
            if path_chars[i] == '[' {
                // Find the closing bracket
                let mut j = i + 1;
                while j < path_chars.len() && path_chars[j] != ']' {
                    j += 1;
                }
                if j >= path_chars.len() {
                    return None; // Malformed path
                }
                
                // Extract index
                let index_str: String = path_chars[i+1..j].iter().collect();
                if let Ok(index) = index_str.parse::<usize>() {
                    current = current.get(index)?;
                } else {
                    return None;
                }
                i = j + 1;
                
                // Skip the dot if present
                if i < path_chars.len() && path_chars[i] == '.' {
                    i += 1;
                }
            } else {
                // Find the next dot or bracket
                let mut j = i;
                while j < path_chars.len() && path_chars[j] != '.' && path_chars[j] != '[' {
                    j += 1;
                }
                
                // Extract field name
                let field_name: String = path_chars[i..j].iter().collect();
                current = current.get(&field_name)?;
                i = j;
                
                // Skip the dot if present
                if i < path_chars.len() && path_chars[i] == '.' {
                    i += 1;
                }
            }
        }
        
        current.as_str()
    }
    
    /// Pre-scan d'un JSON pour détecter les types de registres nécessaires
    pub fn scan_required_registries(&self, json: &serde_json::Value) -> Vec<RegistryDependency> {
        let mut registries = Vec::new();
        self.scan_json_recursive(json, "", &mut registries);
        registries
    }
    
    fn scan_json_recursive(&self, value: &serde_json::Value, path: &str, registries: &mut Vec<RegistryDependency>) {
        match value {
            serde_json::Value::String(s) => {
                // Détecter les patterns de resource locations
                if self.looks_like_resource_location(s) {
                    let is_tag = s.starts_with('#');
                    let registry_type = self.infer_registry_type(path, s);
                    
                    registries.push(RegistryDependency {
                        registry: registry_type,
                        identifier: s.clone(),
                        is_tag,
                    });
                }
            }
            serde_json::Value::Object(obj) => {
                for (key, val) in obj {
                    let new_path = if path.is_empty() {
                        key.clone()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    self.scan_json_recursive(val, &new_path, registries);
                }
            }
            serde_json::Value::Array(arr) => {
                for (index, val) in arr.iter().enumerate() {
                    let new_path = format!("{}[{}]", path, index);
                    self.scan_json_recursive(val, &new_path, registries);
                }
            }
            _ => {}
        }
    }
    
    /// Détecter si une string ressemble à une resource location
    pub fn looks_like_resource_location(&self, s: &str) -> bool {
        // Pattern: namespace:path or #namespace:path
        let s = if let Some(stripped) = s.strip_prefix('#') { stripped } else { s };
        
        s.contains(':') && 
        s.chars().all(|c| c.is_alphanumeric() || c == ':' || c == '_' || c == '/' || c == '.')
    }
    
    /// Inférer le type de registre depuis le chemin JSON
    fn infer_registry_type(&self, path: &str, _value: &str) -> String {
        // Heuristiques basées sur les patterns courants
        match path {
            p if p.contains("item") => "item".to_string(),
            p if p.contains("block") => "block".to_string(),
            p if p.contains("recipe") => "recipe".to_string(),
            p if p.contains("enchantment") => "enchantment".to_string(),
            p if p.contains("biome") => "worldgen/biome".to_string(),
            p if p.contains("damage") => "damage_type".to_string(),
            p if p.contains("sound") => "sound_event".to_string(),
            _ => "unknown".to_string(), // Nécessitera annotation MCDOC
        }
    }
    
    /// Obtenir tous les registres chargés
    pub fn get_loaded_registries(&self) -> Vec<&str> {
        self.registries.keys().map(|s| s.as_str()).collect()
    }
    
    /// Vérifier si un registre est chargé
    pub fn has_registry(&self, name: &str) -> bool {
        self.registries.contains_key(name)
    }
    
    /// Obtenir les statistiques d'un registre
    pub fn get_registry_stats(&self, name: &str) -> Option<(usize, usize)> {
        self.registries.get(name).map(|r| (r.entries.len(), r.tags.len()))
    }
    
    /// Charger registres Minecraft réels depuis JSON
    pub async fn load_minecraft_registries(&mut self, version: &str) -> Result<(), McDocParserError> {
        // Load common Minecraft registries for the specified version
        let registries_to_load = vec![
            "item", "block", "entity_type", "enchantment", 
            "recipe_serializer", "loot_condition_type", "loot_function_type"
        ];
        
        for registry_name in registries_to_load {
            if let Ok(data) = self.fetch_registry_data(registry_name, version).await {
                self.load_registry_from_json(registry_name.to_string(), version.to_string(), &data)?;
            }
            // If fetch fails, registry stays empty (graceful degradation)
        }
        
        Ok(())
    }
    
    /// Charger registres depuis le filesystem local
    pub fn load_registries_from_directory(&mut self, base_path: &str) -> Result<(), McDocParserError> {
        use std::fs;
        use std::path::Path;
        
        let registries_dir = Path::new(base_path);
        if !registries_dir.exists() {
            return Err(McDocParserError::IoError(format!("Registry directory not found: {}", base_path)));
        }
        
        // Load all JSON files in the registries directory
        for entry in fs::read_dir(registries_dir).map_err(|e| McDocParserError::IoError(e.to_string()))? {
            let entry = entry.map_err(|e| McDocParserError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if path.extension() == Some(std::ffi::OsStr::new("json")) {
                let registry_name = path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                
                let content = fs::read_to_string(&path)
                    .map_err(|e| McDocParserError::IoError(e.to_string()))?;
                
                let json: serde_json::Value = serde_json::from_str(&content)
                    .map_err(|e| McDocParserError::JsonParseError(e.to_string()))?;
                
                self.load_registry_from_json(registry_name, "local".to_string(), &json)?;
            }
        }
        
        Ok(())
    }
    
    /// Fetch registry data from Minecraft assets (placeholder for future HTTP/file loading)
    async fn fetch_registry_data(&self, _registry_name: &str, _version: &str) -> Result<serde_json::Value, McDocParserError> {
        // For now, return a minimal valid registry structure
        // In production, this would fetch from Minecraft assets or external API
        Ok(serde_json::json!({
            "entries": {},
            "tags": {}
        }))
    }
    
    /// Charger tous les registres depuis le fichier data.min.json 
    pub fn load_minecraft_data(&mut self, data_json: &serde_json::Value, version: &str) -> Result<(), McDocParserError> {
        if let Some(obj) = data_json.as_object() {
            for (registry_name, entries) in obj {
                if let Some(array) = entries.as_array() {
                    let mut registry = Registry::new(registry_name.clone(), version.to_string());
                    
                    // Add all entries from the array
                    for entry in array {
                        if let Some(entry_str) = entry.as_str() {
                            registry.add_entry(entry_str.to_string());
                        }
                    }
                    
                    self.add_registry(registry);
                }
            }
            Ok(())
        } else {
            Err(McDocParserError::JsonParseError(
                "Expected object at root level".to_string()
            ))
        }
    }
    
    /// Charger les registres de test depuis un fichier JSON utilisateur (pour development/tests)
    pub fn load_test_data(&mut self, data_file_path: &str) -> Result<(), McDocParserError> {
        use std::fs;
        
        let data_content = fs::read_to_string(data_file_path)
            .map_err(|e| McDocParserError::IoError(format!("Failed to read test data from {}: {}", data_file_path, e)))?;
        
        let data_json: serde_json::Value = serde_json::from_str(&data_content)
            .map_err(|e| McDocParserError::JsonParseError(format!("Failed to parse test data: {}", e)))?;
        
        self.load_minecraft_data(&data_json, "1.21")?;
        
        // Log loaded registries for debugging
        println!("✅ Loaded {} registries from test data: {}", self.registries.len(), data_file_path);
        for (name, registry) in &self.registries {
            println!("   {} registry: {} entries", name, registry.entries.len());
        }
        
        Ok(())
    }
}

impl Default for RegistryManager {
    fn default() -> Self {
        Self::new()
    }
} 