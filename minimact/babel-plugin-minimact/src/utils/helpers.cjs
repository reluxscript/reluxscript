/**
 * Utility Helpers
 *
 * General utility functions used throughout the plugin.
 *
 * Functions to move:
 * - escapeCSharpString(str) - Escapes special characters for C# strings
 * - getComponentName(path) - Extracts component name from function/class declaration
 *
 * Utilities:
 * - escapeCSharpString: Handles \, ", \n, \r, \t escaping
 * - getComponentName: Supports FunctionDeclaration, ArrowFunctionExpression, etc.
 *
 * Returns processed string or component name
 */

// TODO: Move the following functions here:
// - escapeCSharpString
// - getComponentName

/**
 * Escape C# string
 */
function escapeCSharpString(str) {
  return str
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

/**
 * Get component name from path
 */
function getComponentName(path) {
  if (path.node.id) {
    return path.node.id.name;
  }

  if (path.parent.type === 'VariableDeclarator') {
    return path.parent.id.name;
  }

  if (path.parent.type === 'ExportNamedDeclaration') {
    return path.node.id ? path.node.id.name : null;
  }

  return null;
}


module.exports = {
  escapeCSharpString,
  getComponentName,
};
