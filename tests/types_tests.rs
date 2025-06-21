use voxel_rsmcdoc::types::{ValidationResult, McDocError, McDocDependency, DatapackResult, MinecraftVersion};

#[test]
fn test_validation_result_creation() {
    let result = ValidationResult::success(vec![]);
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
    assert!(result.dependencies.is_empty());
}

#[test]
fn test_minecraft_version_simple() {
    // MinecraftVersion is now a String type alias
    let version: MinecraftVersion = "1.20.5".to_string();
    assert_eq!(version, "1.20.5");
    
    let version2: MinecraftVersion = "1.21".to_string();
    assert_eq!(version2, "1.21");
}

#[test]
fn test_version_comparison_simple() {
    // Simple string comparison of versions
    let v1_20: MinecraftVersion = "1.20".to_string();
    let v1_20_5: MinecraftVersion = "1.20.5".to_string();
    let v1_21: MinecraftVersion = "1.21".to_string();
    
    // Simple string comparison (real parsing done in TypeScript)
    assert!(v1_20 < v1_21);
    assert!(v1_20_5 != v1_21);
    assert_eq!(v1_20, "1.20");
}

#[test]
fn test_mcdoc_dependency() {
    let dependency = McDocDependency {
        resource_location: "minecraft:diamond_sword".to_string(),
        registry_type: "item".to_string(),
        source_path: "result.item".to_string(),
        source_file: Some("recipes/diamond_sword.json".to_string()),
        is_tag: false,
    };
    
    assert_eq!(dependency.resource_location, "minecraft:diamond_sword");
    assert_eq!(dependency.registry_type, "item");
    assert!(!dependency.is_tag);
}

#[test]
fn test_mcdoc_error() {
    let error = McDocError {
        file: "test.json".to_string(),
        path: "result.item".to_string(),
        message: "Invalid item reference".to_string(),
        error_type: voxel_rsmcdoc::error::ErrorType::Validation,
        line: Some(10),
        column: Some(15),
    };
    
    assert_eq!(error.file, "test.json");
    assert_eq!(error.message, "Invalid item reference");
    assert_eq!(error.line, Some(10));
}

#[test]
fn test_datapack_result() {
    let mut result = DatapackResult::new();
    
    // Test initial state
    assert_eq!(result.total_files, 0);
    assert_eq!(result.valid_files, 0);
    assert!(result.errors.is_empty());
    assert!(result.dependencies.is_empty());
    
    // Test adding file results
    let validation_result = ValidationResult::success(vec![
        McDocDependency {
            resource_location: "minecraft:diamond".to_string(),
            registry_type: "item".to_string(),
            source_path: "ingredients[0]".to_string(),
            source_file: None,
            is_tag: false,
        }
    ]);
    
    result.add_file_result("test.json".to_string(), validation_result);
    
    assert_eq!(result.total_files, 1);
    assert_eq!(result.valid_files, 1);
    assert!(result.errors.is_empty());
    assert!(!result.dependencies.is_empty());
}

#[test]
fn test_error_variants() {
    use voxel_rsmcdoc::error::ErrorType;
    
    let lexer_error = ErrorType::Lexer;
    let validation_error = ErrorType::Validation;
    // JsonError removed as it was unused and consolidated into Validation
    
    // Test that error types can be compared
    assert_ne!(lexer_error, validation_error);
    
    // Test display formatting
    let error_msg = format!("{:?}", validation_error);
    assert!(error_msg.contains("Validation"));
} 