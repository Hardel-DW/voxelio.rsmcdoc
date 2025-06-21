use voxel_rsmcdoc::registry::{Registry, RegistryManager};
use serde_json::json;

#[test]
fn test_registry_creation() {
    let registry = Registry::new("item".to_string(), "1.20".to_string());
    assert_eq!(registry.name, "item");
    assert_eq!(registry.version, "1.20");
    assert!(registry.entries.is_empty());
}

#[test]
fn test_registry_add_entries() {
    let mut registry = Registry::new("item".to_string(), "1.20".to_string());
    registry.entries.insert("minecraft:diamond".to_string());
    registry.entries.insert("minecraft:stick".to_string());
    
    assert!(registry.contains("minecraft:diamond"));
    assert!(registry.contains("minecraft:stick"));
    assert!(!registry.contains("minecraft:nonexistent"));
}

#[test]
fn test_registry_from_json() {
    let json = json!({
        "entries": {
            "minecraft:diamond": {},
            "minecraft:stick": {},
            "minecraft:stone": {}
        },
        "tags": {
            "minecraft:gems": ["minecraft:diamond"],
            "minecraft:tools": ["minecraft:stick"]
        }
    });
    
    let registry = Registry::from_json("item".to_string(), "1.20".to_string(), &json);
    assert!(registry.is_ok());
    
    let registry = registry.unwrap();
    assert!(registry.contains("minecraft:diamond"));
    assert!(registry.contains_tag("minecraft:gems"));
}

#[test]
fn test_registry_manager() {
    let mut manager = RegistryManager::new();
    
    // Test basic functionality without removed methods
    assert!(!manager.has_registry("item"));
    
    // Test registry loading
    let json = json!({
        "entries": {
            "minecraft:diamond_sword": {},
            "minecraft:iron_sword": {}
        }
    });
    
    assert!(manager.load_registry_from_json("item".to_string(), "1.20".to_string(), &json).is_ok());
    assert!(manager.has_registry("item"));
}

#[test]
fn test_scan_required_registries() {
    let manager = RegistryManager::new();
    
    let json = json!({
        "result": "minecraft:diamond_sword",
        "ingredients": ["minecraft:diamond", "minecraft:stick"]
    });
    
    let dependencies = manager.scan_required_registries(&json);
    assert!(!dependencies.is_empty());
    
    // Should detect minecraft: patterns
    let has_minecraft_refs = dependencies.iter().any(|dep| dep.identifier.starts_with("minecraft:"));
    assert!(has_minecraft_refs);
}

#[test]
fn test_validate_resource_location() {
    let mut manager = RegistryManager::new();
    
    // Load a test registry
    let json = json!({
        "entries": {
            "minecraft:diamond_sword": {},
            "minecraft:iron_sword": {}
        },
        "tags": {
            "minecraft:swords": ["minecraft:diamond_sword", "minecraft:iron_sword"]
        }
    });
    
    manager.load_registry_from_json("item".to_string(), "1.20".to_string(), &json).unwrap();
    
    // Test valid resource
    assert!(manager.validate_resource_location("item", "minecraft:diamond_sword", false).unwrap());
    
    // Test invalid resource
    assert!(!manager.validate_resource_location("item", "minecraft:nonexistent", false).unwrap());
    
    // Test valid tag
    assert!(manager.validate_resource_location("item", "minecraft:swords", true).unwrap());
}

#[test]
fn test_load_minecraft_data() {
    let mut manager = RegistryManager::new();
    
    // Test simplified minecraft data loading
    let test_data = json!({
        "item": {
            "entries": {
                "minecraft:diamond_sword": {},
                "minecraft:stick": {}
            }
        },
        "block": {
            "entries": {
                "minecraft:stone": {},
                "minecraft:dirt": {}
            }
        }
    });
    
    // load_minecraft_data removed - load each registry individually
    assert!(manager.load_registry_from_json("item".to_string(), "1.20".to_string(), test_data.get("item").unwrap()).is_ok());
    assert!(manager.load_registry_from_json("block".to_string(), "1.20".to_string(), test_data.get("block").unwrap()).is_ok());
    assert!(manager.has_registry("item"));
    assert!(manager.has_registry("block"));
} 