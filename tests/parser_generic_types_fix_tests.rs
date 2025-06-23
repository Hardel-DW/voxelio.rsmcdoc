//! Tests pour reproduire et corriger les erreurs de parsing des types génériques
//! Issues observées dans equipment.mcdoc avec Layer<T> et WingsLayer<T>

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

#[test]
fn test_generic_type_declaration() {
    let input = r#"
type Layer<T> = struct {
    texture: T,
    dyeable?: Dyeable,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(file) => {
            assert_eq!(file.declarations.len(), 1, "Should parse 1 type declaration");
        },
        Err(errors) => {
            panic!("Should not fail to parse generic type declaration. Errors: {:?}", errors);
        }
    }
}

#[test]
fn test_generic_type_usage_in_field() {
    let input = r#"
struct Layers {
    humanoid?: [Layer<string>],
    wings?: [WingsLayer<string>],
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(file) => {
            assert_eq!(file.declarations.len(), 1, "Should parse 1 struct declaration");
        },
        Err(errors) => {
            panic!("Should not fail to parse generic type usage. Errors: {:?}", errors);
        }
    }
}

#[test]
fn test_complex_generic_with_annotations() {
    let input = r#"
struct Layers {
    humanoid?: [Layer<#[id(registry="texture",path="entity/equipment/humanoid/")] string>],
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(file) => {
            assert_eq!(file.declarations.len(), 1, "Should parse 1 struct declaration");
        },
        Err(errors) => {
            panic!("Should not fail to parse complex generic with annotations. Errors: {:?}", errors);
        }
    }
}

#[test]
fn test_equipment_mcdoc_simplified() {
    let input = r#"
type Layer<T> = struct {
    texture: T,
    dyeable?: Dyeable,
}

struct Dyeable {
    color_when_undyed?: RGB,
}

type WingsLayer<T> = struct {
    ...Layer<T>,
    use_player_texture?: boolean,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(file) => {
            assert_eq!(file.declarations.len(), 3, "Should parse 3 declarations");
        },
        Err(errors) => {
            panic!("Should not fail to parse equipment-like structure. Errors: {:?}", errors);
        }
    }
}

#[test]
fn test_spread_with_generic_type() {
    let input = r#"
type WingsLayer<T> = struct {
    ...Layer<T>,
    use_player_texture?: boolean,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(file) => {
            assert_eq!(file.declarations.len(), 1, "Should parse 1 type declaration");
        },
        Err(errors) => {
            panic!("Should not fail with spread generics after fix. Errors: {:?}", errors);
        }
    }
} 