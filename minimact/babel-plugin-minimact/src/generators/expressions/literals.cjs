/**
 * Literal expression handlers
 */

const t = require('@babel/types');
const { escapeCSharpString } = require('../../utils/helpers.cjs');

/**
 * Generate string literal
 */
function generateStringLiteral(node, inInterpolation) {
  // In string interpolation context, escape the quotes: \"text\"
  // Otherwise use normal quotes: "text"
  if (inInterpolation) {
    return `\\"${escapeCSharpString(node.value)}\\"`;
  } else {
    return `"${escapeCSharpString(node.value)}"`;
  }
}

/**
 * Generate numeric literal
 */
function generateNumericLiteral(node) {
  return String(node.value);
}

/**
 * Generate boolean literal
 */
function generateBooleanLiteral(node) {
  return node.value ? 'true' : 'false';
}

/**
 * Generate null literal
 */
function generateNullLiteral(node) {
  const nodePath = node.__minimactPath || '';
  return `new VNull("${nodePath}")`;
}

/**
 * Generate template literal
 */
function generateTemplateLiteral(node, generateCSharpExpression) {
  // If no expressions, use verbatim string literal (@"...") to avoid escaping issues
  if (node.expressions.length === 0) {
    const text = node.quasis[0].value.raw;
    // Use verbatim string literal (@"...") for multiline or strings with special chars
    // Escape " as "" in verbatim strings
    const escaped = text.replace(/"/g, '""');
    return `@"${escaped}"`;
  }

  // Has expressions - use C# string interpolation
  let result = '$"';
  for (let i = 0; i < node.quasis.length; i++) {
    // Escape special chars in C# interpolated strings
    let text = node.quasis[i].value.raw;
    // Escape { and } by doubling them
    text = text.replace(/{/g, '{{').replace(/}/g, '}}');
    // Escape " as \"
    text = text.replace(/"/g, '\\"');
    result += text;

    if (i < node.expressions.length) {
      const expr = node.expressions[i];
      // Wrap conditional (ternary) expressions in parentheses to avoid ':' conflict in C# interpolation
      const exprCode = generateCSharpExpression(expr);
      const needsParens = t.isConditionalExpression(expr);
      result += '{' + (needsParens ? `(${exprCode})` : exprCode) + '}';
    }
  }
  result += '"';
  return result;
}

module.exports = {
  generateStringLiteral,
  generateNumericLiteral,
  generateBooleanLiteral,
  generateNullLiteral,
  generateTemplateLiteral
};
