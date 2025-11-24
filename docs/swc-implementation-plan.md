# SWC Plugin Implementation Plan

This document outlines what remains to be implemented for logical parity with `index.cjs` and `processComponent.cjs`.

## Current Status

The `generate-swc-visitor.js` script generates the basic SWC plugin structure with:
- Main transformer with `visit_mut_program`
- Specialized visitors: `HookExtractor`, `LocalVariableExtractor`, `HelperFunctionExtractor`, `RenderBodyExtractor`, `ImportExtractor`
- Basic hook extraction: `useState`, `useEffect`, `useRef`, `useClientState`, `useMarkdown`, custom hooks
- Component struct with all fields

## Remaining Implementation Tasks

### Phase 1: Core Extraction (High Priority)

#### 1.1 Props Extraction with TypeScript Types
**Location**: `processComponent.cjs:124-160`

Currently implemented: Basic prop extraction from destructured patterns
Missing:
- [ ] Extract TypeScript type annotations from parameters
- [ ] Handle `TSTypeLiteral` for inline type definitions
- [ ] Map TS types to C# types using `tsTypeToCSharpType`
- [ ] Handle props as single object (`function Component(props)`)

```rust
// Need to implement
fn extract_props_with_types(&mut self, params: &[Param], component: &mut Component) {
    // Check for type annotation: params[0].typeAnnotation?.typeAnnotation
    // If TSTypeLiteral, iterate members for TSPropertySignature
    // Extract type for each prop
}
```

#### 1.2 Event Handler Extraction
**Location**: `processComponent.cjs` (implicit in hooks.cjs)

Missing:
- [ ] Detect event handler functions (onClick, onChange, etc.)
- [ ] Extract handler parameters
- [ ] Track async handlers
- [ ] Distinguish client handlers from server handlers

#### 1.3 useEffect Detailed Analysis
**Location**: `extractors/hooks.cjs`

Currently: Basic dependency array extraction
Missing:
- [ ] Analyze callback body for client-side API usage (window, document, localStorage)
- [ ] Track cleanup functions
- [ ] Set `is_client_side` flag correctly

### Phase 2: Template Extraction (Medium Priority)

#### 2.1 Text Templates
**Location**: `extractors/templates.cjs`

Missing:
- [ ] `extractTemplates(renderBody, component)` - Extract text interpolation templates
- [ ] `extractAttributeTemplates(renderBody, component)` - Extract attribute templates
- [ ] `addTemplateMetadata(component, templates)` - Add metadata to component
- [ ] Generate template strings with bindings

#### 2.2 Loop Templates
**Location**: `extractors/loopTemplates.cjs`

Missing:
- [ ] Detect `.map()` patterns on arrays
- [ ] Extract `stateKey`, `itemVar`, `indexVar`
- [ ] Extract `keyExpression` from JSX key attribute
- [ ] Track nested loops

#### 2.3 Structural Templates
**Location**: `extractors/structuralTemplates.cjs`

Missing:
- [ ] Detect conditional rendering patterns:
  - Ternary: `condition ? <A /> : <B />`
  - Logical AND: `condition && <A />`
- [ ] Extract `conditionBinding`
- [ ] Track truthy/falsy branches

#### 2.4 Conditional Element Templates
**Location**: `extractors/conditionalElementTemplates.cjs`

Missing:
- [ ] Enhanced conditional detection
- [ ] Mark templates as `evaluable` or not
- [ ] Extract full `conditionExpression`

#### 2.5 Expression Templates
**Location**: `extractors/expressionTemplates.cjs`

Missing:
- [ ] Method call templates: `price.toFixed(2)`
- [ ] Binary expression templates: `count * 2 + 1`
- [ ] Member expression templates: `items.length`
- [ ] Unary expression templates: `-count`
- [ ] Transform metadata extraction

### Phase 3: JSX Path Assignment (Medium Priority)

#### 3.1 Hex Path Generator
**Location**: `utils/hexPath.cjs`, `utils/pathAssignment.cjs`

Missing:
- [ ] `HexPathGenerator` struct
- [ ] `assignPathsToJSX(renderBody, path, pathGen, t)` function
- [ ] Generate unique hex keys for each JSX element
- [ ] Add `key` attributes to JSX nodes for hot reload

### Phase 4: Analysis Features (Lower Priority)

#### 4.1 Prop Type Inference
**Location**: `analyzers/propTypeInference.cjs`

Missing:
- [ ] `inferPropTypes(component, body)` - Infer types from usage patterns
- [ ] Analyze how props are used in JSX and expressions

