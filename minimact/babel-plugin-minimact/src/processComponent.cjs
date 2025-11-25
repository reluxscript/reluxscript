/**
 * Component Processor
 *
 * Main entry point for processing a component function/class.
 */

const t = require('@babel/types');
const { getComponentName } = require('./utils/helpers.cjs');
const { tsTypeToCSharpType } = require('./types/typeConversion.cjs');
const { extractHook } = require('./extractors/hooks.cjs');
const { extractLocalVariables } = require('./extractors/localVariables.cjs');
const { inferPropTypes } = require('./analyzers/propTypeInference.cjs');
const { isCustomHook } = require('./analyzers/hookDetector.cjs');
const { analyzeHook } = require('./analyzers/hookAnalyzer.cjs');
const { generateHookClass } = require('./generators/hookClassGenerator.cjs');
const { analyzeImportedHooks } = require('./analyzers/hookImports.cjs');
const {
  extractTemplates,
  extractAttributeTemplates,
  addTemplateMetadata
} = require('./extractors/templates.cjs');
const { extractLoopTemplates } = require('./extractors/loopTemplates.cjs');
const { extractStructuralTemplates } = require('./extractors/structuralTemplates.cjs');
const { extractConditionalElementTemplates } = require('./extractors/conditionalElementTemplates.cjs');
const { extractExpressionTemplates } = require('./extractors/expressionTemplates.cjs');
const { analyzePluginUsage, validatePluginUsage } = require('./analyzers/analyzePluginUsage.cjs');
const { analyzeTimeline } = require('./analyzers/timelineAnalyzer.cjs');
const { HexPathGenerator } = require('./utils/hexPath.cjs');
const { assignPathsToJSX } = require('./utils/pathAssignment.cjs');

/**
 * Process a component function OR custom hook
 */
