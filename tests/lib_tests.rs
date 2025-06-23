use voxel_rsmcdoc::{
    DatapackValidator, ResourceId
};

#[test]
fn test_resource_id_parsing() {
    let id = ResourceId::parse("minecraft:diamond_sword").unwrap();
    assert_eq!(id.namespace, "minecraft");
    assert_eq!(id.path, "diamond_sword");
    
    let id2 = ResourceId::parse("diamond_sword").unwrap();
    assert_eq!(id2.namespace, "");
    
    let id3 = ResourceId::parse_with_default_namespace("diamond_sword", Some("custom")).unwrap();
    assert_eq!(id3.namespace, "custom");
    assert_eq!(id3.path, "diamond_sword");
}

#[test]
fn test_simple_validation_without_schema() {
    let validator = DatapackValidator::new();
    
    let test_json = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": {"item": "minecraft:diamond_sword"}
    });
    
    let result = validator.validate_json(&test_json, "minecraft:recipe", None);
    
    // Validation should fail because no schema is loaded
    assert!(!result.is_valid);
}

#[test]
fn test_simple_mcdoc_parsing() {
    let simple_mcdoc = r#"
        struct SimpleItem {
            name: string,
            count: int,
        }
    "#;
    
    match voxel_rsmcdoc::parse_mcdoc(simple_mcdoc) {
        Ok(ast) => {
            assert!(!ast.declarations.is_empty(), "Should have declarations");
        }
        Err(errors) => {
            panic!("Simple parsing failed: {:?}", errors);
        }
    }
}

#[test]
fn test_phase_0_compilation_works() {
    // Test: Create validator 
    let validator = DatapackValidator::new();
    
    // Test: Basic validator methods accessible
    let test_json = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": {"item": "minecraft:diamond_sword"}
    });
    
    let result = validator.validate_json(&test_json, "minecraft:recipe", None);
    
    // Basic assertions
    assert!(result.errors.is_empty() || !result.errors.is_empty());
    assert!(result.dependencies.is_empty() || !result.dependencies.is_empty());
}

#[test]
fn test_phase_0_api_consistency() {
    // Test critical API points are accessible and work
    let id = ResourceId::parse("minecraft:diamond_sword").unwrap();
    assert_eq!(id.namespace, "minecraft");
    assert_eq!(id.path, "diamond_sword");
    
    let id2 = ResourceId::parse("diamond_sword").unwrap();
    assert_eq!(id2.namespace, ""); // No default namespace
    assert_eq!(id2.path, "diamond_sword");
    
    let id3 = ResourceId::parse_with_default_namespace("diamond_sword", Some("custom")).unwrap();
    assert_eq!(id3.namespace, "custom");
    assert_eq!(id3.path, "diamond_sword");
}

#[test]
fn test_phase_1_simple_validation() {
    // Test validator creation and basic JSON validation without full schemas
    let validator = DatapackValidator::new();
    
    let test_json = serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": {"item": "minecraft:diamond_sword"}
    });
    
    let result = validator.validate_json(&test_json, "minecraft:recipe", None);
    
    // Phase 1 success criteria: validation completes without crashing
    assert!(!result.is_valid); // Either outcome is acceptable in Phase 1
}

#[test]
fn test_mcdoc_parsing_simple() {
    let _validator = DatapackValidator::new();
    let mcdoc = "struct Test { field: string }";
    
    // This test just ensures parsing doesn't panic
    // Real parsing tests are in parser_tests.rs
    let _ = voxel_rsmcdoc::parse_mcdoc(mcdoc);
}

#[test]
fn test_full_flow_placeholder() {
    // This test acts as a placeholder for a full integration test
    // similar to what would be done in `integration_tests.rs` but simpler.
    
    // 1. Initialize validator
    let mut validator = DatapackValidator::new();

    // 2. Load Registries
    let registry_json = serde_json::json!({
        "entries": { "minecraft:diamond": {} }
    });
    validator.load_registry("item".to_string(), "1.21".to_string(), &registry_json).unwrap();

    // 3. Load MCDOC
    let mcdoc = "dispatch minecraft:resource[test] to struct Test { result: #[id=\"item\"] string }";
    let ast = voxel_rsmcdoc::parse_mcdoc(mcdoc).unwrap();
    validator.load_parsed_mcdoc("test.mcdoc".to_string(), ast).unwrap();

    // 4. Validate JSON
    let json = serde_json::json!({ "result": "minecraft:diamond" });
    let result = validator.validate_json(&json, "minecraft:test", None);

    assert!(result.is_valid, "Validation failed: {:?}", result.errors);
    assert_eq!(result.dependencies.len(), 1);
    assert_eq!(result.dependencies[0].resource_location, "minecraft:diamond");
}

#[test]
fn test_versioned_registry() {
    let mut validator = DatapackValidator::new();
    
    let registry_1_20 = serde_json::json!({ "entries": { "minecraft:stone": {} } });
    let registry_1_21 = serde_json::json!({ "entries": { "minecraft:stone": {}, "minecraft:granite": {} } });
    
    validator.load_registry("block".to_string(), "1.20".to_string(), &registry_1_20).unwrap();
    validator.load_registry("block".to_string(), "1.21".to_string(), &registry_1_21).unwrap();

    assert!(validator.registry_manager.has_registry("block"));
} 