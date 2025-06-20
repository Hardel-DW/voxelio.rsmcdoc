//! Système d'erreurs complet pour le parser MCDOC
//! 
//! Support error recovery et reporting détaillé pour IDE integration.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Erreur principale du parser MCDOC
#[derive(Debug, Clone, PartialEq)]
pub enum McDocParserError {
    // Lexer errors
    UnexpectedCharacter { char: char, line: u32, column: u32 },
    UnterminatedString { line: u32, column: u32 },
    InvalidNumber { value: String, line: u32, column: u32 },
    
    // Parser errors  
    UnexpectedToken { expected: String, found: String, line: u32, column: u32 },
    UnclosedParen { line: u32, column: u32 },
    UnclosedBrace { line: u32, column: u32 },
    InvalidAnnotation { annotation: String, line: u32, column: u32 },
    
    // Resolver errors
    ImportNotFound { path: String },
    CircularDependency { cycle: Vec<String> },
    ModuleNotFound { module: String, from: String },
    
    // Validation errors
    ValidationError { message: String, line: u32, column: u32 },
    InvalidRegistry { path: String, value: String, registry: String },
    VersionConstraint { field: String, required_version: String, current_version: String },
    TypeMismatch { expected: String, found: String, path: String },
    ConstraintViolation { constraint: String, value: String, path: String },
    MissingRequiredField { field: String, path: String },
    
    // Resource ID errors
    InvalidResourceId(String),
    
    // JSON errors
    JsonParseError(String),
    
    // IO errors
    IoError(String),
}

/// Types d'erreurs pour catégorisation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorType {
    LexerError,
    ParseError,
    ResolverError,
    ValidationError,
    JsonError,
    IoError,
}

impl fmt::Display for McDocParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            McDocParserError::UnexpectedCharacter { char, line, column } => {
                write!(f, "Unexpected character '{}' at line {}, column {}", char, line, column)
            }
            McDocParserError::UnterminatedString { line, column } => {
                write!(f, "Unterminated string at line {}, column {}", line, column)
            }
            McDocParserError::InvalidNumber { value, line, column } => {
                write!(f, "Invalid number '{}' at line {}, column {}", value, line, column)
            }
            McDocParserError::UnexpectedToken { expected, found, line, column } => {
                write!(f, "Expected '{}', found '{}' at line {}, column {}", expected, found, line, column)
            }
            McDocParserError::UnclosedParen { line, column } => {
                write!(f, "Unclosed parenthesis at line {}, column {}", line, column)
            }
            McDocParserError::UnclosedBrace { line, column } => {
                write!(f, "Unclosed brace at line {}, column {}", line, column)
            }
            McDocParserError::InvalidAnnotation { annotation, line, column } => {
                write!(f, "Invalid annotation '{}' at line {}, column {}", annotation, line, column)
            }
            McDocParserError::ValidationError { message, line, column } => {
                write!(f, "Validation error at line {}, column {}: {}", line, column, message)
            }
            McDocParserError::ImportNotFound { path } => {
                write!(f, "Import not found: '{}'", path)
            }
            McDocParserError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {}", cycle.join(" -> "))
            }
            McDocParserError::ModuleNotFound { module, from } => {
                write!(f, "Module '{}' not found (imported from '{}')", module, from)
            }
            McDocParserError::InvalidRegistry { path, value, registry } => {
                write!(f, "Invalid registry reference at '{}': '{}' not found in registry '{}'", path, value, registry)
            }
            McDocParserError::VersionConstraint { field, required_version, current_version } => {
                write!(f, "Version constraint violation for field '{}': requires '{}', current version is '{}'", field, required_version, current_version)
            }
            McDocParserError::TypeMismatch { expected, found, path } => {
                write!(f, "Type mismatch at '{}': expected '{}', found '{}'", path, expected, found)
            }
            McDocParserError::ConstraintViolation { constraint, value, path } => {
                write!(f, "Constraint violation at '{}': value '{}' violates constraint '{}'", path, value, constraint)
            }
            McDocParserError::MissingRequiredField { field, path } => {
                write!(f, "Missing required field '{}' at '{}'", field, path)
            }
            McDocParserError::InvalidResourceId(id) => {
                write!(f, "Invalid resource identifier: '{}'", id)
            }
            McDocParserError::JsonParseError(msg) => {
                write!(f, "JSON parse error: {}", msg)
            }
            McDocParserError::IoError(msg) => {
                write!(f, "IO error: {}", msg)
            }
        }
    }
}

impl std::error::Error for McDocParserError {}

impl McDocParserError {
    /// Obtenir le type d'erreur pour catégorisation
    pub fn error_type(&self) -> ErrorType {
        match self {
            McDocParserError::UnexpectedCharacter { .. } |
            McDocParserError::UnterminatedString { .. } |
            McDocParserError::InvalidNumber { .. } => ErrorType::LexerError,
            
            McDocParserError::UnexpectedToken { .. } |
            McDocParserError::UnclosedParen { .. } |
            McDocParserError::UnclosedBrace { .. } |
            McDocParserError::InvalidAnnotation { .. } => ErrorType::ParseError,
            
            McDocParserError::ImportNotFound { .. } |
            McDocParserError::CircularDependency { .. } |
            McDocParserError::ModuleNotFound { .. } => ErrorType::ResolverError,
            
            McDocParserError::ValidationError { .. } |
            McDocParserError::InvalidRegistry { .. } |
            McDocParserError::VersionConstraint { .. } |
            McDocParserError::TypeMismatch { .. } |
            McDocParserError::ConstraintViolation { .. } |
            McDocParserError::MissingRequiredField { .. } => ErrorType::ValidationError,
            
            McDocParserError::JsonParseError(_) => ErrorType::JsonError,
            
            McDocParserError::InvalidResourceId(_) |
            McDocParserError::IoError(_) => ErrorType::IoError,
        }
    }
    
    /// Obtenir la position (ligne, colonne) si disponible
    pub fn position(&self) -> Option<(u32, u32)> {
        match self {
            McDocParserError::UnexpectedCharacter { line, column, .. } |
            McDocParserError::UnterminatedString { line, column } |
            McDocParserError::InvalidNumber { line, column, .. } |
            McDocParserError::UnexpectedToken { line, column, .. } |
            McDocParserError::UnclosedParen { line, column } |
            McDocParserError::UnclosedBrace { line, column } |
            McDocParserError::InvalidAnnotation { line, column, .. } |
            McDocParserError::ValidationError { line, column, .. } => {
                Some((*line, *column))
            }
            _ => None,
        }
    }
} 