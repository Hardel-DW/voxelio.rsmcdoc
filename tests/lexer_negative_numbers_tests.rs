//! Tests for negative numbers in array constraints

use voxel_rsmcdoc::lexer::{Lexer, Token};

#[test]
fn test_negative_number_in_range_constraint() {
    // Reproduit le bug exact de model.mcdoc ligne 16
    let input = "float @ -80..80";
    let mut lexer = Lexer::new(input);
    
    let tokens = lexer.tokenize().expect("Should parse negative numbers in constraints");
    
    let expected_tokens = vec![
        Token::Identifier("float"),
        Token::At,
        Token::Number(-80.0),  // Doit lire -80 comme un nombre négatif
        Token::DotDot,
        Token::Number(80.0),
    ];
    
    for (i, (actual, expected)) in tokens.iter().zip(expected_tokens.iter()).enumerate() {
        assert_eq!(
            std::mem::discriminant(&actual.token), 
            std::mem::discriminant(expected),
            "Token {} mismatch: expected {:?}, got {:?}", i, expected, actual.token
        );
        
        // Vérifie la valeur pour les nombres
        if let (Token::Number(actual_val), Token::Number(expected_val)) = (&actual.token, expected) {
            assert_eq!(actual_val, expected_val, "Number value mismatch at token {}", i);
        }
    }
}

#[test] 
fn test_array_constraint_with_negative_range() {
    // Test la syntaxe complète de la ligne problématique
    let input = "[float @ -80..80] @ 3";
    let mut lexer = Lexer::new(input);
    
    let result = lexer.tokenize();
    assert!(result.is_ok(), "Should parse array with negative range constraint: {:?}", result.err());
    
    let tokens = result.unwrap();
    
    // Debug: afficher tous les tokens pour voir la séquence réelle
    println!("Actual tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token.token);
    }
    
    // Test simplifié : vérifier que -80 est bien un nombre négatif et 80 un nombre positif
    let number_tokens: Vec<f64> = tokens.iter()
        .filter_map(|t| if let Token::Number(n) = t.token { Some(n) } else { None })
        .collect();
    
    assert!(number_tokens.contains(&-80.0), "Should contain -80.0, found: {:?}", number_tokens);
    assert!(number_tokens.contains(&80.0), "Should contain 80.0, found: {:?}", number_tokens);
    assert!(number_tokens.contains(&3.0), "Should contain 3.0, found: {:?}", number_tokens);
}

#[test]
fn test_simple_negative_numbers() {
    let test_cases = vec![
        ("-1", vec![Token::Number(-1.0)]),
        ("-42.5", vec![Token::Number(-42.5)]), 
        ("-0", vec![Token::Number(-0.0)]),
        ("-.5", vec![Token::Number(-0.5)]),
    ];
    
    for (input, expected) in test_cases {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect(&format!("Should parse '{}'", input));
        
        // Skip EOF token pour la comparaison
        let actual_tokens: Vec<_> = tokens.iter()
            .filter(|t| !matches!(t.token, Token::Eof))
            .map(|t| &t.token)
            .collect();
        
        assert_eq!(actual_tokens.len(), expected.len(), "Token count mismatch for '{}'", input);
        
        for (actual, expected) in actual_tokens.iter().zip(expected.iter()) {
            if let (Token::Number(a), Token::Number(b)) = (actual, expected) {
                assert_eq!(a, b, "Number value mismatch for '{}'", input);
            }
        }
    }
}

#[test]
fn test_negative_in_context() {
    // Test différents contextes où les nombres négatifs peuvent apparaître
    let test_cases = vec![
        "value: -100",
        "range @ -50..50", 
        "scale?: [float @ -4..4] @ 3",
        "position: [-16..32]",
    ];
    
    for input in test_cases {
        let mut lexer = Lexer::new(input);
        let result = lexer.tokenize();
        assert!(result.is_ok(), "Should parse '{}': {:?}", input, result.err());
    }
} 