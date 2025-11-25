/**
 * Hook Class Generator
 *
 * Generates C# class code for custom hooks
 * Output format:
 *
 * [Hook]
 * public partial class UseCounterHook : MinimactComponent
 * {
 *     [State] private int count = 0;
 *     private void setCount(int value) => SetState("count", value);
 *     private void increment() => setCount(count + 1);
 *     protected override VNode Render() { ... }
 * }
 */

const { generateRenderBody } = require('./renderBody.cjs');
const { assignPathsToJSX } = require('../utils/pathAssignment.cjs');
const { HexPathGenerator } = require('../utils/hexPath.cjs');
const t = require('@babel/types');

/**
 * Generate complete C# class for a hook
 *
 * @param {Object} analysis - Hook analysis from hookAnalyzer
 * @param {Object} component - Component context
 * @returns {string} - C# class code
 */
function generateHookClass(analysis, component) {
  const lines = [];

  // Class declaration with [Hook] attribute
  lines.push(`// ============================================================`);
  lines.push(`// HOOK CLASS - Generated from ${analysis.name}`);
  lines.push(`// ============================================================`);
  lines.push(`[Hook]`);
  lines.push(`public partial class ${analysis.className} : MinimactComponent`);
  lines.push(`{`);

  // Configuration properties (from hook parameters)
  if (analysis.params && analysis.params.length > 0) {
    lines.push(`    // Configuration (from hook arguments)`);
    analysis.params.forEach(param => {
      const csharpType = mapTypeToCSharp(param.type);
      lines.push(`    private ${csharpType} ${param.name} => GetState<${csharpType}>("_config.${param.name}");`);
    });
    lines.push(``);
  }

  // State fields
  if (analysis.states && analysis.states.length > 0) {
    lines.push(`    // Hook state`);
    analysis.states.forEach(state => {
      const csharpType = mapTypeToCSharp(state.type);
      const initialValue = state.initialValue || getDefaultValue(csharpType);
      lines.push(`    [State]`);
      lines.push(`    private ${csharpType} ${state.varName} = ${initialValue};`);
      lines.push(``);
    });
  }

  // Setter methods for state
  if (analysis.states && analysis.states.length > 0) {
    lines.push(`    // State setters`);
    analysis.states.forEach(state => {
      const csharpType = mapTypeToCSharp(state.type);
      lines.push(`    private void ${state.setterName}(${csharpType} value)`);
      lines.push(`    {`);
      lines.push(`        SetState(nameof(${state.varName}), value);`);
      lines.push(`    }`);
      lines.push(``);
    });
  }

  // Methods
  if (analysis.methods && analysis.methods.length > 0) {
    lines.push(`    // Hook methods`);
    analysis.methods.forEach(method => {
      const returnType = mapTypeToCSharp(method.returnType);
      const params = method.params.map(p => {
        const type = mapTypeToCSharp(p.type);
        return `${type} ${p.name}`;
      }).join(', ');

      lines.push(`    private ${returnType} ${method.name}(${params})`);
      lines.push(`    {`);
      // TODO: Transpile method body properly
      lines.push(`        ${method.body}`);
      lines.push(`    }`);
      lines.push(``);
    });
  }

  // Render method (if hook has JSX)
  if (analysis.jsxElements && analysis.jsxElements.length > 0) {
    const jsx = analysis.jsxElements[0]; // Take first JSX element

    lines.push(`    // Hook UI rendering`);
    lines.push(`    protected override VNode Render()`);
    lines.push(`    {`);
    lines.push(`        StateManager.SyncMembersToState(this);`);
    lines.push(``);

    if (jsx && jsx.node) {
      try {
        // Assign hex paths to JSX tree
        const pathGen = new HexPathGenerator();
        assignPathsToJSX(jsx.node, '', pathGen, t);

        // Generate VNode code using existing generator
        const renderCode = generateRenderBody(jsx.node, component, 2);
        lines.push(`        ${renderCode.trim()}`);
      } catch (e) {
        console.error('[hookClassGenerator] Error generating JSX:', e.message);
        console.error(e.stack);
        lines.push(`        // TODO: Generate VNode tree from JSX`);
        lines.push(`        return new VNull("1");`);
      }
    } else {
      lines.push(`        // TODO: Generate VNode tree from JSX`);
      lines.push(`        return new VNull("1");`);
    }

    lines.push(`    }`);
    lines.push(``);

    // Event handlers (if any)
    if (component.eventHandlers && component.eventHandlers.length > 0) {
      lines.push(`    // Event handlers`);
      component.eventHandlers.forEach((handler, index) => {
        lines.push(`    public void Handle${index}(dynamic e)`);
        lines.push(`    {`);
        lines.push(`        ${handler.methodCall}`);
        lines.push(`    }`);
        lines.push(``);
      });
    }
  }

  lines.push(`}`);
  lines.push(``);

  return lines.join('\n');
}

