const t = require('@babel/types');

/**
 * Build member expression path: user.name → "user.name"
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

/**
 * Build optional member expression path: viewModel?.userEmail → "viewModel.userEmail"
 */
function buildOptionalMemberPath(expr) {
  const parts = [];
  let current = expr;

  while (t.isOptionalMemberExpression(current) || t.isMemberExpression(current)) {
    if (t.isIdentifier(current.property)) {
      parts.unshift(current.property.name);
    } else {
      return null; // Computed property
    }
    current = current.object;
  }

  if (t.isIdentifier(current)) {
    parts.unshift(current.name);
    return parts.join('.');
  }

  return null;
}

module.exports = { buildMemberPath, buildOptionalMemberPath };
