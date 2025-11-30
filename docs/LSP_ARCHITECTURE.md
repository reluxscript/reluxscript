# ReluxScript LSP Architecture

## Design Philosophy

**Hybrid Approach: TypeScript + Rust**

- **TypeScript (VS Code Extension):** UI, commands, LSP client
- **Rust (Language Server):** Parsing, semantic analysis, type checking

This maximizes developer experience while maintaining performance for heavy operations.

## Component Breakdown

### 1. Rust Language Server (`source/src/lsp/`)

#### `mod.rs` - Entry Point
```rust
pub async fn start_server()
```
- Initializes stdio communication
- Creates LspService with server instance
- Starts tower-lsp Server

#### `server.rs` - LanguageServer Implementation
```rust
pub struct ReluxScriptLanguageServer {
    client: Client,              // Communication to VS Code
    documents: HashMap<Url, DocumentState>,  // Document cache
}
```

**Key Methods:**
- `initialize()` - Announces server capabilities
- `did_open()` - Parse + analyze new document
- `did_change()` - Reparse on edits
- `hover()` - Type info on hover (TODO)
- `completion()` - Code completions (basic)

#### `diagnostics.rs` - Error Conversion
Converts compiler errors → LSP Diagnostics with:
- Source range (line, column)
- Severity (ERROR, WARNING, INFO)
- Message text
- Source identifier

#### `handlers.rs` - Future Features
Placeholder for:
- Go to definition
- Find references
- Rename
- Code actions

### 2. VS Code Extension (`vscode-extension/`)

#### `src/extension.ts` - Extension Entry
```typescript
export function activate(context: vscode.ExtensionContext)
```

**Responsibilities:**
1. Find LSP binary (`reluxscript-lsp`)
2. Start LanguageClient
3. Register compile commands
4. Handle lifecycle

**LSP Binary Discovery:**
```
1. source/target/debug/reluxscript-lsp[.exe]
2. source/target/release/reluxscript-lsp[.exe]
3. vscode-extension/bin/reluxscript-lsp[.exe]
4. System PATH
```

#### `syntaxes/reluxscript.tmLanguage.json` - Syntax Highlighting

TextMate grammar with patterns for:
- **Keywords:** `plugin`, `writer`, `fn`, `if`, `match`, etc.
- **AST Types:** `Expression`, `Statement`, `Pattern`, etc.
- **Literals:** strings, numbers, booleans
- **Comments:** `//` and `/* */`
- **Operators:** `+`, `==`, `&&`, `=>`, etc.

#### `package.json` - Extension Manifest

Defines:
- Language ID: `reluxscript`
- File extension: `.lux`
- Commands: compile to Babel, SWC, or both
- Activation events

## Communication Flow

### Document Open/Edit

```
1. User opens test.lux in VS Code
   ↓
2. Extension sends textDocument/didOpen (JSON-RPC)
   ↓
3. LSP receives notification
   ↓
4. LSP calls analyze_document()
   - Lexer::tokenize(content)
   - Parser::parse(tokens)
   - SemanticAnalyzer::analyze(ast)
   ↓
5. LSP sends textDocument/publishDiagnostics
   ↓
6. VS Code displays red squiggles
```

### Completion Request

```
1. User types "plu|" (cursor at |)
   ↓
2. Extension sends textDocument/completion
   ↓
3. LSP receives { position: { line: 0, character: 3 } }
   ↓
4. LSP returns completion items:
   [{ label: "plugin", kind: Keyword }, ...]
   ↓
5. VS Code shows completion menu
```

## Data Structures

### DocumentState (Rust)
```rust
struct DocumentState {
    content: String,              // Full text
    version: i32,                 // Increments on change
    ast: Option<Program>,         // Parsed AST (cached)
    diagnostics: Vec<Diagnostic>, // Current errors
}
```

### Diagnostic (LSP)
```rust
struct Diagnostic {
    range: Range,                 // start/end position
    severity: DiagnosticSeverity, // ERROR, WARNING, INFO, HINT
    message: String,              // Error text
    source: String,               // "reluxscript"
}
```

## Performance Considerations

### Current: Full Text Sync
- VS Code sends entire document on each change
- Simple, but inefficient for large files

### Future: Incremental Sync
```rust
text_document_sync: TextDocumentSyncCapability::Options(
    TextDocumentSyncOptions {
        change: Some(TextDocumentSyncKind::INCREMENTAL),
        ...
    }
)
```
- VS Code sends only changed ranges
- More complex but much faster

### Caching Strategy

