//! Lexer for ReluxScript

use crate::lexer::token::{Token, TokenKind, Span};

/// The lexer that tokenizes ReluxScript source code
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
    line_start: usize,
    /// Depth tracking for context-aware newline handling
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.char_indices().peekable(),
            current_pos: 0,
            line: 1,
            column: 1,
            line_start: 0,
            paren_depth: 0,
            bracket_depth: 0,
            brace_depth: 0,
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            let is_eof = matches!(token.kind, TokenKind::Eof);
            tokens.push(token);
            if is_eof {
                break;
            }
        }
        tokens
    }

    /// Get the next token
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let start = self.current_pos;
        let start_line = self.line;
        let start_column = self.column;

        let kind = match self.advance() {
            None => TokenKind::Eof,
            Some((_, c)) => match c {
                // Single-char tokens (track depth for context-aware newlines)
                '(' => {
                    self.paren_depth += 1;
                    TokenKind::LParen
                }
                ')' => {
                    self.paren_depth = self.paren_depth.saturating_sub(1);
                    TokenKind::RParen
                }
                '{' => {
                    self.brace_depth += 1;
                    TokenKind::LBrace
                }
                '}' => {
                    self.brace_depth = self.brace_depth.saturating_sub(1);
                    TokenKind::RBrace
                }
                '[' => {
                    self.bracket_depth += 1;
                    TokenKind::LBracket
                }
                ']' => {
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    TokenKind::RBracket
                }
                ',' => TokenKind::Comma,
                ';' => TokenKind::Semicolon,
                '?' => {
                    if self.peek_char() == Some('.') {
                        self.advance();
                        TokenKind::QuestionDot
                    } else {
                        TokenKind::Question
                    }
                }
                '#' => TokenKind::Hash,
                '^' => TokenKind::Caret,
                '%' => TokenKind::Percent,

                // Potentially multi-char tokens
                '+' => {
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::PlusEq
                    } else {
                        TokenKind::Plus
                    }
                }

                '*' => {
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::StarEq
                    } else {
                        TokenKind::Star
                    }
                }

                '-' => {
                    if self.peek_char() == Some('>') {
                        self.advance();
                        TokenKind::Arrow
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::MinusEq
                    } else {
                        TokenKind::Minus
                    }
                }

                '/' => {
                    if self.peek_char() == Some('/') {
                        self.advance();
                        if self.peek_char() == Some('/') {
                            self.advance();
                            let comment = self.read_line_comment();
                            TokenKind::DocComment(comment)
                        } else {
                            let comment = self.read_line_comment();
                            TokenKind::Comment(comment)
                        }
                    } else if self.peek_char() == Some('*') {
                        self.advance();
                        self.read_block_comment()
                    } else if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::SlashEq
                    } else {
                        TokenKind::Slash
                    }
                }

                '=' => {
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::EqEq
                    } else if self.peek_char() == Some('>') {
                        self.advance();
                        TokenKind::DDArrow
                    } else {
                        TokenKind::Eq
                    }
                }

                '!' => {
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::NotEq
                    } else {
                        TokenKind::Not
                    }
                }

                '<' => {
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::LtEq
                    } else {
                        TokenKind::Lt
                    }
                }

                '>' => {
                    if self.peek_char() == Some('=') {
                        self.advance();
                        TokenKind::GtEq
                    } else {
                        TokenKind::Gt
                    }
                }

                '&' => {
                    if self.peek_char() == Some('&') {
                        self.advance();
                        TokenKind::And
                    } else {
                        TokenKind::Ampersand
                    }
                }

                '|' => {
                    if self.peek_char() == Some('|') {
                        self.advance();
                        TokenKind::Or
                    } else {
                        TokenKind::Pipe
                    }
                }

                ':' => {
                    if self.peek_char() == Some(':') {
                        self.advance();
                        TokenKind::ColonColon
                    } else {
                        TokenKind::Colon
                    }
                }

                '.' => {
                    if self.peek_char() == Some('.') {
                        self.advance();
                        if self.peek_char() == Some('.') {
                            self.advance();
                            TokenKind::DotDotDot
                        } else {
                            TokenKind::DotDot
                        }
                    } else {
                        TokenKind::Dot
                    }
                }

                // String literals
                '"' => self.read_string(),

                // Char literals - treat as single-character strings
                '\'' => self.read_char_as_string(),

                // Newlines (skip if inside delimiters, otherwise create token)
                '\n' => {
                    self.line += 1;
                    self.line_start = self.current_pos;
                    self.column = 1;

                    // Skip newlines when inside any delimiters
                    if self.paren_depth > 0 || self.bracket_depth > 0 || self.brace_depth > 0 {
                        return self.next_token(); // Skip this newline, get next token
                    }

                    TokenKind::Newline
                }

                // Numbers
                c if c.is_ascii_digit() => self.read_number(c),

                // Identifiers and keywords
                c if c.is_alphabetic() || c == '_' => self.read_identifier(c),

                // Unknown character
                c => TokenKind::Error(format!("Unexpected character: '{}'", c)),
            },
        };

        let span = Span::new(start, self.current_pos, start_line, start_column);
        Token::new(kind, span)
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        let result = self.chars.next();
        if let Some((pos, c)) = result {
            self.current_pos = pos + c.len_utf8();
            if c != '\n' {
                self.column += 1;
            }
        }
        result
    }

    fn peek_char(&mut self) -> Option<char> {
        self.chars.peek().map(|(_, c)| *c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek_char() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_line_comment(&mut self) -> String {
        let start = self.current_pos;
        while let Some(c) = self.peek_char() {
            if c == '\n' {
                break;
            }
            self.advance();
        }
        self.source[start..self.current_pos].trim().to_string()
    }

    fn read_block_comment(&mut self) -> TokenKind {
        let mut comment = String::new();
        loop {
            match self.advance() {
                None => return TokenKind::Error("Unterminated block comment".to_string()),
                Some((_, '*')) => {
                    if self.peek_char() == Some('/') {
                        self.advance();
                        break;
                    } else {
                        comment.push('*');
                    }
                }
                Some((_, '\n')) => {
                    self.line += 1;
                    self.line_start = self.current_pos;
                    self.column = 1;
                    comment.push('\n');
                }
                Some((_, c)) => comment.push(c),
            }
        }
        TokenKind::Comment(comment.trim().to_string())
    }

    fn read_string(&mut self) -> TokenKind {
        let mut string = String::new();
        loop {
            match self.advance() {
                None => return TokenKind::Error("Unterminated string".to_string()),
                Some((_, '"')) => break,
                Some((_, '\\')) => {
                    // Handle escape sequences
                    match self.advance() {
                        Some((_, 'n')) => string.push('\n'),
                        Some((_, 't')) => string.push('\t'),
                        Some((_, 'r')) => string.push('\r'),
                        Some((_, '\\')) => string.push('\\'),
                        Some((_, '"')) => string.push('"'),
                        Some((_, c)) => {
                            return TokenKind::Error(format!("Invalid escape sequence: \\{}", c))
                        }
                        None => return TokenKind::Error("Unterminated escape sequence".to_string()),
                    }
                }
                Some((_, c)) => string.push(c),
            }
        }
        TokenKind::StringLit(string)
    }

    fn read_char_as_string(&mut self) -> TokenKind {
        // Read a char literal 'x' and treat it as a single-character string "x"
        match self.advance() {
            None => return TokenKind::Error("Unterminated char literal".to_string()),
            Some((_, '\\')) => {
                // Handle escape sequences
                match self.advance() {
                    Some((_, 'n')) => {
                        if !self.expect_char('\'') {
                            return TokenKind::Error("Unterminated char literal".to_string());
                        }
                        TokenKind::StringLit("\n".to_string())
                    }
                    Some((_, 't')) => {
                        if !self.expect_char('\'') {
                            return TokenKind::Error("Unterminated char literal".to_string());
                        }
                        TokenKind::StringLit("\t".to_string())
                    }
                    Some((_, 'r')) => {
                        if !self.expect_char('\'') {
                            return TokenKind::Error("Unterminated char literal".to_string());
                        }
                        TokenKind::StringLit("\r".to_string())
                    }
                    Some((_, '\\')) => {
                        if !self.expect_char('\'') {
                            return TokenKind::Error("Unterminated char literal".to_string());
                        }
                        TokenKind::StringLit("\\".to_string())
                    }
                    Some((_, '\'')) => {
                        if !self.expect_char('\'') {
                            return TokenKind::Error("Unterminated char literal".to_string());
                        }
                        TokenKind::StringLit("'".to_string())
                    }
                    Some((_, c)) => {
                        TokenKind::Error(format!("Invalid escape sequence in char literal: \\{}", c))
                    }
                    None => TokenKind::Error("Unterminated escape sequence in char literal".to_string()),
                }
            }
            Some((_, '\'')) => {
                TokenKind::Error("Empty char literal".to_string())
            }
            Some((_, c)) => {
                // Regular character
                if !self.expect_char('\'') {
                    return TokenKind::Error("Unterminated char literal".to_string());
                }
                TokenKind::StringLit(c.to_string())
            }
        }
    }

    fn expect_char(&mut self, expected: char) -> bool {
        match self.advance() {
            Some((_, c)) if c == expected => true,
            _ => false,
        }
    }

    fn read_number(&mut self, first: char) -> TokenKind {
        // Check for hex or binary prefix
        if first == '0' {
            if let Some(c) = self.peek_char() {
                if c == 'x' || c == 'X' {
                    self.advance();
                    return self.read_hex_number();
                } else if c == 'b' || c == 'B' {
                    self.advance();
                    return self.read_binary_number();
                }
            }
        }

        let mut number = String::new();
        number.push(first);
        let mut is_float = false;

        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                number.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                // Check if this is a float or a method call
                let next_after_dot = {
                    let mut temp_chars = self.source[self.current_pos..].chars();
                    temp_chars.next(); // skip the dot
                    temp_chars.next()
                };
                if next_after_dot.map_or(false, |c| c.is_ascii_digit()) {
                    is_float = true;
                    number.push(c);
                    self.advance();
                } else {
                    break;
                }
            } else if c == '_' {
                // Allow underscores in numbers (like 1_000_000)
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            match number.parse::<f64>() {
                Ok(n) => TokenKind::FloatLit(n),
                Err(_) => TokenKind::Error(format!("Invalid float: {}", number)),
            }
        } else {
            match number.parse::<i64>() {
                Ok(n) => TokenKind::IntLit(n),
                Err(_) => TokenKind::Error(format!("Invalid integer: {}", number)),
            }
        }
    }

    fn read_hex_number(&mut self) -> TokenKind {
        let mut number = String::new();
        while let Some(c) = self.peek_char() {
            if c.is_ascii_hexdigit() {
                number.push(c);
                self.advance();
            } else if c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if number.is_empty() {
            return TokenKind::Error("Invalid hex number".to_string());
        }
        match i64::from_str_radix(&number, 16) {
            Ok(n) => TokenKind::IntLit(n),
            Err(_) => TokenKind::Error(format!("Invalid hex number: 0x{}", number)),
        }
    }

    fn read_binary_number(&mut self) -> TokenKind {
        let mut number = String::new();
        while let Some(c) = self.peek_char() {
            if c == '0' || c == '1' {
                number.push(c);
                self.advance();
            } else if c == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if number.is_empty() {
            return TokenKind::Error("Invalid binary number".to_string());
        }
        match i64::from_str_radix(&number, 2) {
            Ok(n) => TokenKind::IntLit(n),
            Err(_) => TokenKind::Error(format!("Invalid binary number: 0b{}", number)),
        }
    }

    fn read_identifier(&mut self, first: char) -> TokenKind {
        let mut ident = String::new();
        ident.push(first);

        while let Some(c) = self.peek_char() {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.advance();
            } else {
                break;
            }
        }

        // Check for matches! macro
        if ident == "matches" && self.peek_char() == Some('!') {
            self.advance();
            return TokenKind::Matches;
        }

        // Check for keywords
        match ident.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "const" => TokenKind::Const,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "while" => TokenKind::While,
            "loop" => TokenKind::Loop,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "null" => TokenKind::Null,
            "plugin" => TokenKind::Plugin,
            "writer" => TokenKind::Writer,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "impl" => TokenKind::Impl,
            "use" => TokenKind::Use,
            "pub" => TokenKind::Pub,
            "as" => TokenKind::As,
            "self" => TokenKind::Self_,
            "Self" => TokenKind::SelfType,
            "match" => TokenKind::Match,
            "traverse" => TokenKind::Traverse,
            "using" => TokenKind::Using,
            "capturing" => TokenKind::Capturing,

            // Type keywords
            "Str" => TokenKind::Str,
            "bool" => TokenKind::Bool,
            "i32" => TokenKind::I32,
            "u32" => TokenKind::U32,
            "f64" => TokenKind::F64,
            "Vec" => TokenKind::Vec,
            "Option" => TokenKind::Option,
            "Result" => TokenKind::Result,
            "HashMap" => TokenKind::HashMap,
            "HashSet" => TokenKind::HashSet,
            "CodeBuilder" => TokenKind::CodeBuilder,

            // AST Node Type keywords
            "Program" => TokenKind::Program,
            "FunctionDeclaration" => TokenKind::FunctionDeclaration,
            "VariableDeclaration" => TokenKind::VariableDeclaration,
            "ExpressionStatement" => TokenKind::ExpressionStatement,
            "ReturnStatement" => TokenKind::ReturnStatement,
            "IfStatement" => TokenKind::IfStatement,
            "ForStatement" => TokenKind::ForStatement,
            "WhileStatement" => TokenKind::WhileStatement,
            "BlockStatement" => TokenKind::BlockStatement,
            "Identifier" => TokenKind::Identifier,
            "Literal" => TokenKind::Literal,
            "BinaryExpression" => TokenKind::BinaryExpression,
            "UnaryExpression" => TokenKind::UnaryExpression,
            "CallExpression" => TokenKind::CallExpression,
            "MemberExpression" => TokenKind::MemberExpression,
            "ArrayExpression" => TokenKind::ArrayExpression,
            "ObjectExpression" => TokenKind::ObjectExpression,
            "JSXElement" => TokenKind::JSXElement,
            "JSXFragment" => TokenKind::JSXFragment,
            "JSXAttribute" => TokenKind::JSXAttribute,
            "JSXText" => TokenKind::JSXText,
            "JSXExpressionContainer" => TokenKind::JSXExpressionContainer,

            // Regular identifier
            _ => TokenKind::Ident(ident),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("( ) { } [ ]");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::LParen));
        assert!(matches!(tokens[1].kind, TokenKind::RParen));
        assert!(matches!(tokens[2].kind, TokenKind::LBrace));
        assert!(matches!(tokens[3].kind, TokenKind::RBrace));
        assert!(matches!(tokens[4].kind, TokenKind::LBracket));
        assert!(matches!(tokens[5].kind, TokenKind::RBracket));
    }

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("fn let mut if else for in while return");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::Fn));
        assert!(matches!(tokens[1].kind, TokenKind::Let));
        assert!(matches!(tokens[2].kind, TokenKind::Mut));
        assert!(matches!(tokens[3].kind, TokenKind::If));
        assert!(matches!(tokens[4].kind, TokenKind::Else));
        assert!(matches!(tokens[5].kind, TokenKind::For));
        assert!(matches!(tokens[6].kind, TokenKind::In));
        assert!(matches!(tokens[7].kind, TokenKind::While));
        assert!(matches!(tokens[8].kind, TokenKind::Return));
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / = == != < > <= >= && || !");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::Plus));
        assert!(matches!(tokens[1].kind, TokenKind::Minus));
        assert!(matches!(tokens[2].kind, TokenKind::Star));
        assert!(matches!(tokens[3].kind, TokenKind::Slash));
        assert!(matches!(tokens[4].kind, TokenKind::Eq));
        assert!(matches!(tokens[5].kind, TokenKind::EqEq));
        assert!(matches!(tokens[6].kind, TokenKind::NotEq));
        assert!(matches!(tokens[7].kind, TokenKind::Lt));
        assert!(matches!(tokens[8].kind, TokenKind::Gt));
        assert!(matches!(tokens[9].kind, TokenKind::LtEq));
        assert!(matches!(tokens[10].kind, TokenKind::GtEq));
        assert!(matches!(tokens[11].kind, TokenKind::And));
        assert!(matches!(tokens[12].kind, TokenKind::Or));
        assert!(matches!(tokens[13].kind, TokenKind::Not));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("\"hello world\"");
        let tokens = lexer.tokenize();
        assert!(matches!(&tokens[0].kind, TokenKind::StringLit(s) if s == "hello world"));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 1_000");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::IntLit(42)));
        assert!(matches!(tokens[1].kind, TokenKind::FloatLit(n) if (n - 3.14).abs() < 0.001));
        assert!(matches!(tokens[2].kind, TokenKind::IntLit(1000)));
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz _test");
        let tokens = lexer.tokenize();
        assert!(matches!(&tokens[0].kind, TokenKind::Ident(s) if s == "foo"));
        assert!(matches!(&tokens[1].kind, TokenKind::Ident(s) if s == "bar_baz"));
        assert!(matches!(&tokens[2].kind, TokenKind::Ident(s) if s == "_test"));
    }

    #[test]
    fn test_plugin_declaration() {
        let mut lexer = Lexer::new("plugin MyPlugin { fn visit_program(node: &mut Program) { } }");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::Plugin));
        assert!(matches!(&tokens[1].kind, TokenKind::Ident(s) if s == "MyPlugin"));
        assert!(matches!(tokens[2].kind, TokenKind::LBrace));
        assert!(matches!(tokens[3].kind, TokenKind::Fn));
    }

    #[test]
    fn test_matches_macro() {
        let mut lexer = Lexer::new("matches!(node, FunctionDeclaration)");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::Matches));
        assert!(matches!(tokens[1].kind, TokenKind::LParen));
    }

    #[test]
    fn test_arrow_and_fat_arrow() {
        let mut lexer = Lexer::new("-> =>");
        let tokens = lexer.tokenize();
        assert!(matches!(tokens[0].kind, TokenKind::Arrow));
        assert!(matches!(tokens[1].kind, TokenKind::DDArrow));
    }

    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("// this is a comment\n/// this is a doc comment");
        let tokens = lexer.tokenize();
        assert!(matches!(&tokens[0].kind, TokenKind::Comment(s) if s == "this is a comment"));
        assert!(matches!(tokens[1].kind, TokenKind::Newline));
        assert!(matches!(&tokens[2].kind, TokenKind::DocComment(s) if s == "this is a doc comment"));
    }
}
