# SWC Plugin Implementation Status & Finishing Plan

This document details the current implementation status of the SWC plugin generator and what remains to achieve full parity with `index.cjs` and `processComponent.cjs`.

## Architecture: Main Plugin vs Helper Library

**Critical distinction:**

### Main Plugin (`lib.rs`)
Contains **visitor structure and routing only**:
- Detects AST patterns (hook calls, JSX, etc.)
- Routes to helper functions
- Does NOT contain extraction/transformation logic

### Helper Library (`helpers.rs`)
Contains **all extraction and transformation logic**:
- Generated from template JSON files
- Each Babel helper function becomes a Rust helper function
- `extractUseState()`, `extractUseValidation()`, `generateCSharpExpression()`, etc.

**Example flow:**
```rust
// In HookExtractor visitor (lib.rs) - just routing
fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    match hook_name.as_str() {
        "useState" => extract_use_state(call, &mut self.component),  // calls helper
        "useValidation" => extract_use_validation(call, &mut self.component),
        // ...
    }
}

// In helpers.rs - actual logic (generated from templates)
pub fn extract_use_state(call: &CallExpr, component: &mut Component) {
    // Actual extraction logic here
}
```

---

## Current Implementation Status

### Completed Features

#### Core Structure (lib.rs)
- [x] Main transformer with `visit_mut_program`
- [x] Parent context tracking via stack (`ParentContext` enum)
- [x] Component struct with all fields
- [x] Input file path handling from metadata

#### Visitors Implemented (lib.rs - routing only)
- [x] `HookExtractor` - detects hook calls, routes to helpers
- [x] `LocalVariableExtractor` - non-hook variable declarations
- [x] `HelperFunctionExtractor` - function declarations in component body
- [x] `RenderBodyExtractor` - captures return statement JSX
- [x] `ImportExtractor` - tracks external imports
- [x] `JSXTemplateExtractor` - text and attribute templates
- [x] `LoopExtractor` - .map() patterns
- [x] `StructuralExtractor` - conditional rendering (ternary, &&)
- [x] `ExpressionExtractor` - method calls, binary ops, unary ops, member access

#### Props Extraction
- [x] Destructured props: `function Component({ prop1, prop2 })`
- [x] Single object props: `function Component(props)`
- [x] TypeScript type annotation extraction from `TSTypeLiteral`
- [x] Type conversion: TS types → C# types

#### Template Extraction
- [x] Text templates from JSX expression containers
- [x] Attribute templates from JSX attributes
- [x] Loop templates from `.map()` calls
- [x] Structural templates (ternary conditionals, logical AND)
- [x] Expression templates (method calls, binary/unary expressions)

#### JSX Processing
- [x] `HexPathGenerator` for unique keys
- [x] `assign_hex_paths_to_jsx()` - adds `data-minimact-key` attributes
- [x] Recursive JSX tree traversal

#### File Generation
- [x] C# file generation (basic structure)
- [x] `.templates.json` file generation

#### Helper Stubs
- [x] `helpers.rs` module with stubs for all template-generated functions

---

## Remaining Implementation Tasks

### High Priority - Main Plugin (lib.rs)

#### 1. Hook Detection Completeness
**Current gaps in HookExtractor routing:**
- [ ] `useStateX` - add match case, route to `extract_use_state_x()`
- [ ] `useProtectedState` - add match case, route to `extract_use_protected_state()`
- [ ] `useValidation` - add match case, route to `extract_use_validation()`
- [ ] `useModal` - add match case, route to `extract_use_modal()`
- [ ] `useToggle` - add match case, route to `extract_use_toggle()`
- [ ] `useDropdown` - add match case, route to `extract_use_dropdown()`
- [ ] `useTemplate` - add match case, route to `extract_use_template()`
- [ ] `useRazorMarkdown` - add match case, route to `extract_use_razor_markdown()`
- [ ] `usePub` / `useSub` - add match cases
- [ ] `useMicroTask` / `useMacroTask` - add match cases
- [ ] `useSignalR` - add match case, route to `extract_use_signalr()`
- [ ] `usePredictHint` - add match case
- [ ] `useServerTask` / `usePaginatedServerTask` - add match cases
- [ ] `useMvcState` / `useMvcViewModel` - add match cases

