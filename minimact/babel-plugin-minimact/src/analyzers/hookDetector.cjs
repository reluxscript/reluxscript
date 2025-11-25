/**
 * Hook Detection Module
 *
 * Detects custom hooks based on the pattern:
 * function use{Name}(namespace: string, ...args) { ... }
 *
 * Requirements:
 * 1. Function name starts with 'use'
 * 2. First parameter is named 'namespace' (string type)
 * 3. Contains at least one useState call OR returns JSX
 */

const t = require('@babel/types');

/**
 * Check if a path represents a custom hook definition
 *
 * @param {NodePath} path - Babel AST path
 * @returns {boolean} - True if this is a custom hook
 */
function isCustomHook(path) {
  let node, name, params;

  // Handle function declaration: function useCounter(namespace, start) { ... }
  if (t.isFunctionDeclaration(path.node)) {
    node = path.node;
    name = node.id?.name;
    params = node.params;
  }
  // Handle variable declarator with arrow function: const useCounter = (namespace, start) => { ... }
  else if (t.isVariableDeclarator(path.node)) {
    const init = path.node.init;
    if (t.isArrowFunctionExpression(init) || t.isFunctionExpression(init)) {
      node = init;
      name = path.node.id?.name;
      params = init.params;
    } else {
      return false;
    }
  }
  else {
    return false;
  }

  // Must have a name
  if (!name) return false;

  // Must start with 'use'
  if (!name.startsWith('use')) return false;

  // Must have at least one parameter
  if (!params || params.length === 0) return false;

  // First parameter must be 'namespace'
  const firstParam = params[0];
  if (!isNamespaceParameter(firstParam)) return false;

  return true;
}

/**
 * Check if a parameter is the required 'namespace' parameter
 *
 * @param {Node} param - Parameter node
 * @returns {boolean} - True if this is a valid namespace parameter
 */
function isNamespaceParameter(param) {
  // Handle simple identifier: namespace
  if (t.isIdentifier(param)) {
    return param.name === 'namespace';
  }

  // Handle TypeScript annotation: namespace: string
  if (param.typeAnnotation) {
    const id = param;
    if (t.isIdentifier(id) && id.name === 'namespace') {
      // Optionally verify it's typed as string
      const typeAnnotation = param.typeAnnotation;
      if (t.isTSTypeAnnotation(typeAnnotation)) {
        return t.isTSStringKeyword(typeAnnotation.typeAnnotation);
      }
      return true; // Accept without type annotation
    }
  }

  return false;
}

/**
 * Get the hook name from a path
 *
 * @param {NodePath} path - Babel AST path
 * @returns {string|null} - Hook name or null
 */
function getHookName(path) {
  if (t.isFunctionDeclaration(path.node)) {
    return path.node.id?.name || null;
  }
  if (t.isVariableDeclarator(path.node)) {
    return path.node.id?.name || null;
  }
  return null;
}

/**
 * Get all parameters of a hook (excluding namespace)
 *
 * @param {NodePath} path - Babel AST path
 * @returns {Array} - Array of parameter objects { name, type, defaultValue }
 */
function getHookParameters(path) {
  let params;

  if (t.isFunctionDeclaration(path.node)) {
    params = path.node.params;
  } else if (t.isVariableDeclarator(path.node)) {
    const init = path.node.init;
    if (t.isArrowFunctionExpression(init) || t.isFunctionExpression(init)) {
      params = init.params;
    } else {
      return [];
    }
  } else {
    return [];
  }

  // Skip first parameter (namespace)
  return params.slice(1).map(param => {
    let name, type = 'any', defaultValue = null;

    if (t.isIdentifier(param)) {
      name = param.name;
    } else if (t.isAssignmentPattern(param)) {
      // Has default value: start = 0
      name = param.left.name;
      defaultValue = param.right;
    }

    // Extract TypeScript type if present
    if (param.typeAnnotation && t.isTSTypeAnnotation(param.typeAnnotation)) {
      type = extractTypeString(param.typeAnnotation.typeAnnotation);
    }

    return { name, type, defaultValue };
  });
}

/**
 * Extract TypeScript type as string
 *
 * @param {Node} typeNode - TypeScript type node
 * @returns {string} - Type as string
 */
function extractTypeString(typeNode) {
  if (t.isTSStringKeyword(typeNode)) return 'string';
  if (t.isTSNumberKeyword(typeNode)) return 'number';
  if (t.isTSBooleanKeyword(typeNode)) return 'boolean';
  if (t.isTSAnyKeyword(typeNode)) return 'any';
  if (t.isTSArrayType(typeNode)) {
    const elementType = extractTypeString(typeNode.elementType);
    return `${elementType}[]`;
  }
  if (t.isTSTypeReference(typeNode)) {
    return typeNode.typeName.name || 'any';
  }
  return 'any';
}

/**
 * Get the function body from a hook path
 *
 * @param {NodePath} path - Babel AST path
 * @returns {Node|null} - Function body node or null
 */
function getHookBody(path) {
  if (t.isFunctionDeclaration(path.node)) {
    return path.node.body;
  }
  if (t.isVariableDeclarator(path.node)) {
    const init = path.node.init;
    if (t.isArrowFunctionExpression(init) || t.isFunctionExpression(init)) {
      return init.body;
    }
  }
  return null;
}

/**
 * Check if hook contains useState calls
 *
 * @param {NodePath} path - Babel AST path
 * @returns {boolean} - True if hook uses useState
 */
function containsUseState(path) {
  const body = getHookBody(path);
  if (!body) return false;

  let hasUseState = false;

  path.traverse({
    CallExpression(callPath) {
      if (t.isIdentifier(callPath.node.callee) &&
          callPath.node.callee.name === 'useState') {
        hasUseState = true;
        callPath.stop(); // Stop traversal early
      }
    }
  });

  return hasUseState;
}

/**
 * Check if hook returns JSX
 *
 * @param {NodePath} path - Babel AST path
 * @returns {boolean} - True if hook returns JSX
 */
function returnsJSX(path) {
  const body = getHookBody(path);
  if (!body) return false;

  let hasJSX = false;

  path.traverse({
    JSXElement(jsxPath) {
      hasJSX = true;
      jsxPath.stop();
    },
    JSXFragment(jsxPath) {
      hasJSX = true;
      jsxPath.stop();
    }
  });

  return hasJSX;
}

module.exports = {
  isCustomHook,
  getHookName,
  getHookParameters,
  getHookBody,
  containsUseState,
  returnsJSX,
  isNamespaceParameter,
  extractTypeString
};
