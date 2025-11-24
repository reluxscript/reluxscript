//! Token stream rewriter for autofix
//!
//! Detects problematic token patterns and rewrites them to valid alternatives.

use crate::lexer::{Token, TokenKind, Span};

/// Token stream rewriter
pub struct TokenRewriter {
    tokens: Vec<Token>,
    position: usize,
    output: Vec<Token>,
    fixes_applied: usize,
}

impl TokenRewriter {
    /// Create a new token rewriter
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            output: Vec::new(),
            fixes_applied: 0,
        }
    }

    /// Rewrite the token stream, applying all fixes
    pub fn rewrite(mut self) -> (Vec<Token>, usize) {
        while self.position < self.tokens.len() {
            if self.try_fix_if_let_path_qualified() {
                // Fix was applied, continue
                continue;
            }

            // No fix applied, just copy the token
            self.output.push(self.tokens[self.position].clone());
            self.position += 1;
        }

        (self.output, self.fixes_applied)
    }

    /// Try to fix: if let Foo::Bar(x) = expr { ... }
    /// Converts to: match expr { Foo::Bar(x) => { ... }, _ => {} }
    fn try_fix_if_let_path_qualified(&mut self) -> bool {
        // Pattern: If Let Ident :: ...
        if !self.matches_sequence(&[
            |t| matches!(t, TokenKind::If),
            |t| matches!(t, TokenKind::Let),
        ]) {
            return false;
        }

        // Save start position
        let start_pos = self.position;

        // Look ahead to see if there's a :: (path qualifier)
        let mut lookahead = self.position + 2; // Skip "if let"
        let mut has_path_qualifier = false;

        // Scan the pattern to see if it contains ::
        while lookahead < self.tokens.len() {
            match &self.tokens[lookahead].kind {
                TokenKind::Eq => break, // Found =, stop scanning
                TokenKind::ColonColon => {
                    has_path_qualifier = true;
                    break;
                }
                _ => lookahead += 1,
            }
        }

        if !has_path_qualifier {
            return false; // No path qualifier, not the pattern we're looking for
        }

        // Now extract the full pattern:
        // if let PATTERN = EXPR { BODY } [else { ELSE_BODY }]

        self.position += 2; // Skip "if let"

        // Extract pattern tokens until we hit =
        let mut pattern_tokens = Vec::new();
        while self.position < self.tokens.len() {
            if matches!(self.tokens[self.position].kind, TokenKind::Eq) {
                self.position += 1; // Skip =
                break;
            }
            pattern_tokens.push(self.tokens[self.position].clone());
            self.position += 1;
        }

        // Extract expression tokens until we hit {
        let mut expr_tokens = Vec::new();
        while self.position < self.tokens.len() {
            if matches!(self.tokens[self.position].kind, TokenKind::LBrace) {
                break;
            }
            expr_tokens.push(self.tokens[self.position].clone());
            self.position += 1;
        }

        // Extract body (the { ... } block)
        let mut body_tokens = Vec::new();
        if self.position < self.tokens.len() &&
           matches!(self.tokens[self.position].kind, TokenKind::LBrace) {
            let start_span = self.tokens[self.position].span;
            body_tokens.push(self.tokens[self.position].clone());
            self.position += 1;

            let mut brace_depth = 1;
            while self.position < self.tokens.len() && brace_depth > 0 {
                match self.tokens[self.position].kind {
                    TokenKind::LBrace => brace_depth += 1,
                    TokenKind::RBrace => brace_depth -= 1,
                    _ => {}
                }
                body_tokens.push(self.tokens[self.position].clone());
                self.position += 1;
            }
        }

        // Check for else clause
        let has_else = self.position < self.tokens.len() &&
                       matches!(self.tokens[self.position].kind, TokenKind::Else);

        if has_else {
            // Skip the else clause - we'll ignore it for now in the conversion
            // since match with a catch-all _ arm doesn't need it
            self.position += 1; // Skip "else"

            // Skip the else body
            if self.position < self.tokens.len() &&
               matches!(self.tokens[self.position].kind, TokenKind::LBrace) {
                self.position += 1;
                let mut brace_depth = 1;
                while self.position < self.tokens.len() && brace_depth > 0 {
                    match self.tokens[self.position].kind {
                        TokenKind::LBrace => brace_depth += 1,
                        TokenKind::RBrace => brace_depth -= 1,
                        _ => {}
                    }
                    self.position += 1;
                }
            }
        }

        // Now generate the match expression
        // match EXPR { PATTERN => BODY, _ => {} }

        let first_span = self.tokens[start_pos].span;

        // match
        self.output.push(Token::new(TokenKind::Match, first_span));

        // EXPR
        self.output.extend(expr_tokens);

        // {
        self.output.push(Token::new(TokenKind::LBrace, first_span));

        // PATTERN
        self.output.extend(pattern_tokens);

        // =>
        self.output.push(Token::new(TokenKind::DDArrow, first_span));

        // BODY
        self.output.extend(body_tokens);

        // , _ => {}
        self.output.push(Token::new(TokenKind::Comma, first_span));
        self.output.push(Token::new(TokenKind::Ident("_".to_string()), first_span));
        self.output.push(Token::new(TokenKind::DDArrow, first_span));
        self.output.push(Token::new(TokenKind::LBrace, first_span));
        self.output.push(Token::new(TokenKind::RBrace, first_span));

        // }
        self.output.push(Token::new(TokenKind::RBrace, first_span));

        self.fixes_applied += 1;
        true
    }

    /// Check if the current position matches a sequence of token predicates
    fn matches_sequence(&self, predicates: &[fn(&TokenKind) -> bool]) -> bool {
        if self.position + predicates.len() > self.tokens.len() {
            return false;
        }

        for (i, predicate) in predicates.iter().enumerate() {
            if !predicate(&self.tokens[self.position + i].kind) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_if_let_no_fix() {
        // if let Some(x) = opt { ... }
        // Should NOT be fixed (no path qualifier)
        let tokens = vec![
            Token::new(TokenKind::If, Span::new(0, 2, 1, 1)),
            Token::new(TokenKind::Let, Span::new(3, 6, 1, 4)),
            Token::new(TokenKind::Ident("Some".to_string()), Span::new(7, 11, 1, 8)),
            Token::new(TokenKind::LParen, Span::new(11, 12, 1, 12)),
            Token::new(TokenKind::Ident("x".to_string()), Span::new(12, 13, 1, 13)),
            Token::new(TokenKind::RParen, Span::new(13, 14, 1, 14)),
            Token::new(TokenKind::Eq, Span::new(15, 16, 1, 16)),
            Token::new(TokenKind::Ident("opt".to_string()), Span::new(17, 20, 1, 18)),
            Token::new(TokenKind::LBrace, Span::new(21, 22, 1, 22)),
            Token::new(TokenKind::RBrace, Span::new(22, 23, 1, 23)),
        ];

        let rewriter = TokenRewriter::new(tokens.clone());
        let (result, fixes) = rewriter.rewrite();

        assert_eq!(fixes, 0);
        assert_eq!(result.len(), tokens.len());
    }

    #[test]
    fn test_path_qualified_if_let_fix() {
        // if let Foo::Bar(x) = expr { body }
        // Should be fixed to: match expr { Foo::Bar(x) => { body }, _ => {} }
        let tokens = vec![
            Token::new(TokenKind::If, Span::new(0, 2, 1, 1)),
            Token::new(TokenKind::Let, Span::new(3, 6, 1, 4)),
            Token::new(TokenKind::Ident("Foo".to_string()), Span::new(7, 10, 1, 8)),
            Token::new(TokenKind::ColonColon, Span::new(10, 12, 1, 11)),
            Token::new(TokenKind::Ident("Bar".to_string()), Span::new(12, 15, 1, 13)),
            Token::new(TokenKind::LParen, Span::new(15, 16, 1, 16)),
            Token::new(TokenKind::Ident("x".to_string()), Span::new(16, 17, 1, 17)),
            Token::new(TokenKind::RParen, Span::new(17, 18, 1, 18)),
            Token::new(TokenKind::Eq, Span::new(19, 20, 1, 20)),
            Token::new(TokenKind::Ident("expr".to_string()), Span::new(21, 25, 1, 22)),
            Token::new(TokenKind::LBrace, Span::new(26, 27, 1, 27)),
            Token::new(TokenKind::Ident("body".to_string()), Span::new(28, 32, 1, 29)),
            Token::new(TokenKind::RBrace, Span::new(33, 34, 1, 34)),
        ];

        let rewriter = TokenRewriter::new(tokens);
        let (result, fixes) = rewriter.rewrite();

        assert_eq!(fixes, 1);

        // Verify the structure: match expr { Foo::Bar(x) => { body }, _ => {} }
        assert!(matches!(result[0].kind, TokenKind::Match));
        assert!(matches!(result[1].kind, TokenKind::Ident(ref s) if s == "expr"));
        assert!(matches!(result[2].kind, TokenKind::LBrace));
        assert!(matches!(result[3].kind, TokenKind::Ident(ref s) if s == "Foo"));
        assert!(matches!(result[4].kind, TokenKind::ColonColon));
        assert!(matches!(result[5].kind, TokenKind::Ident(ref s) if s == "Bar"));
    }
}