**Note:** These are just routing additions to lib.rs. Actual extraction logic goes in helpers.rs.

#### 2. Event Handler Detection
**Add to HelperFunctionExtractor or new EventHandlerExtractor:**
- [ ] Detect event handler patterns (onClick, onChange, etc.)
- [ ] Route to `extract_event_handler()` helper

#### 3. Output Generation Enhancement
**In generate_outputs():**
- [ ] Call helper functions for C# generation instead of inline code
- [ ] Add .tsx.keys file generation (needs helper)
- [ ] Add JSX nullification logic

---

### High Priority - Helper Library Stubs (helpers.rs)

#### 4. Add Missing Hook Extraction Stubs
**Each needs correct signature to match Babel:**
- [ ] `extract_use_state_x(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_protected_state(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_validation(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_modal(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_toggle(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_dropdown(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_template(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_razor_markdown(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_pub(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_sub(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_micro_task(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_macro_task(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_signalr(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_predict_hint(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_server_task(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_paginated_server_task(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_mvc_state(call: &CallExpr, component: &mut Component)`
- [ ] `extract_use_mvc_view_model(call: &CallExpr, component: &mut Component)`
- [ ] `extract_custom_hook_call(call: &CallExpr, component: &mut Component, hook_name: &str)`

#### 5. Add Event Handler Extraction Stub
- [ ] `extract_event_handler(func: &FnDecl, component: &mut Component)`

#### 6. Add useEffect Analysis Stubs
- [ ] `analyze_hook_usage(callback: &Expr) -> Vec<String>`
- [ ] `transform_effect_callback(callback: &Expr, hook_calls: &[String]) -> Expr`
- [ ] `transform_handler_function(body: &Expr, params: &[Pat], hook_calls: &[String]) -> Expr`

---

### Medium Priority - Component Struct Fields

#### 7. Add Missing Component Fields
**In component.rs:**
- [ ] `use_protected_state: Vec<UseStateInfo>`
- [ ] `use_razor_markdown: Vec<UseRazorMarkdownInfo>`
- [ ] `use_pub: Vec<UsePubInfo>`
- [ ] `use_sub: Vec<UseSubInfo>`
- [ ] `use_micro_task: Vec<UseMicroTaskInfo>`
- [ ] `use_macro_task: Vec<UseMacroTaskInfo>`
- [ ] `use_predict_hint: Vec<UsePredictHintInfo>`
- [ ] `use_server_task: Vec<UseServerTaskInfo>`
- [ ] `paginated_tasks: Vec<PaginatedTaskInfo>`
- [ ] `use_mvc_state: Vec<UseMvcStateInfo>`
- [ ] `use_mvc_view_model: Vec<UseMvcViewModelInfo>`
- [ ] `client_effects: Vec<ClientEffect>`
- [ ] `custom_hooks: Vec<CustomHookInstance>`
- [ ] `imported_hook_metadata: HashMap<String, HookMetadata>`

---

### Medium Priority - Template Features

#### 5. Conditional Element Templates (Enhanced)
**Current:** Basic structural templates
**Missing:**
- [ ] Full `conditionExpression` extraction
- [ ] `evaluable` flag determination
- [ ] Complex condition analysis

**Location in Babel:** `extractors/conditionalElementTemplates.cjs`

#### 6. Expression Template Enhancements
**Missing:**
- [ ] Arithmetic operation chain analysis (`count * 2 + 1`)
- [ ] Complex multi-variable expressions
- [ ] Transform metadata extraction

**Location in Babel:** `extractors/expressionTemplates.cjs`

---

### Medium Priority - Output Generation

#### 7. Complete C# Code Generation
**Current:** Basic class structure
**Missing:**
- [ ] Full event handler method generation with bodies
- [ ] useEffect → lifecycle method conversion
- [ ] Custom hook instance handling
- [ ] Client handler JavaScript embedding
- [ ] Proper type mapping for all scenarios

