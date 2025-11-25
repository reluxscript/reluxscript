/**
 * Identifier and member expression handlers
 */

const t = require('@babel/types');

/**
 * Generate identifier expression
 */
function generateIdentifier(node, currentComponent) {
  // Special case: 'state' identifier (state proxy)
  // Note: This should only happen as part of member expression (state.key or state["key"])
  // Standalone 'state' reference is unusual - warn but transpile to 'State'
  if (node.name === 'state') {
    console.warn('[Babel Plugin] Naked state reference detected (should be state.key or state["key"])');
    return 'State';
  }

  // Check if this identifier is a custom hook return value
  if (currentComponent && currentComponent.customHooks) {
    for (const hookInstance of currentComponent.customHooks) {
      if (!hookInstance.metadata || !hookInstance.metadata.returnValues) continue;

      // Check if this identifier matches any non-UI return value
      const returnValueIndex = hookInstance.returnValues.indexOf(node.name);
      if (returnValueIndex !== -1) {
        const returnValueMetadata = hookInstance.metadata.returnValues[returnValueIndex];

        // Skip UI return values (they're handled separately as VComponentWrapper)
        if (returnValueMetadata.type === 'jsx') {
          return node.name; // Keep as-is for UI variables
        }

        // Replace with lifted state access for state/method returns
        if (returnValueMetadata.type === 'state') {
          // Access the hook's lifted state: State["hookNamespace.stateVarName"]
          const liftedStatePath = `${hookInstance.namespace}.${returnValueMetadata.name}`;
          return `GetState<dynamic>("${liftedStatePath}")`;
        } else if (returnValueMetadata.type === 'method') {
          // For methods, we can't call them from parent - warn and keep as-is for now
          // In the future, we could emit a method that invokes the child component's method
          console.warn(`[Custom Hook] Cannot access method '${node.name}' from hook '${hookInstance.hookName}' in parent component`);
          return node.name;
        }
      }
    }
  }

  return node.name;
}

/**
 * Generate member expression
 */
function generateMemberExpression(node, generateCSharpExpression, inInterpolation) {
  // Special case: state.key or state["key"] (state proxy)
  if (t.isIdentifier(node.object, { name: 'state' })) {
    if (node.computed) {
      // state["someKey"] or state["Child.key"] → State["someKey"] or State["Child.key"]
      const key = generateCSharpExpression(node.property, inInterpolation);
      return `State[${key}]`;
    } else {
      // state.someKey → State["someKey"]
      const key = node.property.name;
      return `State["${key}"]`;
    }
  }

  const object = generateCSharpExpression(node.object);
  const propertyName = t.isIdentifier(node.property) ? node.property.name : null;

  // Handle ref.current → just ref (refs in C# are the value itself, not a container)
  if (propertyName === 'current' && !node.computed && t.isIdentifier(node.object)) {
    // Check if the object is a ref variable (ends with "Ref")
    if (node.object.name.endsWith('Ref')) {
      return object;  // Return just the ref variable name without .current
    }
  }

  // Handle JavaScript to C# API conversions
  if (propertyName === 'length' && !node.computed) {
    // array.length → array.Count
    return `${object}.Count`;
  }

  // Handle event object property access (e.target.value → e.Target.Value)
  if (propertyName === 'target' && !node.computed) {
    return `${object}.Target`;
  }
  if (propertyName === 'value' && !node.computed) {
    // Capitalize for C# property convention
    return `${object}.Value`;
  }
  if (propertyName === 'checked' && !node.computed) {
    // Capitalize for C# property convention
    return `${object}.Checked`;
  }

  // Handle exception properties (err.message → err.Message)
  if (propertyName === 'message' && !node.computed) {
    return `${object}.Message`;
  }

  // Handle fetch Response properties (response.ok → response.IsSuccessStatusCode)
  if (propertyName === 'ok' && !node.computed) {
    return `${object}.IsSuccessStatusCode`;
  }

  const property = node.computed
    ? `[${generateCSharpExpression(node.property)}]`
    : `.${propertyName}`;
  return `${object}${property}`;
}

/**
 * Generate optional member expression
 */
function generateOptionalMemberExpression(node, generateCSharpExpression, inInterpolation) {
  const object = generateCSharpExpression(node.object, inInterpolation);
  const propertyName = t.isIdentifier(node.property) ? node.property.name : null;

  // Capitalize first letter for C# property convention (userEmail → UserEmail)
  const csharpProperty = propertyName
    ? propertyName.charAt(0).toUpperCase() + propertyName.slice(1)
    : propertyName;

  const property = node.computed
    ? `?[${generateCSharpExpression(node.property, inInterpolation)}]`
    : `?.${csharpProperty}`;
  return `${object}${property}`;
}

module.exports = {
  generateIdentifier,
  generateMemberExpression,
  generateOptionalMemberExpression
};
