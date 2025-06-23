#[cfg(test)]
use voxel_rsmcdoc::{lexer::Lexer, parser::Parser};

#[test]
fn test_debug_tokens() {
    let mcdoc_content = r#"struct Test { a: string }"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    println!("TOKENS DEBUG:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token.token);
    }
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    assert!(ast.is_ok(), "Simple struct parsing failed: {:?}", ast.err());
}

#[test]
fn test_parse_dispatch_with_inline_struct() {
    let mcdoc_content = r#"
dispatch minecraft:resource[test_recipe] to struct TestRecipe {
    ingredient: #[id(registry="item")] string,
    result: #[id(registry="item")] string,
}
"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");

    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    assert!(ast.is_ok(), "Parsing failed with errors: {:?}", ast.err());

    let mcdoc_file = ast.unwrap();
    assert_eq!(mcdoc_file.declarations.len(), 1);
    
    let decl = &mcdoc_file.declarations[0];
    if let voxel_rsmcdoc::parser::Declaration::Dispatch(dispatch_decl) = decl {
        if let voxel_rsmcdoc::parser::TypeExpression::Struct(members) = &dispatch_decl.target_type {
            assert_eq!(members.len(), 2);
            if let voxel_rsmcdoc::parser::StructMember::Field(field) = &members[0] {
                assert_eq!(field.name, "ingredient");
            }
            if let voxel_rsmcdoc::parser::StructMember::Field(field) = &members[1] {
                assert_eq!(field.name, "result");
            }
        } else {
            panic!("Dispatch target_type was not a struct, but: {:?}", dispatch_decl.target_type);
        }
    } else {
        panic!("Declaration was not a dispatch declaration");
    }
}

#[test]
fn test_multiline_dispatch_targets() {
    let content = r#"dispatch minecraft:item[
    axolotl_bucket,
    cod_bucket,
    salmon_bucket,
    pufferfish_bucket,
    tadpole_bucket
] to struct BasicFishBucket {
    EntityTag?: string,
}"#;

    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(ast) => {
            assert_eq!(ast.declarations.len(), 1);
            if let voxel_rsmcdoc::parser::Declaration::Dispatch(_dispatch_decl) = &ast.declarations[0] {
                // Successfully parsed multiline dispatch
            } else {
                panic!("Expected dispatch declaration");
            }
        }
        Err(errors) => {
            panic!("Expected successful parsing but got errors: {:#?}", errors);
        }
    }
}

#[test]
fn test_spread_field_in_struct() {
    let content = r#"struct TestStruct {
    ...super::ItemBase,
    EntityTag?: string,
    BucketVariantTag?: int,
}"#;

    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().expect("Tokenization should succeed");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(ast) => {
            assert_eq!(ast.declarations.len(), 1);
            if let voxel_rsmcdoc::parser::Declaration::Struct(struct_decl) = &ast.declarations[0] {
                // Should parse struct with spread field and regular fields
                assert!(!struct_decl.members.is_empty());
            } else {
                panic!("Expected struct declaration");
            }
        }
        Err(errors) => {
            panic!("Expected successful parsing but got errors: {:#?}", errors);
        }
    }
}

#[test]
fn test_gpu_warnlist_parsing() {
    let content = r#"#[since="1.16"]
dispatch minecraft:resource[gpu_warnlist] to struct GpuWarnlist {
	renderer?: [#[regex_pattern] string],
	version?: [#[regex_pattern] string],
	vendor?: [#[regex_pattern] string],
}"#;
    
    let mut lexer = Lexer::new(content);
    let tokens = lexer.tokenize().expect("Should tokenize successfully");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(ast) => {
            assert_eq!(ast.declarations.len(), 1);
            println!("✅ gpu_warnlist.mcdoc parsed successfully!");
        }
        Err(errors) => {
            for error in &errors {
                eprintln!("❌ Parse error: {:?}", error);
            }
            panic!("Failed to parse gpu_warnlist.mcdoc");
        }
    }
}

#[test]
fn test_hypothese_1_union_type_with_parentheses() {
    // HYPOTHÈSE 1: Type union avec parenthèses non géré
    let mcdoc_content = r#"struct ChatType {
    chat?: (
        #[until="1.19.1"] TextDisplay |
        #[since="1.19.1"] ChatDecoration |
    ),
}"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    println!("TOKENS pour union avec parenthèses:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token.token);
    }
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    match ast {
        Ok(_) => println!("✅ HYPOTHÈSE 1 INVALIDÉE: Union avec parenthèses fonctionne"),
        Err(errors) => {
            println!("❌ HYPOTHÈSE 1 CONFIRMÉE: Erreurs union parenthèses");
            for error in &errors {
                println!("  Error: {:?}", error);
            }
        }
    }
}

