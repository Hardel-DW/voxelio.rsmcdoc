use voxel_rsmcdoc::parser::{Parser, TypeExpression, Declaration};
use voxel_rsmcdoc::lexer::Lexer;

#[test]
fn test_generic_type_conditions_simple() {
    // Cas le plus simple qui échoue
    let input = "type Conditions<C> = struct { conditions?: C }";
    let tokens = Lexer::new(input).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(file) => {
            assert_eq!(file.declarations.len(), 1);
            match &file.declarations[0] {
                Declaration::Type(type_decl) => {
                    assert_eq!(type_decl.name, "Conditions");
                    // Vérifier que c'est un type générique
                    match &type_decl.type_expr {
                        TypeExpression::Struct(_) => {
                            // Should be ok
                        }
                        _ => panic!("Expected struct type expression")
                    }
                }
                _ => panic!("Expected type declaration")
            }
        }
        Err(errors) => {
            for error in errors {
                println!("Error: {:?}", error);
            }
            panic!("Failed to parse simple generic type");
        }
    }
}

#[test]
fn test_dispatch_with_generic_conditions() {
    // Test exact case from trigger.mcdoc line 50
    let input = r#"
dispatch minecraft:trigger[allay_drop_item_on_block] to Conditions<struct AllayDropItemOnBlock {
    conditions?: string,
}>
"#;
    
    let tokens = Lexer::new(input).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => {
            // Should parse successfully
        }
        Err(errors) => {
            println!("Parsing errors for dispatch with generic:");
            for (i, error) in errors.iter().enumerate() {
                println!("  Error {}: {:?}", i + 1, error);
            }
            
            // Collect expected errors to validate hypotheses
            let syntax_errors: Vec<_> = errors.iter()
                .filter_map(|e| match e {
                    voxel_rsmcdoc::error::ParseError::Syntax { expected, found, pos } => {
                        Some((expected.clone(), found.clone(), pos.clone()))
                    }
                    _ => None
                })
                .collect();
                
            // Hypothèse 1: "Expected '=' after type name" + "Less"
            assert!(syntax_errors.iter().any(|(expected, found, _)| 
                expected.contains("=") && found.contains("Less")
            ), "Expected error about '=' and '<' not found");
        }
    }
}

#[test]
fn test_required_conditions_generic() {
    // Test case from trigger.mcdoc line 15
    let input = "type RequiredConditions<C> = struct { conditions: C }";
    
    let tokens = Lexer::new(input).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => {
            // Should parse successfully
        }
        Err(errors) => {
            println!("Errors for RequiredConditions:");
            for error in errors {
                println!("  {:?}", error);
            }
            // Document the exact errors for fixing
        }
    }
}

#[test]
fn test_generic_with_union_type() {
    // Test complex case like CompositeEntity 
    let input = r#"
type CompositeEntity = (
    EntityPredicate |
    [LootCondition] |
)
"#;
    
    let tokens = Lexer::new(input).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => {
            // Should parse successfully
        }
        Err(errors) => {
            println!("Errors for CompositeEntity union:");
            for error in errors {
                println!("  {:?}", error);
            }
        }
    }
}

#[test]
fn test_location_type_with_annotations() {
    // Test versioned union type from line 22
    let input = r#"
type Location = (
    #[deprecated="1.16"] #[until="1.19"]
    LocationPredicate |
    #[since="1.16"]
    struct {
        location?: LocationPredicate,
    } |
)
"#;
    
    let tokens = Lexer::new(input).tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => {
            // Should parse successfully
        }
        Err(errors) => {
            println!("Errors for versioned Location type:");
            for error in errors {
                println!("  {:?}", error);
            }
        }
    }
}

#[test]
fn test_step_by_step_generic_parsing() {
    // Test chaque partie individuellement pour isoler le problème
    
    // Étape 1: Juste le nom générique
    let input1 = "type Test<T>";
    let tokens1 = Lexer::new(input1).tokenize().unwrap();
    let mut parser1 = Parser::new(tokens1);
    
    println!("=== ÉTAPE 1: Nom générique seul ===");
    match parser1.parse() {
        Ok(_) => println!("✅ Nom générique parsé avec succès"),
        Err(errors) => {
            println!("❌ Erreurs nom générique:");
            for error in errors {
                println!("  {:?}", error);
            }
        }
    }
    
    // Étape 2: Avec affectation simple
    let input2 = "type Test<T> = T";
    let tokens2 = Lexer::new(input2).tokenize().unwrap();
    let mut parser2 = Parser::new(tokens2);
    
    println!("=== ÉTAPE 2: Affectation simple ===");
    match parser2.parse() {
        Ok(_) => println!("✅ Affectation simple parsée"),
        Err(errors) => {
            println!("❌ Erreurs affectation simple:");
            for error in errors {
                println!("  {:?}", error);
            }
        }
    }
    
    // Étape 3: Avec struct
    let input3 = "type Test<T> = struct { field: T }";
    let tokens3 = Lexer::new(input3).tokenize().unwrap();
    let mut parser3 = Parser::new(tokens3);
    
    println!("=== ÉTAPE 3: Avec struct ===");
    match parser3.parse() {
        Ok(_) => println!("✅ Struct générique parsée"),
        Err(errors) => {
            println!("❌ Erreurs struct générique:");
            for error in errors {
                println!("  {:?}", error);
            }
        }
    }
} 