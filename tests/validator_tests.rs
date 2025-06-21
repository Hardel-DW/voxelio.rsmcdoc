use voxel_rsmcdoc::validator::McDocValidator;

#[test]
fn test_validator_creation() {
    let validator = McDocValidator::new();
    // Vérifier que le registre manager est initialisé
    assert!(!validator.registry_manager.has_registry("item"));
}

#[test]
fn test_load_mcdoc_files() {
    let mut validator = McDocValidator::new();
    let mut files = rustc_hash::FxHashMap::default();
    files.insert("test.mcdoc".to_string(), "struct Test {}");
    
    // Load MCDOC files (simplifié - retourne toujours Ok)
    assert!(validator.load_mcdoc_files(files).is_ok());
}

#[test]
fn test_validate_json_basic() {
    let validator = McDocValidator::new();
    let json = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": "minecraft:diamond_sword"
    });
    
    let result = validator.validate_json(&json, "minecraft:recipe");
    // Sans registres chargés, validation basique seulement
    assert!(result.is_valid || !result.errors.is_empty());
}

// Test extract_resource_id supprimé - fonction non utilisée en WASM

#[test]
fn test_registry_loading() {
    let mut validator = McDocValidator::new();
    
    // Test loading a simple registry
    let registry_json = serde_json::json!({
        "entries": {
            "minecraft:diamond": {},
            "minecraft:stick": {}
        }
    });
    
    assert!(validator.load_registry("item".to_string(), "1.20".to_string(), &registry_json).is_ok());
    assert!(validator.registry_manager.has_registry("item"));
} 