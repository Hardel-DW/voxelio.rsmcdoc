//! Tests spécifiques pour le bug des annotations avec newlines dans union types

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

#[test]
fn test_union_type_with_annotation_on_newline() {
    let input = r#"
type CompositeEntity = (
	EntityPredicate |
	#[since="1.16"]
	[LootCondition] |
)
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize successfully");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    // Ce test devrait passer après le fix
    assert!(result.is_ok(), "Parser should handle union types with annotations on newlines: {:?}", result);
}

#[test]
fn test_simplified_union_with_annotation_newline() {
    let input = r#"
type Test = (
	string |
	#[since="1.16"]
	int |
)
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize successfully");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle simple union with annotation on newline: {:?}", result);
}

#[test]
fn test_annotation_before_array_in_union() {
    let input = r#"
type Test = (
	EntityPredicate |
	#[since="1.16"]
	[LootCondition] |
)
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok());
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle annotations before array types in unions: {:?}", result);
}

#[test]
fn test_multiple_annotations_in_union() {
    let input = r#"
type ComplexUnion = (
	#[deprecated="1.16"] #[until="1.19"]
	LocationPredicate |
	#[since="1.16"]
	struct {
		field: string,
	} |
)
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok());
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle multiple annotations in union types: {:?}", result);
}

#[test]
fn test_struct_inside_union_with_annotations() {
    let input = r#"
type Location = (
	#[deprecated="1.16"] #[until="1.19"]
	LocationPredicate |
	#[since="1.16"]
	struct {
		field?: string,
	} |
)
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok());
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle anonymous structs with annotations in unions: {:?}", result);
}

#[test]
fn test_exact_trigger_mcdoc_pattern() {
    let input = r#"
type CompositeEntity = (
	EntityPredicate |
	#[since="1.16"]
	[LootCondition] |
)

type Location = (
	#[deprecated="1.16"] #[until="1.19"]
	LocationPredicate |
	#[since="1.16"]
	struct {
		field?: string,
		location?: LocationPredicate,
	} |
)
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize successfully");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle the exact pattern from trigger.mcdoc: {:?}", result);
} 