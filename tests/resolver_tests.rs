use voxel_rsmcdoc::resolver::ImportResolver;
use voxel_rsmcdoc::parser::{McDocFile, ImportStatement, ImportPath};
use voxel_rsmcdoc::lexer::Position;
use voxel_rsmcdoc::error::McDocParserError;

#[test]
fn test_resolve_absolute_import() {
    let resolver = ImportResolver::new();
    let path = ImportPath::Absolute(vec!["minecraft", "item", "ItemStack"]);
    
    let resolved = resolver.resolve_import_path("test/module", &path).unwrap();
    assert_eq!(resolved, "minecraft/item/ItemStack");
}

#[test]
fn test_resolve_relative_import() {
    let resolver = ImportResolver::new();
    let path = ImportPath::Relative(vec!["loot", "LootCondition"]);
    
    let resolved = resolver.resolve_import_path("test/module", &path).unwrap();
    assert_eq!(resolved, "test/loot/LootCondition");
}

#[test]
fn test_topological_sort_simple() {
    let mut resolver = ImportResolver::new();
    
    // Module A (no dependencies)
    let file_a = McDocFile {
        imports: vec![],
        declarations: vec![],
    };
    
    // Module B depends on A
    let file_b = McDocFile {
        imports: vec![ImportStatement {
            path: ImportPath::Absolute(vec!["a"]),
            position: Position { line: 1, column: 1, offset: 0 },
        }],
        declarations: vec![],
    };
    
    resolver.add_module("a".to_string(), file_a);
    resolver.add_module("b".to_string(), file_b);
    
    resolver.resolve_all().unwrap();
    
    let order = resolver.get_resolution_order();
    assert_eq!(order, vec!["a", "b"]);
}

#[test]
fn test_circular_dependency_detection() {
    let mut resolver = ImportResolver::new();
    
    // Module A depends on B
    let file_a = McDocFile {
        imports: vec![ImportStatement {
            path: ImportPath::Absolute(vec!["b"]),
            position: Position { line: 1, column: 1, offset: 0 },
        }],
        declarations: vec![],
    };
    
    // Module B depends on A (cycle!)
    let file_b = McDocFile {
        imports: vec![ImportStatement {
            path: ImportPath::Absolute(vec!["a"]),
            position: Position { line: 1, column: 1, offset: 0 },
        }],
        declarations: vec![],
    };
    
    resolver.add_module("a".to_string(), file_a);
    resolver.add_module("b".to_string(), file_b);
    
    let result = resolver.resolve_all();
    assert!(matches!(result, Err(McDocParserError::CircularDependency { .. })));
} 