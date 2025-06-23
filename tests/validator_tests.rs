use voxel_rsmcdoc::validator::DatapackValidator;
use serde_json::json;

#[test]
fn test_validator_creation() {
    let validator = DatapackValidator::new();
    // Verify that the registry manager is initialized
    assert!(!validator.registry_manager.has_registry("item"));
}

/*
#[test]
fn test_load_mcdoc_files() {
    let mut validator = DatapackValidator::new();
    let mut files = rustc_hash::FxHashMap::default();
    files.insert("test.mcdoc".to_string(), "struct Test {}");
    
    // Load MCDOC files (simplified - always returns Ok)
    assert!(validator.load_mcdoc_files(files).is_ok());
}
*/

#[test]
fn test_validate_json_basic() {
    let validator = DatapackValidator::new();
    let json = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": "minecraft:diamond_sword"
    });
    
    let result = validator.validate_json(&json, "minecraft:recipe", None);
    // Without loaded registries, basic validation only
    assert!(!result.is_valid); // Expect error due to no schema
}

#[test]
fn test_validate_json_with_registry() {
    let mut validator = DatapackValidator::new();
    
    let registry = json!({ "entries": { "minecraft:stone": {} } });
    validator.load_registry("item".to_string(), "1.21".to_string(), &registry).unwrap();
    
    let json = json!({ "item": "minecraft:stone" });
    let result = validator.validate_json(&json, "test:item", None);
    
    assert!(!result.is_valid); // Expect error due to no schema
}

#[test]
fn test_validate_json_missing_dependency() {
    let validator = DatapackValidator::new();
    
    let json = json!({ "item": "minecraft:nonexistent" });
    let result = validator.validate_json(&json, "test:item", None);
    
    assert!(!result.is_valid); // Should be false once validation is implemented
}

/*
#[test]
fn test_get_required_registries() {
    let mut validator = DatapackValidator::new();
    
    let json = json!({
        "type": "crafting_shaped",
        "result": { "item": "minecraft:diamond" },
        "ingredients": [
            { "item": "minecraft:stick" },
            { "tag": "minecraft:planks" }
        ]
    });
    
    let registries = validator.get_required_registries(&json, "recipe");
    assert_eq!(registries, vec!["item", "tag"]);
}
*/

#[test]
fn test_registry_loading() {
    let mut validator = DatapackValidator::new();
    
    // Test loading a simple registry
    let registry_json = json!({
        "entries": {
            "minecraft:stone": {},
            "minecraft:diamond": {}
        }
    });
    
    assert!(validator.load_registry("item".to_string(), "1.z".to_string(), &registry_json).is_ok());
    assert!(validator.registry_manager.has_registry("item"));
} 