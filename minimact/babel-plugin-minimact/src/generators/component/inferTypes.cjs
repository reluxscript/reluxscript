/**
 * Type inference utilities
 */

const t = require('@babel/types');

/**
 * Infer C# type from JavaScript AST node (for client-computed variables)
 */
function inferCSharpTypeFromInit(node) {
  if (!node) return 'dynamic';

  // Array types
  if (t.isArrayExpression(node)) {
    return 'List<dynamic>';
  }

  // Call expressions - try to infer from method name
  if (t.isCallExpression(node)) {
    const callee = node.callee;

    if (t.isMemberExpression(callee) && t.isIdentifier(callee.property)) {
      const method = callee.property.name;

      // Common array methods return arrays
      if (['map', 'filter', 'sort', 'sortBy', 'orderBy', 'slice', 'concat'].includes(method)) {
        return 'List<dynamic>';
      }

      // Aggregation methods return numbers
      if (['reduce', 'sum', 'sumBy', 'mean', 'meanBy', 'average', 'count', 'size'].includes(method)) {
        return 'double';
      }

      // Find methods return single item
      if (['find', 'minBy', 'maxBy', 'first', 'last'].includes(method)) {
        return 'dynamic';
      }

      // String methods
      if (['format', 'toString', 'join'].includes(method)) {
        return 'string';
      }
    }

    // Direct function calls (moment(), _.chain(), etc.)
    return 'dynamic';
  }

  // String operations
  if (t.isTemplateLiteral(node) || t.isStringLiteral(node)) {
    return 'string';
  }

  // Numbers
  if (t.isNumericLiteral(node)) {
    return 'double';
  }

  // Booleans
  if (t.isBooleanLiteral(node)) {
    return 'bool';
  }

  // Binary expressions - try to infer from operation
  if (t.isBinaryExpression(node)) {
    if (['+', '-', '*', '/', '%'].includes(node.operator)) {
      return 'double';
    }
    if (['==', '===', '!=', '!==', '<', '>', '<=', '>='].includes(node.operator)) {
      return 'bool';
    }
  }

  // Logical expressions
  if (t.isLogicalExpression(node)) {
    return 'bool';
  }

  // Default to dynamic
  return 'dynamic';
}

/**
 * Convert TypeScript type to C# type
 */
function tsTypeToCSharpType(tsType) {
  if (!tsType) return 'dynamic';

  switch (tsType.type) {
    case 'TSStringKeyword':
      return 'string';
    case 'TSNumberKeyword':
      return 'double';
    case 'TSBooleanKeyword':
      return 'bool';
    case 'TSVoidKeyword':
      return 'void';
    case 'TSAnyKeyword':
      return 'dynamic';
    case 'TSArrayType':
      const elementType = tsTypeToCSharpType(tsType.elementType);
      return `List<${elementType}>`;
    default:
      return 'dynamic';
  }
}

module.exports = {
  inferCSharpTypeFromInit,
  tsTypeToCSharpType
};
