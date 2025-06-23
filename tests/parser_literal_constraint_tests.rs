//! Tests for parsing literal type constraints
//! Reproduces bug where `type: #[annotation] "literal_value"` fails to parse

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

#[test]
fn test_literal_type_constraint() {
    let input = r#"
struct TestStruct {
    type: #[id="position_source_type"] "block",
    pos: [int] @ 3,
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle literal type constraints: {:?}", result.err());
}

#[test]
fn test_literal_type_constraint_in_complex_structure() {
    let input = r#"
struct VibrationParticleData {	
    arrival_in_ticks: int,
    destination: SafePositionSource,
}

struct SafePositionSource {
    type: #[id="position_source_type"] "block",
    pos: [int] @ 3,
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle literal constraints in complex structures: {:?}", result.err());
}

#[test]
fn test_various_literal_constraints() {
    let input = r#"
struct TestConstraints {
    type: #[id="test"] "literal_string",
    mode: #[id="mode"] "creative",
    version: #[id="version"] "1.21",
    flag: #[id="flag"] true,
    number: #[id="number"] 42,
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle various literal type constraints: {:?}", result.err());
}

#[test]
fn test_literal_constraint_with_union() {
    let input = r#"
struct TestUnion {
    value: (#[id="item"] string | #[id="block"] "stone"),
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle literal constraints in unions: {:?}", result.err());
} 