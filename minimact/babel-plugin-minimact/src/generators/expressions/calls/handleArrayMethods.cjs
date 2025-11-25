const t = require('@babel/types');
const { generateJSXElement } = require('../../jsx.cjs');

/**
 * Handle .map() → .Select()
 */
function handleMap(node, generateCSharpExpression, generateCSharpStatement, currentComponent) {
  if (!t.isMemberExpression(node.callee) ||
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

      // Cast to IEnumerable<dynamic> if we detect dynamic access
      // Check for optional chaining or property access (likely dynamic)
      const needsCast = object.includes('?.') || object.includes('?') || object.includes('.');
      const castedObject = needsCast ? `((IEnumerable<dynamic>)${object})` : object;

      // If the object needs casting (is dynamic), we also need to cast the lambda
      // to prevent CS1977: "Cannot use a lambda expression as an argument to a dynamically dispatched operation"
      const lambdaExpr = `${params} => ${body}`;
      const castedLambda = needsCast ? `(Func<dynamic, dynamic>)(${lambdaExpr})` : lambdaExpr;

      return `${castedObject}.Select(${castedLambda}).ToList()`;
    }
  }
  return null;
}

/**
 * Handle useState/useClientState setters → SetState calls
 */
function handleStateSetters(node, generateCSharpExpression, currentComponent) {
  if (!t.isIdentifier(node.callee) || !currentComponent) {
    return null;
  }

  const setterName = node.callee.name;

  // Check if this is a useState setter
  const useState = [...(currentComponent.useState || []), ...(currentComponent.useClientState || [])]
    .find(state => state.setter === setterName);

  if (useState && node.arguments.length > 0) {
    const newValue = generateCSharpExpression(node.arguments[0]);
    return `SetState(nameof(${useState.name}), ${newValue})`;
  }

  return null;
}

module.exports = {
  handleMap,
  handleStateSetters
};
