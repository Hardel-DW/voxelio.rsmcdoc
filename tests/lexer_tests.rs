use voxel_rsmcdoc::lexer::{Lexer, Token};

#[test]
fn test_basic_tokens() {
    let input = "use struct { } ( ) : :: ;";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    let expected = vec![
        Token::Use,
        Token::Struct,
        Token::LeftBrace,
        Token::RightBrace,
        Token::LeftParen,
        Token::RightParen,
        Token::Colon,
        Token::DoubleColon,
        Token::Semicolon,
        Token::Eof,
    ];
    
    for (actual, expected) in tokens.iter().zip(expected.iter()) {
        assert_eq!(&actual.token, expected);
    }
}

#[test]
fn test_identifiers_and_keywords() {
    let input = "use MyStruct identifier_name true false";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].token, Token::Use);
    assert_eq!(tokens[1].token, Token::Identifier("MyStruct"));
    assert_eq!(tokens[2].token, Token::Identifier("identifier_name"));
    assert_eq!(tokens[3].token, Token::Boolean(true));
    assert_eq!(tokens[4].token, Token::Boolean(false));
}

#[test]
fn test_strings_and_numbers() {
    let input = r#""hello world" 123 45.67 "escaped \" quote""#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].token, Token::String("hello world"));
    assert_eq!(tokens[1].token, Token::Number(123.0));
    assert_eq!(tokens[2].token, Token::Number(45.67));
    assert_eq!(tokens[3].token, Token::String("escaped \\\" quote"));
}

#[test]
fn test_annotations() {
    let input = r#"#[id="item"] #[since="1.20"]"#;
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].token, Token::Annotation(r#"#[id="item"]"#));
    assert_eq!(tokens[1].token, Token::Annotation(r#"#[since="1.20"]"#));
}

#[test]
fn test_comments_are_skipped() {
    let input = "use // line comment\nstruct /* block comment */ MyStruct";
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    
    // Comments should be skipped
    assert_eq!(tokens[0].token, Token::Use);
    assert_eq!(tokens[1].token, Token::Newline);
    assert_eq!(tokens[2].token, Token::Struct);
    assert_eq!(tokens[3].token, Token::Identifier("MyStruct"));
} 