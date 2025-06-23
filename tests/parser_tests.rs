use voxel_rsmcdoc::{
    lexer::Lexer,
    parser::{Declaration, Parser},
};

#[test]
fn test_parse_struct_with_fields() {
    let content = r#"
        struct Test {
            field1: string,
            field2: int,
        }
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);

    if let Declaration::Struct(struct_decl) = &ast.declarations[0] {
        assert_eq!(struct_decl.name, "Test");
        assert_eq!(struct_decl.members.len(), 2);
        if let voxel_rsmcdoc::parser::StructMember::Field(field) = &struct_decl.members[0] {
            assert_eq!(field.name, "field1");
        }
        if let voxel_rsmcdoc::parser::StructMember::Field(field) = &struct_decl.members[1] {
            assert_eq!(field.name, "field2");
        }
    } else {
        panic!("Expected a struct declaration");
    }
}

#[test]
fn test_parse_struct_with_optional_fields() {
    let content = r#"
        struct Test {
            field1?: string,
            field2: int,
        }
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);

    if let Declaration::Struct(struct_decl) = &ast.declarations[0] {
        if let voxel_rsmcdoc::parser::StructMember::Field(field) = &struct_decl.members[0] {
            assert!(field.optional);
        }
        if let voxel_rsmcdoc::parser::StructMember::Field(field) = &struct_decl.members[1] {
            assert!(!field.optional);
        }
    } else {
        panic!("Expected a struct declaration");
    }
}

#[test]
fn test_parse_dispatch_to_named_struct() {
    let content = "dispatch minecraft:test to Test;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_parse_dispatch_with_key_to_named_struct() {
    let content = "dispatch minecraft:test[key] to Test;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_parse_dispatch_to_inline_struct() {
    let content = "dispatch minecraft:test to struct { field: string };";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_parse_enum() {
    let content = r#"
        enum Test: string {
            Variant1 = "v1",
            Variant2 = "v2",
        }
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_parse_type_alias() {
    let content = "type Test = string;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_parse_import_statement() {
    let content = "use a::b::c;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.imports.len(), 1);
}

#[test]
fn test_parse_annotations() {
    let content = r#"
        #[id(registry="minecraft:item")]
        struct Test {
            field: string,
        }
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
    if let Declaration::Struct(s) = &result.declarations[0] {
        assert_eq!(s.annotations.len(), 1);
    } else {
        panic!();
    }
}

#[test]
fn test_union_type() {
    let content = "type MyUnion = string | int;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_array_type() {
    let content = "type MyArray = string[];";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_generic_type() {
    let content = "type MyGeneric = Map<string, int>;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_spread_operator() {
    let content = "dispatch minecraft:recipe to ...minecraft:item;";
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_multiline_dispatch() {
    let content = r#"
        dispatch minecraft:recipe to struct {
            a: string,
            b: int,
        };
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_complex_dispatch() {
    let content = r#"
        dispatch minecraft:recipe[stone, stick] to struct {
            a: string,
            b: int,
        };
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse().unwrap();
    assert_eq!(result.declarations.len(), 1);
}

#[test]
fn test_array_with_annotations() {
    let content = r#"
        struct GpuWarnlist {
            renderer?: [#[regex_pattern] string],
            version?: [#[regex_pattern] string],
        }
    "#;
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);

    let ast = parser.parse().unwrap();
    assert_eq!(ast.declarations.len(), 1);

    if let Declaration::Struct(struct_decl) = &ast.declarations[0] {
        assert_eq!(struct_decl.name, "GpuWarnlist");
        assert_eq!(struct_decl.members.len(), 2);
        
        // Check first field: renderer?: [#[regex_pattern] string]
        if let voxel_rsmcdoc::parser::StructMember::Field(field) = &struct_decl.members[0] {
            assert_eq!(field.name, "renderer");
            assert!(field.optional);
            // Should be an array type
            if let voxel_rsmcdoc::parser::TypeExpression::Array { element_type, .. } = &field.field_type {
                // Element should be string (annotation is parsed but not stored in TypeExpression for now)
                if let voxel_rsmcdoc::parser::TypeExpression::Simple(type_name) = element_type.as_ref() {
                    assert_eq!(*type_name, "string");
                } else {
                    panic!("Expected simple type 'string' as array element");
                }
            } else {
                panic!("Expected array type for renderer field");
            }
        } else {
            panic!("Expected field for renderer");
        }
    } else {
        panic!("Expected a struct declaration");
    }
} 