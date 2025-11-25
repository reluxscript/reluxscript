/**
 * Generate C# code for Plugin elements
 * Transforms <Plugin name="..." state={...} /> to C# PluginNode instances
 *
 * Phase 3: Babel Plugin Integration
 */

const { generateCSharpExpression } = require('./expressions.cjs');

/**
 * Generate C# code for a plugin usage
 * @param {Object} pluginMetadata - Plugin usage metadata from analyzer
 * @param {Object} componentState - Component metadata
 * @returns {string} C# code
 */
function generatePluginNode(pluginMetadata, componentState) {
  const { pluginName, stateBinding, version } = pluginMetadata;

  // Generate state expression
  const stateCode = generateStateExpression(stateBinding, componentState);

  // Generate PluginNode constructor call
  if (version) {
    // Future: Support version-specific plugin loading
    // For now, version is informational only
    return `new PluginNode("${pluginName}", ${stateCode}) /* v${version} */`;
  }

  return `new PluginNode("${pluginName}", ${stateCode})`;
}

/**
 * Generate C# expression for plugin state
 * @param {Object} stateBinding - State binding metadata
 * @param {Object} componentState - Component metadata
 * @returns {string} C# code
 */
function generateStateExpression(stateBinding, componentState) {
  switch (stateBinding.type) {
    case 'identifier':
      // Simple identifier: state={currentTime} -> currentTime
      return stateBinding.name;

    case 'memberExpression':
      // Member expression: state={this.state.time} -> state.time (remove 'this')
      return stateBinding.binding;

    case 'objectExpression':
      // Inline object: state={{ hours: h, minutes: m }}
      return generateInlineObject(stateBinding, componentState);

    case 'complexExpression':
      // Complex expression: evaluate using expression generator
      return generateCSharpExpression(stateBinding.expression);

    default:
      throw new Error(`Unknown state binding type: ${stateBinding.type}`);
  }
}

/**
 * Generate C# code for inline object expression
 * @param {Object} stateBinding - State binding with objectExpression type
 * @param {Object} componentState - Component metadata
 * @returns {string} C# anonymous object code
 */
function generateInlineObject(stateBinding, componentState) {
  const properties = stateBinding.properties;

  if (!properties || properties.length === 0) {
    return 'new { }';
  }

  const propStrings = properties.map(prop => {
    const key = prop.key.name || prop.key.value;
    const value = generateCSharpExpression(prop.value);
    return `${key} = ${value}`;
  });

  return `new { ${propStrings.join(', ')} }`;
}

/**
 * Generate using directives needed for plugins
 * @returns {Array<string>} Using statements
 */
function generatePluginUsings() {
  return [
    'using Minimact.AspNetCore.Core;',
    'using Minimact.AspNetCore.Plugins;'
  ];
}

/**
 * Check if component uses plugins (for conditional using statement inclusion)
 * @param {Object} componentState - Component metadata
 * @returns {boolean}
 */
function usesPlugins(componentState) {
  return componentState.pluginUsages && componentState.pluginUsages.length > 0;
}

/**
 * Generate comment documenting plugin usage
 * @param {Object} pluginMetadata - Plugin metadata
 * @returns {string} C# comment
 */
function generatePluginComment(pluginMetadata) {
  const { pluginName, stateBinding, version } = pluginMetadata;

  const versionInfo = version ? ` (v${version})` : '';
  const stateInfo = stateBinding.stateType
    ? ` : ${stateBinding.stateType}`
    : '';

  return `// Plugin: ${pluginName}${versionInfo}, State: ${stateBinding.binding}${stateInfo}`;
}

/**
 * Generate validation code for plugin state (optional, for runtime safety)
 * @param {Object} pluginMetadata - Plugin metadata
 * @returns {string|null} C# validation code or null
 */
function generatePluginValidation(pluginMetadata) {
  // Future enhancement: Generate runtime validation
  // For now, validation happens in PluginManager
  return null;
}

module.exports = {
  generatePluginNode,
  generateStateExpression,
  generateInlineObject,
  generatePluginUsings,
  generatePluginComment,
  generatePluginValidation,
  usesPlugins
};
