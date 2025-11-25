# Babel Plugin Modularization Guide

## Overview

The babel plugin has been restructured from a single 1235-line file into a modular architecture for better maintainability.

## File Structure

```
babel-plugin-minimact/
├── index-full.cjs              # Original monolithic version (CURRENT)
├── index-modular.cjs           # New modular entry point (TO BE ACTIVATED)
├── src/
│   ├── processComponent.cjs    # Main component processor
│   ├── extractors/
│   │   ├── hooks.cjs           # Hook extraction (useState, useEffect, etc.)
│   │   ├── props.cjs           # Props extraction from parameters
│   │   ├── localVariables.cjs # Local variable extraction
│   │   └── eventHandlers.cjs  # Event handler extraction
│   ├── generators/
│   │   ├── csharpFile.cjs     # Complete C# file generation
│   │   ├── component.cjs      # Component class generation
│   │   ├── jsx.cjs            # JSX → VNode generation
│   │   ├── expressions.cjs    # Expression generation
│   │   ├── runtimeHelpers.cjs # Runtime helper calls
│   │   └── renderBody.cjs     # Render method body
│   ├── analyzers/
│   │   ├── dependencies.cjs   # Dependency analysis
│   │   ├── classification.cjs # Node classification
│   │   └── detection.cjs      # Pattern detection
│   ├── types/
│   │   └── typeConversion.cjs # TS → C# type conversion
│   └── utils/
│       └── helpers.cjs        # General utilities
```

## Function Mapping

### From index-full.cjs → Modular Structure

| Function | Source Lines | Destination Module |
|----------|-------------|-------------------|
| `processComponent` | ~55-146 | `src/processComponent.cjs` |
| `extractHook` | ~152-179 | `src/extractors/hooks.cjs` |
| `extractUseState` | ~184-215 | `src/extractors/hooks.cjs` |
| `extractUseEffect` | ~217-244 | `src/extractors/hooks.cjs` |
| `extractUseRef` | ~246-269 | `src/extractors/hooks.cjs` |
| `extractUseMarkdown` | ~271-292 | `src/extractors/hooks.cjs` |
| `extractUseTemplate` | ~294-304 | `src/extractors/hooks.cjs` |
| `extractLocalVariables` | ~184-214 | `src/extractors/localVariables.cjs` |
| `extractEventHandler` | ~1025-1047 | `src/extractors/eventHandlers.cjs` |
| `analyzeDependencies` | ~306-351 | `src/analyzers/dependencies.cjs` |
| `classifyNode` | ~356-366 | `src/analyzers/classification.cjs` |
| `hasSpreadProps` | ~520-522 | `src/analyzers/detection.cjs` |
| `hasDynamicChildren` | ~524-557 | `src/analyzers/detection.cjs` |
| `hasComplexProps` | ~559-581 | `src/analyzers/detection.cjs` |
| `generateCSharpFile` | ~373-508 | `src/generators/csharpFile.cjs` |
| `generateComponent` | ~510-518 | `src/generators/component.cjs` |
| `generateRenderBody` | ~585-595 | `src/generators/renderBody.cjs` |
| `generateJSXElement` | ~755-833 | `src/generators/jsx.cjs` |
| `generateChildren` | ~835-859 | `src/generators/jsx.cjs` |
| `generateFragment` | ~861-890 | `src/generators/jsx.cjs` |
| `generateRuntimeHelperCall` | ~645-753 | `src/generators/runtimeHelpers.cjs` |
| `generateRuntimeHelperForJSXNode` | ~583-643 | `src/generators/runtimeHelpers.cjs` |
| `generateJSXExpression` | ~861-933 | `src/generators/expressions.cjs` |
| `generateHybridExpression` | ~935-969 | `src/generators/expressions.cjs` |
| `generateConditional` | ~971-990 | `src/generators/expressions.cjs` |
| `generateShortCircuit` | ~992-1010 | `src/generators/expressions.cjs` |
| `generateMapExpression` | ~1012-1023 | `src/generators/expressions.cjs` |
| `generateCSharpExpression` | ~1049-1136 | `src/generators/expressions.cjs` |
| `generateCSharpStatement` | ~1138-1170 | `src/generators/expressions.cjs` |
| `generateAttributeValue` | ~1125-1143 | `src/generators/expressions.cjs` |
| `tsTypeToCSharpType` | ~1072-1101 | `src/types/typeConversion.cjs` |
| `inferType` | ~1106-1117 | `src/types/typeConversion.cjs` |
| `getComponentName` | ~1172-1195 | `src/utils/helpers.cjs` |
| `escapeCSharpString` | ~1197-1204 | `src/utils/helpers.cjs` |

## Migration Steps

1. ✅ **Create module files with comments** (DONE)
2. ⏳ **Move functions to modules** (IN PROGRESS - You do this!)
3. ⏳ **Wire up imports/exports**
4. ⏳ **Test modular version**
5. ⏳ **Switch index.cjs to use modular version**

## How to Move Functions

For each function:

1. Copy function from `index-full.cjs`
2. Paste into appropriate module file
3. Add to module.exports
4. Identify dependencies (other functions it calls)
5. Add require() imports for dependencies
6. Update function to use imported dependencies

## Example

Moving `escapeCSharpString`:

**Before (in index-full.cjs):**
```javascript
function escapeCSharpString(str) {
  return str
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}
```

**After (in src/utils/helpers.cjs):**
```javascript
const t = require('@babel/types');

function escapeCSharpString(str) {
  return str
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

function getComponentName(path) {
  // ... implementation
}

module.exports = {
  escapeCSharpString,
  getComponentName
};
```

## Testing

After moving functions, test with:
```bash
node test-single.js Counter.jsx
```

## Benefits

- **Maintainability**: Easier to find and modify specific functionality
- **Testability**: Can unit test individual modules
- **Collaboration**: Multiple developers can work on different modules
- **Clarity**: Clear separation of concerns
- **Reusability**: Modules can be reused in other contexts
