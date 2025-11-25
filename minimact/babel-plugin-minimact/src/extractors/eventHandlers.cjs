/**
 * Event Handlers Extractor
 */

const t = require('@babel/types');
const generate = require('@babel/generator').default;

/**
 * Detect if handler body is client-only (DOM manipulation, no server state changes)
 * Client-only patterns:
 * - e.currentTarget.style.X = value
 * - e.stopPropagation()
 * - e.preventDefault()
 * - element.classList.add/remove/toggle
 * - element.focus/blur/etc
 *
 * Server patterns (NOT client-only):
 * - setState calls
 * - Method calls on component
 * - await expressions
 */
function isClientOnlyHandler(body) {
  let hasClientOnlyCode = false;
  let hasServerCode = false;

  function checkNode(node) {
    if (!node) return;

    // Server patterns
    if (t.isCallExpression(node)) {
      const callee = node.callee;

      // setState, setXxx calls - SERVER
      if (t.isIdentifier(callee) && (callee.name === 'setState' || callee.name.startsWith('set'))) {
        hasServerCode = true;
      }
    }

    // await - SERVER
    if (t.isAwaitExpression(node)) {
      hasServerCode = true;
    }

    // Client-only patterns
    if (t.isMemberExpression(node)) {
      // e.stopPropagation, e.preventDefault
      if (t.isIdentifier(node.property) &&
          (node.property.name === 'stopPropagation' || node.property.name === 'preventDefault')) {
        hasClientOnlyCode = true;
      }

      // e.currentTarget.style.X, e.target.style.X
      if (t.isMemberExpression(node.object) &&
          t.isIdentifier(node.object.property, { name: 'style' })) {
        hasClientOnlyCode = true;
      }

      // element.classList
      if (t.isIdentifier(node.property, { name: 'classList' })) {
        hasClientOnlyCode = true;
      }

      // element.focus, blur, etc
      if (t.isIdentifier(node.property) &&
          ['focus', 'blur', 'scrollIntoView', 'select'].includes(node.property.name)) {
        hasClientOnlyCode = true;
      }
    }

    // Assignment to style properties
    if (t.isAssignmentExpression(node)) {
      const left = node.left;
      if (t.isMemberExpression(left)) {
        // Check if assigning to style property
        if (t.isMemberExpression(left.object) &&
            t.isIdentifier(left.object.property, { name: 'style' })) {
          hasClientOnlyCode = true;
        }
      }
    }

    // Recursively check children
    for (const key in node) {
      if (node[key] && typeof node[key] === 'object') {
        if (Array.isArray(node[key])) {
          node[key].forEach(child => checkNode(child));
        } else {
          checkNode(node[key]);
        }
      }
    }
  }

  checkNode(body);

  // Only client-only if it has client code AND no server code
  return hasClientOnlyCode && !hasServerCode;
}

/**
 * Extract event handler name
 */