#### 4.2 Custom Hook Detection & Analysis
**Location**: `analyzers/hookDetector.cjs`, `analyzers/hookAnalyzer.cjs`

Currently: Basic detection
Missing:
- [ ] `isCustomHook(path)` - Full validation
- [ ] `analyzeHook(path)` - Extract hook structure:
  - States
  - Methods/event handlers
  - Return values
  - JSX elements (for hooks that return UI)

#### 4.3 Imported Hook Analysis
**Location**: `analyzers/hookImports.cjs`

Missing:
- [ ] `analyzeImportedHooks(programPath, state)` - Detect hooks imported from other files
- [ ] Load and parse hook metadata from external files
- [ ] Store in `component.importedHookMetadata`

#### 4.4 Plugin Usage Analysis
**Location**: `analyzers/analyzePluginUsage.cjs`

Missing:
- [ ] Detect `<Plugin name="..." state={...} />` usage
- [ ] Extract plugin name, version, state binding
- [ ] `validatePluginUsage(pluginUsages)`

#### 4.5 Timeline Analysis
**Location**: `analyzers/timelineAnalyzer.cjs`

Missing:
- [ ] Detect `@minimact/timeline` usage
- [ ] Extract duration, keyframes, state bindings

### Phase 5: Code Generation (High Priority)

#### 5.1 C# File Generation
**Location**: `generators/csharpFile.cjs`

Missing:
- [ ] `generateCSharpFile(components, state)` - Main generator
- [ ] Generate C# class for each component
- [ ] Generate state properties
- [ ] Generate event handler methods
- [ ] Generate Render method with JSX-to-C# conversion

#### 5.2 Hook Class Generation
**Location**: `generators/hookClassGenerator.cjs`

Missing:
- [ ] `generateHookClass(hookAnalysis, context)` - Generate C# class for custom hooks
- [ ] Map hook states to C# properties
- [ ] Map hook methods to C# methods

#### 5.3 Template Map JSON Generation
**Location**: `extractors/templates.cjs`

Missing:
- [ ] `generateTemplateMapJSON(name, templates, attrTemplates, conditionalTemplates)`
- [ ] Write `.templates.json` files

### Phase 6: Output File Generation (High Priority)

#### 6.1 File Writing
**Location**: `index.cjs:174-327`

Missing:
- [ ] Write `.cs` file
- [ ] Write `.tsx.keys` file (JSX with hex keys added)
- [ ] Write `.templates.json` file
- [ ] Write `.timeline-templates.json` file (if timeline exists)
- [ ] Write `.structural-changes.json` file (for hot reload)

#### 6.2 Hot Reload Support
**Location**: `index.cjs:230-324`

Missing:
- [ ] Hook signature extraction and comparison
- [ ] JSX structural change detection (insertions/deletions)
- [ ] Previous key comparison for deletion detection

### Phase 7: JSX Nullification (Medium Priority)

**Location**: `index.cjs:150-163`

Missing:
- [ ] After processing, replace JSX return with `null`
- [ ] Track component paths to nullify
- [ ] Apply after `.tsx.keys` generation

## Implementation Order Recommendation

1. **Phase 5.1** - C# File Generation (enables testing)
2. **Phase 1.1** - Props with TypeScript types
3. **Phase 2.1-2.2** - Text and Loop templates
4. **Phase 3.1** - Hex path assignment
5. **Phase 6.1** - File writing
6. **Phase 2.3-2.5** - Remaining template types
7. **Phase 4.1-4.2** - Analysis features
8. **Phase 6.2** - Hot reload support
9. **Phase 7** - JSX nullification

## Specialized Visitors Needed

Beyond the current visitors, these additional visitors may be needed:

```rust
/// Visitor for JSX template extraction
struct JSXTemplateExtractor<'a> {
    component: &'a mut Component,
    path_generator: &'a mut HexPathGenerator,
    current_path: Vec<usize>,
}

/// Visitor for event handler detection in JSX
struct EventHandlerExtractor<'a> {
    component: &'a mut Component,
}

/// Visitor for conditional rendering patterns
struct ConditionalExtractor<'a> {
    component: &'a mut Component,
}

/// Visitor for loop (.map) patterns
struct LoopExtractor<'a> {
    component: &'a mut Component,
}
```

## Notes

- The current implementation uses `visit_mut_with` with specialized visitors for selective traversal
- Parent context is tracked via a stack for accurate path information
- Each visitor only implements the `visit_mut_*` methods it needs
- Helper function translation will be handled separately using the template JSON files
