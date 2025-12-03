# ReluxScript - Claude Code Context

## What is ReluxScript?

ReluxScript is a DSL for writing AST transformation plugins that compile to both Babel (JavaScript) and SWC (Rust). Users write plugins once in ReluxScript and get both targets.

## SWC Codegen Architecture

The SWC code generation follows a strict 4-stage pipeline:

```
Parser AST → Decorator → Rewriter → Hoister → Emitter
```

### Architecture Invariants

1. **All AST nodes must go through the full pipeline.** Never bypass decoration.

2. **The emitter must remain dumb.** It only prints decorated nodes. No type inference, no pattern transformation, no smart logic.

3. **DecoratedExprKind must never hold raw parser types.** Every variant should contain only decorated types (`DecoratedExpr`, `DecoratedStmt`, `DecoratedPattern`, etc.). This makes it structurally impossible to skip decoration.

4. **Type context flows through the decorator.** The decorator is where:
   - ReluxScript types map to SWC types (`Expression::Identifier` → `Expr::Ident`)
   - Field types are tracked (`node.init` is `Option<Box<Expr>>`)
   - Pattern context determines variant names
   - Field conversions are applied (`.callee` → `.callee.as_expr().unwrap()`)

### Why This Matters

If any node bypasses decoration:
- It skips the rewriter (no unwrap insertions, no pattern transforms)
- It skips the hoister (scope issues)
- The emitter receives raw AST and emits it literally - which is wrong for SWC

### Key Files

- `codegen/swc_decorator.rs` - Decorates raw AST with SWC type metadata
- `codegen/swc_rewriter.rs` - Transforms decorated AST (inserts unwraps, etc.)
- `codegen/swc_hoister.rs` - Handles hoisting and scope
- `codegen/swc_emit.rs` - Dumb emitter, just prints decorated nodes
- `codegen/decorated_ast.rs` - Decorated AST types

## Testing

The primary test case is `minimact_full.lux` - a real-world plugin that exercises most features.

`source/examples/super/super_kitchen_sink.lux` is a stress test containing 50+ intentionally unsupported features. Use it TDD-style: run it, pick an error, fix it, repeat.

## Build Commands

```bash
cd source
cargo build --release
./target/release/relux build path/to/plugin.lux
```
