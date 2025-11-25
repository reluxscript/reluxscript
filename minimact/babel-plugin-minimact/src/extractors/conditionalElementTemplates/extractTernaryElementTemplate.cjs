const t = require('@babel/types');
const generate = require('@babel/generator').default;
const { extractBindingsFromCondition } = require('./extractBindingsFromCondition.cjs');
const { isConditionEvaluableClientSide } = require('./isConditionEvaluableClientSide.cjs');
const { extractElementStructure } = require('./extractElementStructure.cjs');

/**
 * Extract template from ternary expression
 * Example: {myState1 ? <div>Active</div> : <div>Inactive</div>}
 * @param {*} parentPath - Hex path of parent conditional template (for nesting)
 */
function extractTernaryElementTemplate(expr, containerNode, parentPath, stateKeyMap) {
  const test = expr.test;
  const consequent = expr.consequent;
  const alternate = expr.alternate;

  // Check if branches are JSX elements
  const hasConsequent = t.isJSXElement(consequent) || t.isJSXFragment(consequent);
  const hasAlternate = t.isJSXElement(alternate) || t.isJSXFragment(alternate) || t.isNullLiteral(alternate);

  if (!hasConsequent && !hasAlternate) {
    return null; // Not a structural template
  }

  // Extract condition
  const conditionCode = generate(test).code;
  const variableNames = extractBindingsFromCondition(test);

  // Build mapping from variable names to state keys
  const conditionMapping = {};
  const stateKeys = [];

  for (const varName of variableNames) {
    const stateKey = stateKeyMap.get(varName) || varName;
    conditionMapping[varName] = stateKey;
    stateKeys.push(stateKey);
  }

  const isEvaluable = isConditionEvaluableClientSide(test, variableNames);

  // Extract both branches
  const branches = {};

  if (hasConsequent) {
    branches.true = extractElementStructure(consequent);
  }

  if (hasAlternate) {
    if (t.isNullLiteral(alternate)) {
      branches.false = null;
    } else {
      branches.false = extractElementStructure(alternate);
    }
  }

  const template = {
    type: "conditional-element",
    conditionExpression: conditionCode,
    conditionBindings: stateKeys, // ["state_0", "state_1"]
    conditionMapping: conditionMapping, // { "myState1": "state_0", "myState2": "state_1" }
    evaluable: isEvaluable,
    branches,
    operator: "?"
  };

  // Add parent reference if nested
  if (parentPath) {
    template.parentTemplate = parentPath;
  }

  return template;
}

module.exports = { extractTernaryElementTemplate };
