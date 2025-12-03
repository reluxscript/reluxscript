# Refactor: XPath-Based Type Environment

## Summary

Assign a unique XPath to every AST node at parse time. Use these paths as keys in the TypeEnv. Eliminates name collisions and simplifies all downstream passes.

## Problem Statement

The current type system has a fundamental flaw: types are stored by **name** instead of by **path**.

```
TypeEnv: HashMap<String, TypeInfo>
  "self" → KitchenSink
  "self" → ComplexBinding  // Overwrites the previous!
  "node" → FnDecl
  "node" → VarDecl         // Overwrites again!
```

When multiple functions define variables with the same name, they collide. The "fix" of disabling scope push/pop just made the last definition win globally.

## Solution: XPath-Style Keys

Each AST node has a unique path from the root. Use this path as the key:

```
TypeEnv: HashMap<String, TypeInfo>
  "KitchenSink.visit_mut_fn_decl.self" → KitchenSink
  "KitchenSink.visit_mut_fn_decl.self.state" → State
  "KitchenSink.visit_mut_fn_decl.self.state.output" → Str
  "KitchenSink.visit_mut_fn_decl.node" → FnDecl
  "KitchenSink.visit_mut_var_decl.node" → VarDecl
  "ComplexBinding.with_depth.self" → ComplexBinding
  "ComplexBinding.with_depth.self.name" → Str
```

No collisions. No ambiguity. Parent lookups are trivial string operations.

## Path Format

Each AST node gets a unique path assigned at parse time. The path consists of segments, where each segment has a **name** and a **4-digit hex ID** for disambiguation:

```
<name>[<hex>].<name>[<hex>].<name>[<hex>]...
```

Examples:
```
KitchenSink[A1B2].visit_mut_fn_decl[3F01].self[7B4C]                    → &mut KitchenSink
KitchenSink[A1B2].visit_mut_fn_decl[3F01].self[7B4C].state[D2A1]        → State
KitchenSink[A1B2].visit_mut_fn_decl[3F01].self[7B4C].state[D2A1].output[9E3F] → Str
KitchenSink[A1B2].visit_mut_fn_decl[3F01].node[C4D5]                    → &mut FnDecl
KitchenSink[A1B2].visit_mut_var_decl[8E2A].node[F1C3]                   → &mut VarDecl
ComplexBinding[B3C4].with_depth[2D9E].self[A7F2]                        → &ComplexBinding
ComplexBinding[B3C4].with_depth[2D9E].self[A7F2].name[E5B1]             → Str
```

### Why Hex IDs?

**Shadowing** - Same name can appear multiple times in different scopes:
```rust
let x = 1;           // x[A001]
{
    let x = 2;       // x[A002] - different ID, no collision
}
```

**Anonymous scopes** - Blocks, match arms, closures all get IDs:
```
fn[3F01].match[B2C3].arm[D4E5].value[F6A7]
fn[3F01].if[C3D4].then[E5F6].result[A7B8]
fn[3F01].closure[D4E5].captured[F6A7]
```

### ID Generation

The parser maintains a simple counter (or uses position hash). Each new AST node gets the next ID:

```rust
struct Parser {
    next_id: u16,  // Wraps at 65536, but paths are unique anyway
}

impl Parser {
    fn next_hex_id(&mut self) -> String {
        let id = format!("{:04X}", self.next_id);
        self.next_id = self.next_id.wrapping_add(1);
        id
    }
}
```

### Benefits

1. **Guaranteed unique** - No collisions even with shadowing
2. **Human readable** - Names preserved for debugging
3. **Compact** - 4 hex chars per segment
4. **Stable** - Assigned once at parse, never changes
5. **Deterministic** - Same source → same paths (if using counter, not random)

## Implementation Changes

### 0. Parser: Assign Paths to AST Nodes

The parser is the **single source of truth** for XPaths. Every AST node gets a `path` field:

```rust
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    pub path: String,  // Assigned at parse time
}

pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
    pub path: String,
}

pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Block,
    pub span: Span,
    pub path: String,
}
// ... etc for all AST node types
```

The parser tracks current path context as it descends:

```rust
struct Parser {
    current_path: String,
    next_id: u16,
}

impl Parser {
    fn parse_function(&mut self) -> FnDecl {
        let name = self.parse_ident();
        let id = self.next_hex_id();
        let fn_path = format!("{}.{}[{}]", self.current_path, name, id);

        let old_path = std::mem::replace(&mut self.current_path, fn_path.clone());

        let params = self.parse_params();  // Each param gets path like "fn[ID].param_name[ID]"
        let body = self.parse_block();     // Statements get paths like "fn[ID].stmt[ID]"

        self.current_path = old_path;

        FnDecl {
            name,
            params,
            body,
            span: ...,
            path: fn_path,
        }
    }

    fn parse_member_expr(&mut self, object: Expr) -> Expr {
        let property = self.parse_ident();
        let id = self.next_hex_id();

        // Member path extends the object's path
        let member_path = format!("{}.{}[{}]", object.path, property, id);

        Expr {
            kind: ExprKind::Member { object, property },
            span: ...,
            path: member_path,
        }
    }
}
```

