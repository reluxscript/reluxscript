const t = require('@babel/types');

/**
 * Build member expression path (user.profile.name → "user.profile.name")
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

  return parts.length > 0 ? parts.join('.') : null;
}

module.exports = { buildMemberPath };
