const t = require('@babel/types');

/**
 * Build full path from member expression
 *
 * Example: todo.author.name → "todo.author.name"
 */
function buildMemberExpressionPath(expr) {
  const parts = [];
  let current = expr;

  while (t.isMemberExpression(current)) {
    if (t.isIdentifier(current.property)) {
      parts.unshift(current.property.name);
    } else {
      return null; // Computed property (not supported)
    }
    current = current.object;
  }

  if (t.isIdentifier(current)) {
    parts.unshift(current.name);
    return parts.join('.');
  }

  return null;
}

module.exports = { buildMemberExpressionPath };
