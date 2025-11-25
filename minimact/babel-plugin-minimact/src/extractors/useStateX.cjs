/**
 * useStateX Extractor
 * Extracts useStateX hook calls and analyzes transform functions for C# generation
 */

const t = require('@babel/types');
const { generateCSharpExpression } = require('../generators/expressions.cjs');
const { inferType } = require('../types/typeConversion.cjs');

/**
 * Extract useStateX hook and analyze projections
 *
 * @example
 * const [price, setPrice] = useStateX(99, {
 *   targets: {
 *     '.price-display': {
 *       transform: v => `$${v.toFixed(2)}`,
 *       applyIf: ctx => ctx.user.canSeePrice
 *     }
 *   }
 * });
 */
function extractUseStateX(path, component) {
  const node = path.node;

  // Get the variable declarator (const [price, setPrice] = ...)
  const parent = path.parentPath.node;
  if (!t.isVariableDeclarator(parent) || !t.isArrayPattern(parent.id)) {
    console.warn('[useStateX] Expected array pattern destructuring');
    return;
  }

  const [valueBinding, setterBinding] = parent.id.elements;
  if (!t.isIdentifier(valueBinding)) {
    console.warn('[useStateX] Expected identifier for value binding');
    return;
  }

  const varName = valueBinding.name;
  const setterName = setterBinding ? setterBinding.name : `set${varName[0].toUpperCase()}${varName.slice(1)}`;

  // Get initial value and config
  const [initialValueArg, configArg] = node.arguments;

  if (!configArg || !t.isObjectExpression(configArg)) {
    console.warn('[useStateX] Expected config object as second argument');
    return;
  }

  // Extract initial value
  let initialValue = null;
  let initialValueType = 'dynamic';

  if (initialValueArg) {
    if (t.isLiteral(initialValueArg)) {
      initialValue = initialValueArg.value;
      initialValueType = inferType(initialValueArg);
    } else {
      initialValue = generateCSharpExpression(initialValueArg);
      initialValueType = 'dynamic';
    }
  }

  // Extract target projections
  const targets = extractTargets(configArg);

  // Extract sync strategy
  const sync = extractSyncStrategy(configArg);

  // Store useStateX metadata
  component.useStateX = component.useStateX || [];
  component.useStateX.push({
    varName,
    setterName,
    initialValue,
    initialValueType,
    targets,
    sync
  });

  // Track state type
  component.stateTypes = component.stateTypes || new Map();
  component.stateTypes.set(varName, 'useStateX');
}

/**
 * Extract target projection configurations
 */
function extractTargets(configObject) {
  const targets = [];

  // Find targets property
  const targetsProp = configObject.properties.find(
    p => t.isIdentifier(p.key) && p.key.name === 'targets'
  );

  if (!targetsProp || !t.isObjectExpression(targetsProp.value)) {
    return targets;
  }

  // Process each target selector
  targetsProp.value.properties.forEach(target => {
    const selector = target.key.value || target.key.name;
    const targetConfig = target.value;

    if (!t.isObjectExpression(targetConfig)) {
      return;
    }

    const projection = {
      selector,
      transform: null,
      transformId: null,
      transformType: 'none',
      applyIf: null,
      applyAs: 'textContent',
      property: null,
      template: null
    };

    // Extract each property
    targetConfig.properties.forEach(prop => {
      const propName = prop.key.name;
      const propValue = prop.value;

      switch (propName) {
        case 'transform':
          if (t.isArrowFunctionExpression(propValue) || t.isFunctionExpression(propValue)) {
            // Analyze transform function
            const transformAnalysis = analyzeTransformFunction(propValue);
            projection.transform = transformAnalysis.csharpCode;
            projection.transformType = transformAnalysis.type;
          }
          break;

        case 'transformId':
          if (t.isStringLiteral(propValue)) {
            projection.transformId = propValue.value;
            projection.transformType = 'registry';
          }
          break;

        case 'applyIf':
          if (t.isArrowFunctionExpression(propValue) || t.isFunctionExpression(propValue)) {
            // Analyze applyIf condition
            projection.applyIf = analyzeApplyIfCondition(propValue);
          }
          break;

        case 'applyAs':
          if (t.isStringLiteral(propValue)) {
            projection.applyAs = propValue.value;
          }
          break;

        case 'property':
          if (t.isStringLiteral(propValue)) {
            projection.property = propValue.value;
          }
          break;

        case 'template':
          if (t.isStringLiteral(propValue)) {
            projection.template = propValue.value;
          }
          break;
      }
    });

    targets.push(projection);
  });

  return targets;
}

/**
 * Analyze transform function and generate C# equivalent
 *
 * Supports:
 * - Template literals with simple expressions
 * - Method calls (toFixed, toUpperCase, etc.)
 * - Ternary expressions
 * - Property access
 */
