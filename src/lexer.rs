//! Lexer MCDOC avec zero-copy parsing
//! 
//! Tokenise les fichiers MCDOC sans allocation grâce aux lifetimes Rust.

use crate::error::McDocParserError;

/// Token MCDOC avec référence zero-copy au source
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'input> {
    // Identifiers et littéraux
    Identifier(&'input str),
    String(&'input str),           // "hello"
    Number(f64),                   // 123, 123.456
    Boolean(bool),                 // true, false
    
    // Mots-clés
    Use,                          // use
    Struct,                       // struct  
    Enum,                         // enum
    Type,                         // type
    Dispatch,                     // dispatch
    To,                           // to
    Super,                        // super
    
    // Symboles
    LeftParen,                    // (
    RightParen,                   // )
    LeftBrace,                    // {
    RightBrace,                   // }
    LeftBracket,                  // [
    RightBracket,                 // ]
    
    // Operators
    Colon,                        // :
    DoubleColon,                  // ::
    Semicolon,                    // ;
    Comma,                        // ,
    Question,                     // ?
    Pipe,                         // |
    At,                           // @
    Hash,                         // #
    Dot,                          // .
    DotDotDot,                    // ...
    DotDot,                       // ..
    Percent,                      // %
    Equals,                       // =
    LeftAngle,                    // <
    RightAngle,                   // >
    
    // Annotations
    Annotation(&'input str),       // #[id="item"]
    
    // Comments (à ignorer généralement)
    LineComment(&'input str),      // //
    BlockComment(&'input str),     // /* */
    
    // Special
    Eof,
    Newline,
}

/// Position dans le fichier source
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

/// Token avec position dans le source
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos<'input> {
    pub token: Token<'input>,
    pub position: Position,
}

/// Lexer MCDOC avec zero-copy
pub struct Lexer<'input> {
    input: &'input str,
    chars: std::str::Chars<'input>,
    current_pos: Position,
    current_char: Option<char>,
    peek_char: Option<char>,
}

impl<'input> Lexer<'input> {
    /// Créer un nouveau lexer
    pub fn new(input: &'input str) -> Self {
        let mut chars = input.chars();
        let current_char = chars.next();
        let peek_char = chars.next();
        
        Self {
            input,
            chars,
            current_pos: Position { line: 1, column: 1, offset: 0 },
            current_char,
            peek_char,
        }
    }
    
    /// Avancer d'un caractère
    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            self.current_pos.offset += ch.len_utf8();
            
            if ch == '\n' {
                self.current_pos.line += 1;
                self.current_pos.column = 1;
            } else {
                self.current_pos.column += 1;
            }
        }
        
        self.current_char = self.peek_char;
        self.peek_char = self.chars.next();
    }
    
    /// Regarder le caractère suivant sans avancer
    fn peek(&self) -> Option<char> {
        self.peek_char
    }
    
    /// Ignorer les espaces et les commentaires
    fn skip_whitespace_and_comments(&mut self) -> Result<(), McDocParserError> {
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    // Don't skip newlines - they're significant tokens
                    break;
                }
                '/' if self.peek() == Some('/') => {
                    // Skip line comment
                    while self.current_char.is_some() && self.current_char != Some('\n') {
                        self.advance();
                    }
                    // Don't skip the newline - it will be handled as a token
                }
                '/' if self.peek() == Some('*') => {
                    // Skip block comment
                    self.advance(); // Skip '/'
                    self.advance(); // Skip '*'
                    
                    let mut depth = 1;
                    while depth > 0 && self.current_char.is_some() {
                        if self.current_char == Some('/') && self.peek() == Some('*') {
                            depth += 1;
                            self.advance();
                            self.advance();
                        } else if self.current_char == Some('*') && self.peek() == Some('/') {
                            depth -= 1;
                            self.advance();
                            self.advance();
                        } else {
                            self.advance();
                        }
                    }
                    
                    if depth > 0 {
                        return Err(McDocParserError::UnterminatedString {
                            line: self.current_pos.line,
                            column: self.current_pos.column,
                        });
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }
    
    /// Lire un identifiant ou mot-clé
    fn read_identifier(&mut self) -> &'input str {
        let start_offset = self.current_pos.offset;
        
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        
        &self.input[start_offset..self.current_pos.offset]
    }
    
    /// Lire un nombre
    fn read_number(&mut self) -> Result<f64, McDocParserError> {
        let start_offset = self.current_pos.offset;
        let start_pos = self.current_pos;
        
        // Lire la partie entière
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        
        // Lire la partie décimale si présente
        if self.current_char == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance(); // Skip '.'
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        let number_str = &self.input[start_offset..self.current_pos.offset];
        number_str.parse().map_err(|_| McDocParserError::InvalidNumber {
            value: number_str.to_string(),
            line: start_pos.line,
            column: start_pos.column,
        })
    }
    
    /// Lire une chaîne de caractères
    fn read_string(&mut self) -> Result<&'input str, McDocParserError> {
        let start_pos = self.current_pos;
        self.advance(); // Skip opening quote
        
        let content_start = self.current_pos.offset;
        
        while let Some(ch) = self.current_char {
            match ch {
                '"' => {
                    let content = &self.input[content_start..self.current_pos.offset];
                    self.advance(); // Skip closing quote
                    return Ok(content);
                }
                '\\' => {
                    self.advance(); // Skip backslash
                    if self.current_char.is_some() {
                        self.advance(); // Skip escaped character
                    }
                }
                '\n' => {
                    return Err(McDocParserError::UnterminatedString {
                        line: start_pos.line,
                        column: start_pos.column,
                    });
                }
                _ => self.advance(),
            }
        }
        
        Err(McDocParserError::UnterminatedString {
            line: start_pos.line,
            column: start_pos.column,
        })
    }
    
    /// Lire une annotation complète #[...]
    fn read_annotation(&mut self) -> Result<&'input str, McDocParserError> {
        let start_offset = self.current_pos.offset;
        let start_pos = self.current_pos;
        
        self.advance(); // Skip '#'
        
        if self.current_char != Some('[') {
            return Err(McDocParserError::InvalidAnnotation {
                annotation: "#".to_string(),
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        let mut depth = 0;
        while let Some(ch) = self.current_char {
            match ch {
                '[' => {
                    depth += 1;
                    self.advance();
                }
                ']' => {
                    self.advance();
                    depth -= 1;
                    if depth == 0 {
                        return Ok(&self.input[start_offset..self.current_pos.offset]);
                    }
                }
                _ => self.advance(),
            }
        }
        
        Err(McDocParserError::InvalidAnnotation {
            annotation: self.input[start_offset..self.current_pos.offset].to_string(),
            line: start_pos.line,
            column: start_pos.column,
        })
    }
    
    /// Déterminer le type de token pour un identifiant
    fn identifier_to_token(ident: &str) -> Token {
        match ident {
            "use" => Token::Use,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "type" => Token::Type,
            "dispatch" => Token::Dispatch,
            "to" => Token::To,
            "super" => Token::Super,
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            _ => Token::Identifier(ident),
        }
    }
    
    /// Obtenir le prochain token
    pub fn next_token(&mut self) -> Result<TokenWithPos<'input>, McDocParserError> {
        self.skip_whitespace_and_comments()?;
        
        let pos = self.current_pos;
            
            let token = match self.current_char {
                None => Token::Eof,
                Some('\n') => {
                    self.advance();
                    Token::Newline
                }
                Some('(') => {
                    self.advance();
                    Token::LeftParen
                }
                Some(')') => {
                    self.advance();
                    Token::RightParen
                }
                Some('{') => {
                    self.advance();
                    Token::LeftBrace
                }
                Some('}') => {
                    self.advance();
                    Token::RightBrace
                }
                Some('[') => {
                    self.advance();
                    Token::LeftBracket
                }
                Some(']') => {
                    self.advance();
                    Token::RightBracket
                }
                Some(':') if self.peek() == Some(':') => {
                    self.advance();
                    self.advance();
                    Token::DoubleColon
                }
                Some(':') => {
                    self.advance();
                    Token::Colon
                }
                Some(';') => {
                    self.advance();
                    Token::Semicolon
                }
                Some(',') => {
                    self.advance();
                    Token::Comma
                }
                Some('?') => {
                    self.advance();
                    Token::Question
                }
                Some('|') => {
                    self.advance();
                    Token::Pipe
                }
                Some('@') => {
                    self.advance();
                    Token::At
                }
                Some('%') => {
                    self.advance();
                    Token::Percent
                }
                Some('=') => {
                    self.advance();
                    Token::Equals
                }
                Some('#') => {
                    let annotation = self.read_annotation()?;
                    Token::Annotation(annotation)
                }
                Some('.') if self.peek() == Some('.') => {
                    self.advance();
                    self.advance();
                    if self.current_char == Some('.') {
                        self.advance();
                        Token::DotDotDot
                    } else {
                        Token::DotDot
                    }
                }
                Some('.') => {
                    self.advance();
                    Token::Dot
                }
                Some('"') => {
                    let content = self.read_string()?;
                    Token::String(content)
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let number = self.read_number()?;
                    Token::Number(number)
                }
                Some('<') => {
                    self.advance();
                    Token::LeftAngle
                }
                Some('>') => {
                    self.advance();
                    Token::RightAngle
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let ident = self.read_identifier();
                    Self::identifier_to_token(ident)
                }
                Some(ch) => {
                    return Err(McDocParserError::UnexpectedCharacter {
                        char: ch,
                        line: pos.line,
                        column: pos.column,
                    });
                }
            };
            
            Ok(TokenWithPos { token, position: pos })
    }
    
    /// Tokeniser tout le fichier
    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPos<'input>>, McDocParserError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            let is_eof = matches!(token.token, Token::Eof);
            tokens.push(token);
            
            if is_eof {
                break;
            }
        }
        
        Ok(tokens)
    }
} 