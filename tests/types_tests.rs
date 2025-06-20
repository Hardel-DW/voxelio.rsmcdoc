use voxel_rsmcdoc::types::MinecraftVersion;

#[test]
fn test_minecraft_version_parsing() {
    let v1 = MinecraftVersion::parse("1.20").unwrap();
    assert_eq!(v1.major, 1);
    assert_eq!(v1.minor, 20);
    assert_eq!(v1.patch, 0);
    
    let v2 = MinecraftVersion::parse("1.20.5").unwrap();
    assert_eq!(v2.major, 1);
    assert_eq!(v2.minor, 20);
    assert_eq!(v2.patch, 5);
    
    assert!(MinecraftVersion::parse("invalid").is_none());
}

#[test]
fn test_version_comparison() {
    let v1_20 = MinecraftVersion::parse("1.20").unwrap();
    let v1_20_5 = MinecraftVersion::parse("1.20.5").unwrap();
    let v1_21 = MinecraftVersion::parse("1.21").unwrap();
    
    assert!(v1_21.is_at_least(&v1_20));
    assert!(v1_20_5.is_at_least(&v1_20));
    assert!(!v1_20.is_at_least(&v1_20_5));
} 