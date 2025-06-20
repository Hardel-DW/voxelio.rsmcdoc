use voxel_rsmcdoc::registry::{Registry, RegistryManager};

#[test]
fn test_registry_creation() {
    let mut registry = Registry::new("item".to_string(), "1.20.5".to_string());
    registry.add_entry("minecraft:diamond_sword".to_string());
    registry.add_entry("minecraft:iron_sword".to_string());
    
    assert!(registry.contains("minecraft:diamond_sword"));
    assert!(registry.contains("minecraft:iron_sword"));
    assert!(!registry.contains("minecraft:nonexistent"));
}

#[test]
fn test_registry_tags() {
    let mut registry = Registry::new("item".to_string(), "1.20.5".to_string());
    registry.add_tag("minecraft:swords".to_string(), vec![
        "minecraft:diamond_sword".to_string(),
        "minecraft:iron_sword".to_string(),
    ]);
    
    assert!(registry.contains_tag("minecraft:swords"));
    assert!(!registry.contains_tag("minecraft:nonexistent"));
    
    let entries = registry.get_tag_entries("minecraft:swords").unwrap();
    assert_eq!(entries.len(), 2);
}

#[test]
fn test_manager_validation() {
    let mut manager = RegistryManager::new();
    let mut registry = Registry::new("item".to_string(), "1.20.5".to_string());
    
    registry.add_entry("minecraft:diamond_sword".to_string());
    registry.add_tag("minecraft:swords".to_string(), vec!["minecraft:diamond_sword".to_string()]);
    
    manager.add_registry(registry);
    
    // Test valid resource location
    assert!(manager.validate_resource_location("item", "minecraft:diamond_sword", false).unwrap());
    
    // Test valid tag
    assert!(manager.validate_resource_location("item", "minecraft:swords", true).unwrap());
    
    // Test invalid resource location
    assert!(!manager.validate_resource_location("item", "minecraft:nonexistent", false).unwrap());
}

#[test]
fn test_resource_location_detection() {
    let manager = RegistryManager::new();
    
    assert!(manager.looks_like_resource_location("minecraft:diamond_sword"));
    assert!(manager.looks_like_resource_location("#minecraft:swords"));
    assert!(!manager.looks_like_resource_location("not_a_resource"));
    assert!(!manager.looks_like_resource_location("no-colon"));
}

#[test]
fn test_json_path_extraction() {
    let manager = RegistryManager::new();
    let json = serde_json::json!({
        "result": {
            "item": "minecraft:diamond_sword"
        },
        "ingredients": [
            "minecraft:diamond",
            "minecraft:stick"
        ]
    });
    
    assert_eq!(manager.get_json_value_at_path(&json, "result.item"), Some("minecraft:diamond_sword"));
    assert_eq!(manager.get_json_value_at_path(&json, "ingredients[0]"), Some("minecraft:diamond"));
    assert_eq!(manager.get_json_value_at_path(&json, "ingredients[1]"), Some("minecraft:stick"));
    assert_eq!(manager.get_json_value_at_path(&json, "nonexistent"), None);
}

#[test]
fn test_load_minecraft_data() {
    let mut manager = RegistryManager::new();
    
    // Load test data if available
    if let Ok(()) = manager.load_test_data("examples/data.min.json") {
        // Verify some critical registries are loaded
        assert!(manager.has_registry("item"));
        assert!(manager.has_registry("block"));
        assert!(manager.has_registry("entity_type"));
        assert!(manager.has_registry("enchantment"));
        
        // Test some known items exist
        assert!(manager.validate_resource_location("item", "minecraft:diamond_sword", false).unwrap_or(false));
        assert!(manager.validate_resource_location("item", "minecraft:diamond", false).unwrap_or(false));
        assert!(manager.validate_resource_location("block", "minecraft:stone", false).unwrap_or(false));
        
        // Test invalid items don't exist
        assert!(!manager.validate_resource_location("item", "minecraft:nonexistent_item", false).unwrap_or(true));
        
        println!("✅ Minecraft data loading test passed");
    } else {
        println!("⚠️ Test data not available, skipping minecraft data test");
    }
}

#[test]
fn test_data_json_structure() {
    let sample_data = serde_json::json!({
        "item": ["minecraft:diamond_sword", "minecraft:diamond", "minecraft:stick"],
        "block": ["minecraft:stone", "minecraft:dirt", "minecraft:grass_block"],
        "enchantment": ["minecraft:sharpness", "minecraft:protection"]
    });
    
    let mut manager = RegistryManager::new();
    manager.load_minecraft_data(&sample_data, "test").unwrap();
    
    assert!(manager.has_registry("item"));
    assert!(manager.has_registry("block"));
    assert!(manager.has_registry("enchantment"));
    
    assert!(manager.validate_resource_location("item", "minecraft:diamond_sword", false).unwrap());
    assert!(manager.validate_resource_location("block", "minecraft:stone", false).unwrap());
    assert!(manager.validate_resource_location("enchantment", "minecraft:sharpness", false).unwrap());
    
    assert!(!manager.validate_resource_location("item", "minecraft:nonexistent", false).unwrap());
} 