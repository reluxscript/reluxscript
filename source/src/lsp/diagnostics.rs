use tower_lsp::lsp_types::*;

pub fn parse_error_to_diagnostic(error: crate::parser::parser::ParseError) -> Diagnostic {
    Diagnostic {
        range: Range {
            start: Position {
                line: (error.span.start.line.saturating_sub(1)) as u32,
                character: (error.span.start.column.saturating_sub(1)) as u32,
            },
            end: Position {
                line: (error.span.end.line.saturating_sub(1)) as u32,
                character: (error.span.end.column.saturating_sub(1)) as u32,
            },
        },
        severity: Some(DiagnosticSeverity::ERROR),
        message: error.message,
        source: Some("reluxscript".to_string()),
        ..Default::default()
    }
}

pub fn semantic_errors_to_diagnostics(errors: Vec<crate::semantic::SemanticError>) -> Vec<Diagnostic> {
    errors.into_iter().map(|error| {
        Diagnostic {
            range: Range {
                start: Position {
                    line: (error.span.start.line.saturating_sub(1)) as u32,
                    character: (error.span.start.column.saturating_sub(1)) as u32,
                },
                end: Position {
                    line: (error.span.end.line.saturating_sub(1)) as u32,
                    character: (error.span.end.column.saturating_sub(1)) as u32,
                },
            },
            severity: Some(DiagnosticSeverity::ERROR),
            message: error.message,
            source: Some("reluxscript".to_string()),
            ..Default::default()
        }
    }).collect()
}
