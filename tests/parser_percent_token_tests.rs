//! Tests for Percent token parsing in MCDOC patterns

use voxel_rsmcdoc::lexer::Lexer;
use voxel_rsmcdoc::parser::Parser;

#[test]
fn test_percent_key_in_dynamic_reference() {
    let input = r#"
struct TestStruct {
    data?: struct {
        [string]: mcdoc:test[[%key]],
    },
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ Test passed: Percent key in dynamic reference parsed successfully"),
        Err(errors) => {
            println!("❌ Test failed with errors:");
            for error in &errors {
                println!("  {:?}", error);
            }
            panic!("Parser should handle [[%key]] pattern");
        }
    }
}

#[test] 
fn test_percent_unknown_in_dispatch() {
    let input = r#"
dispatch mcdoc:marker_data[%unknown] to any
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ Test passed: Percent unknown in dispatch parsed successfully"),
        Err(errors) => {
            println!("❌ Test failed with errors:");
            for error in &errors {
                println!("  {:?}", error);
            }
            panic!("Parser should handle [%unknown] pattern in dispatch");
        }
    }
}

#[test]
fn test_complete_marker_mcdoc() {
    let input = r#"
#[since="1.17"]
dispatch minecraft:entity[marker] to struct Marker {
	...super::EntityBase,
	/// Any stored data
	#[until="1.21.5"]
	data?: struct {
		[#[dispatcher_key="mcdoc:marker_data"] string]: mcdoc:marker_data[[%key]],
	},
}

dispatch mcdoc:marker_data[%unknown] to any
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(ast) => {
            println!("✅ Test passed: Complete marker MCDOC parsed successfully");
            println!("AST: {:#?}", ast);
        },
        Err(errors) => {
            println!("❌ Test failed with errors:");
            for error in &errors {
                println!("  {:?}", error);
            }
            panic!("Parser should handle complete marker.mcdoc file");
        }
    }
}

#[test]
fn test_lexer_percent_token() {
    let input = "mcdoc:test[[%key]]";
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    println!("Tokens generated:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    // Vérifier qu'on a bien les tokens attendus
    assert!(tokens.len() > 5, "Should have enough tokens");
    
    // Chercher le token Percent
    let has_percent = tokens.iter().any(|t| matches!(t.token, voxel_rsmcdoc::lexer::Token::Percent));
    assert!(has_percent, "Should contain Percent token");
} 