use voxel_rsmcdoc::parser::{Parser, ImportPath, DispatchTarget, TypeExpression, LiteralValue, AnnotationType, AnnotationValue, Declaration, DynamicReferenceType};
use voxel_rsmcdoc::lexer::{Lexer, Position, TokenWithPos, Token};

#[test]
fn test_parse_simple_import() {
    let input = "use ::minecraft::item::ItemStack";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let import = parser.parse_import().unwrap();
    
    match import.path {
        ImportPath::Absolute(segments) => {
            assert_eq!(segments, vec!["minecraft", "item", "ItemStack"]);
        }
        _ => panic!("Expected absolute import"),
    }
}

#[test]
fn test_parse_relative_import() {
    let input = "use super::loot::LootCondition";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let import = parser.parse_import().unwrap();
    
    match import.path {
        ImportPath::Relative(segments) => {
            assert_eq!(segments, vec!["loot", "LootCondition"]);
        }
        _ => panic!("Expected relative import"),
    }
}

#[test]
fn test_parse_simple_struct() {
    let input = "struct Recipe {}";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let struct_decl = parser.parse_struct(Vec::new()).unwrap();
    assert_eq!(struct_decl.name, "Recipe");
}

#[test]
fn test_parse_dispatch_simple() {
    // dispatch minecraft:resource[recipe] to struct {}
    let input = r#"dispatch minecraft:resource[recipe] to struct {}"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_dispatch(Vec::new()).unwrap();
    
    assert_eq!(result.source.registry, "resource");
    assert_eq!(result.targets.len(), 1);
    assert!(matches!(result.targets[0], DispatchTarget::Specific("recipe")));
    assert!(matches!(result.target_type, TypeExpression::Struct(_)));
}

#[test]
fn test_parse_dispatch_multi_target() {
    // dispatch minecraft:recipe_serializer[smelting,blasting,smoking] to Smelting
    let input = r#"dispatch minecraft:recipe_serializer[smelting,blasting,smoking] to Smelting"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_dispatch(Vec::new()).unwrap();
    
    assert_eq!(result.source.registry, "recipe_serializer");
    assert_eq!(result.targets.len(), 3);
    assert!(matches!(result.targets[0], DispatchTarget::Specific("smelting")));
    assert!(matches!(result.targets[1], DispatchTarget::Specific("blasting")));
    assert!(matches!(result.targets[2], DispatchTarget::Specific("smoking")));
}

#[test]
fn test_parse_dispatch_unknown() {
    // dispatch minecraft:recipe_serializer[%unknown] to struct {}
    let input = r#"dispatch minecraft:recipe_serializer[%unknown] to struct {}"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_dispatch(Vec::new()).unwrap();
    
    assert_eq!(result.source.registry, "recipe_serializer");
    assert_eq!(result.targets.len(), 1);
    assert!(matches!(result.targets[0], DispatchTarget::Unknown));
}

#[test]
fn test_parse_enum_with_values() {
    let input = r#"enum(string) GameMode { Creative = "creative", Survival = "survival" }"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_enum(Vec::new()).unwrap();
    
    assert_eq!(result.name, "GameMode");
    assert_eq!(result.base_type, Some("string"));
    assert_eq!(result.variants.len(), 2);
    assert_eq!(result.variants[0].name, "Creative");
    assert!(matches!(result.variants[0].value, Some(LiteralValue::String("creative"))));
}

#[test]  
fn test_parse_type_alias() {
    let input = r#"type ItemStack = struct { item: string, count: int }"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_type_alias(Vec::new()).unwrap();
    
    assert_eq!(result.name, "ItemStack");
    assert!(matches!(result.type_expr, TypeExpression::Struct(_)));
}

