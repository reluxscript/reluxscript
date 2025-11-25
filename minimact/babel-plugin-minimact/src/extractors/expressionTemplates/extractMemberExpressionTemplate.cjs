const { buildMemberPath } = require('./buildMemberPath.cjs');
const { extractStateKey } = require('./extractStateKey.cjs');
const { SUPPORTED_TRANSFORMS } = require('./supportedTransforms.cjs');

/**
 * Extract template from member expression
 *
 * Example: items.length
 * →
 * {
 *   type: 'memberExpression',
 *   binding: 'items.length',
 *   transform: { type: 'property', property: 'length' }
 * }
 */
function extractMemberExpressionTemplate(memberExpr, component, path) {
  // Check for computed property access: item[field]
  if (memberExpr.computed) {
    console.warn('[Minimact Warning] Computed property access detected - skipping template optimization (requires runtime evaluation)');

    // Return a special marker indicating this needs runtime evaluation
    // The C# generator will handle this as dynamic property access
    return {
      type: 'computedMemberExpression',
      isComputed: true,
      requiresRuntimeEval: true,
      object: memberExpr.object,
      property: memberExpr.property,
      path
    };
  }

  const binding = buildMemberPath(memberExpr);
  if (!binding) return null;

  // Get property name (only for non-computed properties)
  const propertyName = memberExpr.property.name;

  // Check if it's a supported property
  if (!SUPPORTED_TRANSFORMS[propertyName]) {
    return null;
  }

  const stateKey = extractStateKey(memberExpr, component);

  return {
    type: 'memberExpression',
    stateKey: stateKey || binding.split('.')[0],
    binding,
    property: propertyName,
    transform: {
      type: SUPPORTED_TRANSFORMS[propertyName].type,
      property: propertyName
    },
    path
  };
}

module.exports = { extractMemberExpressionTemplate };
