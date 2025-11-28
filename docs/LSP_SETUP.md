# ReluxScript LSP Setup Guide

This guide will help you set up the ReluxScript Language Server and VS Code extension.

## Architecture Overview

```
┌─────────────────────────────────────┐
│     VS Code Extension (TS)          │
│  - Syntax highlighting              │
│  - Commands (compile, format)       │
│  - Diagnostics display              │
│  - Hover/completion UI              │
└──────────────┬──────────────────────┘
               │ stdio/JSON-RPC 2.0
               ▼
┌─────────────────────────────────────┐
│   Language Server (Rust)            │
│  - tower-lsp framework              │
│  - JSON-RPC handling                │
│  - Document management              │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   ReluxScript Compiler (Rust)       │
│  - Lexer & Parser                   │
│  - Semantic analyzer                │
│  - Type checker                     │
│  - Decorator/Rewriter/Emitter       │
└─────────────────────────────────────┘
```

## Quick Start

### 1. Build the Language Server

```bash
cd source
cargo build --release --features lsp --bin reluxscript-lsp
```

This creates the language server binary at:
- Linux/Mac: `target/release/reluxscript-lsp`
- Windows: `target/release/reluxscript-lsp.exe`

### 2. Install VS Code Extension Dependencies

```bash
cd vscode-extension
npm install
```

### 3. Compile the Extension

```bash
npm run compile
```

### 4. Test the Extension

#### Option A: Launch Extension Development Host

1. Open `vscode-extension/` in VS Code
2. Press `F5` to launch Extension Development Host
3. In the new window, open a `.lux` file

#### Option B: Install Extension Locally

```bash
cd vscode-extension
npm install -g @vscode/vsce
vsce package
code --install-extension reluxscript-0.1.0.vsix
```

## Development Workflow

### Watch Mode for Extension

```bash
cd vscode-extension
npm run watch
```

This will recompile TypeScript on file changes.

### Rebuild Language Server

```bash
cd source
cargo build --features lsp --bin reluxscript-lsp
```

After rebuilding, restart the Extension Development Host (Ctrl+R in the extension host window).

## Features Implemented

### ✅ Working

- **Syntax Highlighting** - Full TextMate grammar for `.lux` files
- **Document Sync** - Full text synchronization between VS Code and LSP
- **Diagnostics** - Parse errors and semantic errors
- **Basic Completions** - Keywords (plugin, writer, fn, etc.)
- **Commands** - Compile to Babel, SWC, or both

### 🚧 TODO (Stubbed)

- **Hover Information** - Currently returns placeholder
- **Go to Definition** - Not implemented
- **Find References** - Not implemented
- **Document Formatting** - Not implemented
- **Advanced Completions** - Context-aware completions for AST types, fields, etc.

## Troubleshooting

### Language Server Not Starting

**Symptom:** Extension shows warning "ReluxScript language server not found"

**Solutions:**
1. Build the language server: `cargo build --features lsp --bin reluxscript-lsp`
2. Check the path in extension output (View → Output → ReluxScript)
3. Verify binary exists at one of these locations:
   - `source/target/debug/reluxscript-lsp[.exe]`
   - `source/target/release/reluxscript-lsp[.exe]`

### No Syntax Highlighting

**Symptom:** `.lux` files open as plain text

**Solutions:**
1. Check file association: Right-click file → "Change Language Mode" → "ReluxScript"
2. Verify `syntaxes/reluxscript.tmLanguage.json` exists
3. Reload VS Code window (Ctrl+Shift+P → "Reload Window")

### Diagnostics Not Showing

**Symptom:** Parse errors don't show as red squiggles

**Solutions:**
1. Check LSP server is running (View → Output → ReluxScript Language Server)
2. Verify file is recognized as `.lux`
3. Save the file to trigger analysis
4. Check for errors in Developer Tools (Help → Toggle Developer Tools)

## File Structure

```
reluxscript/
├── source/
│   ├── src/
│   │   ├── lsp/                    # LSP implementation
│   │   │   ├── mod.rs              # Module entry, start_server()
│   │   │   ├── server.rs           # LanguageServer trait impl
│   │   │   ├── diagnostics.rs     # Error → Diagnostic conversion
│   │   │   └── handlers.rs        # Future: advanced features
│   │   ├── bin/
│   │   │   └── reluxscript-lsp.rs # Binary entry point
│   │   └── lib.rs                 # Exports lsp module
│   └── Cargo.toml                 # Added tower-lsp, tokio deps
│
└── vscode-extension/
    ├── src/
    │   └── extension.ts           # Extension entry point
    ├── syntaxes/
    │   └── reluxscript.tmLanguage.json  # Syntax highlighting
    ├── package.json               # Extension manifest
    ├── tsconfig.json              # TypeScript config
    └── language-configuration.json # Brackets, comments, etc.
```

## Next Steps

### Phase 1: Enhanced Diagnostics (Recommended First)

Improve error messages with rich context:

```rust
// In diagnostics.rs
pub fn semantic_error_to_diagnostic_with_context(
    error: SemanticError,
    source: &str
) -> Diagnostic {
    // Add code snippets, suggestions, etc.
}
```

### Phase 2: Hover Information

Implement hover provider to show:
- Variable types
- Function signatures
- AST node documentation

```rust
// In server.rs
async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
    // Look up symbol at position
    // Return type info + docs
}
```

### Phase 3: Go to Definition

Navigate to:
- Function definitions
- Variable declarations
- Type definitions
- Imported symbols

### Phase 4: Advanced Completions

Context-aware completions for:
- AST node types based on current pattern context
- Field names based on matched node type
- Method names based on receiver type

### Phase 5: Code Actions

Quick fixes for common errors:
- Add missing imports
- Fix type mismatches
- Suggest `.clone()` for ownership errors

## Testing

### Manual Testing

1. Create a test file `test.lux`:
```lux
plugin RemoveDebugger {
    fn visit_call_expression(node: &mut CallExpression) {
        if node.callee.name == "debugger" {
            // Remove debugger calls
        }
    }
}
```

2. Open in Extension Development Host
3. Verify:
   - Syntax highlighting works
   - No diagnostics (valid code)
   - Completions work when typing

### Test Invalid Code

```lux
plugin Test {
    fn visit_foo(node: &mut Unknown) {
        // Should show: Unknown type 'Unknown'
    }
}
```

Verify diagnostic appears.

## Resources

- **tower-lsp docs:** https://docs.rs/tower-lsp
- **LSP Specification:** https://microsoft.github.io/language-server-protocol/
- **VS Code Extension API:** https://code.visualstudio.com/api
- **TextMate Grammar:** https://macromates.com/manual/en/language_grammars

## Tips

1. **Use Output Panel:** View → Output → select "ReluxScript Language Server" to see LSP logs
2. **Developer Tools:** Help → Toggle Developer Tools to debug extension TypeScript
3. **Restart Extension:** Ctrl+R in Extension Development Host after rebuilding
4. **Test Incrementally:** Add features one at a time, test each thoroughly
