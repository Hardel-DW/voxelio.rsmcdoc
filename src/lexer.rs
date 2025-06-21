//! MCDOC Lexer with zero-copy parsing

use crate::error::ParseError;
use serde::{Deserialize, Serialize};

/// MCDOC Token with zero-copy reference to the source
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'input> {
    Identifier(&'input str),
    String(&'input str),
    Number(f64),
    True,
    False,
    Use,
    Struct,
    Enum,
    Type,
    Dispatch,
    To,
    Super,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    DoubleColon,
    Semicolon,
    Comma,
    Question,
    Pipe,
    At,
    Hash,
    Dot,
    DotDotDot,
    DotDot,
    Percent,
    Equal,
    Equals,
    Less,
    Greater,
    Annotation(&'input str),
    LineComment(&'input str),
    BlockComment(&'input str),
    Eof,
    Newline,
    Whitespace,
}

/// Position in the source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

/// Token with position in the source
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWithPos<'input> {
    pub token: Token<'input>,
    pub position: Position,
}

/// MCDOC Lexer with zero-copy
pub struct Lexer<'input> {
    input: &'input str,
    chars: std::str::Chars<'input>,
    current_pos: Position,
    current_char: Option<char>,
    peek_char: Option<char>,
}

impl<'input> Lexer<'input> {
    /// Create a new lexer
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
    
    /// Advance one character
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
    
    /// Peek at the next character without advancing
    fn peek(&self) -> Option<char> {
        self.peek_char
    }
    
    /// Skip whitespace and comments
    fn skip_whitespace_and_comments(&mut self) -> Result<(), ParseError> {
        while let Some(ch) = self.current_char {
            match ch {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    break;
                }
                '/' if self.peek() == Some('/') => {
                    while self.current_char.is_some() && self.current_char != Some('\n') {
                        self.advance();
                    }
                }
                '/' if self.peek() == Some('*') => {
                    self.advance();
                    self.advance();
                    
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
                        return Err(ParseError::lexer(
                            "Unterminated block comment", 
                            crate::error::SourcePos::new(self.current_pos.line, self.current_pos.column)
                        ));
                    }
                }
                _ => break,
            }
        }
        Ok(())
    }
    
    /// Read an identifier or keyword
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
    
    /// Read a number
    fn read_number(&mut self) -> Result<f64, ParseError> {
        let _start_pos = self.current_pos;
        let start_offset = self.current_pos.offset;
        
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        
        if self.current_char == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            self.advance();
            while let Some(ch) = self.current_char {
                if ch.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }
        
        let number_str = &self.input[start_offset..self.current_pos.offset];
        number_str.parse().map_err(|_| {
            ParseError::lexer(
                format!("Invalid number format: {}", number_str),
                crate::error::SourcePos::new(self.current_pos.line, self.current_pos.column)
            )
        })
    }
    
    /// Read a string literal
    fn read_string(&mut self) -> Result<&'input str, ParseError> {
        let quote_char = self.current_char.unwrap();
        self.advance();
        
        let start_offset = self.current_pos.offset;
        
        while let Some(ch) = self.current_char {
            if ch == quote_char {
                let string_content = &self.input[start_offset..self.current_pos.offset];
                self.advance();
                return Ok(string_content);
            } else if ch == '\\' {
                self.advance();
                if self.current_char.is_some() {
                    self.advance();
                }
            } else {
                self.advance();
            }
        }
        
        Err(ParseError::lexer(
            "Unterminated string literal",
            crate::error::SourcePos::new(self.current_pos.line, self.current_pos.column)
        ))
    }
    
    /// Read a complete annotation #[...]
    fn read_annotation(&mut self) -> Result<&'input str, ParseError> {
        let start_offset = self.current_pos.offset;  // Include the '#'
        self.advance();
        
        if self.current_char != Some('[') {
            return Err(ParseError::lexer(
                "Expected '[' after '#' in annotation",
                crate::error::SourcePos::new(self.current_pos.line, self.current_pos.column)
            ));
        }
        
        self.advance();
        
        let mut bracket_depth = 1;
        while bracket_depth > 0 && self.current_char.is_some() {
            match self.current_char {
                Some('[') => bracket_depth += 1,
                Some(']') => bracket_depth -= 1,
                _ => {}
            }
            self.advance();
        }
        
        if bracket_depth > 0 {
            return Err(ParseError::lexer(
                "Unterminated annotation",
                crate::error::SourcePos::new(self.current_pos.line, self.current_pos.column)
            ));
        }
        
        Ok(&self.input[start_offset..self.current_pos.offset])
    }
    
    /// Determine the token type for an identifier
    fn identifier_to_token(ident: &str) -> Token {
        match ident {
            "use" => Token::Use,
            "struct" => Token::Struct,
            "enum" => Token::Enum,
            "type" => Token::Type,
            "dispatch" => Token::Dispatch,
            "to" => Token::To,
            "super" => Token::Super,
            "true" => Token::True,
            "false" => Token::False,
            _ => Token::Identifier(ident),
        }
    }
    
    /// Get the next token
    pub fn next_token(&mut self) -> Result<TokenWithPos<'input>, ParseError> {
        self.skip_whitespace_and_comments()?;
        
        let pos = self.current_pos;
        
        let token = match self.current_char {
            None => Token::Eof,
            Some('\n') => { self.advance(); Token::Newline }
            Some('(') => { self.advance(); Token::LeftParen }
            Some(')') => { self.advance(); Token::RightParen }
            Some('{') => { self.advance(); Token::LeftBrace }
            Some('}') => { self.advance(); Token::RightBrace }
            Some('[') => { self.advance(); Token::LeftBracket }
            Some(']') => { self.advance(); Token::RightBracket }
            Some(',') => { self.advance(); Token::Comma }
            Some(';') => { self.advance(); Token::Semicolon }
            Some('?') => { self.advance(); Token::Question }
            Some('|') => { self.advance(); Token::Pipe }
            Some('@') => { self.advance(); Token::At }
            Some('%') => { self.advance(); Token::Percent }
            Some('=') => { self.advance(); Token::Equal }
            Some('<') => { self.advance(); Token::Less }
            Some('>') => { self.advance(); Token::Greater }
            Some(':') => {
                self.advance();
                if self.current_char == Some(':') {
                    self.advance();
                    Token::DoubleColon  
                } else {
                    Token::Colon
                }
            }
            Some('.') => {
                self.advance();
                if self.current_char == Some('.') {
                    self.advance();
                    if self.current_char == Some('.') {
                        self.advance();
                        Token::DotDotDot
                    } else {
                        Token::DotDot
                    }
                } else {
                    Token::Dot
                }
            }
            Some('#') => {
                Token::Annotation(self.read_annotation()?)
            }
            Some('"') | Some('\'') => {
                Token::String(self.read_string()?)
            }
            Some(ch) if ch.is_ascii_digit() => {
                Token::Number(self.read_number()?)
            }
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let ident = self.read_identifier();
                Self::identifier_to_token(ident)
            }
            Some(ch) => {
                return Err(ParseError::lexer(
                    format!("Unexpected character: '{}'", ch),
                    crate::error::SourcePos::new(pos.line, pos.column)
                ));
            }
        };
        
        Ok(TokenWithPos { token, position: pos })
    }
    
    /// Tokenize the entire file
    pub fn tokenize(&mut self) -> Result<Vec<TokenWithPos<'input>>, ParseError> {
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