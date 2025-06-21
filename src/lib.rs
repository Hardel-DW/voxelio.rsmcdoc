//! Voxel RSMCDOC - MCDOC Parser in Rust

pub mod lexer;
pub mod parser;
pub mod error;
pub mod types;
pub mod registry;
pub mod validator;
pub mod wasm;

// Main re-exports for compatibility
pub use error::{ParseError, SourcePos, ErrorType};
pub use parser::{Parser, McDocFile, Declaration, StructDeclaration, FieldDeclaration, TypeExpression}; 
pub use lexer::{Lexer, Token, TokenWithPos, Position};
pub use types::*;
pub use registry::Registry;
pub use validator::McDocValidator;

/// Main entry point to parse an MCDOC file
pub fn parse_mcdoc(input: &str) -> Result<McDocFile, Vec<ParseError>> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().map_err(|e| vec![e])?;
    
    let mut parser = Parser::new(tokens);
    parser.parse()
}

/// Resource identifier for Minecraft resources (e.g., "minecraft:diamond_sword")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId {
    pub namespace: String,
    pub path: String,
}

impl ResourceId {
    /// Parse a resource identifier string like "minecraft:diamond_sword"
    pub fn parse(input: &str) -> Result<Self, ParseError> {
        Self::parse_with_default_namespace(input, None)
    }
    
    /// Parse with optional default namespace
    pub fn parse_with_default_namespace(input: &str, default_namespace: Option<&str>) -> Result<Self, ParseError> {
        let parts: Vec<&str> = input.split(':').collect();
        
        match parts.as_slice() {
            [namespace, path] => Ok(ResourceId {
                namespace: namespace.to_string(),
                path: path.to_string(),
            }),
            [path] => {
                match default_namespace {
                    Some(ns) => Ok(ResourceId {
                        namespace: ns.to_string(),
                        path: path.to_string(),
                    }),
                    None => Ok(ResourceId {
                        namespace: String::new(),
                        path: path.to_string(),
                    }),
                }
            },
            _ => Err(ParseError::InvalidResourceId(input.to_string())),
        }
    }
    
    /// Convert back to string format
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.namespace, self.path)
    }
}

/// Registry dependency for dynamic loading
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryDependency {
    pub registry: String,
    pub identifier: String,
    pub is_tag: bool,
}