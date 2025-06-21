//! Parser MCDOC unifié

use crate::lexer::{Token, TokenWithPos, Position};
use crate::error::{ParseError, SourcePos};
use rustc_hash::FxHashMap;

// ================================
// AST STRUCTURES ESSENTIELLES
// ================================

/// Fichier MCDOC principal
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

/// Déclarations top-level
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration<'input> {
    Struct(StructDeclaration<'input>),
    Enum(EnumDeclaration<'input>),
    Type(TypeDeclaration<'input>),
    Dispatch(DispatchDeclaration<'input>),
}

/// Annotation consolidée
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
    pub fields: Vec<FieldDeclaration<'input>>,
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
    Struct(Vec<FieldDeclaration<'input>>),
    Generic {
        name: &'input str,
        type_args: Vec<TypeExpression<'input>>,
    },
    Reference(ImportPath<'input>),
    Spread(SpreadExpression<'input>),
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

// ================================
// PARSER IMPLEMENTATION
// ================================

/// Parser principal unifié
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

    /// Parse complet du fichier MCDOC
    pub fn parse(&mut self) -> Result<McDocFile<'input>, Vec<ParseError>> {
        let mut imports = Vec::new();
        let mut declarations = Vec::new();

        self.skip_whitespace();

        while !self.is_at_end() {
            match self.current_token() {
                Ok(token) => match &token.token {
                    Token::Use => {
                        match self.parse_import() {
                            Ok(import) => imports.push(import),
                            Err(e) => {
                                self.errors.push(e);
                                self.synchronize();
                            }
                        }
                    }
                    Token::Eof => break,
                    _ => {
                        match self.parse_declaration() {
                            Ok(Some(declaration)) => declarations.push(declaration),
                            Ok(None) => self.advance(),
                            Err(e) => {
                                self.errors.push(e);
                                self.synchronize();
                            }
                        }
                    }
                },
                Err(_) => break,
            }
            self.skip_whitespace();
        }

        if self.errors.is_empty() {
            Ok(McDocFile { imports, declarations })
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    // ================================
    // HELPER METHODS
    // ================================

    fn current_pos(&self) -> SourcePos {
        if self.current < self.tokens.len() {
            let pos = self.tokens[self.current].position;
            SourcePos::new(pos.line, pos.column)
        } else {
            SourcePos::new(0, 0)
        }
    }

    fn make_position(&self, pos: SourcePos) -> Position {
        Position {
            line: pos.line,
            column: pos.column,
            offset: 0,
        }
    }

    fn syntax_error(&self, expected: impl Into<String>, found: impl Into<String>) -> ParseError {
        ParseError::syntax(expected, found, self.current_pos())
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match &self.tokens[self.current].token {
                Token::Newline | Token::Whitespace => {
                    self.current += 1;
                }
                _ => break,
            }
        }
    }

    fn current_token(&self) -> Result<&TokenWithPos<'input>, ParseError> {
        self.tokens.get(self.current)
            .ok_or_else(|| ParseError::lexer("Unexpected end of input", self.current_pos()))
    }

    fn check_token(&self, token_type: &Token) -> bool {
        if let Ok(current) = self.current_token() {
            std::mem::discriminant(&current.token) == std::mem::discriminant(token_type)
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.current += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn consume(&mut self, expected_token: &Token, error_msg: &str) -> Result<(), ParseError> {
        if self.check_token(expected_token) {
            self.advance();
            Ok(())
        } else {
            let found = self.current_token()
                .map(|t| format!("{:?}", t.token))
                .unwrap_or_else(|_| "end of input".to_string());
            Err(self.syntax_error(error_msg, found))
        }
    }

    fn current_identifier(&self) -> Result<&'input str, ParseError> {
        match &self.current_token()?.token {
            Token::Identifier(name) => Ok(name),
            // Accept keywords as identifiers in certain contexts
            Token::Type => Ok("type"),
            Token::Use => Ok("use"),
            Token::Struct => Ok("struct"),
            Token::Enum => Ok("enum"),
            Token::Dispatch => Ok("dispatch"),
            Token::To => Ok("to"),
            Token::Super => Ok("super"),
            _ => {
                let found = format!("{:?}", self.current_token()?.token);
                Err(self.syntax_error("identifier", found))
            }
        }
    }

    fn synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            match &self.current_token().map(|t| &t.token) {
                Ok(Token::Struct) | Ok(Token::Enum) | Ok(Token::Type) | 
                Ok(Token::Dispatch) | Ok(Token::Use) => return,
                _ => self.advance(),
            }
        }
    }

    // ================================
    // PARSING METHODS
    // ================================

    pub fn parse_import(&mut self) -> Result<ImportStatement<'input>, ParseError> {
        let pos = self.current_pos();
        self.consume(&Token::Use, "use")?;
        self.skip_whitespace();

        let path = self.parse_import_path()?;
        self.skip_whitespace();

        Ok(ImportStatement { 
            path, 
            position: self.make_position(pos) 
        })
    }

    fn parse_import_path(&mut self) -> Result<ImportPath<'input>, ParseError> {
        let mut path_components = Vec::new();
        
        if self.check_token(&Token::DoubleColon) {
            // Absolute path: ::java::world::item::ItemStack
            self.advance();
            loop {
                let name = self.current_identifier()?;
                path_components.push(name);
                self.advance();

                if self.check_token(&Token::DoubleColon) {
                    self.advance();
                } else {
                    break;
                }
            }
            Ok(ImportPath::Absolute(path_components))
        } else if self.check_token(&Token::Super) {
            // Relative path: super::loot::LootCondition
            self.advance(); // consume super
            self.consume(&Token::DoubleColon, "::")?;
            
            loop {
                let name = self.current_identifier()?;
                path_components.push(name);
                self.advance();

                if self.check_token(&Token::DoubleColon) {
                    self.advance();
                } else {
                    break;
                }
            }
            Ok(ImportPath::Relative(path_components))
        } else {
            // Simple relative path
            let name = self.current_identifier()?;
            path_components.push(name);
            self.advance();
            Ok(ImportPath::Relative(path_components))
        }
    }

    fn parse_declaration(&mut self) -> Result<Option<Declaration<'input>>, ParseError> {
        let pos = self.current_pos();
        
        // Skip any whitespace before parsing annotations
        self.skip_whitespace();
        
        // Parse annotations
        let annotations = self.parse_annotations()?;
        
        // Skip whitespace after annotations
        self.skip_whitespace();
        
        match self.current_token()?.token {
            Token::Struct => {
                let struct_decl = self.parse_struct_declaration(annotations, pos)?;
                Ok(Some(Declaration::Struct(struct_decl)))
            }
            Token::Enum => {
                let enum_decl = self.parse_enum_declaration(annotations, pos)?;
                Ok(Some(Declaration::Enum(enum_decl)))
            }
            Token::Type => {
                let type_decl = self.parse_type_declaration(annotations, pos)?;
                Ok(Some(Declaration::Type(type_decl)))
            }
            Token::Dispatch => {
                let dispatch_decl = self.parse_dispatch_declaration(annotations, pos)?;
                Ok(Some(Declaration::Dispatch(dispatch_decl)))
            }
            _ => Ok(None),
        }
    }

    fn parse_annotations(&mut self) -> Result<Vec<Annotation<'input>>, ParseError> {
        let mut annotations = Vec::new();
        
        while self.check_token(&Token::Hash) {
            let pos = self.current_pos();
            self.advance(); // consume #
            self.consume(&Token::LeftBracket, "[")?;
            
            let name = self.current_identifier()?;
            self.advance();
            
            let data = if self.check_token(&Token::LeftParen) {
                // Complex annotation with parameters
                self.advance(); // consume (
                let mut params = FxHashMap::default();
                
                while !self.check_token(&Token::RightParen) {
                    let key = self.current_identifier()?;
                    self.advance();
                    self.consume(&Token::Equals, "=")?;
                    
                    let value = match &self.current_token()?.token {
                        Token::String(s) => *s,
                        Token::Identifier(i) => *i,
                        _ => return Err(self.syntax_error("string or identifier", "value")),
                    };
                    self.advance();
                    
                    params.insert(key, value);
                    
                    if self.check_token(&Token::Comma) {
                        self.advance();
                    }
                }
                self.consume(&Token::RightParen, ")")?;
                AnnotationData::Complex(params)
            } else if self.check_token(&Token::Equals) {
                // Simple annotation with value
                self.advance(); // consume =
                let value = match &self.current_token()?.token {
                    Token::String(s) => *s,
                    _ => return Err(self.syntax_error("string", "annotation value")),
                };
                self.advance();
                AnnotationData::Simple(value)
            } else {
                // Empty annotation
                AnnotationData::Empty
            };
            
            self.consume(&Token::RightBracket, "]")?;
            
            annotations.push(Annotation {
                name,
                data,
                position: self.make_position(pos),
            });
            
            self.skip_whitespace();
        }
        
        Ok(annotations)
    }

    pub fn parse_struct_declaration(&mut self, annotations: Vec<Annotation<'input>>, pos: SourcePos) -> Result<StructDeclaration<'input>, ParseError> {
        self.consume(&Token::Struct, "struct")?;
        self.skip_whitespace();
        
        let name = self.current_identifier()?;
        self.advance();
        self.skip_whitespace();
        
        self.consume(&Token::LeftBrace, "{")?;
        self.skip_whitespace();
        
        let mut fields = Vec::new();
        while !self.check_token(&Token::RightBrace) {
            let field = self.parse_field_declaration()?;
            fields.push(field);
            
            if self.check_token(&Token::Comma) {
                self.advance();
            }
            self.skip_whitespace();
        }
        
        self.consume(&Token::RightBrace, "}")?;
        
        Ok(StructDeclaration {
            name,
            fields,
            annotations,
            position: self.make_position(pos),
        })
    }

    fn parse_field_declaration(&mut self) -> Result<FieldDeclaration<'input>, ParseError> {
        let pos = self.current_pos();
        let annotations = self.parse_annotations()?;
        
        // Check for spread expression
        if self.check_token(&Token::DotDotDot) {
            self.advance(); // consume ...
            
            let namespace = self.current_identifier()?;
            self.advance();
            self.consume(&Token::Colon, ":")?;
            let registry = self.current_identifier()?;
            self.advance();
            
            // Parse optional dynamic key [[type]] or [[%key]]
            let dynamic_key = if self.check_token(&Token::LeftBracket) {
                self.advance(); // consume [
                self.consume(&Token::LeftBracket, "[")?;
                
                let reference = if self.check_token(&Token::Percent) {
                    self.advance(); // consume %
                    let key = self.current_identifier()?;
                    self.advance();
                    DynamicReferenceType::SpecialKey(key)
                } else {
                    let field = self.current_identifier()?;
                    self.advance();
                    DynamicReferenceType::Field(field)
                };
                
                self.consume(&Token::RightBracket, "]")?;
                self.consume(&Token::RightBracket, "]")?;
                
                Some(DynamicReference {
                    reference,
                    position: self.make_position(self.current_pos()),
                })
            } else {
                None
            };
            
            let field_type = TypeExpression::Spread(SpreadExpression {
                namespace,
                registry,
                dynamic_key,
                position: self.make_position(pos),
            });
            
            return Ok(FieldDeclaration {
                name: "", // Spread expressions don't have names
                field_type,
                optional: false,
                annotations,
                position: self.make_position(pos),
            });
        }
        
        let name = self.current_identifier()?;
        self.advance();
        
        let optional = if self.check_token(&Token::Question) {
            self.advance();
            true
        } else {
            false
        };
        
        self.consume(&Token::Colon, ":")?;
        let field_type = self.parse_type_expression()?;
        
        Ok(FieldDeclaration {
            name,
            field_type,
            optional,
            annotations,
            position: self.make_position(pos),
        })
    }

    pub fn parse_type_expression(&mut self) -> Result<TypeExpression<'input>, ParseError> {
        match &self.current_token()?.token {
            Token::Identifier(name) => {
                let name = *name;
                self.advance();
                Ok(TypeExpression::Simple(name))
            }
            Token::LeftBracket => {
                // Parse array type or dynamic reference [[%key]]
                self.advance(); // consume [
                
                if self.check_token(&Token::LeftBracket) {
                    // Dynamic reference [[%key]] or [[type]]
                    self.advance(); // consume second [
                    
                    let reference = if self.check_token(&Token::Percent) {
                        self.advance(); // consume %
                        let key = self.current_identifier()?;
                        self.advance();
                        DynamicReferenceType::SpecialKey(key)
                    } else {
                        let field = self.current_identifier()?;
                        self.advance();
                        DynamicReferenceType::Field(field)
                    };
                    
                    self.consume(&Token::RightBracket, "]")?;
                    self.consume(&Token::RightBracket, "]")?;
                    
                    let _dynamic_ref = DynamicReference {
                        reference,
                        position: self.make_position(self.current_pos()),
                    };
                    
                    Ok(TypeExpression::Reference(ImportPath::Absolute(vec!["dynamic"])))
                } else {
                    // Regular array [ElementType]
                    let element_type = Box::new(self.parse_type_expression()?);
                    self.consume(&Token::RightBracket, "]")?;
                    
                    Ok(TypeExpression::Array {
                        element_type,
                        constraints: None,
                    })
                }
            }
            Token::DotDotDot => {
                // Spread expression: ...minecraft:recipe_serializer[[type]]
                self.advance(); // consume ...
                
                let namespace = self.current_identifier()?;
                self.advance();
                self.consume(&Token::Colon, ":")?;
                let registry = self.current_identifier()?;
                self.advance();
                
                let _base_path = format!("{}:{}", namespace, registry);
                
                // Parse optional dynamic key [[type]] or [[%key]]
                let dynamic_key = if self.check_token(&Token::LeftBracket) {
                    self.advance(); // consume [
                    self.consume(&Token::LeftBracket, "[")?;
                    
                    let reference = if self.check_token(&Token::Percent) {
                        self.advance(); // consume %
                        let key = self.current_identifier()?;
                        self.advance();
                        DynamicReferenceType::SpecialKey(key)
                    } else {
                        let field = self.current_identifier()?;
                        self.advance();
                        DynamicReferenceType::Field(field)
                    };
                    
                    self.consume(&Token::RightBracket, "]")?;
                    self.consume(&Token::RightBracket, "]")?;
                    
                    Some(DynamicReference {
                        reference,
                        position: self.make_position(self.current_pos()),
                    })
                } else {
                    None
                };
                
                Ok(TypeExpression::Spread(SpreadExpression {
                    namespace,
                    registry,
                    dynamic_key,
                    position: self.make_position(self.current_pos()),
                }))
            }
            Token::LeftParen => {
                // Union type: (string | int | boolean)
                self.advance(); // consume (
                let mut types = Vec::new();
                
                loop {
                    types.push(self.parse_type_expression()?);
                    
                    if self.check_token(&Token::Pipe) {
                        self.advance(); // consume |
                    } else {
                        break;
                    }
                }
                
                self.consume(&Token::RightParen, ")")?;
                Ok(TypeExpression::Union(types))
            }
            Token::Struct => {
                // Inline struct
                self.advance(); // consume struct
                self.consume(&Token::LeftBrace, "{")?;
                
                let mut fields = Vec::new();
                while !self.check_token(&Token::RightBrace) {
                    let field = self.parse_field_declaration()?;
                    fields.push(field);
                    
                    if self.check_token(&Token::Comma) {
                        self.advance();
                    }
                    self.skip_whitespace();
                }
                
                self.consume(&Token::RightBrace, "}")?;
                Ok(TypeExpression::Struct(fields))
            }
            _ => {
                let found = format!("{:?}", self.current_token()?.token);
                Err(self.syntax_error("type expression", found))
            }
        }
    }

    pub fn parse_enum_declaration(&mut self, annotations: Vec<Annotation<'input>>, pos: SourcePos) -> Result<EnumDeclaration<'input>, ParseError> {
        self.consume(&Token::Enum, "enum")?;
        self.skip_whitespace();
        
        // Parse optional base type: enum(string) or just enum
        let base_type = if self.check_token(&Token::LeftParen) {
            self.advance(); // consume (
            let base = self.current_identifier()?;
            self.advance();
            self.consume(&Token::RightParen, ")")?;
            Some(base)
        } else {
            None
        };
        
        self.skip_whitespace();
        let name = self.current_identifier()?;
        self.advance();
        self.skip_whitespace();
        
        self.consume(&Token::LeftBrace, "{")?;
        self.skip_whitespace();
        
        // Parse enum variants
        let mut variants = Vec::new();
        while !self.check_token(&Token::RightBrace) {
            let variant_pos = self.current_pos();
            let variant_annotations = self.parse_annotations()?;
            
            let variant_name = self.current_identifier()?;
            self.advance();
            
            // Parse optional value: = "creative"
            let value = if self.check_token(&Token::Equals) || self.check_token(&Token::Equal) {
                self.advance(); // consume = 
                match &self.current_token()?.token.clone() {
                    Token::String(s) => {
                        let s = *s;
                        self.advance();
                        Some(LiteralValue::String(s))
                    }
                    Token::Number(n) => {
                        let n = *n;
                        self.advance();
                        Some(LiteralValue::Number(n))
                    }
                    Token::True => {
                        self.advance();
                        Some(LiteralValue::Boolean(true))
                    }
                    Token::False => {
                        self.advance();
                        Some(LiteralValue::Boolean(false))
                    }
                    _ => return Err(self.syntax_error("literal value", "enum variant value")),
                }
            } else {
                None
            };
            
            variants.push(EnumVariant {
                name: variant_name,
                value,
                annotations: variant_annotations,
                position: self.make_position(variant_pos),
            });
            
            if self.check_token(&Token::Comma) {
                self.advance();
            }
            self.skip_whitespace();
        }
        
        self.consume(&Token::RightBrace, "}")?;
        
        Ok(EnumDeclaration {
            name,
            base_type,
            variants,
            annotations,
            position: self.make_position(pos),
        })
    }

    pub fn parse_type_declaration(&mut self, annotations: Vec<Annotation<'input>>, pos: SourcePos) -> Result<TypeDeclaration<'input>, ParseError> {
        self.consume(&Token::Type, "type")?;
        
        let name = self.current_identifier()?;
        self.advance();
        
        // Accept both Equal and Equals for compatibility
        if self.check_token(&Token::Equal) {
            self.advance();
        } else if self.check_token(&Token::Equals) {
            self.advance();
        } else {
            return Err(self.syntax_error("=", "type alias assignment"));
        }
        
        let type_expr = self.parse_type_expression()?;
        
        Ok(TypeDeclaration {
            name,
            type_expr,
            annotations,
            position: self.make_position(pos),
        })
    }

    pub fn parse_dispatch_declaration(&mut self, annotations: Vec<Annotation<'input>>, pos: SourcePos) -> Result<DispatchDeclaration<'input>, ParseError> {
        self.consume(&Token::Dispatch, "dispatch")?;
        
        // Parse dispatch source: minecraft:resource[recipe] or minecraft:recipe_serializer[crafting_shaped]
        let _namespace = self.current_identifier()?; // minecraft - kept for future use
        self.advance();
        self.consume(&Token::Colon, ":")?;
        let registry = self.current_identifier()?; // resource or recipe_serializer
        self.advance();
        
        self.consume(&Token::LeftBracket, "[")?;
        
        // Parse targets: [recipe], [smelting,blasting,smoking], [%unknown]
        let mut targets = Vec::new();
        loop {
            match &self.current_token()?.token {
                Token::Identifier(target) => {
                    targets.push(DispatchTarget::Specific(target));
                    self.advance();
                }
                Token::Percent => {
                    self.advance(); // consume %
                    let _unknown_type = self.current_identifier()?; // "unknown" or "key" - kept for future use
                    self.advance();
                    targets.push(DispatchTarget::Unknown);
                }
                _ => break,
            }
            
            if self.check_token(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        
        self.consume(&Token::RightBracket, "]")?;
        
        // Parse optional dynamic key: [[type]] or [[%key]]
        let key = if self.check_token(&Token::LeftBracket) {
            self.advance(); // consume [
            self.consume(&Token::LeftBracket, "[")?;
            
            let key_name = if self.check_token(&Token::Percent) {
                self.advance(); // consume %
                let key = self.current_identifier()?;
                self.advance();
                key
            } else {
                let key = self.current_identifier()?;
                self.advance();
                key
            };
            
            self.consume(&Token::RightBracket, "]")?;
            self.consume(&Token::RightBracket, "]")?;
            Some(key_name)
        } else {
            None
        };
        
        self.consume(&Token::To, "to")?;
        let target_type = self.parse_type_expression()?;
        
        let source = DispatchSource {
            registry,
            key,
            position: self.make_position(pos),
        };
        
        Ok(DispatchDeclaration {
            source,
            targets,
            target_type,
            annotations,
            position: self.make_position(pos),
        })
    }
    
    // ===== WRAPPER METHODS FOR TESTS =====
    
    /// Wrapper method for tests - parse struct with empty annotations
    pub fn parse_struct(&mut self, annotations: Vec<Annotation<'input>>) -> Result<StructDeclaration<'input>, ParseError> {
        let pos = self.current_pos();
        self.parse_struct_declaration(annotations, pos)
    }
    
    /// Wrapper method for tests - parse dispatch with empty annotations  
    pub fn parse_dispatch(&mut self, annotations: Vec<Annotation<'input>>) -> Result<DispatchDeclaration<'input>, ParseError> {
        let pos = self.current_pos();
        self.parse_dispatch_declaration(annotations, pos)
    }
    
    /// Wrapper method for tests - parse enum with empty annotations
    pub fn parse_enum(&mut self, annotations: Vec<Annotation<'input>>) -> Result<EnumDeclaration<'input>, ParseError> {
        let pos = self.current_pos();
        self.parse_enum_declaration(annotations, pos)
    }
    
    /// Wrapper method for tests - parse type alias with empty annotations
    pub fn parse_type_alias(&mut self, annotations: Vec<Annotation<'input>>) -> Result<TypeDeclaration<'input>, ParseError> {
        let pos = self.current_pos();
        self.parse_type_declaration(annotations, pos)
    }
    
    /// Wrapper method for tests - parse single type expression
    pub fn parse_single_type(&mut self) -> Result<TypeExpression<'input>, ParseError> {
        self.parse_type_expression()
    }
} 