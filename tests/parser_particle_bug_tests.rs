//! Tests for the specific particle.mcdoc parsing bug
//! Reproduces the exact structure that was failing

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

#[test]
fn test_particle_vibration_structure() {
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
    assert!(result.is_ok(), "Parser should handle particle vibration structure: {:?}", result.err());
    
    // Verify the structure was parsed correctly
    let file = result.unwrap();
    assert_eq!(file.declarations.len(), 2, "Should have 2 struct declarations");
}

#[test]
fn test_simplified_particle_dispatch() {
    let input = r#"
dispatch minecraft:particle[vibration] to struct VibrationParticle {
    value: VibrationParticleData,
}

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
    assert!(result.is_ok(), "Parser should handle particle dispatch with literal constraints: {:?}", result.err());
}

#[test]
fn test_exact_failing_particle_excerpt() {
    // This reproduces the exact lines that were failing in particle.mcdoc
    let input = r#"
struct SafePositionSource {
    type: #[id="position_source_type"] "block",
    pos: [int] @ 3,
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle SafePositionSource with literal type constraint: {:?}", result.err());
    
    // Verify the parsed structure
    let file = result.unwrap();
    assert_eq!(file.declarations.len(), 1, "Should have 1 struct declaration");
}

#[test]
fn test_block_particle_with_literal_constraints() {
    let input = r#"
dispatch minecraft:particle[block, falling_dust, block_marker, dust_pillar] to struct BlockParticle {
    #[until="1.20.5"]
    value: BlockState,
    #[since="1.20.5"]
    block_state: (#[id="block"] string | BlockState),
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexer should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Parser should handle block particle dispatch: {:?}", result.err());
} 