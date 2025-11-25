# Building the Playground

The playground uses WebAssembly to run the ReluxScript compiler in the browser.

## Prerequisites

```bash
# Install wasm-pack
cargo install wasm-pack
```

## Build Steps

1. **Build the WASM module**:
```bash
cd ../source
wasm-pack build --target web --features wasm,codegen --out-dir ../site/pkg
```

2. **Test locally**:
```bash
cd ../site
npm run dev
```

3. **Visit the playground**:
Open http://localhost:5173/playground.html

## Deployment

When deploying the site, make sure the `public/pkg` directory is included with all the WASM files.

## File Structure

```
site/
├── playground.html          # Playground UI
├── vite.config.js           # Vite configuration for WASM
├── pkg/                     # WASM build output (NOT in public/)
│   ├── reluxscript.js
│   ├── reluxscript_bg.wasm
│   └── reluxscript.d.ts
└── src/
    └── main.js              # Updated with Playground link
```

## Troubleshooting

If you get errors about missing dependencies, make sure you have the `wasm` feature enabled in Cargo.toml and all WASM dependencies are installed.