**Location in Babel:** `generators/csharpFile.cjs`

#### 8. .tsx.keys File Generation
**Challenge:** Requires access to original source code
**Options:**
- [ ] Store original source in transformer
- [ ] Use SWC's codegen with custom key injection
- [ ] Generate from modified AST

**Location in Babel:** `index.cjs:91-148`

#### 9. JSX Nullification
**Not yet implemented:**
- [ ] After processing, replace JSX return with `null`
- [ ] Track component paths to nullify
- [ ] Apply after `.tsx.keys` generation

**Location in Babel:** `index.cjs:150-163`

---

### Lower Priority - Advanced Features

#### 10. Custom Hook Detection & Analysis
**Current:** Basic detection by name pattern
**Missing:**
- [ ] Full `isCustomHook()` validation
- [ ] `analyzeHook()` - extract hook structure:
  - States
  - Methods/event handlers
  - Return values
  - JSX elements (for hooks that return UI)

**Location in Babel:** `analyzers/hookDetector.cjs`, `analyzers/hookAnalyzer.cjs`

#### 11. Imported Hook Analysis
**Not yet implemented:**
- [ ] `analyzeImportedHooks()` - detect hooks imported from other files
- [ ] Load and parse hook metadata from external files
- [ ] Store in `component.importedHookMetadata`

**Location in Babel:** `analyzers/hookImports.cjs`

#### 12. Plugin Usage Analysis
**Not yet implemented:**
- [ ] Detect `<Plugin name="..." state={...} />` in JSX
- [ ] Extract plugin name, version, state binding
- [ ] `validatePluginUsage()`

**Location in Babel:** `analyzers/analyzePluginUsage.cjs`

#### 13. Timeline Analysis
**Not yet implemented:**
- [ ] Detect `@minimact/timeline` usage
- [ ] Extract duration, keyframes, state bindings
- [ ] Generate `.timeline-templates.json`

**Location in Babel:** `analyzers/timelineAnalyzer.cjs`

#### 14. Hot Reload Support
**Not yet implemented:**
- [ ] Hook signature extraction and comparison
- [ ] JSX structural change detection (insertions/deletions)
- [ ] Previous key comparison for deletion detection
- [ ] Generate `.structural-changes.json`

**Location in Babel:** `index.cjs:230-324`

---

## Helper Stubs Summary

All extraction/transformation logic lives in `helpers.rs` as stubs. These will eventually be **generated from template JSON files**.

### Currently Stubbed (in helpers.rs)
- `ts_type_to_csharp_type()` - basic implementation
- `infer_csharp_type()` - basic implementation
- `generate_csharp_expression()` - basic implementation
- `escape_csharp_string()` - basic implementation
- `get_default_value()` - basic implementation
- `is_custom_hook_name()` - basic implementation

### Need to Add as Stubs

#### Hook Extraction (from `extractors/hooks.cjs`)
- [ ] `extract_use_state()` - useState/useClientState
- [ ] `extract_use_state_x()` - declarative state projections
- [ ] `extract_use_protected_state()` - protected state
- [ ] `extract_use_effect()` - with client-side analysis
- [ ] `extract_use_ref()` - refs
- [ ] `extract_use_markdown()` - markdown state
- [ ] `extract_use_razor_markdown()` - razor markdown
- [ ] `extract_use_template()` - template references
- [ ] `extract_use_validation()` - validation rules
- [ ] `extract_use_modal()` - modal state
- [ ] `extract_use_toggle()` - toggle state
- [ ] `extract_use_dropdown()` - dropdown state
- [ ] `extract_use_pub()` / `extract_use_sub()` - pub/sub
- [ ] `extract_use_micro_task()` / `extract_use_macro_task()` - task scheduling
- [ ] `extract_use_signalr()` - SignalR
- [ ] `extract_use_predict_hint()` - prediction hints
- [ ] `extract_use_server_task()` - server tasks
- [ ] `extract_use_paginated_server_task()` - paginated tasks
- [ ] `extract_use_mvc_state()` / `extract_use_mvc_view_model()` - MVC integration
- [ ] `extract_custom_hook_call()` - custom hook instances

