use voxel_rsmcdoc::ResourceId;

#[test]
fn test_resource_id_parsing() {
    // Test with explicit namespace
    let id = ResourceId::parse("minecraft:diamond_sword").unwrap();
    assert_eq!(id.namespace, "minecraft");
    assert_eq!(id.path, "diamond_sword");
    
    // Test without namespace - no default
    let id2 = ResourceId::parse("diamond_sword").unwrap();
    assert_eq!(id2.namespace, ""); // No default namespace
    assert_eq!(id2.path, "diamond_sword");
    
    // Test with custom default namespace
    let id3 = ResourceId::parse_with_default_namespace("diamond_sword", Some("custom")).unwrap();
    assert_eq!(id3.namespace, "custom");
    assert_eq!(id3.path, "diamond_sword");
    
    // Test invalid format
    assert!(ResourceId::parse("too:many:colons").is_err());
} 