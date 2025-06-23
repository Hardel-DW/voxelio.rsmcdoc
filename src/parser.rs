//! Parser MCDOC unifié

use crate::error::{ParseError, SourcePos};
use crate::lexer::{Token, TokenWithPos, Position};
use rustc_hash::FxHashMap;

// ================================
// AST ESSENTIAL STRUCTURES
// ================================

/// Main MCDOC file
#[derive(Debug, Clone, PartialEq)]
pub struct McDocFile<'input> {
    pub imports: Vec<ImportStatement<'input>>,
    pub declarations: Vec<Declaration<'input>>,
}

/// Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStatement<'input> {
    pub path: ImportPath<'input>,
    pub position: Position,
}

/// Import path
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

/// Consolidated annotation
#[derive(Debug, Clone, PartialEq)]
pub struct Annotation<'input> {
    pub name: &'input str,
    pub data: AnnotationData<'input>,
    pub position: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AnnotationData<'input> {
    Simple(&'input str),
    Complex(FxHashMap<&'input str, &'input str>),
    Empty,
}

/// Struct declaration
#[derive(Debug, Clone, PartialEq)]
pub struct StructDeclaration<'input> {
    pub name: &'input str,
    pub members: Vec<StructMember<'input>>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Field declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDeclaration<'input> {
    pub name: &'input str,
    pub field_type: TypeExpression<'input>,
    pub optional: bool,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Struct member (either a field, dynamic field, or a spread)
#[derive(Debug, Clone, PartialEq)]
pub enum StructMember<'input> {
    Field(FieldDeclaration<'input>),
    DynamicField(DynamicFieldDeclaration<'input>),
    Spread(SpreadExpression<'input>),
}

/// Dynamic field declaration like [#[id="mob_effect"] string]: MobEffectPredicate
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicFieldDeclaration<'input> {
    pub key_type: TypeExpression<'input>,
    pub value_type: TypeExpression<'input>,
    pub optional: bool,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Enum declaration
#[derive(Debug, Clone, PartialEq)]
pub struct EnumDeclaration<'input> {
    pub name: &'input str,
    pub base_type: Option<&'input str>,
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

/// Type declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDeclaration<'input> {
    pub name: &'input str,
    pub type_params: Vec<&'input str>, // Generic parameters like <C, T>
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

/// Dispatch source
#[derive(Debug, Clone, PartialEq)]
pub struct DispatchSource<'input> {
    pub registry: &'input str,
    pub key: Option<&'input str>,
    pub position: Position,
}

/// Dispatch target
#[derive(Debug, Clone, PartialEq)]
pub enum DispatchTarget<'input> {
    Specific(&'input str),
    Unknown,
}

/// Type expressions
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpression<'input> {
    Simple(&'input str),
    Array {
        element_type: Box<TypeExpression<'input>>,
        constraints: Option<ArrayConstraints>,
    },
    Union(Vec<TypeExpression<'input>>),
    Struct(Vec<StructMember<'input>>),
    Generic {
        name: &'input str,
        type_args: Vec<TypeExpression<'input>>,
    },
    Reference(ImportPath<'input>),
    Spread(SpreadExpression<'input>),
    Literal(LiteralValue<'input>),
    /// Type with constraints like "float @ -80..80"
    Constrained {
        base_type: Box<TypeExpression<'input>>,
        constraints: TypeConstraints,
    },
}

/// Array constraints
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayConstraints {
    pub min: Option<u32>,
    pub max: Option<u32>,
}

/// Spread expression
#[derive(Debug, Clone, PartialEq)]
pub struct SpreadExpression<'input> {
    pub namespace: &'input str,
    pub registry: &'input str,
    pub dynamic_key: Option<DynamicReference<'input>>,
    pub annotations: Vec<Annotation<'input>>,
    pub position: Position,
}

/// Dynamic reference
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicReference<'input> {
    pub reference: DynamicReferenceType<'input>,
    pub position: Position,
}

/// Dynamic reference type
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicReferenceType<'input> {
    Field(&'input str),
    SpecialKey(&'input str),
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue<'input> {
    String(&'input str),
    Number(f64),
    Boolean(bool),
}

/// Type constraints (like @ -80..80)
#[derive(Debug, Clone, PartialEq)]
pub struct TypeConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
}

// ================================
// PARSER IMPLEMENTATION
// ================================

/// Main unified parser
pub struct Parser<'input> {
    tokens: Vec<TokenWithPos<'input>>,
    current: usize,
    errors: Vec<ParseError>,
}

impl<'input> Parser<'input> {
    pub fn new(tokens: Vec<TokenWithPos<'input>>) -> Self {
        Self {
            tokens,
            current: 0,
            errors: Vec::new(),
        }
    }

    /// Full parse of the MCDOC file
    pub fn parse(&mut self) -> Result<McDocFile<'input>, Vec<ParseError>> {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        self.skip_whitespace();

        while !self.is_at_end() {
            match self.current_token() {
                Ok(token) => match &token.token {
                    Token::Use => match self.parse_import() {
                        Ok(import) => {
                            imports.push(import);
                            if self.check_token(Token::Semicolon) {
                                self.advance();
                            }
                        },
                        Err(e) => {
                            self.errors.push(e);
                            self.synchronize();
                        }
                    },
                    Token::Eof => break,
                    _ => match self.parse_declaration() {
                        Ok(Some(declaration)) => {
                            declarations.push(declaration);
                            if self.check_token(Token::Semicolon) {
                                self.advance();
                            }
                        },
                        Ok(None) => self.advance(),
                        Err(e) => {
                            self.errors.push(e);
                            self.synchronize();
                        }
                    },
                },
                Err(_) => break,
            }
            self.skip_whitespace();
        }

        if self.errors.is_empty() {
            Ok(McDocFile {
                imports,
                declarations,
            })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    // ================================
    // HELPER METHODS
    // ================================

    fn current_pos(&self) -> Position {
        self.tokens
            .get(self.current)
            .map(|t| t.position)
            .unwrap_or_default()
    }

    fn syntax_error(&self, expected: impl Into<String>, found: impl Into<String>) -> ParseError {
        let pos = self.current_pos();
        ParseError::Syntax {
            expected: expected.into(),
            found: found.into(),
            pos: SourcePos { line: pos.line, column: pos.column }
        }
    }

    fn skip_whitespace(&mut self) {
        while let Ok(token) = self.current_token() {
            if matches!(
                token.token,
                Token::Whitespace | Token::Newline | Token::LineComment(_) | Token::BlockComment(_)
            ) {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current_token(&self) -> Result<&TokenWithPos<'input>, ParseError> {
        self.tokens
            .get(self.current)
            .ok_or_else(|| self.syntax_error("token", "EOF"))
    }

    fn check_token(&self, token_type: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        // Use std::mem::discriminant to compare enum variants without their data
        std::mem::discriminant(&self.current_token().unwrap().token) == std::mem::discriminant(&token_type)
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || 
        (self.current < self.tokens.len() && 
         matches!(self.tokens[self.current].token, Token::Eof))
    }

    fn consume(&mut self, expected_token: Token, error_msg: &str) -> Result<(), ParseError> {
        self.skip_whitespace();
        if self.check_token(expected_token) {
            self.advance();
            Ok(())
        } else {
            Err(self.syntax_error(error_msg, format!("{:?}", self.current_token().unwrap().token)))
        }
    }

    fn current_identifier(&mut self) -> Result<&'input str, ParseError> {
        self.skip_whitespace();
        let token_with_pos = self.current_token()?.clone();
        match token_with_pos.token {
            Token::Identifier(s) => {
                self.advance();
                Ok(s)
            },
            // Allow keywords as field names
            Token::Type => {
                self.advance();
                Ok("type")
            },
            Token::Struct => {
                self.advance();
                Ok("struct")
            },
            Token::Enum => {
                self.advance();
                Ok("enum")
            },
            Token::Dispatch => {
                self.advance();
                Ok("dispatch")
            },
            Token::Use => {
                self.advance();
                Ok("use")
            },
            Token::To => {
                self.advance();
                Ok("to")
            },
            Token::Super => {
                self.advance();
                Ok("super")
            },
            Token::True => {
                self.advance();
                Ok("true")
            },
            Token::False => {
                self.advance();
                Ok("false")
            },
            _ => Err(self.syntax_error(
                "identifier",
                format!("{:?}", token_with_pos.token),
            )),
        }
    }

    /// Parse special identifiers that can include patterns like %unknown, %key
    fn current_identifier_or_special(&mut self) -> Result<&'input str, ParseError> {
        self.skip_whitespace();
        let token_with_pos = self.current_token()?.clone();
        match token_with_pos.token {
            Token::Identifier(s) => {
                self.advance();
                Ok(s)
            },
            Token::Percent => {
                // Handle %unknown, %key patterns
                self.advance(); // consume %
                
                // Get the identifier after %
                if let Ok(next_token) = self.current_token() {
                    if let Token::Identifier(name) = next_token.token {
                        self.advance(); // consume the identifier
                        // For now, return the name without % for simplicity
                        // Later we can extend this to return the full pattern
                        Ok(name)
                    } else {
                        Err(self.syntax_error("identifier after %", format!("{:?}", next_token.token)))
                    }
                } else {
                    Err(self.syntax_error("identifier after %", "end of input"))
                }
            },
            // Allow keywords as special identifiers too
            Token::Type => {
                self.advance();
                Ok("type")
            },
            Token::Struct => {
                self.advance();
                Ok("struct")
            },
            Token::Enum => {
                self.advance();
                Ok("enum")
            },
            Token::Dispatch => {
                self.advance();
                Ok("dispatch")
            },
            Token::Use => {
                self.advance();
                Ok("use")
            },
            Token::To => {
                self.advance();
                Ok("to")
            },
            Token::Super => {
                self.advance();
                Ok("super")
            },
            Token::True => {
                self.advance();
                Ok("true")
            },
            Token::False => {
                self.advance();
                Ok("false")
            },
            _ => Err(self.syntax_error(
                "identifier or special pattern",
                format!("{:?}", token_with_pos.token),
            )),
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.check_token(Token::Newline) {
                return;
            }
            match self.current_token().unwrap().token {
                Token::Struct | Token::Enum | Token::Type | Token::Dispatch | Token::Use => return,
                _ => self.advance(),
            }
        }
    }

    // ================================
    // MAIN PARSING LOGIC
    // ================================

    pub fn parse_import(&mut self) -> Result<ImportStatement<'input>, ParseError> {
        let pos = self.current_pos();
        self.consume(Token::Use, "Expected 'use'")?;
        let path = self.parse_import_path()?;
        Ok(ImportStatement {
            path,
            position: pos,
        })
    }

    fn parse_import_path(&mut self) -> Result<ImportPath<'input>, ParseError> {
        let mut segments = Vec::new();
        let mut is_relative = false;

        self.skip_whitespace();

        if self.check_token(Token::DoubleColon) {
            self.advance(); // consume ::
        } else if self.check_token(Token::Super) {
            is_relative = true;
            self.advance();
            self.consume(Token::DoubleColon, "Expected '::' after 'super'")?;
        }

        loop {
            segments.push(self.current_identifier()?);
            
            if self.check_token(Token::DoubleColon) {
                self.advance();
            } else {
                break;
            }
        }

        if is_relative {
            Ok(ImportPath::Relative(segments))
        } else {
            Ok(ImportPath::Absolute(segments))
        }
    }

    fn parse_declaration(&mut self) -> Result<Option<Declaration<'input>>, ParseError> {
        let annotations = self.parse_annotations()?;
        let pos = self.current_pos();

        self.skip_whitespace();
        if self.is_at_end() {
            return Ok(None);
        }

        let token = self.current_token()?.token.clone();
        match token {
            Token::Struct => Ok(Some(Declaration::Struct(
                self.parse_struct_declaration(annotations, pos)?,
            ))),
            Token::Enum => Ok(Some(Declaration::Enum(
                self.parse_enum_declaration(annotations, pos)?,
            ))),
            Token::Type => Ok(Some(Declaration::Type(
                self.parse_type_declaration(annotations, pos)?,
            ))),
            Token::Dispatch => Ok(Some(Declaration::Dispatch(
                self.parse_dispatch_declaration(annotations, pos)?,
            ))),
            _ => {
                if annotations.is_empty() {
                    let found = format!("{:?}", self.current_token()?.token);
                    self.errors
                        .push(self.syntax_error("declaration keyword", found));
                    self.synchronize();
                    Ok(None)
                } else {
                    Err(self.syntax_error("declaration keyword", "annotations only"))
                }
            }
        }
    }

    fn parse_annotations(&mut self) -> Result<Vec<Annotation<'input>>, ParseError> {
        let mut annotations = Vec::new();
        
        while let Ok(token) = self.current_token() {
            if let Token::Annotation(text) = token.token.clone() {
                let pos = token.position;
                self.advance();
                
                // Simple annotation parsing: #[name(key=value)] or #[name=value] or #[name]
                let annotation_text = text.trim_start_matches("#[").trim_end_matches(']');
                let (name, data) = if let Some(paren_pos) = annotation_text.find('(') {
                // Complex: #[name(key=value)]
                let name = annotation_text[..paren_pos].trim();
                let params_text = annotation_text[paren_pos + 1..].trim_end_matches(')');
                
                let mut map = FxHashMap::default();
                for param in params_text.split(',') {
                    if let Some(eq_pos) = param.find('=') {
                        let key = param[..eq_pos].trim();
                        let value = param[eq_pos + 1..].trim_matches('"');
                        map.insert(key, value);
                    }
                }
                (name, AnnotationData::Complex(map))
            } else if let Some(eq_pos) = annotation_text.find('=') {
                // Simple: #[name=value]
                let name = annotation_text[..eq_pos].trim();
                let value = annotation_text[eq_pos + 1..].trim_matches('"');
                (name, AnnotationData::Simple(value))
            } else {
                // Empty: #[name]
                (annotation_text, AnnotationData::Empty)
            };
                
                annotations.push(Annotation {
                    name,
                    data,
                    position: pos,
                });
            } else {
                break;
            }
        }
        
        Ok(annotations)
    }

    pub fn parse_struct_declaration(
        &mut self,
        annotations: Vec<Annotation<'input>>,
        pos: Position,
    ) -> Result<StructDeclaration<'input>, ParseError> {
        self.consume(Token::Struct, "Expected 'struct'")?;
        let name = self.current_identifier()?;
        
        self.consume(Token::LeftBrace, "Expected '{' to start struct body")?;
        let mut members = Vec::new();
        self.skip_whitespace();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            members.push(self.parse_struct_member()?);
            self.skip_whitespace();
        }
        self.consume(Token::RightBrace, "Expected '}' to end struct body")?;

        Ok(StructDeclaration {
            name,
            members,
            annotations,
            position: pos,
        })
    }

    fn parse_struct_member(&mut self) -> Result<StructMember<'input>, ParseError> {
        self.skip_whitespace(); // Skip any whitespace before parsing
        
        // Parse annotations first (they can apply to both spreads and fields)
        let annotations = self.parse_annotations()?;
        
        // CORRECTION: Skip whitespace after parsing annotations to properly position cursor
        self.skip_whitespace();
        
        // Check if it's a spread operator
        if self.check_token(Token::DotDotDot) {
            self.advance(); // consume ...
            
            // The spread can be followed by:
            // 1. A type expression like `struct { field: type }`
            // 2. A namespace::identifier like `super::ItemBase` or `minecraft::item`
            
            if self.check_token(Token::Struct) {
                // This is a spread of a struct type: ...struct { ... }
                // Parse the struct directly instead of calling parse_type_expression
                // to avoid double-parsing annotations
                self.advance(); // consume 'struct'
                self.consume(Token::LeftBrace, "Expected '{' after 'struct'")?;
                
                let mut members = Vec::new();
                self.skip_whitespace();
                
                while !self.check_token(Token::RightBrace) && !self.is_at_end() {
                    let member = self.parse_struct_member()?;
                    members.push(member);
                    self.skip_whitespace();
                }
                
                self.consume(Token::RightBrace, "Expected '}' to end struct body")?;
                
                // Skip any trailing comma
                if self.check_token(Token::Comma) {
                    self.advance();
                }
                
                // Create a struct type expression and return it as a spread
                // For now we treat spread structs as simple spreads
                return Ok(StructMember::Spread(SpreadExpression {
                    namespace: "",  // No namespace for inline structs
                    registry: "",   // No registry for inline structs  
                    dynamic_key: None,
                    annotations,
                    position: self.current_pos(),
                }));
            } else {
                // Smart parsing: detect different spread patterns
                let (namespace, registry) = if self.check_token(Token::Super) || self.check_token(Token::DoubleColon) {
                    // Handle import path: super::ItemBase or ::absolute::path
                    let namespace = if self.check_token(Token::Super) {
                        self.advance(); // consume super
                        self.consume(Token::DoubleColon, "Expected '::' after 'super'")?;
                        "super"
                    } else if self.check_token(Token::DoubleColon) {
                        self.advance(); // consume ::
                        ""
                    } else {
                        ""
                    };
                    
                    let registry = self.current_identifier()?;
                    (namespace, registry)
                } else {
                    // Check if it's a namespace:registry pattern or generic type
                    let name = self.current_identifier()?;
                    
                    if self.check_token(Token::Colon) {
                        // Pattern: minecraft:test_instance[[type]]
                        self.advance(); // consume :
                        let registry = self.current_identifier()?;
                        (name, registry)
                    } else if self.check_token(Token::Less) {
                        // Pattern: Layer<T> - parse as generic type
                        self.current = self.current.saturating_sub(1); // Back up to reparse
                        let spread_type = self.parse_single_type()?;
                        
                        match spread_type {
                            TypeExpression::Generic { name, .. } => (name, ""),
                            TypeExpression::Simple(name) => (name, ""),
                            _ => ("", "")
                        }
                    } else {
                        // Simple name
                        (name, "")
                    }
                };
                
                // Handle dynamic reference like [[type]] or [[%key]]
                let dynamic_key = if self.check_token(Token::LeftBracket) && 
                   self.tokens.get(self.current + 1).map(|t| &t.token) == Some(&Token::LeftBracket) {
                    self.advance(); // consume first [
                    self.advance(); // consume second [
                    
                    // Allow % patterns and identifiers in dynamic references
                    let key = self.current_identifier_or_special()?;
                    
                    self.consume(Token::RightBracket, "Expected ']' in dynamic reference")?;
                    self.consume(Token::RightBracket, "Expected ']]' in dynamic reference")?;
                    
                    Some(DynamicReference {
                        reference: DynamicReferenceType::Field(key),
                        position: self.current_pos(),
                    })
                } else {
                    None
                };
                
                // Skip any trailing comma
                if self.check_token(Token::Comma) {
                    self.advance();
                }
                
                Ok(StructMember::Spread(SpreadExpression {
                    namespace,
                    registry,
                    dynamic_key,
                    annotations,
                    position: self.current_pos(),
                }))
            }
        } else if self.check_token(Token::LeftBracket) {
            // Parse dynamic field: [#[id="mob_effect"] string]: MobEffectPredicate
            let pos = self.current_pos();
            self.advance(); // consume [
            
            // Parse the key type (e.g., #[id="mob_effect"] string)
            let key_type = self.parse_type_expression()?;
            
            self.consume(Token::RightBracket, "Expected ']' after dynamic field key type")?;
            
            let optional = if self.check_token(Token::Question) {
                self.advance();
                true
            } else {
                false
            };

            self.consume(Token::Colon, "Expected ':' after dynamic field key")?;

            // Parse value type
            let value_type = self.parse_type_expression()?;

            if self.check_token(Token::Comma) {
                self.advance();
            }

            Ok(StructMember::DynamicField(DynamicFieldDeclaration {
                key_type,
                value_type,
                optional,
                annotations,
                position: pos,
            }))
        } else {
            // Parse as regular field - but we already have annotations, so pass them
            let pos = self.current_pos();
            let name = self.current_identifier()?;
            
            let optional = if self.check_token(Token::Question) {
                self.advance();
                true
            } else {
                false
            };

            self.consume(Token::Colon, "Expected ':' after field name")?;

            // Parse type annotations (like #[id(...)] before the type)
            let type_annotations = self.parse_annotations()?;
            
            let field_type = self.parse_type_expression()?;

            if self.check_token(Token::Comma) {
                self.advance();
            }

            // Combine field annotations and type annotations
            let mut all_annotations = annotations;
            all_annotations.extend(type_annotations);

            Ok(StructMember::Field(FieldDeclaration {
                name,
                field_type,
                optional,
                annotations: all_annotations,
                position: pos,
            }))
        }
    }

    #[allow(dead_code)]
    fn parse_field_declaration(&mut self) -> Result<FieldDeclaration<'input>, ParseError> {
        let field_annotations = self.parse_annotations()?;
        let pos = self.current_pos();
        let name = self.current_identifier()?;
        
        let optional = if self.check_token(Token::Question) {
            self.advance();
            true
        } else {
            false
        };

        self.consume(Token::Colon, "Expected ':' after field name")?;

        // Parse type annotations (like #[id(...)] before the type)
        let type_annotations = self.parse_annotations()?;
        
        let field_type = self.parse_type_expression()?;

        if self.check_token(Token::Comma) {
            self.advance();
        }

        // Combine field annotations and type annotations
        let mut all_annotations = field_annotations;
        all_annotations.extend(type_annotations);

        Ok(FieldDeclaration {
            name,
            field_type,
            optional,
            annotations: all_annotations,
            position: pos,
        })
    }

    pub fn parse_type_expression(&mut self) -> Result<TypeExpression<'input>, ParseError> {
        let mut type_expr = self.parse_single_type()?;

        // Check for constraints on simple types: int @ 1..10
        if self.check_token(Token::At) {
            self.advance(); // consume @
            let _constraints = self.parse_array_constraints()?;
            // For now, we ignore constraints on simple types and just return the type
            // In a full implementation, we'd extend TypeExpression to support constraints
        }

        // Check for array type with optional constraints: [element_type] @ 1..10
        if self.check_token(Token::LeftBracket) {
            self.advance(); // consume [
            self.consume(Token::RightBracket, "Expected ']' after type in array declaration")?;
            
            // Parse optional constraints: @ 1..10 or @ 5.. or @ ..5
            let constraints = if self.check_token(Token::At) {
                self.advance(); // consume @
                self.parse_array_constraints()?
            } else {
                None
            };

            type_expr = TypeExpression::Array {
                element_type: Box::new(type_expr),
                constraints,
            };
        }

        // Check for union type
        if self.check_token(Token::Pipe) {
            self.advance();
            let mut types = vec![type_expr];

            loop {
                // Skip optional trailing pipe before closing paren/brace
                self.skip_whitespace();
                if self.check_token(Token::RightParen) || self.check_token(Token::RightBrace) || 
                   self.check_token(Token::Comma) || self.is_at_end() {
                    break;
                }
                
                types.push(self.parse_single_type()?);
                self.skip_whitespace();
                if self.check_token(Token::Pipe) {
                    self.advance();
                } else {
                    break;
                }
            }

            type_expr = TypeExpression::Union(types);
        }

        Ok(type_expr)
    }

    /// Parse array constraints like 1..10, 5.., ..5, or just 5
    fn parse_array_constraints(&mut self) -> Result<Option<ArrayConstraints>, ParseError> {
        let token = self.current_token()?.token.clone();
        
        match token {
            Token::Number(num) => {
                self.advance();
                
                // Check if it's a range: 5..10 or 5..
                if self.check_token(Token::DotDot) {
                    self.advance(); // consume ..
                    
                    let max = if !self.is_at_end() {
                        if let Ok(next_token) = self.current_token() {
                            if let Token::Number(n) = &next_token.token {
                                let num = *n;
                                self.advance();
                                Some(num as u32)
                            } else {
                                None // No max specified: 5..
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    Ok(Some(ArrayConstraints {
                        min: Some(num as u32),
                        max,
                    }))
                } else {
                    // Just a single number: exactly this count
                    Ok(Some(ArrayConstraints {
                        min: Some(num as u32),
                        max: Some(num as u32),
                    }))
                }
            }
            Token::DotDot => {
                // Range starting from beginning: ..10
                self.advance(); // consume ..
                
                if !self.is_at_end() {
                    if let Ok(next_token) = self.current_token() {
                        if let Token::Number(n) = &next_token.token {
                            let num = *n;
                            self.advance();
                            Ok(Some(ArrayConstraints {
                                min: None,
                                max: Some(num as u32),
                            }))
                        } else {
                            Err(self.syntax_error("number after '..'", format!("{:?}", next_token.token)))
                        }
                    } else {
                        Err(self.syntax_error("number after '..'", "end of input"))
                    }
                } else {
                    Err(self.syntax_error("number after '..'", "end of input"))
                }
            }
            _ => {
                // No valid constraint found
                Ok(None)
            }
        }
    }

    pub fn parse_enum_declaration(
        &mut self,
        annotations: Vec<Annotation<'input>>,
        pos: Position,
    ) -> Result<EnumDeclaration<'input>, ParseError> {
        self.consume(Token::Enum, "Expected 'enum'")?;
        
        // Support both syntaxes: enum(string) Test and enum Test: string
        let (base_type, name) = if self.check_token(Token::LeftParen) {
            // enum(string) Test
            self.advance();
            let bt = self.current_identifier()?;
            self.consume(Token::RightParen, "Expected ')' after enum base type")?;
            let name = self.current_identifier()?;
            (Some(bt), name)
        } else {
            // enum Test: string or enum Test
            let name = self.current_identifier()?;
            let base_type = if self.check_token(Token::Colon) {
                self.advance();
                Some(self.current_identifier()?)
            } else {
                None
            };
            (base_type, name)
        };
        
        self.consume(Token::LeftBrace, "Expected '{' to start enum body")?;
        let mut variants = Vec::new();
        self.skip_whitespace();
        while !self.check_token(Token::RightBrace) && !self.is_at_end() {
            let var_annotations = self.parse_annotations()?;
            let var_pos = self.current_pos();
            let var_name = self.current_identifier()?;
            
            let value = if self.check_token(Token::Equal) {
                self.advance();
                let token = self.current_token()?.token.clone();
                let lit = match token {
                    Token::String(s) => LiteralValue::String(s),
                    Token::Number(n) => LiteralValue::Number(n),
                    Token::True => LiteralValue::Boolean(true),
                    Token::False => LiteralValue::Boolean(false),
                    _ => {
                        return Err(self
                            .syntax_error("literal", "other"))
                    }
                };
                self.advance();
                Some(lit)
            } else {
                None
            };

            variants.push(EnumVariant {
                name: var_name,
                value,
                annotations: var_annotations,
                position: var_pos,
            });

            if self.check_token(Token::Comma) {
                self.advance();
            }
            self.skip_whitespace();
        }
        self.consume(Token::RightBrace, "Expected '}' to end enum body")?;

        Ok(EnumDeclaration {
            name,
            base_type,
            variants,
            annotations,
            position: pos,
        })
    }

    pub fn parse_type_declaration(
        &mut self,
        annotations: Vec<Annotation<'input>>,
        pos: Position,
    ) -> Result<TypeDeclaration<'input>, ParseError> {
        self.consume(Token::Type, "Expected 'type'")?;
        let name = self.current_identifier()?;
        
        // Parse generic parameters if present: <T, U, V>
        let type_params = if self.check_token(Token::Less) {
            self.advance(); // consume <
            let mut params = Vec::new();
            
            loop {
                let param = self.current_identifier()?;
                params.push(param);
                
                if self.check_token(Token::Comma) {
                    self.advance(); // consume comma
                    self.skip_whitespace(); // skip space after comma
                } else {
                    break;
                }
            }
            
            self.consume(Token::Greater, "Expected '>' after generic parameters")?;
            params
        } else {
            Vec::new()
        };
        
        self.consume(Token::Equal, "Expected '=' after type name")?;

        let type_expr = self.parse_type_expression()?;

        Ok(TypeDeclaration {
            name,
            type_params,
            type_expr,
            annotations,
            position: pos,
        })
    }

    pub fn parse_dispatch_declaration(
        &mut self,
        annotations: Vec<Annotation<'input>>,
        pos: Position,
    ) -> Result<DispatchDeclaration<'input>, ParseError> {
        self.consume(Token::Dispatch, "Expected 'dispatch'")?;
        
        // Parse registry path (e.g., "minecraft:resource[test_recipe]")
        let registry = self.current_identifier()?;
        self.consume(Token::Colon, "Expected ':'")?;
        let _path = self.current_identifier()?;
        
        let key = if self.check_token(Token::LeftBracket) {
            self.advance();
            self.skip_whitespace(); // Skip whitespace after opening bracket
            
            // Parse key name - can be identifier, string literal, or %pattern
            let key_name = match &self.current_token()?.token {
                Token::Identifier(name) => {
                    let result = *name;
                    self.advance();
                    result
                }
                Token::String(value) => {
                    let result = *value;
                    self.advance();
                    result
                }
                Token::Percent => {
                    // Handle %unknown, %key patterns
                    self.current_identifier_or_special()?
                }
                _ => return Err(self.syntax_error("identifier, string, or % pattern", format!("{:?}", self.current_token()?.token)))
            };
            
            // Skip additional targets for now (multiple dispatch keys)
            while self.check_token(Token::Comma) {
                self.advance();
                self.skip_whitespace(); // Skip whitespace and newlines after comma
                match &self.current_token()?.token {
                    Token::Identifier(_) | Token::String(_) => {
                        self.advance();
                        self.skip_whitespace(); // Skip whitespace after identifier
                    }
                    Token::Percent => {
                        // Handle % patterns in multiple targets
                        self.current_identifier_or_special()?;
                        self.skip_whitespace();
                    }
                    _ => return Err(self.syntax_error("identifier, string, or % pattern", format!("{:?}", self.current_token()?.token)))
                }
            }
            
            self.skip_whitespace(); // Skip whitespace before closing bracket
            self.consume(Token::RightBracket, "Expected ']'")?;
            Some(key_name)
        } else {
            None
        };

        self.consume(Token::To, "Expected 'to'")?;
        
        // Parse the target type expression
        let target_type = self.parse_type_expression()?;

        Ok(DispatchDeclaration {
            source: DispatchSource {
                registry,
                key,
                position: pos,
            },
            targets: vec![], // TODO: parse targets
            target_type,
            annotations,
            position: pos,
        })
    }

    pub fn parse_single_type(&mut self) -> Result<TypeExpression<'input>, ParseError> {
        self.skip_whitespace();
        
        // Parse annotations before the type (for cases like #[regex_pattern] string)
        let _type_annotations = self.parse_annotations()?;
        
        // CRITICAL FIX: Skip whitespace/newlines after annotations
        self.skip_whitespace();
        
        match &self.current_token()?.token {
            Token::Identifier(name) => {
                let type_name = *name;
                self.advance();
                
                // Check for namespace reference: mcdoc:block_states
                if self.check_token(Token::Colon) {
                    self.advance(); // consume :
                    let registry = self.current_identifier()?;
                    
                    // Check for dynamic reference: [[block]] or [[%key]]
                    if self.check_token(Token::LeftBracket) && 
                       self.tokens.get(self.current + 1).map(|t| &t.token) == Some(&Token::LeftBracket) {
                        self.advance(); // consume first [
                        self.advance(); // consume second [
                        
                        // Allow % patterns in dynamic references
                        let key = self.current_identifier_or_special()?;
                        
                        self.consume(Token::RightBracket, "Expected ']' in dynamic reference")?;
                        self.consume(Token::RightBracket, "Expected ']]' in dynamic reference")?;
                        
                        Ok(TypeExpression::Spread(SpreadExpression {
                            namespace: type_name,
                            registry,
                            dynamic_key: Some(DynamicReference {
                                reference: DynamicReferenceType::Field(key),
                                position: self.current_pos(),
                            }),
                            annotations: Vec::new(),
                            position: self.current_pos(),
                        }))
                    }
                    // Check for simple dispatch reference: minecraft:block_entity[moving_piston]
                    else if self.check_token(Token::LeftBracket) {
                        self.advance(); // consume [
                        let key = self.current_identifier()?;
                        self.consume(Token::RightBracket, "Expected ']' in dispatch reference")?;
                        
                        // This is a dispatch reference, return as simple reference with the key
                        Ok(TypeExpression::Reference(ImportPath::Absolute(vec![type_name, registry, key])))
                    } else {
                        // Simple namespace reference
                        Ok(TypeExpression::Reference(ImportPath::Absolute(vec![type_name, registry])))
                    }
                }
                // Check for generic type: Map<string, int>
                else if self.check_token(Token::Less) {
                    self.advance(); // consume <
                    let mut type_args = Vec::new();
                    
                    loop {
                        type_args.push(self.parse_single_type()?);
                        
                        if self.check_token(Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    
                    self.consume(Token::Greater, "Expected '>' after generic arguments")?;
                    
                    Ok(TypeExpression::Generic {
                        name: type_name,
                        type_args,
                    })
                } else {
                    // Handle simple type with potential array constraints: int[] @ 4
                    let mut type_expr = TypeExpression::Simple(type_name);
                    
                    // Check for array type with optional constraints: [element_type] @ 1..10
                    if self.check_token(Token::LeftBracket) {
                        self.advance(); // consume [
                        self.consume(Token::RightBracket, "Expected ']' after type in array declaration")?;
                        
                        // Parse optional constraints: @ 1..10 or @ 5.. or @ ..5
                        let constraints = if self.check_token(Token::At) {
                            self.advance(); // consume @
                            self.parse_array_constraints()?
                        } else {
                            None
                        };

                        type_expr = TypeExpression::Array {
                            element_type: Box::new(type_expr),
                            constraints,
                        };
                    }
                    
                    Ok(type_expr)
                }
            }
            Token::DotDotDot => {
                // Spread operator: ...minecraft:item
                self.advance(); // consume ...
                let namespace = self.current_identifier()?;
                self.consume(Token::Colon, "Expected ':' after namespace in spread")?;
                let registry = self.current_identifier()?;
                
                Ok(TypeExpression::Spread(SpreadExpression {
                    namespace,
                    registry,
                    dynamic_key: None,
                    annotations: Vec::new(), // No annotations in type context
                    position: self.current_pos(),
                }))
            }
            Token::LeftBracket => {
                // Array type [element_type] @ constraints? ou [element_type @ internal_constraints] @ external_constraints?
                self.advance(); // consume [
                let mut element_type = self.parse_single_type()?;
                
                // Gérer les contraintes internes à l'élément : [float @ -80..80]
                if self.check_token(Token::At) {
                    self.advance(); // consume @
                    let internal_constraints = self.parse_type_constraints()?;
                    
                    if let Some(constraints) = internal_constraints {
                        element_type = TypeExpression::Constrained {
                            base_type: Box::new(element_type),
                            constraints,
                        };
                    }
                }
                
                self.consume(Token::RightBracket, "Expected ']' after array element type")?;
                
                // Parse optional external constraints: @ 1..10 or @ 5.. or @ ..5  
                let constraints = if self.check_token(Token::At) {
                    self.advance(); // consume @
                    self.parse_array_constraints()?
                } else {
                    None
                };
                
                Ok(TypeExpression::Array {
                    element_type: Box::new(element_type),
                    constraints,
                })
            }
            Token::Struct => {
                self.advance(); // consume 'struct'
                
                // Check if there's a struct name or immediate {
                if let Ok(token) = self.current_token() {
                    match &token.token {
                        Token::Identifier(name) => {
                            // Named struct: struct TestRecipe { ... }
                            let _struct_name = *name;
                            self.advance(); // consume struct name
                            self.consume(Token::LeftBrace, "Expected '{' after struct name")?;
                            
                            let mut members = Vec::new();
                            self.skip_whitespace();
                            
                            while !self.check_token(Token::RightBrace) && !self.is_at_end() {
                                let member = self.parse_struct_member()?;
                                members.push(member);
                                self.skip_whitespace();
                            }
                            
                            self.consume(Token::RightBrace, "Expected '}' to end struct body")?;
                            // For now, treat named struct same as anonymous struct
                            Ok(TypeExpression::Struct(members))
                        }
                        Token::LeftBrace => {
                            // Anonymous struct: struct { ... }
                            self.consume(Token::LeftBrace, "Expected '{' after 'struct'")?;
                            
                            let mut members = Vec::new();
                            self.skip_whitespace();
                            
                            while !self.check_token(Token::RightBrace) && !self.is_at_end() {
                                let member = self.parse_struct_member()?;
                                members.push(member);
                                self.skip_whitespace();
                            }
                            
                            self.consume(Token::RightBrace, "Expected '}' to end struct body")?;
                            Ok(TypeExpression::Struct(members))
                        }
                        _ => Err(self.syntax_error("struct name or '{'", format!("{:?}", token.token)))
                    }
                } else {
                    Err(self.syntax_error("struct body", "end of input"))
                }
            }
            Token::LeftParen => {
                // Parenthesized type expression: (type1 | type2)
                self.advance(); // consume (
                let type_expr = self.parse_type_expression()?; // Parse the inner type expression
                self.consume(Token::RightParen, "Expected ')' after parenthesized type")?;
                Ok(type_expr)
            }
            Token::String(s) => {
                // String literal type constraint: #[id="test"] "literal_value"
                let value = *s;
                self.advance();
                Ok(TypeExpression::Literal(LiteralValue::String(value)))
            }
            Token::Number(n) => {
                // Number literal type constraint: #[id="test"] 42
                let value = *n;
                self.advance();
                Ok(TypeExpression::Literal(LiteralValue::Number(value)))
            }
            Token::True => {
                // Boolean literal type constraint: #[id="test"] true
                self.advance();
                Ok(TypeExpression::Literal(LiteralValue::Boolean(true)))
            }
            Token::False => {
                // Boolean literal type constraint: #[id="test"] false
                self.advance();
                Ok(TypeExpression::Literal(LiteralValue::Boolean(false)))
            }
            _ => Err(self.syntax_error("type", format!("{:?}", self.current_token()?.token)))
        }
    }

    /// Parse type constraints like @ -80..80, @ 5.., @ ..5, or @ 5
    fn parse_type_constraints(&mut self) -> Result<Option<TypeConstraints>, ParseError> {
        let token = self.current_token()?.token.clone();
        
        match token {
            Token::Number(num) => {
                self.advance();
                
                // Check if it's a range: -80..80 or 5..
                if self.check_token(Token::DotDot) {
                    self.advance(); // consume ..
                    
                                         let max = if !self.is_at_end() {
                         if let Ok(next_token) = self.current_token() {
                             if let Token::Number(n) = &next_token.token {
                                 let num = *n;
                                 self.advance();
                                 Some(num)
                             } else {
                                 None // No max specified: 5..
                             }
                         } else {
                             None
                         }
                     } else {
                         None
                     };
                    
                    Ok(Some(TypeConstraints {
                        min: Some(num),
                        max,
                    }))
                } else {
                    // Just a single number: exactly this value
                    Ok(Some(TypeConstraints {
                        min: Some(num),
                        max: Some(num),
                    }))
                }
            }
            Token::DotDot => {
                // Range starting from beginning: ..80
                self.advance(); // consume ..
                
                if !self.is_at_end() {
                    if let Ok(next_token) = self.current_token() {
                        if let Token::Number(n) = &next_token.token {
                            let num = *n;
                            self.advance();
                            Ok(Some(TypeConstraints {
                                min: None,
                                max: Some(num),
                            }))
                        } else {
                            Err(self.syntax_error("number after '..'", format!("{:?}", next_token.token)))
                        }
                    } else {
                        Err(self.syntax_error("number after '..'", "end of input"))
                    }
                } else {
                    Err(self.syntax_error("number after '..'", "end of input"))
                }
            }
            _ => {
                // No valid constraint found
                Ok(None)
            }
        }
    }
} 