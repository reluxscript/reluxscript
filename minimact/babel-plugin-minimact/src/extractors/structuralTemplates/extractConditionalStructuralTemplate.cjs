const t = require('@babel/types');
const { extractBinding } = require('./extractBinding.cjs');
const { extractStateKey } = require('./extractStateKey.cjs');
const { extractElementOrFragmentTemplate } = require('./extractElementOrFragmentTemplate.cjs');

/**
 * Extract structural template from ternary conditional
 *
 * Example: {isLoggedIn ? <Dashboard /> : <LoginForm />}
 * →
 * {
 *   type: 'conditional',
 *   conditionBinding: 'isLoggedIn',
 *   branches: {
 *     'true': ElementTemplate { tag: 'Dashboard', ... },
 *     'false': ElementTemplate { tag: 'LoginForm', ... }
 *   }
 * }
 */
function extractConditionalStructuralTemplate(conditionalExpr, component, path) {
  const test = conditionalExpr.test;
  const consequent = conditionalExpr.consequent;
  const alternate = conditionalExpr.alternate;

  // Extract condition binding
  const conditionBinding = extractBinding(test, component);
  if (!conditionBinding) {
    console.warn('[Structural Template] Could not extract condition binding');
    return null;
  }

  // Check if both branches are JSX elements (structural change)
  const hasTrueBranch = t.isJSXElement(consequent) || t.isJSXFragment(consequent);
  const hasFalseBranch = t.isJSXElement(alternate) || t.isJSXFragment(alternate) || t.isNullLiteral(alternate);

  if (!hasTrueBranch && !hasFalseBranch) {
    // Not a structural template (probably just conditional text)
    return null;
  }

  // Extract templates for both branches
  const branches = {};

  if (hasTrueBranch) {
    const trueBranch = extractElementOrFragmentTemplate(consequent, component);
    if (trueBranch) {
      branches['true'] = trueBranch;
    }
  }

  if (hasFalseBranch) {
    if (t.isNullLiteral(alternate)) {
      branches['false'] = { type: 'Null' };
    } else {
      const falseBranch = extractElementOrFragmentTemplate(alternate, component);
      if (falseBranch) {
        branches['false'] = falseBranch;
      }
    }
  }

  // Determine state key (for C# attribute)
  const stateKey = extractStateKey(test, component);

  return {
    type: 'conditional',
    stateKey: stateKey || conditionBinding,
    conditionBinding,
    branches,
    path
  };
}

module.exports = { extractConditionalStructuralTemplate };
