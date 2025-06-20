//! Parser MCDOC - Construction AST à partir des tokens
//! 
//! Supporte error recovery pour une meilleure expérience IDE.

use crate::lexer::{Token, TokenWithPos, Position};
use crate::error::McDocParserError;
use rustc_hash::FxHashMap;

/// AST Node pour MCDOC
#[derive(Debug, Clone, PartialEq)]
pub struct McDocFile<'input> {
    pub imports: Vec<ImportStatement<'input>>,
    pub declarations: Vec<Declaration<'input>>,
}

/// Import statement: use ::path::to::Module
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStatement<'input> {
    pub path: ImportPath<'input>,
    pub position: Position,
}

/// Import path: either absolute (::...) or relative (super::...)
#[derive(Debug, Clone, PartialEq)]
pub enum ImportPath<'input> {
    Absolute(Vec<&'input str>),
    Relative(Vec<&'input str>),
}

/// Top-level declarations
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration<'input> {
    Struct(StructDeclaration<'input>),
    Enum(EnumDeclaration<'input>),
    Type(TypeDeclaration<'input>),
    Dispatch(DispatchDeclaration<'input>),
}

/// Struct declaration
#[derive(Debug, Clone, PartialEq)]
pub struct StructDeclaration<'input> {
    pub name: &'input str,
    pub fields: Vec<FieldDeclaration<'input>>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Field in a struct
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclaration<'input> {
    pub name: &'input str,
    pub field_type: TypeExpression<'input>,
    pub optional: bool,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Enum declaration
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclaration<'input> {
    pub name: &'input str,
    pub base_type: Option<&'input str>, // e.g., enum(string)
    pub variants: Vec<EnumVariant<'input>>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Enum variant
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant<'input> {
    pub name: &'input str,
    pub value: Option<LiteralValue<'input>>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Type alias declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration<'input> {
    pub name: &'input str,
    pub type_expr: TypeExpression<'input>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Dispatch declaration
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchDeclaration<'input> {
    pub source: DispatchSource<'input>,
    pub targets: Vec<DispatchTarget<'input>>,
    pub target_type: TypeExpression<'input>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Dispatch source (e.g., minecraft:resource[recipe])
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchSource<'input> {
    pub registry: &'input str,
    pub key: Option<&'input str>,
    pub position: Position,
}

/// Dispatch target (e.g., [crafting_shaped])
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchTarget<'input> {
    Specific(&'input str),
    Unknown, // %unknown
}

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpression<'input> {
    /// Simple type (e.g., string, int)
    Simple(&'input str),
    /// Array type (e.g., [string])
    Array {
        element_type: Box<TypeExpression<'input>>,
        constraints: Option<ArrayConstraints>,
    },
    /// Union type (e.g., string | int)
    Union(Vec<TypeExpression<'input>>),
    /// Struct type (inline)
    Struct(Vec<FieldDeclaration<'input>>),
    /// Generic type (e.g., Tag<E>)
    Generic {
        name: &'input str,
        type_args: Vec<TypeExpression<'input>>,
    },
    /// Reference type (e.g., ::path::Type)
    Reference(ImportPath<'input>),
    /// Spread expression (e.g., ...minecraft:recipe_serializer[[type]])
    Spread(SpreadExpression<'input>),
}

/// Array constraints (e.g., @ 1..10)
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayConstraints {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// Annotation structure supporting complex MCDOC annotations
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation<'input> {
    pub annotation_type: AnnotationType<'input>,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationType<'input> {
    /// Simple annotation: #[id="value"]
    Simple { name: &'input str, value: &'input str },
    /// Complex annotation with parameters: #[id(registry="item", tags="allowed")]
    Complex { name: &'input str, params: FxHashMap<&'input str, AnnotationValue<'input>> },
    /// Version constraint: #[since="1.20.5"]
    Since(&'input str),
    /// Version constraint: #[until="1.19"]
    Until(&'input str),
    /// Custom annotation with raw content for unrecognized patterns
    Raw(&'input str),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationValue<'input> {
    String(&'input str),
    Array(Vec<&'input str>),
    Boolean(bool),
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue<'input> {
    String(&'input str),
    Number(f64),
    Boolean(bool),
}

/// Spread expression for dispatch inheritance
#[derive(Debug, Clone, PartialEq)]
pub struct SpreadExpression<'input> {
    pub base_path: &'input str,
    pub dynamic_key: Option<DynamicReference<'input>>,
    pub position: Position,
}

/// Dynamic reference like [[type]] or [[%key]]
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicReference<'input> {
    pub reference: DynamicReferenceType<'input>,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DynamicReferenceType<'input> {
    /// Field reference: [[type]]
    Field(&'input str),
    /// Special key reference: [[%key]]
    SpecialKey(&'input str),
}

/// Parser state with error recovery
pub struct Parser<'input> {
    tokens: Vec<TokenWithPos<'input>>,
    current: usize,
    errors: Vec<McDocParserError>,
}

impl<'input> Parser<'input> {
    /// Create new parser
    pub fn new(tokens: Vec<TokenWithPos<'input>>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }
    
    /// Parse complete MCDOC file
    pub fn parse(&mut self) -> Result<McDocFile<'input>, Vec<McDocParserError>> {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();
        
        // Skip leading newlines
        self.skip_newlines();
        
        while !self.is_at_end() {
            // Parse imports first
            if self.match_token(&Token::Use) {
                match self.parse_import() {
                    Ok(import) => imports.push(import),
                    Err(e) => {
                        self.errors.push(e);
                        self.synchronize();
                        continue;
                    }
                }
            } else {
                // Parse declarations
                match self.parse_declaration() {
                    Ok(Some(decl)) => declarations.push(decl),
                    Ok(None) => {
                        self.advance();
                    },
                    Err(e) => {
                        self.errors.push(e);
                        self.synchronize();
                        continue;
                    }
                }
            }
            
            // Skip newlines between declarations
            self.skip_newlines();
        }
        
        if self.errors.is_empty() {
            Ok(McDocFile { imports, declarations })
        } else {
            Err(self.errors.clone())
        }
    }
    
    /// Parse import statement
    pub fn parse_import(&mut self) -> Result<ImportStatement<'input>, McDocParserError> {
        let position = self.current_position();
        self.consume(&Token::Use, "Expected 'use'")?;
        
        let path = self.parse_import_path()?;
        
        Ok(ImportStatement { path, position })
    }
    
    /// Parse import path
    fn parse_import_path(&mut self) -> Result<ImportPath<'input>, McDocParserError> {
        let mut segments = Vec::new();
        let is_absolute = self.match_token(&Token::DoubleColon);
        
        if is_absolute {
            self.advance(); // consume ::
        } else if self.match_token(&Token::Super) {
            self.advance();
            if !self.match_token(&Token::DoubleColon) {
                return Err(McDocParserError::UnexpectedToken {
                    expected: "::".to_string(),
                    found: self.current_token_string(),
                    line: self.current_position().line,
                    column: self.current_position().column,
                });
            }
            self.advance(); // consume ::
        }
        
        // Parse path segments
        loop {
            if let Token::Identifier(name) = &self.current_token()?.token {
                segments.push(*name);
                self.advance();
                
                if self.match_token(&Token::DoubleColon) {
                    self.advance();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if segments.is_empty() {
            return Err(McDocParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            });
        }
        
        Ok(if is_absolute {
            ImportPath::Absolute(segments)
        } else {
            ImportPath::Relative(segments)
        })
    }
    
    /// Parse top-level declaration
    fn parse_declaration(&mut self) -> Result<Option<Declaration<'input>>, McDocParserError> {
        let annotations = self.parse_annotations()?;
        
        match &self.current_token()?.token {
            Token::Struct => Ok(Some(Declaration::Struct(self.parse_struct(annotations)?))),
            Token::Enum => Ok(Some(Declaration::Enum(self.parse_enum(annotations)?))),
            Token::Type => Ok(Some(Declaration::Type(self.parse_type_alias(annotations)?))),
            Token::Dispatch => Ok(Some(Declaration::Dispatch(self.parse_dispatch(annotations)?))),
            Token::Eof => Ok(None),
            _ => {
                return Err(McDocParserError::UnexpectedToken {
                    expected: "declaration".to_string(),
                    found: self.current_token_string(),
                    line: self.current_position().line,
                    column: self.current_position().column,
                });
            }
        }
    }
    
    /// Parse annotations
    fn parse_annotations(&mut self) -> Result<Vec<Annotation<'input>>, McDocParserError> {
        let mut annotations = Vec::new();
        
        while let Token::Annotation(content) = &self.current_token()?.token {
            annotations.push(Annotation {
                annotation_type: self.parse_annotation_type(content)?,
                position: self.current_position(),
            });
            self.advance();
        }
        
        Ok(annotations)
    }
    
    /// Parse annotation type from content string
    pub fn parse_annotation_type(&self, content: &'input str) -> Result<AnnotationType<'input>, McDocParserError> {
        // Remove # and [ ] brackets
        let content = content.trim_start_matches('#').trim_start_matches('[').trim_end_matches(']');
        
        // Handle version constraints first
        if content.starts_with("since=") {
            let version = content.strip_prefix("since=").unwrap().trim_matches('"');
            return Ok(AnnotationType::Since(version));
        }
        
        if content.starts_with("until=") {
            let version = content.strip_prefix("until=").unwrap().trim_matches('"');
            return Ok(AnnotationType::Until(version));
        }
        
        // Check for complex annotation with parameters: name(param1="value1", param2="value2")
        if let Some(paren_pos) = content.find('(') {
            let name = content[..paren_pos].trim();
            let params_str = &content[paren_pos + 1..];
            
            if let Some(end_paren) = params_str.rfind(')') {
                let params_str = &params_str[..end_paren];
                let mut params = FxHashMap::default();
                
                // Parse parameters: key="value", key2=["array", "values"]
                // Need to handle nested arrays properly, not just split on commas
                let mut i = 0;
                let chars: Vec<char> = params_str.chars().collect();
                
                while i < chars.len() {
                    // Skip whitespace
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i >= chars.len() { break; }
                    
                    // Find key
                    let key_start = i;
                    while i < chars.len() && chars[i] != '=' {
                        i += 1;
                    }
                    if i >= chars.len() { break; }
                    
                    let key = params_str[key_start..i].trim();
                    i += 1; // Skip '='
                    
                    // Skip whitespace after =
                    while i < chars.len() && chars[i].is_whitespace() {
                        i += 1;
                    }
                    if i >= chars.len() { break; }
                    
                    // Parse value
                    let value_start = i;
                    let value = if chars[i] == '[' {
                        // Array value - find matching ]
                        let mut bracket_count = 0;
                        while i < chars.len() {
                            if chars[i] == '[' { bracket_count += 1; }
                            else if chars[i] == ']' { 
                                bracket_count -= 1;
                                if bracket_count == 0 {
                                    i += 1;
                                    break;
                                }
                            }
                            i += 1;
                        }
                        
                        let array_str = &params_str[value_start..i];
                        let array_content = &array_str[1..array_str.len() - 1];
                        let items: Vec<&str> = array_content
                            .split(',')
                            .map(|s| s.trim().trim_matches('"'))
                            .collect();
                        AnnotationValue::Array(items)
                    } else if chars[i] == '"' {
                        // String value - find matching "
                        i += 1; // Skip opening "
                        let string_start = i;
                        while i < chars.len() && chars[i] != '"' {
                            if chars[i] == '\\' { i += 1; } // Skip escaped char
                            i += 1;
                        }
                        let string_content = &params_str[string_start..i];
                        i += 1; // Skip closing "
                        AnnotationValue::String(string_content)
                    } else {
                        // Boolean or unquoted value
                        while i < chars.len() && chars[i] != ',' {
                            i += 1;
                        }
                        let bool_str = params_str[value_start..i].trim();
                        if bool_str == "true" || bool_str == "false" {
                            AnnotationValue::Boolean(bool_str == "true")
                        } else {
                            AnnotationValue::String(bool_str)
                        }
                    };
                    
                    params.insert(key, value);
                    
                    // Skip comma and whitespace
                    while i < chars.len() && (chars[i] == ',' || chars[i].is_whitespace()) {
                        i += 1;
                    }
                }
                
                return Ok(AnnotationType::Complex { name, params });
            }
        }
        
        // Check for simple annotation: name="value"
        if let Some(eq_pos) = content.find('=') {
            let name = content[..eq_pos].trim();
            let value = content[eq_pos + 1..].trim().trim_matches('"');
            return Ok(AnnotationType::Simple { name, value });
        }
        
        // Fallback to raw annotation
        Ok(AnnotationType::Raw(content))
    }
    
    /// Parse struct declaration
    pub fn parse_struct(&mut self, annotations: Vec<Annotation<'input>>) -> Result<StructDeclaration<'input>, McDocParserError> {
        let position = self.current_position();
        self.consume(&Token::Struct, "Expected 'struct'")?;
        
        let name = if let Token::Identifier(name) = &self.current_token()?.token {
            let n = *name;
            self.advance();
            n
        } else {
            return Err(McDocParserError::UnexpectedToken {
                expected: "struct name".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            });
        };
        
        self.consume(&Token::LeftBrace, "Expected '{'")?;
        self.skip_newlines();
        
        let mut fields = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Parse field annotations
            let field_annotations = self.parse_annotations()?;
            
            // Check for spread expression
            if self.match_token(&Token::DotDotDot) {
                self.advance();
                let spread_expr = self.parse_spread_expression()?;
                
                // Create a synthetic field for the spread expression
                fields.push(FieldDeclaration {
                    name: "_spread", // Synthetic name for spread fields
                    field_type: spread_expr,
                    optional: false,
                    annotations: field_annotations,
                    position: self.current_position(),
                });
            } else {
                // Parse field name (can be keyword or identifier)
                let field_name = match &self.current_token()?.token {
                    Token::Identifier(name) => {
                        let n = *name;
                        self.advance();
                        n
                    }
                    Token::Type => {
                        // Handle 'type' keyword as field name
                        self.advance();
                        "type"
                    }
                    _ => {
                        self.synchronize();
                        continue;
                    }
                };
                
                // Check for optional marker
                let optional = if self.match_token(&Token::Question) {
                    self.advance();
                    true
                } else {
                    false
                };
                
                self.consume(&Token::Colon, "Expected ':' after field name")?;
                
                // Parse field type
                let field_type = self.parse_type_expression()?;
                
                fields.push(FieldDeclaration {
                    name: field_name,
                    field_type,
                    optional,
                    annotations: field_annotations,
                    position: self.current_position(),
                });
            }
            
            // Skip optional comma and newlines
            if self.match_token(&Token::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }
        
        self.consume(&Token::RightBrace, "Expected '}'")?;
        
        Ok(StructDeclaration {
            name,
            fields,
            annotations,
            position,
        })
    }
    
    /// Parse type expression (unions, arrays, constraints)
    fn parse_type_expression(&mut self) -> Result<TypeExpression<'input>, McDocParserError> {
        let mut types = Vec::new();
        
        // Parse first type
        types.push(self.parse_single_type()?);
        
        // Check for union (|)
        while self.match_token(&Token::Pipe) {
            self.advance();
            // Skip any newlines after the pipe
            self.skip_newlines();
            
            // Check for trailing pipe (e.g., "TypeA | TypeB |")
            // If we encounter ), }, ], or EOF after a pipe, it's a trailing pipe
            if self.check(&Token::RightParen) || self.check(&Token::RightBrace) || 
               self.check(&Token::RightBracket) || self.is_at_end() {
                break;
            }
            
            types.push(self.parse_single_type()?);
        }
        
        if types.len() == 1 {
            Ok(types.into_iter().next().unwrap())
        } else {
            Ok(TypeExpression::Union(types))
        }
    }
    
    /// Parse single type (array, simple, struct, spread)
    pub fn parse_single_type(&mut self) -> Result<TypeExpression<'input>, McDocParserError> {
        // Skip newlines before parsing annotations
        self.skip_newlines();
        
        // Check for type annotations first
        let _type_annotations = self.parse_annotations()?;
        
        // Skip newlines after annotations
        self.skip_newlines();
        
        match &self.current_token()?.token {
            Token::LeftBracket => {
                // Array type [ElementType] ou Dynamic Reference [[field]]
                self.advance();
                
                // Check for dynamic reference [[...]]
                if self.match_token(&Token::LeftBracket) {
                    self.advance();
                    let _dynamic_ref = self.parse_dynamic_reference()?;
                    self.consume(&Token::RightBracket, "Expected ']' for dynamic reference")?;
                    self.consume(&Token::RightBracket, "Expected ']' to close dynamic reference")?;
                    return Ok(TypeExpression::Reference(ImportPath::Absolute(vec!["dynamic_reference"])));
                }
                
                let element_type = Box::new(self.parse_type_expression()?);
                self.consume(&Token::RightBracket, "Expected ']'")?;
                
                // Parse optional constraints @ min..max
                let constraints = if self.match_token(&Token::At) {
                    self.advance();
                    Some(self.parse_array_constraints()?)
                } else {
                    None
                };
                
                Ok(TypeExpression::Array { element_type, constraints })
            }
            Token::DotDotDot => {
                // Spread expression ...minecraft:recipe_serializer[[type]]
                self.advance();
                self.parse_spread_expression()
            }
            Token::Struct => {
                // Inline struct: struct { ... } or struct Name { ... }
                self.advance();
                
                // Check if there's a struct name
                let _struct_name = if let Token::Identifier(name) = &self.current_token()?.token {
                    // Named inline struct: struct LootTable { ... }
                    // For now, we'll skip the name and just parse the fields
                    let struct_name = *name;
                    self.advance();
                    Some(struct_name)
                } else {
                    None
                };
                
                self.consume(&Token::LeftBrace, "Expected '{'")?;
                
                let mut fields = Vec::new();
                while !self.check(&Token::RightBrace) && !self.is_at_end() {
                    self.skip_newlines();
                    
                    if self.check(&Token::RightBrace) {
                        break;
                    }
                    
                    // Parse field annotations
                    let field_annotations = self.parse_annotations()?;
                    
                    // Skip newlines after annotations
                    self.skip_newlines();
                    
                    // Check for spread field: ...TypeName
                    if self.match_token(&Token::DotDotDot) {
                        self.advance();
                        let spread_type = self.parse_type_expression()?;
                        
                        // Create a pseudo-field for the spread
                        fields.push(FieldDeclaration {
                            name: "...", // Special marker for spread
                            field_type: spread_type,
                            optional: false,
                            annotations: field_annotations,
                            position: self.current_position(),
                        });
                        
                        // Handle optional comma and/or newlines
                        if self.match_token(&Token::Comma) {
                            self.advance();
                        }
                        self.skip_newlines();
                        continue;
                    }
                    
                    // Parse field name
                    let field_name = if let Token::Identifier(name) = &self.current_token()?.token {
                        let field_name = *name;
                        self.advance();
                        field_name
                    } else if let Token::Type = &self.current_token()?.token {
                        // Handle 'type' as field name
                        self.advance();
                        "type"
                    } else {
                        break;
                    };
                    
                    // Check for optional field marker '?'
                    let optional = if self.match_token(&Token::Question) {
                        self.advance();
                        true
                    } else {
                        false
                    };
                    
                    self.consume(&Token::Colon, "Expected ':' after field name")?;
                    let field_type = self.parse_type_expression()?;
                    
                    fields.push(FieldDeclaration {
                        name: field_name,
                        field_type,
                        optional,
                        annotations: field_annotations,
                        position: self.current_position(),
                    });
                    
                                        // Handle optional comma and/or newlines
                    if self.match_token(&Token::Comma) {
                        self.advance();
                    }
                    self.skip_newlines();
                }
                
                self.consume(&Token::RightBrace, "Expected '}'")?;
                Ok(TypeExpression::Struct(fields))
            }
            Token::Identifier(name) => {
                let type_name = *name;
                self.advance();
                
                // Check for generic parameters <T>
                if self.match_token(&Token::LeftAngle) {
                    self.advance();
                    let mut type_args = Vec::new();
                    
                    while !self.check(&Token::RightAngle) && !self.is_at_end() {
                        type_args.push(self.parse_type_expression()?);
                        if self.match_token(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    self.consume(&Token::RightAngle, "Expected '>'")?;
                    Ok(TypeExpression::Generic { name: type_name, type_args })
                } else if self.match_token(&Token::At) {
                    // Scalar type constraint: int @ 1..
                    self.advance();
                    let _constraints = self.parse_array_constraints()?;
                    // For now, we'll ignore the constraints and just return the simple type
                    // In a full implementation, we'd want to store these constraints
                    Ok(TypeExpression::Simple(type_name))
                } else {
                    Ok(TypeExpression::Simple(type_name))
                }
            }
            Token::DoubleColon => {
                // Reference type ::path::Type
                let path = self.parse_import_path()?;
                Ok(TypeExpression::Reference(path))
            }
            Token::LeftParen => {
                // Parenthesized expression - parse as union type
                self.advance();
                let expr = self.parse_type_expression()?;
                self.consume(&Token::RightParen, "Expected ')' after parenthesized type expression")?;
                Ok(expr)
            }
            _ => Err(McDocParserError::UnexpectedToken {
                expected: "type expression".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            })
        }
    }
    
        /// Parse spread expression: ...minecraft:recipe_serializer[[type]] or ...TypeName
    fn parse_spread_expression(&mut self) -> Result<TypeExpression<'input>, McDocParserError> {
        let position = self.current_position();
        
        // Parse first identifier
        let first_identifier = if let Token::Identifier(id) = &self.current_token()?.token {
            let i = *id;
            self.advance();
            i
        } else {
            return Err(McDocParserError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            });
        };

        // Format as namespace:identifier - Generic parsing without hardcoding
        let base_path = if self.match_token(&Token::Colon) {
            self.advance();
            if let Token::Identifier(second_identifier) = &self.current_token()?.token {
                let second = *second_identifier;
                self.advance();
                // Format as "namespace:identifier" generically
                format!("{}:{}", first_identifier, second).leak()
            } else {
                return Err(McDocParserError::UnexpectedToken {
                    expected: "identifier after namespace".to_string(),
                    found: self.current_token_string(),
                    line: self.current_position().line,
                    column: self.current_position().column,
                });
            }
        } else {
            // Simple identifier
            first_identifier
        };
        
        // Parse optional dynamic reference [[...]]
        let dynamic_key = if self.match_token(&Token::LeftBracket) {
            self.advance();
            if self.match_token(&Token::LeftBracket) {
                self.advance();
                let dynamic_ref = self.parse_dynamic_reference()?;
                self.consume(&Token::RightBracket, "Expected ']' for dynamic reference")?;
                self.consume(&Token::RightBracket, "Expected ']' to close dynamic reference")?;
                Some(dynamic_ref)
            } else {
                // Put back the bracket
                self.current -= 1;
                None
            }
        } else {
            None
        };
        
        Ok(TypeExpression::Spread(SpreadExpression {
            base_path,
            dynamic_key,
            position,
        }))
    }

    /// Parse dynamic reference: type or %key
    fn parse_dynamic_reference(&mut self) -> Result<DynamicReference<'input>, McDocParserError> {
        let position = self.current_position();
        
        match &self.current_token()?.token {
            Token::Percent => {
                // Special key reference: %key
                self.advance();
                if let Token::Identifier(key) = &self.current_token()?.token {
                    let k = *key;
                    self.advance();
                    Ok(DynamicReference {
                        reference: DynamicReferenceType::SpecialKey(k),
                        position,
                    })
                } else {
                    Err(McDocParserError::UnexpectedToken {
                        expected: "key name after %".to_string(),
                        found: self.current_token_string(),
                        line: self.current_position().line,
                        column: self.current_position().column,
                    })
                }
            }
            Token::Identifier(field) => {
                // Field reference: type
                let f = *field;
                self.advance();
                Ok(DynamicReference {
                    reference: DynamicReferenceType::Field(f),
                    position,
                })
            }
            Token::Type => {
                // Handle 'type' keyword as field name in dynamic reference
                self.advance();
                Ok(DynamicReference {
                    reference: DynamicReferenceType::Field("type"),
                    position,
                })
            }
            _ => Err(McDocParserError::UnexpectedToken {
                expected: "field name or %key".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            })
        }
    }
    
    /// Parse array constraints @ min..max
    fn parse_array_constraints(&mut self) -> Result<ArrayConstraints, McDocParserError> {
        let mut min = None;
        let mut max = None;
        
        // Parse min value
        if let Token::Number(num) = &self.current_token()?.token {
            min = Some(*num as u32);
            self.advance();
        }
        
        // Parse .. separator
        if self.match_token(&Token::DotDot) {
            self.advance();
            
            // Parse max value
            if let Token::Number(num) = &self.current_token()?.token {
                max = Some(*num as u32);
                self.advance();
            }
        } else if min.is_some() {
            // Single number means exact size
            max = min;
        }
        
        Ok(ArrayConstraints { min, max })
    }
    
    /// Parse enum declaration - enum(string) GameMode { Creative = "creative" }
    pub fn parse_enum(&mut self, annotations: Vec<Annotation<'input>>) -> Result<EnumDeclaration<'input>, McDocParserError> {
        let position = self.current_position();
        self.consume(&Token::Enum, "Expected 'enum'")?;
        
        // Parse optional type specifier: enum(string) or enum(int)
        let base_type = if self.match_token(&Token::LeftParen) {
            self.advance();
            
            let type_name = if let Token::Identifier(name) = &self.current_token()?.token {
                let n = Some(*name);
                self.advance();
                n
            } else {
                return Err(McDocParserError::UnexpectedToken {
                    expected: "type identifier".to_string(),
                    found: self.current_token_string(),
                    line: self.current_position().line,
                    column: self.current_position().column,
                });
            };
            
            self.consume(&Token::RightParen, "Expected ')' after enum type")?;
            type_name
        } else {
            None
        };
        
        // Parse enum name
        let name = if let Token::Identifier(name) = &self.current_token()?.token {
            let n = *name;
            self.advance();
            n
        } else {
            return Err(McDocParserError::UnexpectedToken {
                expected: "enum name".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            });
        };
        
        // Parse enum body { Variant1 = "value1", Variant2 = "value2" }
        self.consume(&Token::LeftBrace, "Expected '{' after enum name")?;
        self.skip_newlines();
        
        let mut variants = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Parse variant annotations (optional)
            let variant_annotations = self.parse_annotations()?;
            
            // Skip newlines after annotations
            self.skip_newlines();
            
            // Parse variant name
            let variant_name = if let Token::Identifier(name) = &self.current_token()?.token {
                let n = *name;
                self.advance();
                n
            } else {
                self.synchronize();
                continue;
            };
            
            // Parse optional value assignment: = "creative" or = 42
            let value = if self.match_token(&Token::Equals) {
                self.advance();
                
                match &self.current_token()?.token {
                    Token::String(s) => {
                        let val = Some(LiteralValue::String(*s));
                        self.advance();
                        val
                    }
                    Token::Number(n) => {
                        let val = Some(LiteralValue::Number(*n));
                        self.advance();
                        val
                    }
                    Token::Boolean(b) => {
                        let val = Some(LiteralValue::Boolean(*b));
                        self.advance();
                        val
                    }
                    _ => {
                        return Err(McDocParserError::UnexpectedToken {
                            expected: "literal value".to_string(),
                            found: self.current_token_string(),
                            line: self.current_position().line,
                            column: self.current_position().column,
                        });
                    }
                }
            } else {
                None
            };
            
            variants.push(EnumVariant {
                name: variant_name,
                value,
                annotations: variant_annotations,
                position: self.current_position(),
            });
            
            // Skip optional comma and newlines
            if self.match_token(&Token::Comma) {
                self.advance();
            }
            self.skip_newlines();
        }
        
        self.consume(&Token::RightBrace, "Expected '}' after enum variants")?;
        
        Ok(EnumDeclaration {
            name,
            base_type,
            variants,
            annotations,
            position,
        })
    }
    
    /// Parse type alias - type ItemStack = struct { item: string, count?: int }
    pub fn parse_type_alias(&mut self, annotations: Vec<Annotation<'input>>) -> Result<TypeDeclaration<'input>, McDocParserError> {
        let position = self.current_position();
        self.consume(&Token::Type, "Expected 'type'")?;
        
        // Parse type alias name
        let name = if let Token::Identifier(name) = &self.current_token()?.token {
            let n = *name;
            self.advance();
            n
        } else {
            return Err(McDocParserError::UnexpectedToken {
                expected: "type alias name".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            });
        };
        
        // Consume '=' assignment
        self.consume(&Token::Equals, "Expected '=' after type alias name")?;
        
        // Parse the type expression
        let type_expr = self.parse_type_expression()?;
        
        Ok(TypeDeclaration {
            name,
            type_expr,
            annotations,
            position,
        })
    }
    
    /// Parse dispatch declaration - CŒUR DE MCDOC
    pub fn parse_dispatch(&mut self, annotations: Vec<Annotation<'input>>) -> Result<DispatchDeclaration<'input>, McDocParserError> {
        let position = self.current_position();
        self.consume(&Token::Dispatch, "Expected 'dispatch'")?;
        
        // Parse dispatch source: minecraft:resource[recipe] ou minecraft:recipe_serializer[crafting_shaped]
        let (dispatch_source, targets) = self.parse_dispatch_source()?;
        
        // Consume 'to' keyword
        self.consume(&Token::To, "Expected 'to' after dispatch source")?;
        
        // Parse target type (struct Recipe, CraftingShaped, etc.)
        let target_type = self.parse_type_expression()?;
        
        Ok(DispatchDeclaration {
            source: dispatch_source,
            targets,
            target_type,
            annotations,
            position,
        })
    }
    
    /// Parse dispatch source: minecraft:resource[recipe] or minecraft:recipe_serializer[crafting_shaped,smelting]
    fn parse_dispatch_source(&mut self) -> Result<(DispatchSource<'input>, Vec<DispatchTarget<'input>>), McDocParserError> {
        let position = self.current_position();
        
        // Parse registry namespace (minecraft) - consume but don't store since it's not used in current logic
        if let Token::Identifier(_name) = &self.current_token()?.token {
            self.advance();
        } else {
            return Err(McDocParserError::UnexpectedToken {
                expected: "namespace identifier".to_string(),
                found: self.current_token_string(),
                line: position.line,
                column: position.column,
            });
        };
        
        // Consume ':'
        self.consume(&Token::Colon, "Expected ':' after namespace")?;
        
        // Parse registry name (resource, recipe_serializer, etc.)
        let registry = if let Token::Identifier(name) = &self.current_token()?.token {
            let n = *name;
            self.advance();
            n
        } else {
            return Err(McDocParserError::UnexpectedToken {
                expected: "registry identifier".to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            });
        };
        
        // Build registry identifier generically - no hardcoding
        let full_registry = registry; // Keep registry as-is, let user code handle namespace logic
        
        // Parse dispatch targets [recipe] or [crafting_shaped,smelting] or [%unknown]
        self.consume(&Token::LeftBracket, "Expected '[' after registry")?;
        
        let mut targets = Vec::new();
        
        loop {
            match &self.current_token()?.token {
                Token::Identifier(target) => {
                    targets.push(DispatchTarget::Specific(*target));
                    self.advance();
                }
                Token::Percent => {
                    // %unknown or %fallback
                    self.advance();
                    if let Token::Identifier(special) = &self.current_token()?.token {
                        if *special == "unknown" {
                            targets.push(DispatchTarget::Unknown);
                        }
                        self.advance();
                    } else {
                        return Err(McDocParserError::UnexpectedToken {
                            expected: "identifier after %".to_string(),
                            found: self.current_token_string(),
                            line: self.current_position().line,
                            column: self.current_position().column,
                        });
                    }
                }
                _ => break,
            }
            
            // Check for comma (multiple targets)
            if self.match_token(&Token::Comma) {
                self.advance();
                continue;
            } else {
                break;
            }
        }
        
        self.consume(&Token::RightBracket, "Expected ']' after dispatch targets")?;
        
        if targets.is_empty() {
            return Err(McDocParserError::UnexpectedToken {
                expected: "at least one dispatch target".to_string(),
                found: "empty targets".to_string(),
                line: position.line,
                column: position.column,
            });
        }
        
        // Check for dispatch key [[type]] after the targets
        let dispatch_key = if self.check(&Token::LeftBracket) && 
                              self.tokens.get(self.current + 1).map(|t| &t.token) == Some(&Token::LeftBracket) {
            Some(self.parse_dispatch_key()?)
        } else {
            None
        };

        let source = DispatchSource {
            registry: full_registry,
            key: dispatch_key,
            position,
        };
        
        Ok((source, targets))
    }

    /// Parse dispatch key like [[type]] or [[%key]]
    fn parse_dispatch_key(&mut self) -> Result<&'input str, McDocParserError> {
        let position = self.current_position();
        
        // Consume first '['
        self.consume(&Token::LeftBracket, "Expected '[' for dispatch key")?;
        // Consume second '['
        self.consume(&Token::LeftBracket, "Expected second '[' for dispatch key")?;
        
        // Parse the key identifier (can be keyword or identifier)
        let key = match &self.current_token()?.token {
            Token::Identifier(name) => {
                let k = *name;
                self.advance();
                k
            }
            Token::Type => {
                // Handle 'type' keyword as identifier in dispatch key context
                self.advance();
                "type"
            }
            Token::Percent => {
                // Handle %key syntax
                self.advance();
                match &self.current_token()?.token {
                    Token::Identifier(name) => {
                        let k = *name;
                        self.advance();
                        k
                    }
                    Token::Type => {
                        self.advance();
                        "type"
                    }
                    _ => {
                        return Err(McDocParserError::UnexpectedToken {
                            expected: "identifier after %".to_string(),
                            found: self.current_token_string(),
                            line: position.line,
                            column: position.column,
                        });
                    }
                }
            }
            _ => {
                return Err(McDocParserError::UnexpectedToken {
                    expected: "identifier or % in dispatch key".to_string(),
                    found: self.current_token_string(),
                    line: position.line,
                    column: position.column,
                });
            }
        };
        
        // Consume closing brackets
        self.consume(&Token::RightBracket, "Expected ']' after dispatch key")?;
        self.consume(&Token::RightBracket, "Expected second ']' after dispatch key")?;
        
        Ok(key)
    }
    
    // Helper methods
    
    fn current_token(&self) -> Result<&TokenWithPos<'input>, McDocParserError> {
        self.tokens.get(self.current).ok_or_else(|| {
            McDocParserError::UnexpectedToken {
                expected: "token".to_string(),
                found: "EOF".to_string(),
                line: 0,
                column: 0,
            }
        })
    }
    
    fn current_token_string(&self) -> String {
        self.current_token()
            .map(|t| format!("{:?}", t.token))
            .unwrap_or_else(|_| "EOF".to_string())
    }
    
    fn current_position(&self) -> Position {
        self.current_token()
            .map(|t| t.position)
            .unwrap_or(Position { line: 0, column: 0, offset: 0 })
    }
    
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        matches!(self.current_token().map(|t| &t.token), Ok(Token::Eof))
    }
    
    fn check(&self, token_type: &Token) -> bool {
        if let Ok(current) = self.current_token() {
            std::mem::discriminant(&current.token) == std::mem::discriminant(token_type)
        } else {
            false
        }
    }
    
    fn match_token(&self, token_type: &Token) -> bool {
        self.check(token_type)
    }
    
    fn consume(&mut self, token_type: &Token, message: &str) -> Result<(), McDocParserError> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(McDocParserError::UnexpectedToken {
                expected: message.to_string(),
                found: self.current_token_string(),
                line: self.current_position().line,
                column: self.current_position().column,
            })
        }
    }
    
    fn skip_newlines(&mut self) {
        while self.match_token(&Token::Newline) {
            self.advance();
        }
    }
    
    /// Error recovery - advance until we find a synchronization point
    fn synchronize(&mut self) {
        self.advance();
        
        while !self.is_at_end() {
            match &self.current_token().unwrap().token {
                Token::Struct | Token::Enum | Token::Type | Token::Dispatch | Token::Use => return,
                _ => self.advance(),
            }
        }
    }
} 