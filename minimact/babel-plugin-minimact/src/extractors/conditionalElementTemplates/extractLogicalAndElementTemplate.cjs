const t = require('@babel/types');
const generate = require('@babel/generator').default;
const { extractLeftSideOfAnd } = require('./extractLeftSideOfAnd.cjs');
const { extractBindingsFromCondition } = require('./extractBindingsFromCondition.cjs');
const { isConditionEvaluableClientSide } = require('./isConditionEvaluableClientSide.cjs');
const { extractElementStructure } = require('./extractElementStructure.cjs');

/**
 * Extract template from logical AND expression
 * Example: {myState1 && !myState2 && <div>{myState3}</div>}
 * @param {*} parentPath - Hex path of parent conditional template (for nesting)
 */
function extractLogicalAndElementTemplate(expr, containerNode, parentPath, stateKeyMap) {
  const right = expr.right;

  // Check if right side is JSX element (structural)
  if (!t.isJSXElement(right) && !t.isJSXFragment(right)) {
    return null;
  }

  // Extract full condition expression
  const condition = extractLeftSideOfAnd(expr);
  const conditionCode = generate(condition).code;

  // Extract bindings from condition (variable names)
  const variableNames = extractBindingsFromCondition(condition);

  // Build mapping from variable names to state keys
  const conditionMapping = {};
  const stateKeys = [];

  for (const varName of variableNames) {
    const stateKey = stateKeyMap.get(varName) || varName;
    conditionMapping[varName] = stateKey;
    stateKeys.push(stateKey);
  }

  // Can we evaluate this condition client-side?
  const isEvaluable = isConditionEvaluableClientSide(condition, variableNames);

  // Extract element structure
  const elementStructure = extractElementStructure(right);

  if (!elementStructure) {
    return null;
  }

  const template = {
    type: "conditional-element",
    conditionExpression: conditionCode,
    conditionBindings: stateKeys, // ["state_0", "state_1"]
    conditionMapping: conditionMapping, // { "myState1": "state_0", "myState2": "state_1" }
    evaluable: isEvaluable,
    branches: {
      true: elementStructure,
      false: null
    },
    operator: "&&"
  };

  // Add parent reference if nested
  if (parentPath) {
    template.parentTemplate = parentPath;
  }

  return template;
}

module.exports = { extractLogicalAndElementTemplate };
