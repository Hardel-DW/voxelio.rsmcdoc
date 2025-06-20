//! # Voxel-RSMCDOC
//! 
//! Parser MCDOC en Rust pour validation ultra-rapide de datapacks Minecraft.
//! 
//! ## Architecture
//! 
//! 1. **Lexer** : Tokenisation zero-copy avec lifetimes
//! 2. **Parser** : Construction AST avec error recovery
//! 3. **Resolver** : Résolution imports avec topological sort
//! 4. **Validator** : Validation JSON avec registry integration
//! 
//! ## Performance Targets
//! 
//! - Small (100 files): <20ms total
//! - Medium (500 files): <70ms total  
//! - Large (1000 files): <150ms total

pub mod error;
pub mod lexer;
pub mod parser;
pub mod registry;
pub mod resolver;
pub mod types;
pub mod validator;

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-exports for public API
pub use validator::McDocValidator;
pub use types::{McDocError, McDocDependency, ValidationResult, DatapackResult};
pub use error::{McDocParserError, ErrorType};

/// Resource identifier for Minecraft resources (e.g., "minecraft:diamond_sword")
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceId {
    pub namespace: String,
    pub path: String,
}

impl ResourceId {
    /// Parse a resource identifier string like "minecraft:diamond_sword"
    /// If no namespace provided, it stays as-is without default namespace
    pub fn parse(input: &str) -> Result<Self, McDocParserError> {
        Self::parse_with_default_namespace(input, None)
    }
    
    /// Parse with optional default namespace
    pub fn parse_with_default_namespace(input: &str, default_namespace: Option<&str>) -> Result<Self, McDocParserError> {
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
                        namespace: String::new(), // No default namespace
                        path: path.to_string(),
                    }),
                }
            },
            _ => Err(McDocParserError::InvalidResourceId(input.to_string())),
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

 