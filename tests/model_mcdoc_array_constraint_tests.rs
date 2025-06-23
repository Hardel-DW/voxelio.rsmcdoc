//! Test unitaire spécifique pour le bug model.mcdoc - contraintes d'array imbriquées
//! Test case: [float @ -80..80] @ 3

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

/// Test du cas exact qui échoue dans model.mcdoc ligne 15 (translation field)
#[test]
fn test_nested_array_constraints_from_model_mcdoc() {
    // Cas exact du model.mcdoc qui échoue
    let input = "[float @ -80..80] @ 3";
    
    println!("🧪 TEST MODEL.MCDOC - Input: {}", input);
    
    // 1. TOKENISATION 
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    
    assert!(tokens_result.is_ok(), "❌ Tokenisation failed: {:?}", tokens_result.err());
    let tokens = tokens_result.unwrap();
    
    println!("🔍 Tokens générés:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    // 2. PARSING
    let mut parser = Parser::new(tokens);
    let type_result = parser.parse_type_expression();
    
    println!("🔍 Résultat parsing: {:?}", type_result);
    
    if let Err(ref error) = type_result {
        println!("❌ Erreur détaillée:");
        println!("  - {:?}", error);
        if let Some(pos) = error.position() {
            println!("  - Position: Ligne {}, Colonne {}", pos.line, pos.column);
        }
    }
    
    // Pour l'instant, on s'attend à ce que le parsing réussisse même si les contraintes internes sont ignorées
    // TODO: Une fois les contraintes internes implémentées, il faudra vérifier leur structure
    assert!(type_result.is_ok(), "❌ Parsing failed on nested array constraints");
    
    // Vérifier structure AST basique - array avec contraintes externes
    if let Ok(ast) = type_result {
        match ast {
            voxel_rsmcdoc::parser::TypeExpression::Array { element_type, constraints } => {
                println!("✅ Array type parsé avec succès");
                println!("  - Element type: {:?}", element_type);
                println!("  - Constraints: {:?}", constraints);
                
                // Vérifier contraintes externes (@ 3)
                if let Some(constraints) = constraints {
                    assert_eq!(constraints.min, Some(3));
                    assert_eq!(constraints.max, Some(3));
                } else {
                    panic!("❌ Expected array constraints @ 3");
                }
                
                // Maintenant les contraintes internes devraient être parsées correctement
                match *element_type {
                    voxel_rsmcdoc::parser::TypeExpression::Constrained { base_type, constraints } => {
                        println!("✅ Contraintes internes correctement parsées!");
                        println!("  - Type de base: {:?}", base_type);
                        println!("  - Contraintes: min={:?}, max={:?}", constraints.min, constraints.max);
                        
                        // Vérifier les contraintes internes (-80..80)
                        assert_eq!(constraints.min, Some(-80.0));
                        assert_eq!(constraints.max, Some(80.0));
                        
                        // Vérifier le type de base (float)
                        match *base_type {
                            voxel_rsmcdoc::parser::TypeExpression::Simple(type_name) => {
                                assert_eq!(type_name, "float");
                            }
                            _ => panic!("❌ Expected Simple('float') as base type"),
                        }
                    }
                    voxel_rsmcdoc::parser::TypeExpression::Simple(type_name) => {
                        panic!("❌ Expected Constrained type but got Simple({}). Constraints not implemented?", type_name);
                    }
                    _ => panic!("❌ Unexpected element type structure: {:?}", element_type),
                }
            }
            _ => panic!("❌ Expected Array type, got {:?}", ast),
        }
    }
}

/// Test des variations de contraintes négatives
#[test]
fn test_negative_number_constraints_variations() {
    let test_cases = [
        ("int @ -100..100", "Simple negative constraint"),
        ("float @ -80.5..80.5", "Float negative constraint"),
        ("[int @ -10..10] @ 5", "Array with negative constraint"),
        ("(float @ -45..45 | int)", "Union with negative constraint"),
    ];
    
    for (input, description) in test_cases {
        println!("🧪 TEST: {} - {}", description, input);
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("Tokenisation should work");
        
        let mut parser = Parser::new(tokens);
        let result = parser.parse_type_expression();
        
        if let Err(ref error) = result {
            println!("❌ Failed {}: {:?}", description, error);
        }
        
        // Pour l'instant, certains peuvent échouer car les contraintes de types simples ne sont pas gérées
        // On se concentre sur le cas d'array qui était le problème principal  
        if input.contains('[') {
            assert!(result.is_ok(), "❌ {} should parse correctly", description);
        } else {
            println!("⚠️ {} pas encore supporté (contraintes simples)", description);
        }
    }
}

/// Test du contexte complet model.mcdoc struct field
#[test] 
fn test_model_mcdoc_struct_field_context() {
    let input = r#"
    struct ItemTransform {
        rotation?: [float] @ 3,
        translation?: [float @ -80..80] @ 3,
        scale?: [float @ -4..4] @ 3,
    }
    "#;
    
    println!("🧪 TEST STRUCT COMPLET - Model.mcdoc context");
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Tokenisation should work");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    println!("🔍 Résultat struct complet: {:?}", result);
    
    if let Err(ref errors) = result {
        println!("❌ Erreurs de parsing:");
        for (i, error) in errors.iter().enumerate() {
            println!("  {}: {:?}", i, error);
        }
    }
    
    assert!(result.is_ok(), "❌ Struct ItemTransform should parse correctly");
} 