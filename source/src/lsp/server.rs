use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::parser::Parser;
use crate::semantic::SemanticAnalyzer;

pub struct ReluxScriptLanguageServer {
    client: Client,
    documents: Arc<Mutex<HashMap<Url, DocumentState>>>,
}

struct DocumentState {
    content: String,
    version: i32,
    ast: Option<crate::parser::ast::Program>,
    diagnostics: Vec<Diagnostic>,
}

impl ReluxScriptLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn analyze_document(&self, uri: &Url, content: &str) -> Vec<Diagnostic> {
        use crate::lsp::diagnostics::{parse_error_to_diagnostic, semantic_errors_to_diagnostics};

        let mut diagnostics = Vec::new();

        // Tokenize and parse
        let tokens = match crate::lexer::Lexer::tokenize(content) {
            Ok(tokens) => tokens,
            Err(e) => {
                // Lexer error
                diagnostics.push(Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 0 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Lexer error: {}", e),
                    source: Some("reluxscript".to_string()),
                    ..Default::default()
                });
                return diagnostics;
            }
        };

        // Parse
        match Parser::new(tokens).parse() {
            Ok(ast) => {
                // Semantic analysis
                match SemanticAnalyzer::new().analyze(&ast) {
                    Ok(_) => {
                        // Success - no diagnostics
                    }
                    Err(errors) => {
                        // Convert semantic errors to diagnostics
                        diagnostics.extend(semantic_errors_to_diagnostics(errors));
                    }
                }
            }
            Err(parse_error) => {
                // Convert parse error to diagnostic
                diagnostics.push(parse_error_to_diagnostic(parse_error));
            }
        }

        diagnostics
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ReluxScriptLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ":".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "ReluxScript language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let content = params.text_document.text;
        let version = params.text_document.version;

        // Analyze document
        let diagnostics = self.analyze_document(&uri, &content).await;

        // Store document state
        let mut documents = self.documents.lock().await;
        documents.insert(
            uri.clone(),
            DocumentState {
                content: content.clone(),
                version,
                ast: None,
                diagnostics: diagnostics.clone(),
            },
        );

        // Publish diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.into_iter().next() {
            let content = change.text;

            // Analyze document
            let diagnostics = self.analyze_document(&uri, &content).await;

            // Update document state
            let mut documents = self.documents.lock().await;
            if let Some(doc) = documents.get_mut(&uri) {
                doc.content = content;
                doc.version = version;
                doc.diagnostics = diagnostics.clone();
            }

            // Publish diagnostics
            self.client
                .publish_diagnostics(uri, diagnostics, Some(version))
                .await;
        }
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // TODO: Get hover info from AST at position
        // For now, return placeholder
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "ReluxScript hover info".to_string(),
            )),
            range: None,
        }))
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        use crate::lsp::completions::*;

        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let documents = self.documents.lock().await;
        let doc = match documents.get(uri) {
            Some(doc) => doc,
            None => return Ok(None),
        };

        // Get the line and character position
        let lines: Vec<&str> = doc.content.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;

        if line_idx >= lines.len() {
            return Ok(None);
        }

        let line = lines[line_idx];
        let text_before_cursor = &line[..char_idx.min(line.len())];

        // Detect completion context
        let mut completions = Vec::new();

        // Check if we're after a dot (field/method completion)
        if text_before_cursor.ends_with('.') {
            // Extract the identifier before the dot
            let before_dot = text_before_cursor.trim_end_matches('.');
            if let Some(last_word) = before_dot.split_whitespace().last() {
                // Try to determine the type of the identifier
                // For now, use simple heuristics
                if last_word == "node" || last_word.starts_with("node") {
                    // Assume it's a common AST node, offer common fields
                    completions.extend(get_field_completions_for_type("CallExpression"));
                    completions.extend(get_field_completions_for_type("MemberExpression"));
                    completions.extend(get_field_completions_for_type("Identifier"));
                } else if last_word.ends_with("_str") || last_word.contains("name") {
                    // String methods
                    completions.extend(get_method_completions_for_type("String"));
                } else if last_word.ends_with("_vec") || last_word.contains("items") {
                    // Vec methods
                    completions.extend(get_method_completions_for_type("Vec"));
                }
            }
        }
        // Check if we're in a type position (after : or <)
        else if text_before_cursor.ends_with(':') || text_before_cursor.ends_with('<') || text_before_cursor.contains(": ") {
            // Type completions
            completions.extend(get_ast_type_completions());
            completions.extend(get_builtin_type_completions());
        }
        // Check if we're starting a visitor method name
        else if text_before_cursor.contains("fn visit_") || text_before_cursor.ends_with("visit_") {
            // Snippet completions for visitor methods
            completions.extend(get_snippet_completions());
        }
        // Check if we're in a match arm or if-let pattern
        else if text_before_cursor.contains("match ") || text_before_cursor.contains("if let ") || text_before_cursor.contains("=> ") {
            // Pattern completions
            completions.extend(get_common_pattern_completions());
            completions.extend(get_keyword_completions());
        }
        // Default: offer everything
        else {
            completions.extend(get_keyword_completions());
            completions.extend(get_ast_type_completions());
            completions.extend(get_builtin_type_completions());
            completions.extend(get_macro_completions());
            completions.extend(get_snippet_completions());
        }

        // Remove duplicates
        completions.dedup_by(|a, b| a.label == b.label);

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn formatting(&self, _params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        // TODO: Implement formatting
        Ok(None)
    }
}
