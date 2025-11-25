/**
 * Analyze <Plugin name="..." state={...} /> JSX elements in React components
 * Detects plugin usage and extracts metadata for C# code generation
 *
 * Phase 3: Babel Plugin Integration
 *
 * Transforms:
 *   <Plugin name="Clock" state={currentTime} />
 *
 * To C# code:
 *   new PluginNode("Clock", currentTime)
 */

const t = require('@babel/types');

/**
 * Analyze JSX tree for Plugin elements
 * @param {Object} path - Babel path to component function
 * @param {Object} componentState - Component metadata being built
 * @returns {Array} Array of plugin usage metadata
 */
function analyzePluginUsage(path, componentState) {
  const pluginUsages = [];

  path.traverse({
    JSXElement(jsxPath) {
      const openingElement = jsxPath.node.openingElement;

      // Check if this is a <Plugin> element
      if (!isPluginElement(openingElement)) {
        return;
      }

      try {
        const pluginMetadata = extractPluginMetadata(openingElement, componentState);
        pluginUsages.push(pluginMetadata);

        // Log for debugging
        console.log(`[analyzePluginUsage] Found plugin usage: ${pluginMetadata.pluginName}`);
      } catch (error) {
        console.error(`[analyzePluginUsage] Error analyzing plugin:`, error.message);
        throw error;
      }
    }
  });

  return pluginUsages;
}

/**
 * Check if JSX element is a <Plugin> component
 * @param {Object} openingElement - JSX opening element
 * @returns {boolean}
 */
function isPluginElement(openingElement) {
  // Check for <Plugin> or <Plugin.Something>
  const name = openingElement.name;

  if (t.isJSXIdentifier(name)) {
    return name.name === 'Plugin';
  }

  if (t.isJSXMemberExpression(name)) {
    return name.object.name === 'Plugin';
  }

  return false;
}

/**
 * Extract plugin metadata from JSX element
 * @param {Object} openingElement - JSX opening element
 * @param {Object} componentState - Component metadata
 * @returns {Object} Plugin metadata
 */
function extractPluginMetadata(openingElement, componentState) {
  const nameAttr = findAttribute(openingElement.attributes, 'name');
  const stateAttr = findAttribute(openingElement.attributes, 'state');
  const versionAttr = findAttribute(openingElement.attributes, 'version');

  // Validate required attributes
  if (!nameAttr) {
    throw new Error('Plugin element requires "name" attribute');
  }

  if (!stateAttr) {
    throw new Error('Plugin element requires "state" attribute');
  }

  // Extract plugin name (must be a string literal)
  const pluginName = extractPluginName(nameAttr);

  // Extract state binding (can be expression or identifier)
  const stateBinding = extractStateBinding(stateAttr, componentState);

  // Extract optional version
  const version = versionAttr ? extractVersion(versionAttr) : null;

  return {
    pluginName,
    stateBinding,
    version,
    // Store original JSX for reference
    jsxElement: openingElement
  };
}

/**
 * Find attribute by name in JSX attributes
 * @param {Array} attributes - JSX attributes
 * @param {string} name - Attribute name to find
 * @returns {Object|null}
 */
function findAttribute(attributes, name) {
  return attributes.find(attr =>
    t.isJSXAttribute(attr) && attr.name.name === name
  );
}

/**
 * Extract plugin name from name attribute
 * Must be a string literal (e.g., name="Clock")
 * @param {Object} nameAttr - JSX attribute node
 * @returns {string}
 */
function extractPluginName(nameAttr) {
  const value = nameAttr.value;

  // String literal: name="Clock"
  if (t.isStringLiteral(value)) {
    return value.value;
  }

  // JSX expression: name={"Clock"} (also a string literal)
  if (t.isJSXExpressionContainer(value) && t.isStringLiteral(value.expression)) {
    return value.expression.value;
  }

  throw new Error('Plugin "name" attribute must be a string literal (e.g., name="Clock")');
}

/**
 * Extract state binding from state attribute
 * Can be an identifier or expression
 * @param {Object} stateAttr - JSX attribute node
 * @param {Object} componentState - Component metadata
 * @returns {Object} State binding metadata
 */