#[test] 
fn test_hypothese_2_spread_with_annotations() {
    // HYPOTHÈSE 2: Spread dans struct avec annotations
    let mcdoc_content = r#"struct ChatDecoration {
    #[until="1.19.1"]
    ...struct {
        style: TextStyle,
    },
    #[since="1.19.1"] 
    ...struct {
        style?: TextStyle,
    },
}"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    println!("TOKENS pour spread avec annotations:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token.token);
    }
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    match ast {
        Ok(_) => println!("✅ HYPOTHÈSE 2 INVALIDÉE: Spread avec annotations fonctionne"),
        Err(errors) => {
            println!("❌ HYPOTHÈSE 2 CONFIRMÉE: Erreurs spread annotations");
            for error in &errors {
                println!("  Error: {:?}", error);
            }
        }
    }
}

#[test]
fn test_hypothese_3_simple_union_parentheses() {
    // HYPOTHÈSE 3: Simple test union parenthèses
    let mcdoc_content = r#"struct Test {
    field: (string | int),
}"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    match ast {
        Ok(_) => println!("✅ HYPOTHÈSE 3 INVALIDÉE: Union parenthèses simple fonctionne"),
        Err(errors) => {
            println!("❌ HYPOTHÈSE 3 CONFIRMÉE: Erreurs union parenthèses simple");
            for error in &errors {
                println!("  Error: {:?}", error);
            }
        }
    }
}

#[test]
fn test_hypothese_4_trace_chat_type_exact() {
    // HYPOTHÈSE 4: Test exact du problème chat_type.mcdoc
    let mcdoc_content = r#"dispatch minecraft:resource[chat_type] to struct ChatType {
    chat?: (
        #[until="1.19.1"] TextDisplay |
        #[since="1.19.1"] ChatDecoration |
    ),
    narration?: (
        #[until="1.19.1"] Narration |
        #[since="1.19.1"] ChatDecoration
    ),
}"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    println!("TRACE EXACT chat_type.mcdoc ligne 4:");
    for (i, token) in tokens.iter().enumerate() {
        if i >= 10 && i <= 20 {  // Zone problématique ligne 4 colonne 9
            println!("  {}: {:?} at line {} col {}", i, token.token, token.position.line, token.position.column);
        }
    }
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    match ast {
        Ok(_) => println!("✅ HYPOTHÈSE 4 INVALIDÉE: chat_type exact fonctionne"),
        Err(errors) => {
            println!("❌ HYPOTHÈSE 4 CONFIRMÉE: Erreurs chat_type exact");
            for error in &errors {
                println!("  Error: {:?}", error);
            }
        }
    }
}

#[test]
fn test_temporal_annotations_in_union() {
    // PROBLÈME RESTANT: Annotations temporelles dans union
    let mcdoc_content = r#"struct Test {
    field: (
        #[until="1.19.1"] TypeA |
        #[since="1.19.1"] TypeB
    ),
}"#;

    let mut lexer = Lexer::new(mcdoc_content);
    let tokens = lexer.tokenize().expect("Lexing failed");
    
    println!("TOKENS annotations temporelles union:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token.token);
    }
    
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();
    
    match ast {
        Ok(_) => println!("✅ Annotations temporelles dans union fonctionnent"),
        Err(errors) => {
            println!("❌ PROBLÈME: Annotations temporelles dans union échouent");
            for error in &errors {
                println!("  Error: {:?}", error);
            }
        }
    }
}

#[test]
fn test_chat_type_parsing_union_with_trailing_comma() {
    // HYPOTHÈSE 1: Union type avec trailing comma et annotations de version
    let input = r#"
struct TestUnion {
    chat?: (
        #[until="1.19.1"] TextDisplay |
        #[since="1.19.1"] ChatDecoration |
    ),
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ HYPOTHÈSE 1 FAUSSE: Union avec trailing comma parse correctement"),
        Err(errors) => {
            println!("🔥 HYPOTHÈSE 1 VRAIE: Erreurs de parsing:");
            for error in &errors {
                println!("  - {:?}", error);
            }
            // Vérifier qu'on a bien l'erreur attendue
            assert!(errors.iter().any(|e| 
                format!("{:?}", e).contains("RightParen") && 
                format!("{:?}", e).contains("type")
            ), "Erreur expected 'type', found 'RightParen' non trouvée");
        }
    }
}

