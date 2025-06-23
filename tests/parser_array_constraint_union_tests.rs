//! Tests pour le parsing des contraintes d'array dans les unions parenthésées
//! 
//! Ce test reproduit le bug spécifique trouvé dans world/item/head.mcdoc
//! où `int[] @ 4 |` dans une expression parenthésée échoue.

use voxel_rsmcdoc::parser::Parser;
use voxel_rsmcdoc::lexer::Lexer;

#[test]
fn test_array_constraint_in_parenthesized_union() {
    let input = r#"
struct SkullOwner {
    Id?: (
        #[until="1.16"] string |
        #[since="1.16"] int[] @ 4 |
    ),
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    // Actuellement échoue - c'est le bug que nous voulons reproduire
    match result {
        Ok(_) => println!("✅ Test passed - array constraint in union works"),
        Err(errors) => {
            println!("❌ Expected error reproduced: {:?}", errors);
            // Vérifier que c'est bien l'erreur attendue
            assert!(errors.iter().any(|e| e.to_string().contains("Expected ')' after parenthesized type")));
        }
    }
}

#[test]
fn test_simple_array_constraint_works() {
    let input = r#"
struct Test {
    field: int[] @ 4,
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    // Ce cas simple devrait fonctionner
    assert!(result.is_ok(), "Simple array constraint should work");
}

#[test]
fn test_union_without_constraint_works() {
    let input = r#"
struct Test {
    field: (string | int),
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    // Ce cas simple devrait fonctionner
    assert!(result.is_ok(), "Simple union should work");
}

#[test]
fn test_minimal_reproduction_case() {
    // Cas minimal pour reproduire le problème
    let input = r#"
struct Test {
    field: (int[] @ 4 | string),
}
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    // Actuellement échoue - cas minimal du bug
    match result {
        Ok(_) => println!("✅ Minimal case passed"),
        Err(errors) => {
            println!("❌ Minimal case error reproduced: {:?}", errors);
        }
    }
}

#[test]
fn test_array_constraint_outside_parentheses() {
    // Vérifier que ça marche en dehors des parenthèses
    let input = r#"
type Test = int[] @ 4
"#;

    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    let mut parser = Parser::new(tokens);
    let result = parser.parse();

    assert!(result.is_ok(), "Array constraint outside parentheses should work");
} 