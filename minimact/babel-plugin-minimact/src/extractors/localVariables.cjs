/**
 * Local Variables Extractor
 */

const t = require('@babel/types');
const { generateCSharpExpression } = require('../generators/expressions.cjs');
const { tsTypeToCSharpType } = require('../types/typeConversion.cjs');

/**
 * Check if an expression uses external libraries
 */
function usesExternalLibrary(node, externalImports, visited = new WeakSet()) {
  if (!node || visited.has(node)) return false;
  visited.add(node);

  // Direct identifier match
  if (t.isIdentifier(node) && externalImports.has(node.name)) {
    return true;
  }

  // Member expression (_.sortBy, moment().format)
  if (t.isMemberExpression(node)) {
    return usesExternalLibrary(node.object, externalImports, visited);
  }

  // Call expression (_.sortBy(...), moment(...))
  if (t.isCallExpression(node)) {
    return usesExternalLibrary(node.callee, externalImports, visited) ||
           node.arguments.some(arg => usesExternalLibrary(arg, externalImports, visited));
  }

  // Binary/Logical expressions
  if (t.isBinaryExpression(node) || t.isLogicalExpression(node)) {
    return usesExternalLibrary(node.left, externalImports, visited) ||
           usesExternalLibrary(node.right, externalImports, visited);
  }

  // Conditional expression
  if (t.isConditionalExpression(node)) {
    return usesExternalLibrary(node.test, externalImports, visited) ||
           usesExternalLibrary(node.consequent, externalImports, visited) ||
           usesExternalLibrary(node.alternate, externalImports, visited);
  }

  // Array expressions
  if (t.isArrayExpression(node)) {
    return node.elements.some(el => el && usesExternalLibrary(el, externalImports, visited));
  }

  // Object expressions
  if (t.isObjectExpression(node)) {
    return node.properties.some(prop =>
      t.isObjectProperty(prop) && usesExternalLibrary(prop.value, externalImports, visited)
    );
  }

  // Arrow functions and function expressions
  if (t.isArrowFunctionExpression(node) || t.isFunctionExpression(node)) {
    return usesExternalLibrary(node.body, externalImports, visited);
  }

  // Block statement
  if (t.isBlockStatement(node)) {
    return node.body.some(stmt => usesExternalLibrary(stmt, externalImports, visited));
  }

  // Return statement
  if (t.isReturnStatement(node)) {
    return usesExternalLibrary(node.argument, externalImports, visited);
  }

  // Expression statement
  if (t.isExpressionStatement(node)) {
    return usesExternalLibrary(node.expression, externalImports, visited);
  }

  return false;
}

/**
 * Extract local variables (const/let/var) from function body
 */
function extractLocalVariables(path, component, types) {
  const declarations = path.node.declarations;

  for (const declarator of declarations) {
    // Skip if it's a hook call (already handled)
    if (t.isCallExpression(declarator.init)) {
      const callee = declarator.init.callee;
      if (t.isIdentifier(callee) && callee.name.startsWith('use')) {
        continue; // Skip hook calls
      }
    }

    // Check if this is an event handler (arrow function or function expression)
    if (t.isIdentifier(declarator.id) && declarator.init) {
      const varName = declarator.id.name;

      // If it's an arrow function or function expression
      if (t.isArrowFunctionExpression(declarator.init) || t.isFunctionExpression(declarator.init)) {
        // Check if the function body uses external libraries
        const usesExternal = usesExternalLibrary(declarator.init.body, component.externalImports);

        if (usesExternal) {
          // Mark as client-computed function
          component.clientComputedVars.add(varName);

          component.localVariables.push({
            name: varName,
            type: 'dynamic', // Will be refined to Func<> in generator
            initialValue: 'null',
            isClientComputed: true,
            isFunction: true,
            init: declarator.init
          });
        } else {
          // Regular event handler
          component.eventHandlers.push({
            name: varName,
            body: declarator.init.body,
            params: declarator.init.params
          });
        }
        continue;
      }

      // Check if this variable uses external libraries
      const isClientComputed = usesExternalLibrary(declarator.init, component.externalImports);

      if (isClientComputed) {
        // Mark as client-computed
        component.clientComputedVars.add(varName);
      }

      // Otherwise, treat as a regular local variable
      const initValue = generateCSharpExpression(declarator.init);

      // Try to infer type from TypeScript annotation or initial value
      let varType = 'var'; // C# var for type inference
      if (declarator.id.typeAnnotation?.typeAnnotation) {
        varType = tsTypeToCSharpType(declarator.id.typeAnnotation.typeAnnotation);
      }

      component.localVariables.push({
        name: varName,
        type: varType,
        initialValue: initValue,
        isClientComputed: isClientComputed,  // NEW: Flag for client-computed
        init: declarator.init  // NEW: Store AST node for type inference
      });
    }
  }
}

module.exports = {
  extractLocalVariables,
  usesExternalLibrary
};
