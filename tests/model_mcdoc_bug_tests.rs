//! Test for the specific model.mcdoc bug reported by user

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

#[test]
fn test_model_mcdoc_line16_translation_field() {
    // Ligne 16 exacte du model.mcdoc qui causait l'erreur
    let input = "translation?: [float @ -80..80] @ 3,";
    
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    
    assert!(tokens_result.is_ok(), 
        "Should tokenize model.mcdoc line 16 without error: {:?}", 
        tokens_result.err()
    );
    
    let tokens = tokens_result.unwrap();
    
    // Vérifier que les nombres négatifs sont correctement tokenisés
    let mut found_negative_80 = false;
    let mut found_positive_80 = false;
    
    for token in &tokens {
        if let voxel_rsmcdoc::lexer::Token::Number(n) = &token.token {
            if *n == -80.0 {
                found_negative_80 = true;
            } else if *n == 80.0 {
                found_positive_80 = true;
            }
        }
    }
    
    assert!(found_negative_80, "Should find -80.0 in tokens");
    assert!(found_positive_80, "Should find 80.0 in tokens");
}

#[test]
fn test_model_mcdoc_struct_field_parsing() {
    // Test simplifié du champ problématique seulement
    let input = "translation?: [float @ -80..80] @ 3";
    
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    assert!(tokens_result.is_ok(), "Should tokenize translation field: {:?}", tokens_result.err());
    
    // Le test important est que le lexer n'échoue plus
    let tokens = tokens_result.unwrap();
    
    // Vérifier qu'on a les nombres attendus
    let numbers: Vec<f64> = tokens.iter()
        .filter_map(|t| if let voxel_rsmcdoc::lexer::Token::Number(n) = &t.token { Some(*n) } else { None })
        .collect();
    
    assert!(numbers.contains(&-80.0), "Should contain -80.0");
    assert!(numbers.contains(&80.0), "Should contain 80.0");
    assert!(numbers.contains(&3.0), "Should contain 3.0");
}

#[test]
fn test_model_element_rotation_angle() {
    // Test d'une autre partie de model.mcdoc avec des nombres négatifs
    let input = r#"
    angle: (
        (-45.0 | -22.5 | 0.0 | 22.5 | 45.0) |
        float @ -45..45 |
    ),
    "#;
    
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    
    assert!(tokens_result.is_ok(), 
        "Should parse rotation angle with negative values: {:?}", 
        tokens_result.err()
    );
    
    let tokens = tokens_result.unwrap();
    
    // Vérifier que tous les nombres négatifs sont correctement parsés
    let negative_numbers: Vec<f64> = tokens.iter()
        .filter_map(|t| {
            if let voxel_rsmcdoc::lexer::Token::Number(n) = &t.token {
                if *n < 0.0 { Some(*n) } else { None }
            } else { None }
        })
        .collect();
    
    assert!(negative_numbers.contains(&-45.0), "Should contain -45.0: {:?}", negative_numbers);
    assert!(negative_numbers.contains(&-22.5), "Should contain -22.5: {:?}", negative_numbers);
    
    // Il devrait y avoir au moins ces nombres négatifs
    assert!(negative_numbers.len() >= 2, "Should have at least 2 negative numbers: {:?}", negative_numbers);
} 