//! Système d'erreurs MCDOC

use serde::{Deserialize, Serialize};
use std::fmt;

/// Position dans le source code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePos {
    pub line: u32,
    pub column: u32,
}

impl SourcePos {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

/// Erreur principale du parser MCDOC - Version consolidée
#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    /// Erreurs lexicales
    Lexer { 
        message: String, 
        pos: SourcePos,
    },
    
    /// Erreurs de parsing
    Syntax { 
        expected: String, 
        found: String, 
        pos: SourcePos,
    },
    
    /// Erreurs de résolution de modules
    Resolution { 
        message: String,
        path: Option<String>,
    },
    
    /// Erreurs de validation
    Validation { 
        message: String,
        path: String,
        pos: Option<SourcePos>,
    },
    
    /// Erreurs generiques avec context
    Context {
        message: String,
        context: String,
        pos: Option<SourcePos>,
    },
    
    /// Erreur d'ID de ressource invalide
    InvalidResourceId(String),
    
    /// Module not found
    ModuleNotFound {
        module: String,
        from: String,
    },
    
    /// Circular dependency
    CircularDependency {
        cycle: Vec<String>,
    },
    

}

/// Alias pour compatibilité avec validator.rs et resolver.rs
pub type McDocParserError = ParseError;

/// Types d'erreurs pour catégorisation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorType {
    Lexer,
    Syntax, 
    Resolution,
    Validation,
    Context,
    InvalidResourceId,
    ModuleNotFound,
    CircularDependency
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Lexer { message, pos } => {
                write!(f, "{} at {}:{}", message, pos.line, pos.column)
            }
            ParseError::Syntax { expected, found, pos } => {
                write!(f, "Expected '{}', found '{}' at {}:{}", expected, found, pos.line, pos.column)
            }
            ParseError::Resolution { message, path } => {
                match path {
                    Some(p) => write!(f, "{} (path: {})", message, p),
                    None => write!(f, "{}", message),
                }
            }
            ParseError::Validation { message, path, pos } => {
                match pos {
                    Some(p) => write!(f, "{} at '{}' ({}:{})", message, path, p.line, p.column),
                    None => write!(f, "{} at '{}'", message, path),
                }
            }
            ParseError::Context { message, context, pos } => {
                match pos {
                    Some(p) => write!(f, "{} in {} ({}:{})", message, context, p.line, p.column),
                    None => write!(f, "{} in {}", message, context),
                }
            }
            ParseError::InvalidResourceId(id) => {
                write!(f, "Invalid resource identifier: '{}'", id)
            }
            ParseError::ModuleNotFound { module, from } => {
                write!(f, "Module not found: {} from {}", module, from)
            }
            ParseError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {:?}", cycle)
            }

        }
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Factory methods pour création rapide d'erreurs
    pub fn lexer(message: impl Into<String>, pos: SourcePos) -> Self {
        Self::Lexer { message: message.into(), pos }
    }
    
    pub fn syntax(expected: impl Into<String>, found: impl Into<String>, pos: SourcePos) -> Self {
        Self::Syntax { 
            expected: expected.into(), 
            found: found.into(), 
            pos 
        }
    }
    
    pub fn resolution(message: impl Into<String>, path: Option<String>) -> Self {
        Self::Resolution { message: message.into(), path }
    }
    
    pub fn validation(message: impl Into<String>, path: impl Into<String>) -> Self {
        Self::Validation { 
            message: message.into(), 
            path: path.into(), 
            pos: None 
        }
    }
    
    pub fn validation_at(message: impl Into<String>, path: impl Into<String>, pos: SourcePos) -> Self {
        Self::Validation { 
            message: message.into(), 
            path: path.into(), 
            pos: Some(pos) 
        }
    }
    
    /// Obtenir le type d'erreur
    pub fn error_type(&self) -> ErrorType {
        match self {
            ParseError::Lexer { .. } => ErrorType::Lexer,
            ParseError::Syntax { .. } => ErrorType::Syntax,
            ParseError::Resolution { .. } => ErrorType::Resolution,
            ParseError::Validation { .. } => ErrorType::Validation,
            ParseError::Context { .. } => ErrorType::Context,
            ParseError::InvalidResourceId(_) => ErrorType::InvalidResourceId,
            ParseError::ModuleNotFound { .. } => ErrorType::ModuleNotFound,
            ParseError::CircularDependency { .. } => ErrorType::CircularDependency,
        }
    }
    
    /// Obtenir la position si disponible
    pub fn position(&self) -> Option<SourcePos> {
        match self {
            ParseError::Lexer { pos, .. } |
            ParseError::Syntax { pos, .. } => Some(*pos),
            ParseError::Validation { pos, .. } |
            ParseError::Context { pos, .. } => *pos,
                        ParseError::Resolution { .. } |
            ParseError::InvalidResourceId(_) |
            ParseError::ModuleNotFound { .. } |
            ParseError::CircularDependency { .. } => None,
            
        }
    }
} 