function processComponent(path, state) {
  const componentName = getComponentName(path);

  if (!componentName) return;

  // 🔥 CUSTOM HOOK DETECTION - Process hooks before checking uppercase
  if (isCustomHook(path)) {
    console.log(`[Custom Hook] Detected: ${componentName}`);
    return processCustomHook(path, state);
  }

  if (componentName[0] !== componentName[0].toUpperCase()) return; // Not a component

  state.file.minimactComponents = state.file.minimactComponents || [];

  // 🔥 NEW: Analyze imported hooks FIRST (before component processing)
  // This allows us to detect hook metadata for cross-file imports
  // NOTE: We pass state.file.path (Program) not path (Function), since imports are at file level
  console.log(`[DEBUG Hook Import] Analyzing imports for ${componentName}...`);
  const importedHooks = analyzeImportedHooks(state.file.path, state);
  console.log(`[DEBUG Hook Import] Found ${importedHooks.size} imported hooks`);

  if (importedHooks.size > 0) {
    console.log(`[Hook Import] Found ${importedHooks.size} imported hook(s) in ${componentName}:`);
    importedHooks.forEach((metadata, hookName) => {
      console.log(`  - ${hookName} from ${metadata.filePath}`);
    });
  }

  const component = {
    name: componentName,
    props: [],
    useState: [],
    useClientState: [],
    useStateX: [], // Declarative state projections
    useEffect: [],
    useRef: [],
    useMarkdown: [],
    useTemplate: null,
    useValidation: [],
    useModal: [],
    useToggle: [],
    useDropdown: [],
    customHooks: [], // Custom hook instances (useCounter, useForm, etc.)
    importedHookMetadata: importedHooks, // 🔥 NEW: Store imported hook metadata
    eventHandlers: [],
    clientHandlers: [], // 🔥 NEW: Client-side event handlers (JavaScript functions)
    clientEffects: [], // 🔥 NEW: Client-side effects (JavaScript callbacks for useEffect)
    localVariables: [], // Local variables (const/let/var) in function body
    helperFunctions: [], // Helper functions declared in function body
    renderBody: null,
    pluginUsages: [], // Plugin instances (<Plugin name="..." state={...} />)
    stateTypes: new Map(), // Track which hook each state came from
    dependencies: new Map(), // Track dependencies per JSX node
    externalImports: new Set(), // Track external library identifiers
    clientComputedVars: new Set() // Track variables using external libs
  };

  // Track external imports at file level
  state.file.path.traverse({
    ImportDeclaration(importPath) {
      const source = importPath.node.source.value;

      // Skip Minimact imports, relative imports, and CSS imports
      if (source.startsWith('minimact') ||
          source.startsWith('.') ||
          source.startsWith('/') ||
          source.endsWith('.css') ||
          source.endsWith('.scss') ||
          source.endsWith('.sass')) {
        return;
      }

      // Track external library identifiers
      importPath.node.specifiers.forEach(spec => {
        if (t.isImportDefaultSpecifier(spec)) {
          // import _ from 'lodash'
          component.externalImports.add(spec.local.name);
        } else if (t.isImportSpecifier(spec)) {
          // import { sortBy } from 'lodash'
          component.externalImports.add(spec.local.name);
        } else if (t.isImportNamespaceSpecifier(spec)) {
          // import * as _ from 'lodash'
          component.externalImports.add(spec.local.name);
        }
      });
    }
  });

  // Extract props from function parameters
  const params = path.node.params;
  if (params.length > 0 && t.isObjectPattern(params[0])) {
    // Destructured props: function Component({ prop1, prop2 })
    // Check if there's a type annotation on the parameter
    const paramTypeAnnotation = params[0].typeAnnotation?.typeAnnotation;

    for (const property of params[0].properties) {
      if (t.isObjectProperty(property) && t.isIdentifier(property.key)) {
        let propType = 'dynamic';

        // Try to extract type from TypeScript annotation
        if (paramTypeAnnotation && t.isTSTypeLiteral(paramTypeAnnotation)) {
          const propName = property.key.name;
          const tsProperty = paramTypeAnnotation.members.find(
            member => t.isTSPropertySignature(member) &&
                     t.isIdentifier(member.key) &&
                     member.key.name === propName
          );
          if (tsProperty && tsProperty.typeAnnotation) {
            propType = tsTypeToCSharpType(tsProperty.typeAnnotation.typeAnnotation);
          }
        }

        component.props.push({
          name: property.key.name,
          type: propType
        });
      }
    }
  } else if (params.length > 0 && t.isIdentifier(params[0])) {
    // Props as single object: function Component(props)
    // Use 'dynamic' to allow property access
    component.props.push({
      name: params[0].name,
      type: 'dynamic'
    });
  }

  // Find function body
  const body = path.node.body.type === 'BlockStatement'
    ? path.node.body
    : t.blockStatement([t.returnStatement(path.node.body)]);

  // Extract hooks and local variables
  path.traverse({
    CallExpression(hookPath) {
      extractHook(hookPath, component);
    },

    VariableDeclaration(varPath) {
      // Only extract local variables at the top level of the function body
      if (varPath.getFunctionParent() === path && varPath.parent.type === 'BlockStatement') {
        extractLocalVariables(varPath, component, t);
      }
    },

    FunctionDeclaration(funcPath) {
      // Only extract helper functions at the top level of the component body
      // (not nested functions inside other functions)
      if (funcPath.getFunctionParent() === path && funcPath.parent.type === 'BlockStatement') {
        const funcName = funcPath.node.id.name;

        // Skip custom hooks (they're processed separately)
        if (isCustomHook(funcPath)) {
          funcPath.skip(); // Don't traverse into this hook
          return;
        }

        const params = funcPath.node.params.map(param => {
          if (t.isIdentifier(param)) {
            // Simple parameter: (name)
            const paramType = param.typeAnnotation?.typeAnnotation
              ? tsTypeToCSharpType(param.typeAnnotation.typeAnnotation)
              : 'dynamic';
            return { name: param.name, type: paramType };
          }
          return { name: 'param', type: 'dynamic' };
        });

        const returnType = funcPath.node.returnType?.typeAnnotation
          ? tsTypeToCSharpType(funcPath.node.returnType.typeAnnotation)
          : 'void';

        const isAsync = funcPath.node.async;

        component.helperFunctions.push({
          name: funcName,
          params,
          returnType,
          isAsync,
          body: funcPath.node.body // Store the function body AST
        });
      }
    },

    ReturnStatement(returnPath) {
      if (returnPath.getFunctionParent() === path) {
        // Store a REFERENCE to the actual live AST node (not a clone!)
        // We'll add keys to THIS node, and it will persist in the Program tree
        component.renderBody = returnPath.node.argument;
      }
    }
  });

  // Infer prop types from usage BEFORE replacing JSX with null
  // Pass the entire function body to analyze all usage (including JSX)
  inferPropTypes(component, body);

  // Extract templates from JSX for hot reload (BEFORE replacing JSX with null)
  if (component.renderBody) {
    // 🔥 CRITICAL: Assign hex paths to all JSX nodes FIRST
    // This ensures all extractors use the same paths (no recalculation!)
    const pathGen = new HexPathGenerator();
    const structuralChanges = []; // Track insertions for hot reload

    // Check if this is a hot reload by looking for previous .tsx.keys file
    const fs = require('fs');
    const nodePath = require('path');
    const inputFilePath = state.file.opts.filename;
    const keysFilePath = inputFilePath ? inputFilePath + '.keys' : null;
    const isHotReload = keysFilePath && fs.existsSync(keysFilePath);

    assignPathsToJSX(component.renderBody, '', pathGen, t, null, null, structuralChanges, isHotReload);
    console.log(`[Minimact Hex Paths] ✅ Assigned hex paths to ${componentName} JSX tree${isHotReload ? ' (hot reload mode)' : ''}`);

    // Store structural changes on component for later processing
    if (structuralChanges.length > 0) {
      component.structuralChanges = structuralChanges;
      console.log(`[Hot Reload] Found ${structuralChanges.length} structural changes in ${componentName}`);
    }

    const textTemplates = extractTemplates(component.renderBody, component);
    const attrTemplates = extractAttributeTemplates(component.renderBody, component);
    const allTemplates = { ...textTemplates, ...attrTemplates };

    // Add template metadata to component
    addTemplateMetadata(component, allTemplates);

    console.log(`[Minimact Templates] Extracted ${Object.keys(allTemplates).length} templates from ${componentName}`);

    // Extract loop templates for predictive rendering (.map() patterns)
    const loopTemplates = extractLoopTemplates(component.renderBody, component);
    component.loopTemplates = loopTemplates;

    if (loopTemplates.length > 0) {
      console.log(`[Minimact Loop Templates] Extracted ${loopTemplates.length} loop templates from ${componentName}:`);
      loopTemplates.forEach(lt => {
        console.log(`  - ${lt.stateKey}.map(${lt.itemVar} => ...)`);
      });
    }

    // Extract structural templates for conditional rendering (Phase 5)
    const structuralTemplates = extractStructuralTemplates(component.renderBody, component);
    component.structuralTemplates = structuralTemplates;

    if (structuralTemplates.length > 0) {
      console.log(`[Minimact Structural Templates] Extracted ${structuralTemplates.length} structural templates from ${componentName}:`);
      structuralTemplates.forEach(st => {
        console.log(`  - ${st.type === 'conditional' ? 'Ternary' : 'Logical AND'}: ${st.conditionBinding}`);
      });
    }

    // Extract conditional element templates (Phase 5.5 - Enhanced)
    const conditionalElementTemplates = extractConditionalElementTemplates(component.renderBody, component);
    component.conditionalElementTemplates = conditionalElementTemplates;

    if (Object.keys(conditionalElementTemplates).length > 0) {
      console.log(`[Minimact Conditional Element Templates] Extracted ${Object.keys(conditionalElementTemplates).length} conditional element templates from ${componentName}:`);
      Object.entries(conditionalElementTemplates).forEach(([path, template]) => {
        const evaluableMarker = template.evaluable ? '✅' : '⚠️';
        console.log(`  - ${evaluableMarker} ${path}: ${template.conditionExpression}`);
      });
    }

    // Extract expression templates for computed values (Phase 6)
    const expressionTemplates = extractExpressionTemplates(component.renderBody, component);
    component.expressionTemplates = expressionTemplates;

    if (expressionTemplates.length > 0) {
      console.log(`[Minimact Expression Templates] Extracted ${expressionTemplates.length} expression templates from ${componentName}:`);
      expressionTemplates.forEach(et => {
        if (et.method) {
          console.log(`  - ${et.binding}.${et.method}(${et.args?.join(', ') || ''})`);
        } else if (et.operator) {
          console.log(`  - ${et.operator}${et.binding}`);
        } else if (et.bindings) {
          console.log(`  - ${et.bindings.join(', ')}`);
        } else {
          console.log(`  - ${JSON.stringify(et)}`);
        }
      });
    }

    // Analyze timeline usage (@minimact/timeline)
    const timeline = analyzeTimeline(path, componentName);
    if (timeline) {
      component.timeline = timeline;
      console.log(`[Minimact Timeline] Found timeline in ${componentName}:`);
      console.log(`  - Duration: ${timeline.duration}ms`);
      console.log(`  - Keyframes: ${timeline.keyframes.length}`);
      console.log(`  - State bindings: ${timeline.stateBindings.size}`);
    }

    // Analyze plugin usage (Phase 3: Plugin System)
    const pluginUsages = analyzePluginUsage(path, component);
    component.pluginUsages = pluginUsages;

    if (pluginUsages.length > 0) {
      // Validate plugin usage
      validatePluginUsage(pluginUsages);

      console.log(`[Minimact Plugins] Found ${pluginUsages.length} plugin usage(s) in ${componentName}:`);
      pluginUsages.forEach(plugin => {
        const versionInfo = plugin.version ? ` v${plugin.version}` : '';
        console.log(`  - <Plugin name="${plugin.pluginName}"${versionInfo} state={${plugin.stateBinding.binding}} />`);
      });
    }
  }

  // Detect which top-level helper functions are referenced by this component
  if (state.file.topLevelFunctions && state.file.topLevelFunctions.length > 0) {
    const referencedFunctionNames = new Set();

    // Traverse the component to find all function calls
    path.traverse({
      CallExpression(callPath) {
        if (t.isIdentifier(callPath.node.callee)) {
          const funcName = callPath.node.callee.name;
          // Check if this matches a top-level function
          const helperFunc = state.file.topLevelFunctions.find(f => f.name === funcName);
          if (helperFunc) {
            referencedFunctionNames.add(funcName);
          }
        }
      }
    });

    // Add referenced functions to component's topLevelHelperFunctions array
    component.topLevelHelperFunctions = state.file.topLevelFunctions
      .filter(f => referencedFunctionNames.has(f.name))
      .map(f => ({
        name: f.name,
        node: f.node
      }));

    if (component.topLevelHelperFunctions.length > 0) {
      console.log(`[Minimact Helpers] Component '${componentName}' references ${component.topLevelHelperFunctions.length} helper function(s):`);
      component.topLevelHelperFunctions.forEach(f => {
        console.log(`  - ${f.name}()`);
      });
    }
  }

  // 🔥 NEW: Generate C# classes for imported hooks
  // After component processing is complete, check if any imported hooks were used
  if (component.customHooks && component.customHooks.length > 0) {
    const generatedHookClasses = new Set();  // Track to avoid duplicates

    component.customHooks.forEach(hookInstance => {
      // Check if this hook has metadata (was imported) and hasn't been generated yet
      if (hookInstance.metadata && !generatedHookClasses.has(hookInstance.className)) {
        // Create a minimal component context for the hook
        const hookComponentContext = {
          name: hookInstance.className,
          stateTypes: new Map(),
          dependencies: new Map(),
          externalImports: new Set(),
          clientComputedVars: new Set(),
          eventHandlers: []
        };

        // Generate the hook class from metadata
        const hookClass = generateHookClass(hookInstance.metadata, hookComponentContext);

        // Create hook component structure (same as processCustomHook)
        const hookComponent = {
          name: hookInstance.className,
          isHook: true, // Flag to identify this as a hook class
          hookData: hookClass, // Store the generated C# code
          hookAnalysis: hookInstance.metadata, // Store the analysis data
          props: [],
          useState: (hookInstance.metadata.states || []).map(s => ({
            varName: s.varName,
            setterName: s.setterName,
            initialValue: s.initialValue,
            type: s.type
          })),
          useClientState: [],
          useStateX: [],
          useEffect: [],
          useRef: [],
          useMarkdown: [],
          useTemplate: null,
          useValidation: [],
          useModal: [],
          useToggle: [],
          useDropdown: [],
          eventHandlers: (hookInstance.metadata.eventHandlers || []),
          localVariables: [],
          helperFunctions: [],
          renderBody: hookInstance.metadata.jsxElements,
          pluginUsages: [],
          stateTypes: new Map(),
          dependencies: new Map(),
          externalImports: new Set(),
          clientComputedVars: new Set(),
          templates: {}
        };

        // Add hook component to the components list
        state.file.minimactComponents.push(hookComponent);
        generatedHookClasses.add(hookInstance.className);

        console.log(`[Custom Hook] Generated C# class for imported hook: ${hookInstance.className}`);
      }
    });
  }

  // Store the component path so we can nullify JSX later (after .tsx.keys generation)
  if (!state.file.componentPathsToNullify) {
    state.file.componentPathsToNullify = [];
  }
  state.file.componentPathsToNullify.push(path);

  state.file.minimactComponents.push(component);
}

