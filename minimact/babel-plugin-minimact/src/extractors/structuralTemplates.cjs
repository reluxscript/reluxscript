/**
 * Structural Template Extractor (Phase 5)
 *
 * Extracts templates for conditional rendering patterns where the DOM structure changes.
 * This handles cases like loading states, authentication states, error boundaries, etc.
 *
 * Examples:
 * - {isLoading ? <Spinner /> : <Content />}
 * - {user ? <Dashboard /> : <LoginForm />}
 * - {error && <ErrorMessage />}
 *
 * Architecture:
 * - Build time: Detect conditional patterns and extract both branches
 * - Runtime: Store structural templates with condition binding
 * - Prediction: Choose correct branch based on current state
 */

const { traverseJSX } = require('./structuralTemplates/traverseJSX.cjs');

/**
 * Extract structural templates from JSX render body
 *
 * Returns array of structural template metadata:
 * [
 *   {
 *     type: 'conditional',
 *     stateKey: 'isLoggedIn',
 *     conditionBinding: 'isLoggedIn',
 *     branches: {
 *       'true': { type: 'Element', tag: 'div', ... },
 *       'false': { type: 'Element', tag: 'div', ... }
 *     }
 *   }
 * ]
 */
function extractStructuralTemplates(renderBody, component) {
  if (!renderBody) return [];

  const structuralTemplates = [];

  // Start traversal
  traverseJSX(renderBody, [], structuralTemplates, component);

  return structuralTemplates;
}

module.exports = {
  extractStructuralTemplates
};