**Key principle**: Downstream passes (semantic, decorator, rewriter, emitter) **never compute paths**. They just read `node.path`.

### 1. TypeEnv Changes

```rust
pub struct TypeEnv {
    /// Types indexed by XPath
    types: HashMap<String, TypeInfo>,

    /// Struct definitions (still by name, these are global)
    structs: HashMap<String, HashMap<String, TypeInfo>>,

    /// Enum definitions (still by name, these are global)
    enums: HashMap<String, HashMap<String, Option<Vec<TypeInfo>>>>,
}

impl TypeEnv {
    /// Define a type at a specific path
    pub fn define_at_path(&mut self, path: &str, ty: TypeInfo) {
        self.types.insert(path.to_string(), ty);
    }

    /// Look up type by exact path
    pub fn lookup_path(&self, path: &str) -> Option<&TypeInfo> {
        self.types.get(path)
    }

    /// Look up type for a member expression by building parent chain
    /// e.g., "self.state.output" tries:
    ///   1. Exact match for "ctx.self.state.output"
    ///   2. Look up "ctx.self.state" → State, then State.output → Str
    pub fn resolve_member_type(&self, ctx: &str, member_path: &str) -> Option<TypeInfo> {
        // First try exact match
        let full_path = format!("{}.{}", ctx, member_path);
        if let Some(ty) = self.types.get(&full_path) {
            return Some(ty.clone());
        }

        // Otherwise resolve step by step
        let parts: Vec<&str> = member_path.split('.').collect();
        let mut current_type = self.types.get(&format!("{}.{}", ctx, parts[0]))?;

        for part in &parts[1..] {
            current_type = self.get_field_type(current_type, part)?;
        }

        Some(current_type.clone())
    }

    fn get_field_type(&self, ty: &TypeInfo, field: &str) -> Option<&TypeInfo> {
        match ty {
            TypeInfo::Struct { name, fields } => fields.get(field),
            TypeInfo::Ref { inner, .. } => self.get_field_type(inner, field),
            _ => None,
        }
    }
}
```

### 2. Type Checker Changes

Track current path context as we traverse:

```rust
pub struct TypeChecker {
    env: TypeEnv,
    /// Current path context, e.g., "KitchenSink.visit_mut_fn_decl"
    current_path: String,
    /// Counter for anonymous scopes (match arms, if branches)
    scope_counter: usize,
}

impl TypeChecker {
    fn check_function(&mut self, f: &FnDecl) {
        let old_path = self.current_path.clone();
        self.current_path = format!("{}.{}", self.current_path, f.name);

        // Define parameters with full paths
        for param in &f.params {
            let path = format!("{}.{}", self.current_path, param.name);
            let ty = self.resolve_param_type(&param.ty);
            self.env.define_at_path(&path, ty);
        }

        // Check body
        self.check_block(&f.body);

        self.current_path = old_path;
    }

    fn check_let(&mut self, let_stmt: &LetStmt) {
        let init_type = self.infer_expr(&let_stmt.init);
        let path = format!("{}.{}", self.current_path, let_stmt.pattern.name());
        self.env.define_at_path(&path, init_type);
    }

    fn check_match(&mut self, match_expr: &MatchExpr) {
        let scrutinee_type = self.infer_expr(&match_expr.scrutinee);

        for (i, arm) in match_expr.arms.iter().enumerate() {
            let old_path = self.current_path.clone();
            self.current_path = format!("{}.match_{}.arm_{}", self.current_path, self.scope_counter, i);

            // Bind pattern variables with narrowed types
            self.bind_pattern(&arm.pattern, &scrutinee_type);

            self.check_block(&arm.body);
            self.current_path = old_path;
        }
        self.scope_counter += 1;
    }

    /// Infer type of member expression and cache intermediate paths
    fn infer_member_expr(&mut self, member: &MemberExpr) -> TypeInfo {
        let obj_type = self.infer_expr(&member.object);
        let field_type = self.get_field_type(&obj_type, &member.property);

        // Cache the member path for decorator lookup
        if let Some(path) = self.expr_to_path(&Expr::Member(member.clone())) {
            let full_path = format!("{}.{}", self.current_path, path);
            self.env.define_at_path(&full_path, field_type.clone());
        }

        field_type
    }

    /// Convert expression to path string
    fn expr_to_path(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(id) => Some(id.name.clone()),
            Expr::Member(m) => {
                let obj_path = self.expr_to_path(&m.object)?;
                Some(format!("{}.{}", obj_path, m.property))
            }
            _ => None,
        }
    }
}
```

