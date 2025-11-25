const t = require('@babel/types');
const { extractBinding } = require('./extractBinding.cjs');
const { extractStateKey } = require('./extractStateKey.cjs');
const { extractElementOrFragmentTemplate } = require('./extractElementOrFragmentTemplate.cjs');

/**
 * Extract structural template from logical AND
 *
 * Example: {error && <ErrorMessage />}
 * →
 * {
 *   type: 'logicalAnd',
 *   conditionBinding: 'error',
 *   branches: {
 *     'truthy': ElementTemplate { tag: 'ErrorMessage', ... },
 *     'falsy': { type: 'Null' }
 *   }
 * }
 */
function extractLogicalAndTemplate(logicalExpr, component, path) {
  const left = logicalExpr.left;
  const right = logicalExpr.right;

  // Extract condition binding from left side
  const conditionBinding = extractBinding(left, component);
  if (!conditionBinding) {
    return null;
  }

  // Check if right side is JSX element (structural change)
  if (!t.isJSXElement(right) && !t.isJSXFragment(right)) {
    return null;
  }

  // Extract template for truthy case
  const truthyBranch = extractElementOrFragmentTemplate(right, component);
  if (!truthyBranch) {
    return null;
  }

  const stateKey = extractStateKey(left, component);

  return {
    type: 'logicalAnd',
    stateKey: stateKey || conditionBinding,
    conditionBinding,
    branches: {
      'truthy': truthyBranch,
      'falsy': { type: 'Null' }
    },
    path
  };
}

module.exports = { extractLogicalAndTemplate };
