/**
 * Operator expression handlers
 */

const t = require('@babel/types');

/**
 * Generate unary expression
 */
function generateUnaryExpression(node, generateCSharpExpression, inInterpolation) {
  const argument = generateCSharpExpression(node.argument, inInterpolation);
  const operator = node.operator;
  return `${operator}${argument}`;
}

/**
 * Generate binary expression
 */
function generateBinaryExpression(node, generateCSharpExpression) {
  // Helper function to get operator precedence (higher = tighter binding)
  const getPrecedence = (op) => {
    if (op === '*' || op === '/' || op === '%') return 3;
    if (op === '+' || op === '-') return 2;
    if (op === '==' || op === '!=' || op === '===' || op === '!==' ||
        op === '<' || op === '>' || op === '<=' || op === '>=') return 1;
    return 0;
  };

  const currentPrecedence = getPrecedence(node.operator);

  // Generate left side, wrap in parentheses if needed
  let left = generateCSharpExpression(node.left);
  if (t.isBinaryExpression(node.left)) {
    const leftPrecedence = getPrecedence(node.left.operator);
    // Wrap in parentheses if left has lower precedence
    if (leftPrecedence < currentPrecedence) {
      left = `(${left})`;
    }
  }

  // Generate right side, wrap in parentheses if needed
  let right = generateCSharpExpression(node.right);
  if (t.isBinaryExpression(node.right)) {
    const rightPrecedence = getPrecedence(node.right.operator);
    // Wrap in parentheses if right has lower or equal precedence
    // Equal precedence on right needs parens for left-associative operators
    if (rightPrecedence <= currentPrecedence) {
      right = `(${right})`;
    }
  }

  // Convert JavaScript operators to C# operators
  let operator = node.operator;
  if (operator === '===') operator = '==';
  if (operator === '!==') operator = '!=';
  return `${left} ${operator} ${right}`;
}

/**
 * Generate logical expression
 */
function generateLogicalExpression(node, generateCSharpExpression) {
  const left = generateCSharpExpression(node.left);
  const right = generateCSharpExpression(node.right);

  if (node.operator === '||') {
    // JavaScript: a || b
    // C#: a ?? b (null coalescing)
    return `(${left}) ?? (${right})`;
  } else if (node.operator === '&&') {
    // Check if right side is a boolean expression (comparison, logical, etc.)
    const rightIsBooleanExpr = t.isBinaryExpression(node.right) ||
                                t.isLogicalExpression(node.right) ||
                                t.isUnaryExpression(node.right);

    if (rightIsBooleanExpr) {
      // JavaScript: a && (b > 0)
      // C#: (a) && (b > 0) - boolean AND
      return `(${left}) && (${right})`;
    } else {
      // JavaScript: a && <jsx> or a && someValue
      // C#: a != null ? value : VNull (for objects)
      const nodePath = node.__minimactPath || '';
      return `(${left}) != null ? (${right}) : new VNull("${nodePath}")`;
    }
  }

  return `${left} ${node.operator} ${right}`;
}

/**
 * Generate conditional (ternary) expression
 */
function generateConditionalExpression(node, generateCSharpExpression) {
  // Handle ternary operator: test ? consequent : alternate
  // Children are always in normal C# expression context, not interpolation context
  const test = generateCSharpExpression(node.test, false);
  const consequent = generateCSharpExpression(node.consequent, false);
  const alternate = generateCSharpExpression(node.alternate, false);
  return `(${test}) ? ${consequent} : ${alternate}`;
}

/**
 * Generate assignment expression
 */
function generateAssignmentExpression(node, generateCSharpExpression, inInterpolation) {
  const left = generateCSharpExpression(node.left, inInterpolation);
  const right = generateCSharpExpression(node.right, inInterpolation);
  const operator = node.operator; // =, +=, -=, etc.
  return `${left} ${operator} ${right}`;
}

module.exports = {
  generateUnaryExpression,
  generateBinaryExpression,
  generateLogicalExpression,
  generateConditionalExpression,
  generateAssignmentExpression
};
