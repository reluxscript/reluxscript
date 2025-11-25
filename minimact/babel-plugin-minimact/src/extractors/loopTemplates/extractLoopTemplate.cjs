const t = require('@babel/types');
const { extractArrayBinding } = require('./extractArrayBinding.cjs');
const { extractJSXFromCallback } = require('./extractJSXFromCallback.cjs');
const { extractElementTemplate } = require('./extractElementTemplate.cjs');
const { extractKeyBinding } = require('./extractKeyBinding.cjs');

/**
 * Extract loop template from .map() call expression
 *
 * Example:
 * todos.map((todo, index) => <li key={todo.id}>{todo.text}</li>)
 */
function extractLoopTemplate(mapCallExpr) {
  // Get array binding (the object being mapped)
  const arrayBinding = extractArrayBinding(mapCallExpr.callee.object);
  if (!arrayBinding) {
    console.warn('[Loop Template] Could not extract array binding from .map()');
    return null;
  }

  // Get callback function (arrow function or function expression)
  const callback = mapCallExpr.arguments[0];
  if (!t.isArrowFunctionExpression(callback) && !t.isFunctionExpression(callback)) {
    console.warn('[Loop Template] .map() callback is not a function');
    return null;
  }

  // Get item and index parameter names
  const itemVar = callback.params[0] ? callback.params[0].name : 'item';
  const indexVar = callback.params[1] ? callback.params[1].name : null;

  // Get JSX element returned by callback
  const jsxElement = extractJSXFromCallback(callback);
  if (!jsxElement) {
    console.warn('[Loop Template] .map() callback does not return JSX element');
    return null;
  }

  // Extract item template from JSX element
  const itemTemplate = extractElementTemplate(jsxElement, itemVar, indexVar);
  if (!itemTemplate) {
    console.warn('[Loop Template] Could not extract item template from JSX');
    return null;
  }

  // Extract key binding
  const keyBinding = extractKeyBinding(jsxElement, itemVar, indexVar);

  return {
    stateKey: arrayBinding,  // For C# attribute: which state variable triggers this template
    arrayBinding,            // Runtime: which array to iterate
    itemVar,                 // Runtime: variable name for each item
    indexVar,                // Runtime: variable name for index (optional)
    keyBinding,              // Runtime: expression for React key (optional)
    itemTemplate             // Runtime: template for each list item
  };
}

module.exports = { extractLoopTemplate };
