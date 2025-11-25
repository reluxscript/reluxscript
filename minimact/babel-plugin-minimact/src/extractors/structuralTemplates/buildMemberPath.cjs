const t = require('@babel/types');

/**
 * Build member expression path
 */
function buildMemberPath(expr) {
  const parts = [];
  let current = expr;

  while (t.isMemberExpression(current)) {
    if (t.isIdentifier(current.property)) {
      parts.unshift(current.property.name);
    }
    current = current.object;
  }

  if (t.isIdentifier(current)) {
    parts.unshift(current.name);
  }

  return parts.join('.');
}

module.exports = { buildMemberPath };