function extractEventHandler(value, component) {
  if (t.isStringLiteral(value)) {
    return value.value;
  }

  if (t.isJSXExpressionContainer(value)) {
    const expr = value.expression;

    if (t.isArrowFunctionExpression(expr) || t.isFunctionExpression(expr)) {
      // Inline arrow function - extract to named method
      // Use combined count of both server and client handlers for unique names
      const totalHandlers = component.eventHandlers.length + (component.clientHandlers ? component.clientHandlers.length : 0);
      const handlerName = `Handle${totalHandlers}`;

      // Check if the function is async
      const isAsync = expr.async || false;

      // Detect curried functions (functions that return functions)
      // Pattern: (e) => (id) => action(id)
      // This is invalid for event handlers because the returned function is never called
      if (t.isArrowFunctionExpression(expr.body) || t.isFunctionExpression(expr.body)) {
        // Generate a handler that throws a helpful error
        component.eventHandlers.push({
          name: handlerName,
          body: null, // Will be handled specially in component generator
          params: expr.params,
          capturedParams: [],
          isAsync: false,
          isCurriedError: true // Flag to generate error throw
        });

        return handlerName;
      }

      // Simplify common pattern: (e) => func(e.target.value)
      // Transform to: (value) => func(value)
      let body = expr.body;
      let params = expr.params;

      if (t.isCallExpression(body) && params.length === 1 && t.isIdentifier(params[0])) {
        const eventParam = params[0].name; // e.g., "e"
        const args = body.arguments;

        // Check if any argument is e.target.value
        const transformedArgs = args.map(arg => {
          if (t.isMemberExpression(arg) &&
              t.isMemberExpression(arg.object) &&
              t.isIdentifier(arg.object.object, { name: eventParam }) &&
              t.isIdentifier(arg.object.property, { name: 'target' }) &&
              t.isIdentifier(arg.property, { name: 'value' })) {
            // Replace e.target.value with direct value parameter
            return t.identifier('value');
          }
          return arg;
        });

        // If we transformed any args, update the body and param name
        if (transformedArgs.some((arg, i) => arg !== args[i])) {
          body = t.callExpression(body.callee, transformedArgs);
          params = [t.identifier('value')];
        }
      }

      // Check if we're inside a .map() context and capture those variables
      const capturedParams = component.currentMapContext ? component.currentMapContext.params : [];

      // Handle parameter destructuring
      // Convert ({ target: { value } }) => ... into (e) => ... with unpacking in body
      const hasDestructuring = params.some(p => t.isObjectPattern(p));
      let processedBody = body;
      let processedParams = params;

      if (hasDestructuring && params.length === 1 && t.isObjectPattern(params[0])) {
        // Extract destructured properties
        const destructuringStatements = [];
        const eventParam = t.identifier('e');

        function extractDestructured(pattern, path = []) {
          if (t.isObjectPattern(pattern)) {
            for (const prop of pattern.properties) {
              if (t.isObjectProperty(prop)) {
                const key = t.isIdentifier(prop.key) ? prop.key.name : null;
                if (key && t.isIdentifier(prop.value)) {
                  // Simple: { value } or { target: { value } }
                  const varName = prop.value.name;
                  const accessPath = [...path, key];
                  destructuringStatements.push({ varName, accessPath });
                } else if (key && t.isObjectPattern(prop.value)) {
                  // Nested: { target: { value } }
                  extractDestructured(prop.value, [...path, key]);
                }
              }
            }
          }
        }

        extractDestructured(params[0]);
        processedParams = [eventParam];

        // Prepend destructuring assignments to body
        if (destructuringStatements.length > 0) {
          const assignments = destructuringStatements.map(({ varName, accessPath }) => {
            // Build e.Target.Value access chain
            let access = eventParam;
            for (const key of accessPath) {
              const capitalizedKey = key.charAt(0).toUpperCase() + key.slice(1);
              access = t.memberExpression(access, t.identifier(capitalizedKey));
            }
            return t.variableDeclaration('var', [
              t.variableDeclarator(t.identifier(varName), access)
            ]);
          });

          // Wrap body in block statement with destructuring
          if (t.isBlockStatement(body)) {
            processedBody = t.blockStatement([...assignments, ...body.body]);
          } else {
            processedBody = t.blockStatement([...assignments, t.expressionStatement(body)]);
          }
        }
      }

      // Check if this is a client-only handler
      const isClientOnly = isClientOnlyHandler(processedBody);

      // 🔥 NEW: Import helper functions from hooks.cjs
      const { analyzeHookUsage, transformHandlerFunction } = require('./hooks.cjs');

      // 🔥 NEW: ALWAYS generate client-side handler
      const hookCalls = analyzeHookUsage(t.arrowFunctionExpression(processedParams, processedBody));

      // Transform to regular function with hook mappings
      const transformedFunction = transformHandlerFunction(processedBody, processedParams, hookCalls);
      const jsCode = generate(transformedFunction).code;

      // Add to clientHandlers collection
      if (!component.clientHandlers) {
        component.clientHandlers = [];
      }
      component.clientHandlers.push({
        name: handlerName,
        jsCode: jsCode,
        hookCalls: hookCalls
      });

      // 🔥 ALSO add to eventHandlers if it modifies state (not client-only)
      if (!isClientOnly) {
        // Server handler - add to eventHandlers collection
        component.eventHandlers.push({
          name: handlerName,
          body: processedBody,
          params: processedParams,
          capturedParams: capturedParams,  // e.g., ['item', 'index']
          isAsync: isAsync  // Track if handler is async
        });
      }

      // Return handler registration string
      // If there are captured params, append them as colon-separated interpolations
      // Format: "Handle0:{item}:{index}" - matches client's existing "Method:arg1:arg2" parser
      if (capturedParams.length > 0) {
        const capturedRefs = capturedParams.map(p => `{${p}}`).join(':');
        return `${handlerName}:${capturedRefs}`;
      }

      return handlerName;
    }

    if (t.isIdentifier(expr)) {
      return expr.name;
    }

    if (t.isCallExpression(expr)) {
      // () => someMethod() - extract
      const handlerName = `Handle${component.eventHandlers.length}`;

      // Check if we're inside a .map() context and capture those variables
      const capturedParams = component.currentMapContext ? component.currentMapContext.params : [];

      component.eventHandlers.push({
        name: handlerName,
        body: expr,
        capturedParams: capturedParams  // e.g., ['item', 'index']
      });

      // Return handler registration string
      // If there are captured params, append them as colon-separated interpolations
      // Format: "Handle0:{item}:{index}" - matches client's existing "Method:arg1:arg2" parser
      if (capturedParams.length > 0) {
        const capturedRefs = capturedParams.map(p => `{${p}}`).join(':');
        return `${handlerName}:${capturedRefs}`;
      }

      return handlerName;
    }
  }

  return 'UnknownHandler';
}



module.exports = {
  extractEventHandler
};
