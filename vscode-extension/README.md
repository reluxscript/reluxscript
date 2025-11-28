# ReluxScript VS Code Extension

Language support for [ReluxScript](https://github.com/reluxscript/reluxscript) - Write AST transformations once, compile to both Babel and SWC.

## Features

- **Syntax Highlighting** - Full syntax highlighting for `.lux` files
- **Real-time Diagnostics** - Parse and semantic errors as you type
- **IntelliSense** - Code completion for keywords, AST types, and more
- **Hover Information** - Type information and documentation on hover
- **Go to Definition** - Navigate to symbol definitions
- **Compile Commands** - Quick commands to compile to Babel, SWC, or both

## Requirements

You need to have the ReluxScript compiler installed:

```bash
cargo install reluxscript --features lsp
```

Or build from source:

```bash
git clone https://github.com/reluxscript/reluxscript
cd reluxscript/source
cargo build --release --features lsp --bin reluxscript-lsp
```

## Extension Commands

- `ReluxScript: Compile to Babel` - Compile current file to JavaScript (Babel plugin)
- `ReluxScript: Compile to SWC` - Compile current file to Rust (SWC plugin)
- `ReluxScript: Compile to Both` - Compile to both targets

## Development

To develop the extension:

1. Install dependencies:
   ```bash
   npm install
   ```

2. Build the extension:
   ```bash
   npm run compile
   ```

3. Press F5 in VS Code to launch Extension Development Host

## Release Notes

### 0.1.0

Initial release:
- Syntax highlighting
- Language server integration
- Basic diagnostics
- Compile commands
