/**
 * Prop Type Inference
 * Infers C# types for props based on how they're used in the component
 */

const t = require('@babel/types');

/**
 * Infer prop types from usage in the component body
 */
function inferPropTypes(component, body) {
  const propUsage = {};

  // Initialize tracking for each prop
  for (const prop of component.props) {
    propUsage[prop.name] = {
      usedAsBoolean: false,
      usedAsNumber: false,
      usedAsString: false,
      usedAsArray: false,
      usedAsObject: false,
      hasArrayMethods: false,
      hasNumberOperations: false
    };
  }

  // Traverse the body to analyze prop usage
  function analyzePropUsage(node) {
    if (!node) return;

    // Handle BlockStatement (function body)
    if (t.isBlockStatement(node)) {
      for (const statement of node.body) {
        analyzePropUsage(statement);
      }
      return;
    }

    // Handle VariableDeclaration
    if (t.isVariableDeclaration(node)) {
      for (const declarator of node.declarations) {
        if (declarator.init) {
          analyzePropUsage(declarator.init);
        }
      }
      return;
    }

    // Handle ReturnStatement
    if (t.isReturnStatement(node)) {
      analyzePropUsage(node.argument);
      return;
    }

    // Handle ExpressionStatement
    if (t.isExpressionStatement(node)) {
      analyzePropUsage(node.expression);
      return;
    }

    // Check if prop is used in conditional context (implies boolean)
    if (t.isConditionalExpression(node)) {
      const testName = extractPropName(node.test);
      if (testName && propUsage[testName]) {
        propUsage[testName].usedAsBoolean = true;
      }
      analyzePropUsage(node.consequent);
      analyzePropUsage(node.alternate);
    }

    // Check if prop is used in logical expression (implies boolean)
    if (t.isLogicalExpression(node)) {
      const leftName = extractPropName(node.left);
      if (leftName && propUsage[leftName]) {
        propUsage[leftName].usedAsBoolean = true;
      }
      analyzePropUsage(node.right);
    }

    // Check if prop is used with .map(), .filter(), etc (implies array)
    if (t.isCallExpression(node) && t.isMemberExpression(node.callee)) {
      const objectName = extractPropName(node.callee.object);
      const methodName = t.isIdentifier(node.callee.property) ? node.callee.property.name : null;

      if (objectName && propUsage[objectName]) {
        if (methodName === 'map' || methodName === 'filter' || methodName === 'forEach' ||
            methodName === 'find' || methodName === 'some' || methodName === 'every' ||
            methodName === 'reduce' || methodName === 'sort' || methodName === 'slice') {
          propUsage[objectName].usedAsArray = true;
          propUsage[objectName].hasArrayMethods = true;
        }
      }

      // Recurse into arguments
      for (const arg of node.arguments) {
        analyzePropUsage(arg);
      }
    }

    // Check if prop is used in arithmetic operations (implies number)
    if (t.isBinaryExpression(node)) {
      if (['+', '-', '*', '/', '%', '>', '<', '>=', '<='].includes(node.operator)) {
        const leftName = extractPropName(node.left);
        const rightName = extractPropName(node.right);

        if (leftName && propUsage[leftName]) {
          propUsage[leftName].usedAsNumber = true;
          propUsage[leftName].hasNumberOperations = true;
        }
        if (rightName && propUsage[rightName]) {
          propUsage[rightName].usedAsNumber = true;
          propUsage[rightName].hasNumberOperations = true;
        }
      }

      analyzePropUsage(node.left);
      analyzePropUsage(node.right);
    }

    // Check member access for .length (could be array or string)
    if (t.isMemberExpression(node)) {
      const objectName = extractPropName(node.object);
      const propertyName = t.isIdentifier(node.property) ? node.property.name : null;

      if (objectName && propUsage[objectName]) {
        if (propertyName === 'length') {
          // Could be array or string, mark both
          propUsage[objectName].usedAsArray = true;
          propUsage[objectName].usedAsString = true;
        } else if (propertyName) {
          // Accessing a property implies object
          propUsage[objectName].usedAsObject = true;
        }
      }

      analyzePropUsage(node.object);
      if (node.computed) {
        analyzePropUsage(node.property);
      }
    }

    // Recurse into JSX elements
    if (t.isJSXElement(node)) {
      for (const child of node.children) {
        analyzePropUsage(child);
      }
      for (const attr of node.openingElement.attributes) {
        if (t.isJSXAttribute(attr) && t.isJSXExpressionContainer(attr.value)) {
          analyzePropUsage(attr.value.expression);
        }
      }
    }

    if (t.isJSXExpressionContainer(node)) {
      analyzePropUsage(node.expression);
    }

    // Recurse into arrow functions
    if (t.isArrowFunctionExpression(node)) {
      analyzePropUsage(node.body);
    }

    // Recurse into arrays
    if (Array.isArray(node)) {
      for (const item of node) {
        analyzePropUsage(item);
      }
    }
  }

  analyzePropUsage(body);

  // Now infer types based on usage patterns
  for (const prop of component.props) {
    if (prop.type !== 'dynamic') {
      // Already has explicit type from TypeScript, don't override
      continue;
    }

    const usage = propUsage[prop.name];

    if (usage.hasArrayMethods) {
      // Definitely an array if array methods are called
      prop.type = 'List<dynamic>';
    } else if (usage.usedAsArray && !usage.hasNumberOperations) {
      // Used as array (e.g., .length on array)
      prop.type = 'List<dynamic>';
    } else if (usage.usedAsBoolean && !usage.usedAsNumber && !usage.usedAsString && !usage.usedAsObject && !usage.usedAsArray) {
      // Used only as boolean
      prop.type = 'bool';
    } else if (usage.hasNumberOperations && !usage.usedAsBoolean && !usage.usedAsArray) {
      // Used in arithmetic operations
      prop.type = 'double';
    } else if (usage.usedAsObject && !usage.usedAsArray && !usage.usedAsBoolean) {
      // Used as object with property access
      prop.type = 'dynamic';
    } else {
      // Keep as dynamic for complex cases
      prop.type = 'dynamic';
    }
  }
}

/**
 * Extract prop name from an expression
 */
function extractPropName(node) {
  if (t.isIdentifier(node)) {
    return node.name;
  }
  if (t.isMemberExpression(node)) {
    return extractPropName(node.object);
  }
  return null;
}

module.exports = {
  inferPropTypes
};
