use reluxscript::{Lexer, TokenRewriter};
use std::fs;

#[test]
fn test_autofix_on_minimal_original() {
    // Load the original file with path-qualified if-let patterns
    let source = fs::read_to_string("tests/codegen/minimal/minimal_original.rsc")
        .expect("Failed to read test file");

    println!("\n=== Testing on minimal_original.rsc ===");

    // Tokenize
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    println!("Original token count: {}", tokens.len());

    // Apply autofix
    let rewriter = TokenRewriter::new(tokens);
    let (_fixed_tokens, fixes_applied) = rewriter.rewrite();

    println!("Fixes applied: {}", fixes_applied);

    // We expect to find these patterns:
    // 1. Statement::ExpressionStatement
    // 2. Expression::CallExpression
    // 3. Expression::MemberExpression
    // 4. Statement::VariableDeclaration
    // 5. Expression::Identifier (first one)
    // 6. Expression::ArrayExpression
    // Note: Nested patterns like Some(Expression::Identifier(...)) are not currently detected

    assert!(fixes_applied >= 4, "Should detect at least 4 if-let patterns with path qualifiers");
    println!("✓ Successfully detected and fixed {} patterns", fixes_applied);
}

#[test]
fn test_autofix_detection() {
    // Small snippet with path-qualified if-let
    let source = r#"
        fn test() {
            if let Expression::CallExpression(call) = expr {
                process(call);
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    let rewriter = TokenRewriter::new(tokens);
    let (fixed_tokens, fixes_applied) = rewriter.rewrite();

    println!("\nSmall snippet test:");
    println!("Fixes applied: {}", fixes_applied);
    assert_eq!(fixes_applied, 1, "Should detect and fix one if-let pattern");
}