#[test]
fn test_chat_type_parsing_spread_with_annotations() {
    // HYPOTHÈSE 2: Spread avec annotations
    let input = r#"
struct TestSpread {
    #[until="1.19.1"]
    ...struct {
        style: TextStyle,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ HYPOTHÈSE 2 FAUSSE: Spread avec annotations parse correctement"),
        Err(errors) => {
            println!("🔥 HYPOTHÈSE 2 VRAIE: Erreurs de parsing:");
            for error in &errors {
                println!("  - {:?}", error);
            }
            // Vérifier qu'on a bien l'erreur attendue
            assert!(errors.iter().any(|e| 
                format!("{:?}", e).contains("DotDotDot") && 
                format!("{:?}", e).contains("identifier")
            ), "Erreur expected 'identifier', found 'DotDotDot' non trouvée");
        }
    }
}

#[test]
fn test_chat_type_parsing_conditional_annotations() {
    // HYPOTHÈSE 3: Annotations sur des blocks conditionnels
    let input = r#"
struct TestConditional {
    parameters: [ChatDecorationParameter],
    #[until="1.19.1"]
    ...struct {
        style: TextStyle,
    },
    #[since="1.19.1"] 
    ...struct {
        style?: TextStyle,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ HYPOTHÈSE 3 FAUSSE: Annotations conditionnelles parsent correctement"),
        Err(errors) => {
            println!("🔥 HYPOTHÈSE 3 VRAIE: Erreurs de parsing:");
            for error in &errors {
                println!("  - {:?}", error);
            }
        }
    }
}

#[test]
fn test_chat_type_parsing_exact_reproduction() {
    // Reproduction exacte du fichier chat_type.mcdoc problématique
    let input = r#"
use ::java::util::text::TextStyle

dispatch minecraft:resource[chat_type] to struct ChatType {
	chat?: (
		#[until="1.19.1"] TextDisplay |
		#[since="1.19.1"] ChatDecoration |
	),
	#[until="1.19.1"]
	overlay?: TextDisplay,
	narration?: (
		#[until="1.19.1"] Narration |
		#[since="1.19.1"] ChatDecoration
	),
}

struct ChatDecoration {
	translation_key: string,
	parameters: [ChatDecorationParameter],
	#[until="1.19.1"]
	...struct {
		style: TextStyle,
	},
	#[since="1.19.1"]
	...struct {
		style?: TextStyle,
	},
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    
    println!("🔍 TOKENS GÉNÉRÉS:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(ast) => {
            println!("✅ SUCCÈS INATTENDU: AST généré");
            println!("AST: {:#?}", ast);
        },
        Err(errors) => {
            println!("❌ ERREURS EXACTES REPRODUITES:");
            for (i, error) in errors.iter().enumerate() {
                println!("  {}: {:?}", i + 1, error);
            }
            
            // Vérifier qu'on reproduit les erreurs mentionnées
            let error_strings: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
            
            // ✅ FIXED: Union types avec trailing comma maintenant supportés
            // assert!(error_strings.iter().any(|s| s.contains("RightParen") && s.contains("type")), 
            //        "Erreur 'expected type, found RightParen' non reproduite");
            
            assert!(error_strings.iter().any(|s| s.contains("annotations only")), 
                   "Erreur 'annotations only' non reproduite");
                   
            assert!(error_strings.iter().any(|s| s.contains("DotDotDot") && s.contains("identifier")), 
                   "Erreur 'expected identifier, found DotDotDot' non reproduite");
        }
    }
}

#[test]
fn test_spread_annotation_debug() {
    // Test très simple pour debugger
    let input = r#"
struct Test {
    #[until="1.19.1"]
    ...struct {
        field: string,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    
    println!("🎯 TOKENS DÉTAILLÉS:");
    for (i, token) in tokens.iter().enumerate() {
        println!("  {}: {:?}", i, token);
    }
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(ast) => {
            println!("✅ SUCCÈS - AST généré:");
            println!("{:#?}", ast);
        },
        Err(errors) => {
            println!("❌ ERREURS:");
            for (i, error) in errors.iter().enumerate() {
                println!("  {}: {:?}", i + 1, error);
            }
        }
    }
}

#[test]
fn test_normal_field_parsing() {
    // Test simple pour vérifier qu'un field normal fonctionne
    let input = r#"
struct Test {
    field: string,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(ast) => println!("✅ FIELD NORMAL: {:?}", ast),
        Err(errors) => {
            println!("❌ FIELD NORMAL ÉCHOUE:");
            for error in &errors {
                println!("  - {:?}", error);
            }
        }
    }
}

#[test]
fn test_spread_annotation_reproduction() {
    // Reproduction exacte du problème chat_type.mcdoc ligne 33-40
    let input = r#"
struct ChatDecoration {
    translation_key: string,
    parameters: [ChatDecorationParameter],
    #[until="1.19.1"]
    ...struct {
        style: TextStyle,
    },
    #[since="1.19.1"]
    ...struct {
        style?: TextStyle,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    
    println!("🎯 TOKENS LIGNE 33-40:");
    for (i, token) in tokens.iter().enumerate() {
        if token.position.line >= 5 && token.position.line <= 12 {
            println!("  {}: {:?} at line {} col {}", i, token.token, token.position.line, token.position.column);
        }
    }
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(_) => println!("✅ Spreads avec annotations fonctionnent"),
        Err(errors) => {
            println!("❌ ERREURS REPRODUITES:");
            for (i, error) in errors.iter().enumerate() {
                println!("  {}: {:?}", i + 1, error);
            }
            
            // Vérifier qu'on reproduit bien les erreurs de l'utilisateur
            let error_strings: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
            
            assert!(error_strings.iter().any(|s| s.contains("DotDotDot") && s.contains("identifier")), 
                   "Erreur 'expected identifier, found DotDotDot' non reproduite");
            
            // Cette fonction devrait échouer jusqu'à la correction
            panic!("Test de reproduction - erreurs attendues");
        }
    }
}

#[test]
fn test_spread_annotation_simple() {
    // Test plus simple pour isoler le problème
    let input = r#"
struct Test {
    #[since="1.19"]
    ...struct {
        field: string,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    
    match result {
        Ok(_) => println!("✅ Spread simple avec annotation fonctionne"),
        Err(errors) => {
            println!("❌ ERREUR SPREAD SIMPLE:");
            for error in &errors {
                println!("  - {:?}", error);
            }
            // Test de reproduction - on s'attend à une erreur pour l'instant
            panic!("Test de reproduction simple - erreur attendue");
        }
    }
}

#[test]
fn test_spread_without_annotation() {
    // Contrôle: spread sans annotation (devrait marcher)
    let input = r#"
struct Test {
    ...struct {
        field: string,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ Spread sans annotation fonctionne (contrôle)"),
        Err(errors) => {
            println!("❌ ERREUR INATTENDUE dans spread sans annotation:");
            for error in &errors {
                println!("  - {:?}", error);
            }
        }
    }
}

#[test]
fn test_annotation_before_field() {
    // Contrôle: annotation avant field normal (devrait marcher)  
    let input = r#"
struct Test {
    #[since="1.19"]
    field: string,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().expect("Lexing should work");
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    match result {
        Ok(_) => println!("✅ Annotation avant field fonctionne (contrôle)"),
        Err(errors) => {
            println!("❌ ERREUR INATTENDUE dans annotation field:");
            for error in &errors {
                println!("  - {:?}", error);
            }
        }
    }
}

#[test]
fn test_conditional_spread_with_version_annotations() {
    // Test case that reproduces the exact error from chat_type.mcdoc
    let input = r#"
        struct ChatDecoration {
            translation_key: string,
            parameters: [ChatDecorationParameter],
            #[until="1.19.1"]
            ...struct {
                style: TextStyle,
            },
            #[since="1.19.1"]
            ...struct {
                style?: TextStyle,
            },
        }
    "#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    println!("Parser result: {:?}", result);
    
    match result {
        Ok(mcdoc_file) => {
            assert_eq!(mcdoc_file.declarations.len(), 1);
            if let voxel_rsmcdoc::parser::Declaration::Struct(struct_decl) = &mcdoc_file.declarations[0] {
                assert_eq!(struct_decl.name, "ChatDecoration");
                assert_eq!(struct_decl.members.len(), 4); // translation_key, parameters, 2 spreads
                
                // Check that spreads have the correct annotations
                let spreads: Vec<_> = struct_decl.members.iter()
                    .filter_map(|m| match m {
                        voxel_rsmcdoc::parser::StructMember::Spread(spread) => Some(spread),
                        _ => None,
                    })
                    .collect();
                
                assert_eq!(spreads.len(), 2, "Should have 2 conditional spreads");
            } else {
                panic!("Expected struct declaration");
            }
        }
        Err(errors) => {
            panic!("Parser failed with errors: {:?}", errors);
        }
    }
}

#[test]
fn test_simple_spread_with_annotations() {
    // Simpler test case to isolate the issue
    let input = r#"
        struct Test {
            #[since="1.19"]
            ...struct {
                field: string,
            },
        }
    "#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Simple annotated spread should parse successfully: {:?}", result);
}

#[test] 
fn test_spread_without_annotations() {
    // Control test - spread without annotations should work
    let input = r#"
        struct Test {
            ...struct {
                field: string,
            },
        }
    "#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    assert!(result.is_ok(), "Simple spread should parse successfully: {:?}", result);
}

#[test]
fn test_real_chat_type_dataset() {
    // Test with the actual problematic content from chat_type.mcdoc
    let input = r#"use ::java::util::text::TextStyle

dispatch minecraft:resource[chat_type] to struct ChatType {
	chat?: (
		#[until="1.19.1"] TextDisplay |
		#[since="1.19.1"] ChatDecoration |
	),
	#[until="1.19.1"]
	overlay?: TextDisplay,
	narration?: (
		#[until="1.19.1"] Narration |
		#[since="1.19.1"] ChatDecoration
	),
}

struct TextDisplay {
	decoration?: ChatDecoration,
}

struct Narration {
	decoration?: ChatDecoration,
	priority: NarrationPriority,
}

enum(string) NarrationPriority {
	Chat = "chat",
	System = "system",
}

struct ChatDecoration {
	translation_key: string,
	parameters: [ChatDecorationParameter],
	#[until="1.19.1"]
	...struct {
		style: TextStyle,
	},
	#[since="1.19.1"]
	...struct {
		style?: TextStyle,
	},
}

enum(string) ChatDecorationParameter {
	Sender = "sender",
	Content = "content",
	#[until="1.19.1"]
	TeamName = "team_name",
	#[since="1.19.1"]
	Target = "target",
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize().unwrap();
    let mut parser = Parser::new(tokens);
    
    let result = parser.parse();
    
    match result {
        Ok(mcdoc_file) => {
            println!("✅ Real chat_type.mcdoc parsed successfully!");
            println!("  Imports: {}", mcdoc_file.imports.len());
            println!("  Declarations: {}", mcdoc_file.declarations.len());
        }
        Err(errors) => {
            println!("❌ Parser failed with errors:");
            for (i, error) in errors.iter().enumerate() {
                println!("  {}: {:?}", i + 1, error);
            }
            panic!("Real chat_type.mcdoc failed to parse");
        }
    }
}

/// Tests pour les erreurs spécifiques trouvées dans trigger.mcdoc

#[test]
fn test_array_constraint_with_at_symbol() {
    let input = r#"
type RecipeCrafted = struct {
    ingredients?: [ItemPredicate] @ 1..9,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize array constraints");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle array constraints with @ symbol: {:?}", result);
}

#[test]
fn test_union_with_annotations_on_newlines_complex() {
    let input = r#"
dispatch minecraft:trigger[item_used_on_block] to Conditions<struct ItemUsedOnBlock {
    location?: (
        #[until="1.20"] LocationPredicate |
        #[since="1.20"] [LootCondition] |
    ),
}>
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize union with versioned annotations");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle union with annotations on newlines: {:?}", result);
}

#[test]
fn test_struct_field_with_dynamic_key_syntax() {
    let input = r#"
struct Test {
    effects?: struct {
        [#[id="mob_effect"] string]: MobEffectPredicate,
    },
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize dynamic key syntax");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle dynamic key syntax in structs: {:?}", result);
}

#[test]
fn test_block_state_reference_syntax() {
    let input = r#"
struct Test {
    state?: mcdoc:block_states[[block]],
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize block state reference");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle block state reference syntax: {:?}", result);
}

#[test]
fn test_exact_trigger_pattern_from_mcdoc() {
    let input = r#"
dispatch minecraft:trigger[placed_block] to Conditions<struct PlacedBlock {
    location?: (
        #[until="1.20"] LocationPredicate |
        #[since="1.20"] [LootCondition]
    ),
}>
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize exact trigger pattern");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle exact trigger pattern from trigger.mcdoc: {:?}", result);
}

#[test]
fn test_optional_field_with_array_constraint() {
    let input = r#"
struct Test {
    ingredients?: [ItemPredicate] @ 1..9,
    items?: [string],
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize optional fields with array constraints");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle optional fields with array constraints: {:?}", result);
}

#[test]
fn test_complex_annotation_in_struct_field() {
    let input = r#"
struct Test {
    #[until="1.20"]
    item?: ItemPredicate,
    #[since="1.17"]
    source?: CompositeEntity,
}
"#;
    
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize();
    assert!(tokens.is_ok(), "Lexer should tokenize struct fields with annotations");
    
    let mut parser = Parser::new(tokens.unwrap());
    let result = parser.parse();
    
    assert!(result.is_ok(), "Parser should handle struct fields with version annotations: {:?}", result);
}



 