1. **Parse once, use many times**
   ```rust
   if let Some(doc) = documents.get(uri) {
       if doc.version == old_version {
           return doc.diagnostics.clone(); // Cached!
       }
   }
   ```

2. **Incremental reparsing (future)**
   - Only reparse affected scope
   - Reuse unchanged subtrees

## Error Handling

### Graceful Degradation

```rust
match Parser::parse(content) {
    Ok(ast) => {
        // Continue to semantic analysis
    }
    Err(parse_error) => {
        // Return parse diagnostic, stop here
        return vec![parse_error_to_diagnostic(parse_error)];
    }
}
```

No crash if code is invalid - just show diagnostics.

### Span Mapping

ReluxScript uses 1-indexed lines/columns:
```rust
Position {
    line: (error.span.start.line - 1) as u32,    // LSP is 0-indexed
    character: (error.span.start.column - 1) as u32,
}
```

## Feature Flags

```toml
[features]
lsp = ["tower-lsp", "tokio"]
```

Why feature flags?
- **Optional dependency:** LSP not needed for CLI compiler
- **Faster builds:** Don't compile tokio unless needed
- **Cleaner separation:** LSP is an add-on, not core

## Testing Strategy

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_error_to_diagnostic() {
        let error = ParseError { ... };
        let diagnostic = parse_error_to_diagnostic(error);
        assert_eq!(diagnostic.severity, DiagnosticSeverity::ERROR);
    }
}
```

### Integration Tests (VS Code)
```typescript
// vscode-extension/src/test/
suite('Extension Tests', () => {
    test('Syntax highlighting for keywords', async () => {
        // Open .lux file
        // Check token scopes
    });
});
```

## Extending the LSP

### Adding Hover Info

1. **Rust:** Store symbol table in DocumentState
```rust
struct DocumentState {
    symbols: HashMap<Position, Symbol>,
}

struct Symbol {
    name: String,
    type_info: TypeInfo,
    documentation: String,
}
```

2. **Rust:** Implement hover handler
```rust
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    let pos = params.text_document_position_params.position;
    if let Some(symbol) = self.symbols.get(&pos) {
        return Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**{}**\n\nType: {}", symbol.name, symbol.type_info),
            }),
            range: None,
        }));
    }
    Ok(None)
}
```

### Adding Go to Definition

1. **Rust:** Track definition locations
```rust
struct Symbol {
    definition: Location,  // Where it's defined
    references: Vec<Location>,  // Where it's used
}
```

2. **Rust:** Implement definition handler
```rust
async fn definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
    // Look up symbol at cursor
    // Return definition location
}
```

## Known Limitations

1. **No workspace support yet** - Each file analyzed independently
2. **No incremental parsing** - Reparses entire file on change
3. **Basic completions only** - Not context-aware yet
4. **No cross-file resolution** - `use` statements not followed

## Future Roadmap

### Phase 1: Core Features (Weeks 1-2)
- ✅ Document sync
- ✅ Diagnostics
- ✅ Syntax highlighting
- 🚧 Hover info
- 🚧 Go to definition

### Phase 2: Intelligence (Weeks 3-4)
- 🚧 Context-aware completions
- 🚧 Find references
- 🚧 Rename refactoring

### Phase 3: Advanced (Weeks 5-6)
- 🚧 Code actions (quick fixes)
- 🚧 Document formatting
- 🚧 Semantic tokens (better highlighting)

### Phase 4: Performance (Ongoing)
- 🚧 Incremental parsing
- 🚧 Workspace support
- 🚧 Multi-file analysis

## Debugging

### Enable LSP Logging

Set environment variable:
```bash
RUST_LOG=tower_lsp=debug reluxscript-lsp
```

### VS Code Developer Tools

1. Help → Toggle Developer Tools
2. Console tab shows extension logs
3. Network tab shows JSON-RPC messages

### Attach Debugger to LSP

```bash
# Terminal 1: Run LSP manually
RUST_LOG=debug cargo run --features lsp --bin reluxscript-lsp

# Terminal 2: Tell VS Code to connect
# (Modify extension to use TCP instead of stdio)
```

## Best Practices

1. **Keep emitter dumb** - All logic in parser/semantic analysis
2. **Cache aggressively** - Reparse only when necessary
3. **Fail gracefully** - Invalid code should show diagnostics, not crash
4. **Test with real code** - Use actual .lux files from examples/
5. **Profile regularly** - Watch for slow operations on large files

---

**The hybrid LSP gives you the best of both worlds: TypeScript for VS Code integration, Rust for performance!** ⚡