### 3. Decorator Changes

The decorator becomes much simpler - just look up paths:

```rust
impl SwcDecorator {
    /// Current function context for path lookups
    current_context: String,

    fn decorate_expr(&mut self, expr: &Expr) -> DecoratedExpr {
        let swc_type = self.lookup_expr_type(expr);

        match expr {
            Expr::Ident(id) => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Ident { name: id.name.clone(), .. },
                    metadata: SwcExprMetadata {
                        swc_type,
                        ..
                    },
                }
            }
            Expr::Member(mem) => {
                let decorated_object = self.decorate_expr(&mem.object);
                DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(decorated_object),
                        property: mem.property.clone(),
                        ..
                    },
                    metadata: SwcExprMetadata {
                        swc_type,  // Already resolved!
                        ..
                    },
                }
            }
            // ... other cases
        }
    }

    fn lookup_expr_type(&self, expr: &Expr) -> String {
        if let Some(path) = self.expr_to_path(expr) {
            let full_path = format!("{}.{}", self.current_context, path);
            if let Some(ty) = self.semantic_type_env.lookup_path(&full_path) {
                return self.type_info_to_string(ty);
            }
        }
        "Unknown".to_string()
    }

    fn expr_to_path(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(id) => Some(id.name.clone()),
            Expr::Member(m) => {
                let obj = self.expr_to_path(&m.object)?;
                Some(format!("{}.{}", obj, m.property))
            }
            _ => None,
        }
    }
}
```

## Benefits

1. **No collisions** - Every binding has a unique path
2. **No scope management** - Paths encode scope implicitly
3. **Simple lookups** - Decorator just reads, no inference
4. **Debuggable** - Can dump the entire type map and see exactly what's defined where
5. **Parent resolution trivial** - `"a.b.c"` parent is `"a.b"`, just string ops

## Migration Steps

1. Add `current_path: String` to TypeChecker
2. Add `define_at_path()` and `lookup_path()` to TypeEnv
3. Update TypeChecker to build paths as it traverses
4. Update Decorator to use path-based lookups
5. Remove old name-based `define()` and `lookup()` for variables
6. Keep name-based lookups for struct/enum definitions (they're global)

## Testing

The kitchen sink test should pass with this change. Specifically:

```rust
// In visit_mut_fn_decl
self.state.output = "string"
```

Should resolve:
- `"KitchenSink.visit_mut_fn_decl.self"` → `&mut KitchenSink`
- `"KitchenSink.visit_mut_fn_decl.self.state"` → `State`
- `"KitchenSink.visit_mut_fn_decl.self.state.output"` → `Str`

And the assignment decorator sees `left_type = "Str"`, sets `needs_to_string = true`.

## Rewriter and Paths

The rewriter transforms `DecoratedExpr` into new `DecoratedExpr`. Question: do generated nodes need paths?

**Answer: No.**

The pipeline is:
```
Parser → AST with paths
    ↓
Semantic → TypeEnv (path → type)
    ↓
Decorator → DecoratedAST (types already resolved, stored in metadata)
    ↓
Rewriter → Transformed DecoratedAST (types in metadata, no lookups needed)
    ↓
Emitter → Rust code (just reads metadata, doesn't need paths)
```

After decoration, types are **embedded in the node metadata**. The rewriter reads `expr.metadata.swc_type`, not the TypeEnv. So:

1. **Rewriter preserves source paths** - When wrapping `"foo"` → `"foo".to_string()`, the inner literal keeps its original path
2. **Generated wrapper nodes get no path** (or a synthetic one like `"<generated>"`) - doesn't matter since nobody looks them up
3. **Type info travels in metadata** - `needs_to_string`, `swc_type`, etc. are in `SwcExprMetadata`, not looked up by path

The rewriter just transforms structure. It doesn't need to resolve types - that's already done.

## Open Questions

1. **Method chains** - `foo.bar().baz` - the call result is anonymous. Path could be `foo[ID].bar()[ID].baz[ID]` where `bar()` segment represents the call.
2. **Index expressions** - `arr[0]` - path could be `arr[ID].[0][ID]` or `arr[ID].index[ID]`
3. **Temporaries** - `(a + b).foo` - the binary expr needs a path for the member access to extend. Could be `<binary>[ID].foo[ID]`

## Appendix: Current Pain Points This Fixes

1. `self` type collisions between impl blocks
2. `node` type collisions between visitor methods
3. Narrowed types from match arms leaking to other arms
4. Complex lookup priority logic in decorator (type_env vs semantic_type_env vs current_params)
5. Debug difficulty - can't tell which "self" a type refers to
