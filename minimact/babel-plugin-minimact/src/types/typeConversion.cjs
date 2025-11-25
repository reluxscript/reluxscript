/**
 * Type Conversion
 */

const t = require('@babel/types');

/**
 * Convert TypeScript type annotation to C# type
 */
function tsTypeToCSharpType(tsType) {
  if (!tsType) return 'dynamic';

  // TSStringKeyword -> string
  if (t.isTSStringKeyword(tsType)) return 'string';

  // TSNumberKeyword -> double
  if (t.isTSNumberKeyword(tsType)) return 'double';

  // TSBooleanKeyword -> bool
  if (t.isTSBooleanKeyword(tsType)) return 'bool';

  // TSAnyKeyword -> dynamic
  if (t.isTSAnyKeyword(tsType)) return 'dynamic';

  // TSArrayType -> List<T>
  if (t.isTSArrayType(tsType)) {
    const elementType = tsTypeToCSharpType(tsType.elementType);
    return `List<${elementType}>`;
  }

  // TSTypeLiteral (object type) -> dynamic
  if (t.isTSTypeLiteral(tsType)) return 'dynamic';

  // TSTypeReference (custom types, interfaces)
  if (t.isTSTypeReference(tsType)) {
    // Handle @minimact/mvc type mappings
    if (t.isIdentifier(tsType.typeName)) {
      const typeName = tsType.typeName.name;

      // Map @minimact/mvc types to C# types
      const typeMap = {
        'decimal': 'decimal',
        'int': 'int',
        'int32': 'int',
        'int64': 'long',
        'long': 'long',
        'float': 'float',
        'float32': 'float',
        'float64': 'double',
        'double': 'double',
        'short': 'short',
        'int16': 'short',
        'byte': 'byte',
        'Guid': 'Guid',
        'DateTime': 'DateTime',
        'DateOnly': 'DateOnly',
        'TimeOnly': 'TimeOnly'
      };

      if (typeMap[typeName]) {
        return typeMap[typeName];
      }
    }

    // Other type references default to dynamic
    return 'dynamic';
  }

  // Default to dynamic for full JSX semantics
  return 'dynamic';
}

/**
 * Infer C# type from initial value
 */
function inferType(node) {
  if (!node) return 'dynamic';

  if (t.isStringLiteral(node)) return 'string';
  if (t.isNumericLiteral(node)) {
    // Check if the number has a decimal point
    // If the value is a whole number, use int; otherwise use double
    const value = node.value;
    return Number.isInteger(value) ? 'int' : 'double';
  }
  if (t.isBooleanLiteral(node)) return 'bool';
  if (t.isNullLiteral(node)) return 'dynamic';
  if (t.isArrayExpression(node)) return 'List<dynamic>';
  if (t.isObjectExpression(node)) return 'dynamic';

  return 'dynamic';
}


module.exports = {
  inferType,
  tsTypeToCSharpType
};
