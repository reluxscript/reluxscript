const t = require('@babel/types');
const { extractBinding } = require('./extractBinding.cjs');
const { extractStateKey } = require('./extractStateKey.cjs');
const { SUPPORTED_TRANSFORMS } = require('./supportedTransforms.cjs');

/**
 * Extract template from method call
 *
 * Example: price.toFixed(2)
 * →
 * {
 *   type: 'methodCall',
 *   binding: 'price',
 *   method: 'toFixed',
 *   args: [2],
 *   transform: { type: 'numberFormat', method: 'toFixed', args: [2] }
 * }
 */
function extractMethodCallTemplate(callExpr, component, path) {
  const callee = callExpr.callee;
  const args = callExpr.arguments;

  // Get binding (e.g., 'price' from price.toFixed())
  const binding = extractBinding(callee.object);
  if (!binding) return null;

  // Get method name
  const methodName = callee.property.name;

  // Check if this is a supported transformation
  if (!SUPPORTED_TRANSFORMS[methodName]) {
    console.warn(`[Expression Template] Unsupported method: ${methodName}`);
    return null;
  }

  // Extract arguments
  const extractedArgs = args.map(arg => {
    if (t.isNumericLiteral(arg)) return arg.value;
    if (t.isStringLiteral(arg)) return arg.value;
    if (t.isBooleanLiteral(arg)) return arg.value;
    return null;
  }).filter(a => a !== null);

  // Determine state key
  const stateKey = extractStateKey(callee.object, component);

  return {
    type: 'methodCall',
    stateKey: stateKey || binding,
    binding,
    method: methodName,
    args: extractedArgs,
    transform: {
      type: SUPPORTED_TRANSFORMS[methodName].type,
      method: methodName,
      args: extractedArgs
    },
    path
  };
}

module.exports = { extractMethodCallTemplate };
