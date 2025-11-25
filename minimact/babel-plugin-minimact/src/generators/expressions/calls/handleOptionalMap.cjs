const t = require('@babel/types');
const { generateJSXElement } = require('../../jsx.cjs');

/**
 * Handle optional .map() call: array?.map(...)
 */
function handleOptionalMap(node, generateCSharpExpression, generateCSharpStatement, currentComponent) {
  if (!t.isOptionalMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'map' })) {
    return null;
  }

  const object = generateCSharpExpression(node.callee.object);
  if (node.arguments.length > 0) {
    const callback = node.arguments[0];
    if (t.isArrowFunctionExpression(callback)) {
      const paramNames = callback.params.map(p => p.name);
      // C# requires parentheses for 0 or 2+ parameters
      const params = paramNames.length === 1
        ? paramNames[0]
        : `(${paramNames.join(', ')})`;

      // Handle JSX in arrow function body
      let body;
      if (t.isBlockStatement(callback.body)) {
        body = `{ ${callback.body.body.map(stmt => generateCSharpStatement(stmt)).join(' ')} }`;
      } else if (t.isJSXElement(callback.body) || t.isJSXFragment(callback.body)) {
        // JSX element - use generateJSXElement with currentComponent context
        // Store map context for event handler closure capture
        // For nested maps, we need to ACCUMULATE params, not replace them
        const previousMapContext = currentComponent ? currentComponent.currentMapContext : null;
        const previousParams = previousMapContext ? previousMapContext.params : [];
        if (currentComponent) {
          // Combine previous params with current params for nested map support
          currentComponent.currentMapContext = { params: [...previousParams, ...paramNames] };
        }
        body = generateJSXElement(callback.body, currentComponent, 0);
        // Restore previous context
        if (currentComponent) {
          currentComponent.currentMapContext = previousMapContext;
        }
      } else {
        body = generateCSharpExpression(callback.body);
      }

      // Cast to IEnumerable<dynamic> for optional chaining (likely dynamic)
      const castedObject = `((IEnumerable<dynamic>)${object})`;

      // Cast result to List<dynamic> for ?? operator compatibility
      // Anonymous types from Select need explicit Cast<dynamic>() before ToList()
      return `${castedObject}?.Select(${params} => ${body})?.Cast<dynamic>().ToList()`;
    }
  }
  return null;
}

module.exports = { handleOptionalMap };