#### Type Conversion (from `types/typeConversion.cjs`)
- [ ] `infer_type()` - full type inference from values

#### Prop Type Inference (from `analyzers/propTypeInference.cjs`)
- [ ] `infer_prop_types()` - analyze usage patterns

#### Template Extraction (from `extractors/`)
- [ ] `extract_templates()` - from `templates.cjs`
- [ ] `extract_attribute_templates()` - from `templates.cjs`
- [ ] `extract_conditional_element_templates()` - from `conditionalElementTemplates.cjs`

#### Hook Analysis (from `analyzers/`)
- [ ] `analyze_hook()` - from `hookAnalyzer.cjs`
- [ ] `analyze_imported_hooks()` - from `hookImports.cjs`

#### Code Generation (from `generators/`)
- [ ] `generate_hook_class()` - from `hookClassGenerator.cjs`
- [ ] `generate_csharp_file()` - from `csharpFile.cjs`

#### Client-Side Execution (from `extractors/hooks.cjs`)
- [ ] `analyze_hook_usage()` - detect hooks used in callback
- [ ] `transform_effect_callback()` - transform for client execution
- [ ] `transform_handler_function()` - transform event handlers

#### Plugin/Timeline (from `analyzers/`)
- [ ] `analyze_plugin_usage()` - from `analyzePluginUsage.cjs`
- [ ] `analyze_timeline()` - from `timelineAnalyzer.cjs`

---

## Recommended Implementation Order

### Phase 1: Complete Routing (lib.rs)
**Goal:** Main plugin detects all patterns and routes to stubs
1. Add all missing hook match cases to `HookExtractor`
2. Add event handler detection
3. Add all missing Component struct fields
4. Ensure all visitors route to helper stubs

### Phase 2: Complete Stubs (helpers.rs)
**Goal:** All helper functions exist as stubs with correct signatures
1. Add all hook extraction stubs
2. Add event handler extraction stub
3. Add type inference stubs
4. Add template extraction stubs
5. Add code generation stubs

### Phase 3: Basic Functionality
**Goal:** Plugin compiles and produces basic output
1. Implement basic `extract_use_state()` stub with real logic
2. Implement basic `generate_csharp_expression()` stub
3. Test with simple components

### Phase 4: Template-Based Generation
**Goal:** Generate real implementations from template JSON files
1. Create script to parse template JSON files
2. Generate Rust implementations from `rust_translation` fields
3. Replace stubs with generated code
4. Test with complex components

### Phase 5: Full Feature Parity
**Goal:** Match all Babel plugin features
1. Complete C# code generation
2. .tsx.keys file generation
3. Hot reload support (structural changes, hook signatures)
4. Custom hook analysis
5. Timeline/plugin analysis

---

## Architecture Notes

### Visitor Pattern
The SWC implementation uses specialized visitors passed to `visit_mut_with()` to mimic Babel's `path.traverse()` with selective handlers. Each visitor only implements the `visit_mut_*` methods it needs.

### Parent Context
Parent context is tracked via a stack (`parent_stack`) to provide information similar to Babel's `path.parent`. The `ParentContext` enum captures the relevant parent types.

### File Generation
Output files are generated in `generate_outputs()` after all components are processed. This happens at the end of `visit_mut_program`.

### Helper Functions
Helper functions are defined as stubs in `helpers.rs` and will eventually be generated from the template JSON files. This allows the main plugin code to compile while the helper implementations are developed separately.

---

## Testing Strategy

1. **Unit tests**: Test individual extractors with sample AST nodes
2. **Integration tests**: Process complete component files and verify output
3. **Comparison tests**: Compare SWC output with Babel plugin output for same input
4. **Template tests**: Verify generated templates match expected format

---

## Dependencies to Add to Cargo.toml

```toml
[dependencies]
swc_core = { version = "0.90", features = ["ecma_plugin_transform"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```
