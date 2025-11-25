/**
 * Conditional Element Template Extractor
 *
 * Extracts complete element structure templates for conditional rendering.
 * This enables the client to construct entire element trees instantly when
 * conditions change, without waiting for server round-trip.
 *
 * Examples:
 * - {myState1 && <div>Content</div>}
 * - {myState1 && !myState2 && <div>{myState3}</div>}
 * - {condition ? <Active /> : <Inactive />}
 *
 * The client can evaluate simple boolean conditions and construct the full
 * DOM tree from the template, providing instant feedback.
 */

const { traverseJSX } = require('./conditionalElementTemplates/traverseJSX.cjs');

/**
 * Extract conditional element templates from JSX render body
 *
 * Returns object keyed by hex path:
 * {
 *   "1.2": {
 *     type: "conditional-element",
 *     conditionExpression: "myState1",
 *     conditionBindings: ["state_0"],  // ← Resolved to state keys!
 *     branches: {
 *       true: { element structure },
 *       false: null
 *     }
 *   }
 * }
 */
function extractConditionalElementTemplates(renderBody, component) {
  if (!renderBody) return {};

  const conditionalTemplates = {};

  // Build mapping from variable name → state key
  const stateKeyMap = new Map();

  // Map useState variables to state_N keys
  if (component.useState) {
    component.useState.forEach((state, index) => {
      stateKeyMap.set(state.name, `state_${index}`);
    });
  }

  // Map useRef variables to ref_N keys
  if (component.useRef) {
    component.useRef.forEach((ref, index) => {
      stateKeyMap.set(ref.name, `ref_${index}`);
    });
  }

  // Map props (they use the prop name as-is)
  if (component.props) {
    component.props.forEach((prop) => {
      stateKeyMap.set(prop.name, prop.name);
    });
  }

  // Start traversal
  traverseJSX(renderBody, null, conditionalTemplates, stateKeyMap);

  return conditionalTemplates;
}

module.exports = {
  extractConditionalElementTemplates
};