/**
 * Process a custom hook function
 * Generates a [Hook] class that extends MinimactComponent
 */
function processCustomHook(path, state) {
  const hookName = getComponentName(path);

  console.log(`[Custom Hook] Processing ${hookName}...`);

  // Analyze the hook structure
  const hookAnalysis = analyzeHook(path);

  if (!hookAnalysis) {
    console.error(`[Custom Hook] Failed to analyze ${hookName}`);
    return;
  }

  console.log(`[Custom Hook] Analysis complete:`, {
    states: hookAnalysis.states?.length || 0,
    methods: hookAnalysis.methods?.length || 0,
    hasJSX: !!hookAnalysis.jsxElements,
    returns: hookAnalysis.returnValues?.length || 0
  });

  // Create a minimal component context for the hook
  const hookComponentContext = {
    name: hookAnalysis.className,
    stateTypes: new Map(),
    dependencies: new Map(),
    externalImports: new Set(),
    clientComputedVars: new Set(),
    eventHandlers: []
  };

  // Generate the hook class C# code
  const hookClass = generateHookClass(hookAnalysis, hookComponentContext);

  console.log(`[Custom Hook] Generated class: ${hookClass.name || hookAnalysis.className}`);

  // Store hook class in state (as a special component)
  state.file.minimactComponents = state.file.minimactComponents || [];

  // Convert hook class to component-like structure for C# generation
  const hookComponent = {
    name: hookAnalysis.className,
    isHook: true, // Flag to identify this as a hook class
    hookData: hookClass, // Store the generated C# code
    hookAnalysis: hookAnalysis, // Store the analysis data
    props: [],
    useState: (hookAnalysis.states || []).map(s => ({
      varName: s.varName,
      setterName: s.setterName,
      initialValue: s.initialValue,
      type: s.type
    })),
    useClientState: [],
    useStateX: [],
    useEffect: [],
    useRef: [],
    useMarkdown: [],
    useTemplate: null,
    useValidation: [],
    useModal: [],
    useToggle: [],
    useDropdown: [],
    eventHandlers: (hookAnalysis.eventHandlers || []),
    localVariables: [],
    helperFunctions: [],
    renderBody: hookAnalysis.jsxElements,
    pluginUsages: [],
    stateTypes: new Map(),
    dependencies: new Map(),
    externalImports: new Set(),
    clientComputedVars: new Set(),
    templates: {}
  };

  state.file.minimactComponents.push(hookComponent);

  // Don't nullify JSX for hooks (they need their render method)
  // (processComponent adds components to componentPathsToNullify, but we skip that here)

  console.log(`[Custom Hook] ✅ ${hookName} processed successfully`);
}

module.exports = {
  processComponent
};