#[test]
fn test_parse_simple_annotation() {
    let parser = Parser::new(vec![]);
    let result = parser.parse_annotation_type(r#"id="item""#).unwrap();
    
    match result {
        AnnotationType::Simple { name, value } => {
            assert_eq!(name, "id");
            assert_eq!(value, "item");
        }
        _ => panic!("Expected simple annotation"),
    }
}

#[test]
fn test_parse_complex_annotation() {
    let parser = Parser::new(vec![]);
    let result = parser.parse_annotation_type(r#"id(registry="item", tags="allowed")"#).unwrap();
    
    match result {
        AnnotationType::Complex { name, params } => {
            assert_eq!(name, "id");
            assert_eq!(params.len(), 2);
            
            match params.get("registry").unwrap() {
                AnnotationValue::String(val) => assert_eq!(*val, "item"),
                _ => panic!("Expected string value"),
            }
            
            match params.get("tags").unwrap() {
                AnnotationValue::String(val) => assert_eq!(*val, "allowed"),
                _ => panic!("Expected string value"),
            }
        }
        _ => panic!("Expected complex annotation"),
    }
}

#[test]
fn test_parse_annotation_with_array() {
    let parser = Parser::new(vec![]);
    let result = parser.parse_annotation_type(r#"id(registry="item", exclude=["air", "void"])"#).unwrap();
    
    match result {
        AnnotationType::Complex { name, params } => {
            assert_eq!(name, "id");
            
            match params.get("exclude").unwrap() {
                AnnotationValue::Array(items) => {
                    assert_eq!(items.len(), 2);
                    assert_eq!(items[0], "air");
                    assert_eq!(items[1], "void");
                }
                _ => panic!("Expected array value"),
            }
        }
        _ => panic!("Expected complex annotation"),
    }
}

#[test]
fn test_parse_version_annotations() {
    let parser = Parser::new(vec![]);
    
    let since_result = parser.parse_annotation_type(r#"since="1.20.5""#).unwrap();
    match since_result {
        AnnotationType::Since(version) => assert_eq!(version, "1.20.5"),
        _ => panic!("Expected since annotation"),
    }
    
    let until_result = parser.parse_annotation_type(r#"until="1.19""#).unwrap();
    match until_result {
        AnnotationType::Until(version) => assert_eq!(version, "1.19"),
        _ => panic!("Expected until annotation"),
    }
}

#[test]
fn test_parse_spread_expression() {
    let tokens = vec![
        TokenWithPos { token: Token::DotDotDot, position: Position { line: 1, column: 1, offset: 0 } },
        TokenWithPos { token: Token::Identifier("minecraft"), position: Position { line: 1, column: 4, offset: 3 } },
        TokenWithPos { token: Token::Colon, position: Position { line: 1, column: 13, offset: 12 } },
        TokenWithPos { token: Token::Identifier("recipe_serializer"), position: Position { line: 1, column: 14, offset: 13 } },
        TokenWithPos { token: Token::LeftBracket, position: Position { line: 1, column: 31, offset: 30 } },
        TokenWithPos { token: Token::LeftBracket, position: Position { line: 1, column: 32, offset: 31 } },
        TokenWithPos { token: Token::Type, position: Position { line: 1, column: 33, offset: 32 } },
        TokenWithPos { token: Token::RightBracket, position: Position { line: 1, column: 37, offset: 36 } },
        TokenWithPos { token: Token::RightBracket, position: Position { line: 1, column: 38, offset: 37 } },
    ];
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_single_type().unwrap();
    
    match result {
        TypeExpression::Spread(spread) => {
            assert_eq!(spread.base_path, "minecraft:recipe_serializer");
            assert!(spread.dynamic_key.is_some());
            
            let dynamic_ref = spread.dynamic_key.unwrap();
            match dynamic_ref.reference {
                DynamicReferenceType::Field(field) => assert_eq!(field, "type"),
                _ => panic!("Expected field reference"),
            }
        }
        _ => panic!("Expected spread expression"),
    }
}

#[test]
fn test_parse_dynamic_reference_special_key() {
    let tokens = vec![
        TokenWithPos { token: Token::LeftBracket, position: Position { line: 1, column: 1, offset: 0 } },
        TokenWithPos { token: Token::LeftBracket, position: Position { line: 1, column: 2, offset: 1 } },
        TokenWithPos { token: Token::Percent, position: Position { line: 1, column: 3, offset: 2 } },
        TokenWithPos { token: Token::Identifier("key"), position: Position { line: 1, column: 4, offset: 3 } },
        TokenWithPos { token: Token::RightBracket, position: Position { line: 1, column: 7, offset: 6 } },
        TokenWithPos { token: Token::RightBracket, position: Position { line: 1, column: 8, offset: 7 } },
    ];
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse_single_type().unwrap();
    
    // Should handle dynamic reference in array context
    match result {
        TypeExpression::Reference(_) => {
            // This is correct - dynamic references are handled as special references
        }
        _ => panic!("Expected reference type for dynamic reference"),
    }
}

#[test]
fn test_parse_dispatch_with_dynamic_key() {
    // dispatch minecraft:recipe_serializer[crafting_shaped][[type]] to struct {}
    let input = r#"dispatch minecraft:recipe_serializer[crafting_shaped][[type]] to struct {}"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_dispatch(Vec::new()).unwrap();
    
    assert_eq!(result.source.registry, "recipe_serializer");
    assert_eq!(result.source.key, Some("type"));
    assert_eq!(result.targets.len(), 1);
    assert!(matches!(result.targets[0], DispatchTarget::Specific("crafting_shaped")));
}

#[test]
fn test_parse_dispatch_with_special_key() {
    // dispatch minecraft:effect_component[%unknown][[%key]] to EffectComponent
    let input = r#"dispatch minecraft:effect_component[%unknown][[%key]] to EffectComponent"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse_dispatch(Vec::new()).unwrap();
    
    assert_eq!(result.source.registry, "effect_component");
    assert_eq!(result.source.key, Some("key"));
    assert_eq!(result.targets.len(), 1);
    assert!(matches!(result.targets[0], DispatchTarget::Unknown));
}

#[test]
fn test_parse_complex_struct_with_spread() {
    let input = r#"struct Recipe { type: string, ...minecraft:recipe_serializer[[type]] }"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse().unwrap();
    
    assert_eq!(result.declarations.len(), 1);
    if let Declaration::Struct(struct_decl) = &result.declarations[0] {
        assert_eq!(struct_decl.name, "Recipe");
        assert_eq!(struct_decl.fields.len(), 2);
        
        // First field should be regular field
        assert_eq!(struct_decl.fields[0].name, "type");
        
        // Second field should have spread expression type
        match &struct_decl.fields[1].field_type {
            TypeExpression::Spread(spread) => {
                assert_eq!(spread.base_path, "minecraft:recipe_serializer");
                assert!(spread.dynamic_key.is_some());
            }
            _ => panic!("Expected spread expression in second field"),
        }
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn debug_complex_struct_tokens() {
    let input = r#"struct Recipe { type: string, ...minecraft:recipe_serializer[[type]] }"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    println!("Generated tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("{}: {:?}", i, token);
    }
    
    assert!(tokens.len() > 0);
}

#[test]
fn debug_actual_loot_table_parsing() {
    use std::fs;
    
    let mcdoc_content = fs::read_to_string("examples/mcdoc/data/loot/mod.mcdoc")
        .expect("Failed to read loot table MCDOC");
    
    let mut lexer = Lexer::new(&mcdoc_content);
    let tokens = lexer.tokenize().expect("Failed to tokenize MCDOC");
    
    println!("Total tokens: {}", tokens.len());
    println!("Last 10 tokens:");
    for (i, token) in tokens.iter().rev().take(10).enumerate() {
        println!("  -{}: {:?}", i+1, token);
    }
    
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("✅ Full loot table parsing successful!");
            println!("   - Parsed {} declarations", ast.declarations.len());
        }
        Err(errors) => {
            println!("❌ Full loot table parsing failed with {} errors:", errors.len());
            for (i, error) in errors.iter().enumerate() {
                println!("   Error {}: {:?}", i+1, error);
            }
        }
    }
} 