function analyzeTransformFunction(arrowFn) {
  const param = arrowFn.params[0]; // 'v'
  const paramName = param ? param.name : 'v';
  const body = arrowFn.body;

  // Template literal: `$${v.toFixed(2)}`
  if (t.isTemplateLiteral(body)) {
    return {
      type: 'template',
      csharpCode: generateCSharpFromTemplate(body, paramName)
    };
  }

  // Ternary: v > 10 ? 'High' : 'Low'
  if (t.isConditionalExpression(body)) {
    return {
      type: 'ternary',
      csharpCode: generateCSharpFromTernary(body, paramName)
    };
  }

  // Method call: v.toUpperCase()
  if (t.isCallExpression(body)) {
    return {
      type: 'method-call',
      csharpCode: generateCSharpFromMethodCall(body, paramName)
    };
  }

  // Member expression: v.firstName
  if (t.isMemberExpression(body)) {
    return {
      type: 'property-access',
      csharpCode: generateCSharpFromMemberExpression(body, paramName)
    };
  }

  // Fallback: complex
  return {
    type: 'complex',
    csharpCode: null
  };
}

/**
 * Generate C# code from template literal
 * Example: `$${v.toFixed(2)}` → $"${v.ToString("F2")}"
 */
function generateCSharpFromTemplate(templateLiteral, paramName) {
  let csharpCode = '$"';

  for (let i = 0; i < templateLiteral.quasis.length; i++) {
    const quasi = templateLiteral.quasis[i];
    csharpCode += quasi.value.raw;

    if (i < templateLiteral.expressions.length) {
      const expr = templateLiteral.expressions[i];
      csharpCode += '{' + generateCSharpFromExpression(expr, paramName) + '}';
    }
  }

  csharpCode += '"';
  return csharpCode;
}

/**
 * Generate C# code from ternary expression
 * Example: v > 10 ? 'High' : 'Low' → v > 10 ? "High" : "Low"
 */
function generateCSharpFromTernary(ternary, paramName) {
  const test = generateCSharpFromExpression(ternary.test, paramName);
  const consequent = generateCSharpFromExpression(ternary.consequent, paramName);
  const alternate = generateCSharpFromExpression(ternary.alternate, paramName);

  return `${test} ? ${consequent} : ${alternate}`;
}

/**
 * Generate C# code from method call
 * Example: v.toFixed(2) → v.ToString("F2")
 */
function generateCSharpFromMethodCall(callExpr, paramName) {
  if (t.isMemberExpression(callExpr.callee)) {
    const object = generateCSharpFromExpression(callExpr.callee.object, paramName);
    const method = callExpr.callee.property.name;
    const args = callExpr.arguments;

    // Map JS methods to C# equivalents
    const methodMap = {
      'toFixed': (args) => {
        const decimals = args[0] && t.isNumericLiteral(args[0]) ? args[0].value : 2;
        return `ToString("F${decimals}")`;
      },
      'toUpperCase': () => 'ToUpper()',
      'toLowerCase': () => 'ToLower()',
      'toString': () => 'ToString()',
      'trim': () => 'Trim()',
      'length': () => 'Length'
    };

    const csharpMethod = methodMap[method] ? methodMap[method](args) : `${method}()`;
    return `${object}.${csharpMethod}`;
  }

  return 'null';
}

/**
 * Generate C# code from member expression
 * Example: v.firstName → v.FirstName
 */
function generateCSharpFromMemberExpression(memberExpr, paramName) {
  const object = generateCSharpFromExpression(memberExpr.object, paramName);
  const property = memberExpr.property.name;

  // Pascal case the property name for C#
  const csharpProperty = property.charAt(0).toUpperCase() + property.slice(1);

  return `${object}.${csharpProperty}`;
}

/**
 * Generate C# code from any expression
 */
function generateCSharpFromExpression(expr, paramName) {
  if (t.isIdentifier(expr)) {
    return expr.name === paramName || expr.name === 'v' ? 'v' : expr.name;
  }

  if (t.isStringLiteral(expr)) {
    return `"${expr.value}"`;
  }

  if (t.isNumericLiteral(expr)) {
    return expr.value.toString();
  }

  if (t.isBooleanLiteral(expr)) {
    return expr.value ? 'true' : 'false';
  }

  if (t.isMemberExpression(expr)) {
    return generateCSharpFromMemberExpression(expr, paramName);
  }

  if (t.isCallExpression(expr)) {
    return generateCSharpFromMethodCall(expr, paramName);
  }

  if (t.isBinaryExpression(expr)) {
    const left = generateCSharpFromExpression(expr.left, paramName);
    const right = generateCSharpFromExpression(expr.right, paramName);
    const operator = expr.operator;
    return `${left} ${operator} ${right}`;
  }

  return 'null';
}

/**
 * Analyze applyIf condition
 * Example: ctx => ctx.user.isAdmin → "ctx => ctx.User.IsAdmin"
 */
function analyzeApplyIfCondition(arrowFn) {
  const param = arrowFn.params[0]; // 'ctx'
  const paramName = param ? param.name : 'ctx';
  const body = arrowFn.body;

  const csharpCondition = generateCSharpFromExpression(body, paramName);

  return {
    csharpCode: `${paramName} => ${csharpCondition}`,
    type: 'arrow'
  };
}

/**
 * Extract sync strategy
 */
function extractSyncStrategy(configObject) {
  const syncProp = configObject.properties.find(
    p => t.isIdentifier(p.key) && p.key.name === 'sync'
  );

  if (!syncProp || !t.isStringLiteral(syncProp.value)) {
    return 'immediate';
  }

  return syncProp.value.value;
}

module.exports = {
  extractUseStateX
};
