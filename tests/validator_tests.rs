use voxel_rsmcdoc::validator::McDocValidator;

#[test]
fn test_validator_creation() {
    let validator = McDocValidator::new();
    assert!(validator.registry_manager.get_loaded_registries().is_empty());
}

#[test]
fn test_resource_id_extraction() {
    let validator = McDocValidator::new();
    
    let path1 = "data/minecraft/recipes/diamond_sword.json";
    assert_eq!(validator.extract_resource_id_from_path(path1), "minecraft:diamond_sword");
    
    let path2 = "data/mymod/loot_tables/chests/dungeon.json";
    assert_eq!(validator.extract_resource_id_from_path(path2), "mymod:dungeon");
}

#[test]
fn test_json_validation_basic() {
    let validator = McDocValidator::new();
    
    let json = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": {
            "item": "minecraft:diamond_sword"
        }
    });
    
    let result = validator.validate_json(&json, "minecraft:diamond_sword_recipe");
    
    // Should detect dependencies even without registry validation
    assert!(!result.dependencies.is_empty());
}

#[test]
fn test_version_setting() {
    let mut validator = McDocValidator::new();
    
    assert!(validator.set_minecraft_version("1.20.5").is_ok());
    assert!(validator.set_minecraft_version("invalid").is_err());
} 