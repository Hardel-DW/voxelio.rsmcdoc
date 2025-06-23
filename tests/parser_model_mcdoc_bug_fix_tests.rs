//! Tests unitaires spécifiques pour le bug model.mcdoc
//! Applique la méthodologie de debugging stricte avec 3 hypothèses distinctes

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

/// HYPOTHÈSE 1: Arrays avec contraintes imbriquées [float @ -80..80] @ 3
#[test]
fn test_hypothesis_1_nested_array_constraints() {
    // Test seulement la partie type problématique
    let input = "[float @ -80..80] @ 3";
    
    println!("🧪 TEST HYPOTHÈSE 1 - Input: {}", input);
    
    // 1. TOKENISATION
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    
    assert!(tokens_result.is_ok(), "❌ Tokenisation failed: {:?}", tokens_result.err());
    let tokens = tokens_result.unwrap();
    
    println!("🔍 Tokens générés:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    // 2. PARSING DU TYPE
    let mut parser = Parser::new(tokens);
    let type_result = parser.parse_type_expression();
    
    println!("🔍 Résultat parsing type: {:?}", type_result);
    
    if let Err(ref error) = type_result {
        println!("❌ Erreur de parsing type:");
        println!("  - {:?}", error);
    }
    
    // Ce test doit réussir une fois corrigé
    assert!(type_result.is_ok(), "❌ Parser failed on nested array constraints type");
    
    // Vérifier la structure AST attendue
    if let Ok(ast) = type_result {
        match ast {
            voxel_rsmcdoc::parser::TypeExpression::Array { element_type, constraints } => {
                println!("✅ Array type parsé correctement");
                println!("  - Element type: {:?}", element_type);
                println!("  - Constraints: {:?}", constraints);
                
                // Vérifier que les contraintes externes sont correctes (@ 3)
                if let Some(constraints) = constraints {
                    assert_eq!(constraints.min, Some(3));
                    assert_eq!(constraints.max, Some(3));
                } else {
                    panic!("❌ Expected array constraints");
                }
            }
            _ => panic!("❌ Expected Array type, got {:?}", ast),
        }
    }
}

/// HYPOTHÈSE 2: Nombres négatifs dans les contraintes
#[test]
fn test_hypothesis_2_negative_numbers_in_constraints() {
    // Test seulement la partie contrainte avec nombres négatifs
    let input = "int @ -80..80";
    
    println!("🧪 TEST HYPOTHÈSE 2 - Input: {}", input);
    
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    
    assert!(tokens_result.is_ok(), "❌ Tokenisation failed: {:?}", tokens_result.err());
    let tokens = tokens_result.unwrap();
    
    println!("🔍 Tokens pour nombres négatifs:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    // Vérifier que -80 est tokenisé comme Number(-80.0) et pas Minus + Number(80.0)
    let negative_number_found = tokens.iter().any(|t| {
        if let voxel_rsmcdoc::lexer::Token::Number(n) = &t.token {
            *n == -80.0
        } else {
            false
        }
    });
    
    assert!(negative_number_found, "❌ Nombre négatif -80 pas trouvé dans les tokens");
    
    let mut parser = Parser::new(tokens);
    let type_result = parser.parse_type_expression();
    
    println!("🔍 Résultat parsing type avec nombres négatifs: {:?}", type_result);
    
    // Ce test doit réussir - vérifie que les contraintes avec nombres négatifs parsent correctement
    assert!(type_result.is_ok(), "❌ Parser failed on negative number constraints");
}

/// HYPOTHÈSE 3: Structure complète du champ model.mcdoc
#[test]
fn test_hypothesis_3_complete_model_field_structure() {
    // Structure complète problématique
    let input = r#"
    struct ModelDisplay {
        [CustomizableItemDisplayContext]: struct ItemTransform {
            rotation?: [float] @ 3,
            translation?: [float @ -80..80] @ 3,
            scale?: [float @ -4..4] @ 3,
        },
    }
    "#;
    
    println!("🧪 TEST HYPOTHÈSE 3 - Structure complète");
    
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    
    assert!(tokens_result.is_ok(), "❌ Tokenisation failed: {:?}", tokens_result.err());
    let tokens = tokens_result.unwrap();
    
    println!("🔍 Nombre total de tokens: {}", tokens.len());
    
    let mut parser = Parser::new(tokens);
    let parse_result = parser.parse();
    
    println!("🔍 Résultat parsing structure complète: {:?}", parse_result);
    
    if let Err(ref errors) = parse_result {
        println!("❌ Erreurs de parsing:");
        for (i, error) in errors.iter().enumerate() {
            println!("  {}: {:?}", i, error);
        }
    }
    
    // Ce test doit réussir une fois corrigé
    assert!(parse_result.is_ok(), "❌ Parser failed on complete model structure");
}

/// TEST DE TRAÇAGE: Tracer ligne par ligne le parsing problématique
#[test]
fn test_trace_parsing_step_by_step() {
    // Cas minimal reproduisant le bug
    let input = "[float @ -80..80] @ 3";
    
    println!("🔍 TRAÇAGE DÉTAILLÉ - Input: {}", input);
    
    // ÉTAPE 1: Tokenisation
    let mut lexer = Lexer::new(input);
    let tokens_result = lexer.tokenize();
    assert!(tokens_result.is_ok(), "Tokenisation failed");
    let tokens = tokens_result.unwrap();
    
    println!("🔍 ÉTAPE 1 - Tokens:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  Token[{}]: {:?} at pos {:?}", i, token.token, token.position);
    }
    
    // ÉTAPE 2: Parsing avec traçage
    let mut parser = Parser::new(tokens);
    
    // Tenter de parser comme type expression
    let type_result = parser.parse_type_expression();
    
    println!("🔍 ÉTAPE 2 - Résultat parse_type_expression: {:?}", type_result);
    
    // Analyser où exactement ça échoue
    if let Err(ref error) = type_result {
        println!("❌ ERREUR DÉTAILLÉE:");
        println!("  Message: {:?}", error);
        if let Some(pos) = error.position() {
            println!("  Position: Ligne {}, Colonne {}", pos.line, pos.column);
        } else {
            println!("  Position: Non disponible");
        }
    }
}

/// TEST DE VALIDATION: Exemple d'array 2D correct
#[test]
fn test_array_2d_syntax_validation() {
    // Test que la syntaxe d'array 2D fonctionne correctement
    let simple_case = "[string] @ 3";
    let complex_case = "[float @ -4..4] @ 3";
    
    for (name, input) in [("simple", simple_case), ("complex", complex_case)] {
        println!("🧪 TEST ARRAY 2D {} - Input: {}", name, input);
        
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().expect("Tokenisation should work");
        
        let mut parser = Parser::new(tokens);
        let result = parser.parse_type_expression();
        
        println!("🔍 Résultat {}: {:?}", name, result);
        
        match result {
            Ok(ast) => println!("✅ {} parsed successfully: {:?}", name, ast),
            Err(error) => {
                println!("❌ {} failed: {:?}", name, error);
                assert!(false, "Array 2D syntax should parse correctly for {}", name);
            }
        }
    }
} 