/**
 * Loop Template Extractor
 *
 * Extracts parameterized loop templates from .map() expressions for predictive rendering.
 * This enables 100% coverage for list rendering patterns with O(1) memory.
 *
 * Architecture:
 * - Build time: Detect .map() patterns and extract item templates
 * - Runtime (Rust predictor): Use Babel-generated templates as primary source
 * - Fallback: Rust runtime extraction if Babel can't generate template
 *
 * Example:
 * {todos.map(todo => <li>{todo.text}</li>)}
 * →
 * LoopTemplate {
 *   arrayBinding: "todos",
 *   itemVar: "todo",
 *   itemTemplate: ElementTemplate {
 *     tag: "li",
 *     children: [TextTemplate { template: "{0}", bindings: ["item.text"] }]
 *   }
 * }
 */

const { traverseJSX } = require('./loopTemplates/traverseJSX.cjs');

/**
 * Extract all loop templates from JSX render body
 *
 * Returns array of loop template metadata:
 * [
 *   {
 *     stateKey: "todos",
 *     arrayBinding: "todos",
 *     itemVar: "todo",
 *     indexVar: "index",
 *     keyBinding: "item.id",
 *     itemTemplate: { ... }
 *   }
 * ]
 */
function extractLoopTemplates(renderBody, component) {
  if (!renderBody) return [];

  const loopTemplates = [];

  // Start traversal
  traverseJSX(renderBody, loopTemplates);

  return loopTemplates;
}

module.exports = {
  extractLoopTemplates
};
