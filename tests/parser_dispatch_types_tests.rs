#[cfg(test)]
mod tests {
    use voxel_rsmcdoc::parser::Parser;
    use voxel_rsmcdoc::lexer::Lexer;

    #[test]
    fn test_dispatch_to_dispatch_syntax() {
        // Test the specific syntax that fails in moving_piston.mcdoc
        let input = r#"
            dispatch minecraft:block[moving_piston] to minecraft:block_entity[moving_piston]
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.is_ok(), "Lexer should tokenize successfully");

        let mut parser = Parser::new(tokens.unwrap());
        let result = parser.parse();
        
        // This should not fail, but currently does
        assert!(result.is_ok(), "Parser should handle dispatch-to-dispatch syntax: {:?}", result.err());
    }

    #[test]
    fn test_dispatch_with_registry_param() {
        // Test dispatch with registry parameters
        let input = r#"
            dispatch minecraft:recipe_serializer[crafting_shaped] to struct CraftingShaped {
                type: string,
            }
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.is_ok());

        let mut parser = Parser::new(tokens.unwrap());
        let result = parser.parse();
        assert!(result.is_ok(), "Parser should handle dispatch with registry parameters: {:?}", result.err());
    }

    #[test]
    fn test_multiple_dispatch_targets() {
        // Test dispatch with multiple targets
        let input = r#"
            dispatch minecraft:recipe_serializer[smelting,blasting,smoking] to struct Smelting {
                ingredient: string,
            }
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.is_ok());

        let mut parser = Parser::new(tokens.unwrap());
        let result = parser.parse();
        assert!(result.is_ok(), "Parser should handle multiple dispatch targets: {:?}", result.err());
    }

    #[test]
    fn test_dispatch_to_existing_dispatch() {
        // This is the exact pattern from moving_piston.mcdoc
        let input = r#"
            dispatch minecraft:block_entity[moving_piston] to struct MovingPiston {
                blockState?: string,
            }
            
            dispatch minecraft:block[moving_piston] to minecraft:block_entity[moving_piston]
        "#;

        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize();
        assert!(tokens.is_ok(), "Lexer should work");

        let mut parser = Parser::new(tokens.unwrap());
        let result = parser.parse();
        assert!(result.is_ok(), "Parser should handle dispatch-to-dispatch reference: {:?}", result.err());
    }
} 