function extractStateBinding(stateAttr, componentState) {
  const value = stateAttr.value;

  if (!t.isJSXExpressionContainer(value)) {
    throw new Error('Plugin "state" attribute must be a JSX expression (e.g., state={currentTime})');
  }

  const expression = value.expression;

  // Simple identifier: state={currentTime}
  if (t.isIdentifier(expression)) {
    return {
      type: 'identifier',
      name: expression.name,
      binding: expression.name,
      stateType: inferStateType(expression.name, componentState)
    };
  }

  // Member expression: state={this.state.time}
  if (t.isMemberExpression(expression)) {
    const binding = generateBindingPath(expression);
    return {
      type: 'memberExpression',
      binding,
      expression: expression,
      stateType: inferStateType(binding, componentState)
    };
  }

  // Object expression: state={{ hours: h, minutes: m }}
  if (t.isObjectExpression(expression)) {
    return {
      type: 'objectExpression',
      binding: '__inline_object__',
      properties: expression.properties,
      expression: expression
    };
  }

  // Any other expression (will be evaluated at runtime)
  return {
    type: 'complexExpression',
    binding: '__complex__',
    expression: expression
  };
}

/**
 * Extract version from version attribute
 * @param {Object} versionAttr - JSX attribute node
 * @returns {string|null}
 */
function extractVersion(versionAttr) {
  const value = versionAttr.value;

  if (t.isStringLiteral(value)) {
    return value.value;
  }

  if (t.isJSXExpressionContainer(value) && t.isStringLiteral(value.expression)) {
    return value.expression.value;
  }

  return null;
}

/**
 * Generate binding path from member expression
 * e.g., this.state.time -> "state.time"
 * @param {Object} expression - Member expression AST node
 * @returns {string}
 */
function generateBindingPath(expression) {
  const parts = [];

  function traverse(node) {
    if (t.isIdentifier(node)) {
      // Skip 'this' prefix
      if (node.name !== 'this') {
        parts.unshift(node.name);
      }
    } else if (t.isMemberExpression(node)) {
      if (t.isIdentifier(node.property)) {
        parts.unshift(node.property.name);
      }
      traverse(node.object);
    }
  }

  traverse(expression);
  return parts.join('.');
}

/**
 * Infer state type from binding name and component metadata
 * @param {string} bindingName - Name of the state binding
 * @param {Object} componentState - Component metadata
 * @returns {string|null}
 */
function inferStateType(bindingName, componentState) {
  // Check useState declarations
  if (componentState.useState) {
    const stateDecl = componentState.useState.find(s =>
      s.name === bindingName || s.setterName === bindingName
    );
    if (stateDecl) {
      return stateDecl.type || 'object';
    }
  }

  // Check props
  if (componentState.props) {
    const prop = componentState.props.find(p => p.name === bindingName);
    if (prop) {
      return prop.type || 'object';
    }
  }

  // Check local variables
  if (componentState.localVariables) {
    const localVar = componentState.localVariables.find(v => v.name === bindingName);
    if (localVar) {
      return localVar.type || 'object';
    }
  }

  // Default to object if we can't infer
  return 'object';
}

/**
 * Validate plugin usage (called after analysis)
 * @param {Array} pluginUsages - Array of plugin usage metadata
 * @throws {Error} If validation fails
 */
function validatePluginUsage(pluginUsages) {
  for (const plugin of pluginUsages) {
    // Validate plugin name format
    if (!/^[A-Za-z][A-Za-z0-9]*$/.test(plugin.pluginName)) {
      throw new Error(
        `Invalid plugin name "${plugin.pluginName}". ` +
        `Plugin names must start with a letter and contain only letters and numbers.`
      );
    }

    // Validate state binding
    if (plugin.stateBinding.binding === '__complex__') {
      console.warn(
        `[analyzePluginUsage] Complex expression used for plugin "${plugin.pluginName}" state. ` +
        `This will be evaluated at runtime.`
      );
    }

    // Validate version format if provided
    if (plugin.version && !/^\d+\.\d+\.\d+$/.test(plugin.version)) {
      console.warn(
        `[analyzePluginUsage] Invalid semver format for plugin "${plugin.pluginName}": ${plugin.version}`
      );
    }
  }
}

module.exports = {
  analyzePluginUsage,
  validatePluginUsage,
  isPluginElement,
  extractPluginMetadata
};
