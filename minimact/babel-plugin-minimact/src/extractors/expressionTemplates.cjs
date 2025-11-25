/**
 * Expression Template Extractor (Phase 6)
 *
 * Extracts templates for computed values and transformations.
 * This handles cases like number formatting, arithmetic, string operations, etc.
 *
 * Examples:
 * - {price.toFixed(2)}
 * - {count * 2 + 1}
 * - {name.toUpperCase()}
 * - {items.length}
 *
 * Architecture:
 * - Build time: Detect expression patterns and extract transformation metadata
 * - Runtime: Store expression templates with bindings and transforms
 * - Prediction: Apply transforms to current state values
 *
 * Security Note:
 * Only safe, whitelisted transformations are supported. No arbitrary JavaScript execution.
 */

const { traverseJSX } = require('./expressionTemplates/traverseJSX.cjs');
const { SUPPORTED_TRANSFORMS } = require('./expressionTemplates/supportedTransforms.cjs');

/**
 * Extract expression templates from JSX render body
 *
 * Returns array of expression template metadata:
 * [
 *   {
 *     type: 'expression',
 *     template: '${0}',
 *     bindings: ['price'],
 *     transforms: [
 *       { type: 'toFixed', args: [2] }
 *     ]
 *   }
 * ]
 */
function extractExpressionTemplates(renderBody, component) {
  if (!renderBody) return [];

  const expressionTemplates = [];

  // Start traversal
  traverseJSX(renderBody, [], expressionTemplates, component);

  return expressionTemplates;
}

module.exports = {
  extractExpressionTemplates,
  SUPPORTED_TRANSFORMS
};
