/**
 * Array expression handlers
 */

const t = require('@babel/types');

/**
 * Generate array expression
 */
function generateArrayExpression(node, generateCSharpExpression) {
  // Check if array contains spread elements
  const hasSpread = node.elements.some(e => t.isSpreadElement(e));

  if (hasSpread) {
    // Handle spread operator: [...array, item] → array.Concat(new[] { item }).ToList()
    const parts = [];
    let currentLiteral = [];

    for (const element of node.elements) {
      if (t.isSpreadElement(element)) {
        // Flush current literal elements
        if (currentLiteral.length > 0) {
          const literalCode = currentLiteral.map(e => generateCSharpExpression(e)).join(', ');
          parts.push(`new[] { ${literalCode} }`);
          currentLiteral = [];
        }
        // Add spread array (cast to preserve type)
        parts.push(generateCSharpExpression(element.argument));
      } else {
        currentLiteral.push(element);
      }
    }

    // Flush remaining literals
    if (currentLiteral.length > 0) {
      const literalCode = currentLiteral.map(e => generateCSharpExpression(e)).join(', ');
      parts.push(`new[] { ${literalCode} }`);
    }

    // Combine with Concat
    if (parts.length === 1) {
      return `${parts[0]}.ToList()`;
    } else {
      const concats = parts.slice(1).map(p => `.Concat(${p})`).join('');
      return `${parts[0]}${concats}.ToList()`;
    }
  }

  // No spread - simple array literal
  const elements = node.elements.map(e => generateCSharpExpression(e)).join(', ');

  // Infer type from first element if all are string literals
  if (node.elements.length > 0 && node.elements.every(e => t.isStringLiteral(e))) {
    return `new List<string> { ${elements} }`;
  }

  // Use List<dynamic> for empty arrays to be compatible with dynamic LINQ results
  const listType = elements.length === 0 ? 'dynamic' : 'object';
  return `new List<${listType}> { ${elements} }`;
}

module.exports = {
  generateArrayExpression
};
