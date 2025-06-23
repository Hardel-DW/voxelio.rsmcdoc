use voxel_rsmcdoc::parser::Parser;
use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::error::ParseError;

#[cfg(test)]
mod parser_gametest_bug_tests {
    use super::*;

    // Test avec le VRAI contenu du fichier gametest qui cause problème
    #[test]
    fn test_real_gametest_problematic_lines() {
        let input = r#"
dispatch minecraft:resource[test_instance] to struct TestInstance {
	type: #[id="test_instance_type"] string,
	...minecraft:test_instance[[type]],
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // DEBUG: Afficher les tokens pour voir ce qui est parsé
        println!("🔍 TOKENS:");
        for (i, token) in tokens.iter().enumerate() {
            if i >= 15 && i <= 25 {  // Autour de la ligne problématique
                println!("  {}: {:?}", i, token);
            }
        }
        
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        match result {
            Ok(ast) => {
                println!("✅ Parsing réussi: {:?}", ast);
                // Vérifier que le spread est bien parsé avec la dynamic key
                assert!(format!("{:?}", ast).contains("dynamic_key: Some"));
                assert!(format!("{:?}", ast).contains("namespace: \"minecraft\""));
                assert!(format!("{:?}", ast).contains("registry: \"test_instance\""));
            }
            Err(errors) => {
                println!("❌ Erreurs trouvées:");
                for error in &errors {
                    println!("  - {:?}", error);
                }
                panic!("Parsing should now succeed after fix, but got errors: {:?}", errors);
            }
        }
    }

    // Test avec ligne 23 qui pose problème: max_ticks: int @ 1..,
    #[test]
    fn test_line_23_debug() {
        let input = r#"
struct TestData {
	max_ticks: int @ 1..,
	setup_ticks?: int @ 0..,
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        
        // DEBUG: Afficher tous les tokens
        println!("🔍 ALL TOKENS for line 23 issue:");
        for (i, token) in tokens.iter().enumerate() {
            println!("  {}: {:?}", i, token);
        }
        
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        
        match result {
            Ok(ast) => println!("✅ Line 23 parsing succeeded: {:?}", ast),
            Err(errors) => {
                println!("❌ Line 23 errors:");
                for error in &errors {
                    println!("  - {:?}", error);
                }
            }
        }
    }

    // Test H1: Parser ne gère pas les spreads avec namespace
    #[test]
    fn test_spread_with_namespace() {
        let input = r#"
struct Test {
    ...minecraft:test_instance[[type]],
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let result = parser.parse();
        match result {
            Ok(_) => println!("✅ Spread with namespace works"),
            Err(errors) => {
                println!("❌ Spread errors: {:?}", errors);
                // Vérifie qu'on a bien l'erreur attendue
                assert!(errors.iter().any(|e| matches!(e, ParseError::Syntax { expected, found, .. } 
                    if expected.contains("identifier") && found.contains("Colon"))));
            }
        }
    }

    // Test H2: Parser ne gère pas int @ constraints
    #[test]
    fn test_int_with_at_constraint() {
        let input = r#"
struct Test {
    max_ticks: int @ 1..,
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let result = parser.parse();
        match result {
            Ok(_) => println!("✅ int @ constraint works"),
            Err(errors) => {
                println!("❌ int @ errors: {:?}", errors);
                // Vérifie qu'on a bien l'erreur "expected identifier, found At"
                assert!(errors.iter().any(|e| matches!(e, ParseError::Syntax { expected, found, .. } 
                    if expected.contains("identifier") && found.contains("At"))));
            }
        }
    }

    // Test H3: Parser avec commentaires triple slash
    #[test]
    fn test_triple_slash_comments() {
        let input = r#"
struct Test {
    /// This is a doc comment
    field: string,
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let result = parser.parse();
        // Ce test doit passer - les commentaires doivent être ignorés
        assert!(result.is_ok(), "Triple slash comments should be parsed correctly");
    }

    // Test H4: Cas simplifié du problème gametest
    #[test]
    fn test_simplified_gametest_structure() {
        let input = r#"
dispatch minecraft:test_instance[function] to struct FunctionTestInstance {
    environment: string,
    max_ticks: int,
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let result = parser.parse();
        assert!(result.is_ok(), "Simplified gametest structure should parse correctly: {:?}", result);
    }

    // Test H5: Le vrai fichier gametest complet (version minimale)
    #[test]
    fn test_real_gametest_minimal() {
        let input = r#"
dispatch minecraft:resource[test_instance] to struct TestInstance {
    type: string,
}

struct TestData {
    max_ticks: int,
    setup_ticks: int,
}
"#;
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        
        let result = parser.parse();
        assert!(result.is_ok(), "Minimal gametest should parse: {:?}", result);
    }
} 