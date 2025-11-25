const t = require('@babel/types');

/**
 * Extract JSX element from callback function body
 */
function extractJSXFromCallback(callback) {
  const body = callback.body;

  // Arrow function with direct JSX return: (...) => <li>...</li>
  if (t.isJSXElement(body)) {
    return body;
  }

  // Arrow function or function expression with block body
  if (t.isBlockStatement(body)) {
    // Find return statement
    for (const stmt of body.body) {
      if (t.isReturnStatement(stmt) && t.isJSXElement(stmt.argument)) {
        return stmt.argument;
      }
    }
  }

  // Expression wrapped in parentheses or conditional
  if (t.isConditionalExpression(body)) {
    // Handle ternary: condition ? <div/> : <span/>
    // For now, just take the consequent (true branch)
    if (t.isJSXElement(body.consequent)) {
      return body.consequent;
    }
  }

  if (t.isLogicalExpression(body) && body.operator === '&&') {
    // Handle logical AND: condition && <div/>
    if (t.isJSXElement(body.right)) {
      return body.right;
    }
  }

  return null;
}

module.exports = { extractJSXFromCallback };