/**
 * Map TypeScript/JavaScript type to C# type
 *
 * @param {string} jsType - JS/TS type
 * @returns {string} - C# type
 */
function mapTypeToCSharp(jsType) {
  const typeMap = {
    'string': 'string',
    'number': 'int',
    'boolean': 'bool',
    'any': 'dynamic',
    'void': 'void',
    'object': 'object',
    'any[]': 'List<dynamic>',
    'string[]': 'List<string>',
    'number[]': 'List<int>',
    'boolean[]': 'List<bool>'
  };

  return typeMap[jsType] || 'dynamic';
}

/**
 * Get default value for C# type
 *
 * @param {string} csharpType - C# type
 * @returns {string} - Default value
 */
function getDefaultValue(csharpType) {
  const defaults = {
    'int': '0',
    'bool': 'false',
    'string': '""',
    'dynamic': 'null',
    'object': 'null'
  };

  // Handle List types
  if (csharpType.startsWith('List<')) {
    return `new ${csharpType}()`;
  }

  return defaults[csharpType] || 'null';
}

/**
 * Generate VComponentWrapper usage code
 *
 * @param {Object} analysis - Hook analysis
 * @param {string} namespace - Hook namespace
 * @param {string} hexPath - Hex path for component
 * @param {Array} args - Hook arguments
 * @returns {string} - VComponentWrapper instantiation code
 */
function generateVComponentWrapper(analysis, namespace, hexPath, args) {
  const lines = [];

  lines.push(`new VComponentWrapper`);
  lines.push(`{`);
  lines.push(`    ComponentName = "${namespace}",`);
  lines.push(`    ComponentType = "${analysis.className}",`);
  lines.push(`    HexPath = "${hexPath}",`);
  lines.push(`    InitialState = new Dictionary<string, object>`);
  lines.push(`    {`);

  // Initialize state fields
  if (analysis.states && analysis.states.length > 0) {
    analysis.states.forEach((state, index) => {
      const comma = index < analysis.states.length - 1 || args.length > 0 ? ',' : '';
      lines.push(`        ["${state.varName}"] = ${state.initialValue || getDefaultValue(mapTypeToCSharp(state.type))}${comma}`);
    });
  }

  // Initialize config parameters
  if (args && args.length > 0) {
    args.forEach((arg, index) => {
      const comma = index < args.length - 1 ? ',' : '';
      lines.push(`        ["_config.${analysis.params[index].name}"] = ${arg}${comma}`);
    });
  }

  lines.push(`    }`);
  lines.push(`}`);

  return lines.join('\n');
}

/**
 * Generate state access code for parent component
 *
 * @param {Object} analysis - Hook analysis
 * @param {string} namespace - Hook namespace
 * @param {Array} returnBindings - Variable names from destructuring
 * @returns {Array} - Array of { varName, stateKey, code }
 */
function generateStateAccess(analysis, namespace, returnBindings) {
  const accessors = [];

  if (!analysis.returnValues || returnBindings.length === 0) {
    return accessors;
  }

  analysis.returnValues.forEach((returnValue, index) => {
    if (index >= returnBindings.length) return;

    const varName = returnBindings[index];

    if (returnValue.type === 'state') {
      // Access state value: var count = State["namespace.count"];
      const stateKey = `${namespace}.${returnValue.name}`;
      const csharpType = mapTypeToCSharp('any'); // TODO: Infer correct type
      accessors.push({
        varName: varName,
        stateKey: stateKey,
        code: `var ${varName} = GetState<${csharpType}>("${stateKey}");`
      });
    }
    else if (returnValue.type === 'setter') {
      // Generate method to call setter
      accessors.push({
        varName: varName,
        stateKey: null,
        code: `// ${varName} - method call via state manipulation`
      });
    }
    else if (returnValue.type === 'method') {
      // Generate method wrapper
      accessors.push({
        varName: varName,
        stateKey: null,
        code: `// ${varName} - method call via hook instance`
      });
    }
    // JSX is handled via VComponentWrapper, no accessor needed
  });

  return accessors;
}

module.exports = {
  generateHookClass,
  generateVComponentWrapper,
  generateStateAccess,
  mapTypeToCSharp,
  getDefaultValue
};
