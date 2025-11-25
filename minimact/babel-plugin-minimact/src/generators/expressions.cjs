/**
 * Expression Generators
 */

const t = require('@babel/types');
const { escapeCSharpString } = require('../utils/helpers.cjs');
const { analyzeDependencies } = require('../analyzers/dependencies.cjs');
const { classifyNode } = require('../analyzers/classification.cjs');
const { generateRuntimeHelperForJSXNode } = require('./runtimeHelpers.cjs');
const { generateJSXElement } = require('./jsx.cjs');
const { getPathFromNode } = require('../utils/pathAssignment.cjs');

// Import extracted expression handlers
const {
  generateStringLiteral,
  generateNumericLiteral,
  generateBooleanLiteral,
  generateNullLiteral,
  generateTemplateLiteral
} = require('./expressions/literals.cjs');

const {
  generateUnaryExpression,
  generateBinaryExpression,
  generateLogicalExpression,
  generateConditionalExpression,
  generateAssignmentExpression
} = require('./expressions/operators.cjs');

const {
  generateIdentifier,
  generateMemberExpression,
  generateOptionalMemberExpression
} = require('./expressions/identifiers.cjs');

const { generateArrayExpression } = require('./expressions/arrays.cjs');
const { generateObjectExpression } = require('./expressions/objects.cjs');
const { generateFunctionExpression } = require('./expressions/functions.cjs');
const { generateCallExpression, generateOptionalCallExpression } = require('./expressions/calls.cjs');
const { generateNewExpression } = require('./expressions/newExpressions.cjs');

// Module-level variable to store current component context
// This allows useState setter detection without threading component through all calls
let currentComponent = null;

/**
 * Convert C# string truthy checks to proper boolean expressions
 * Wraps non-boolean expressions with MinimactHelpers.ToBool()
 */
function convertStringToBool(csharpTest, originalTestNode) {
  // If it's already a boolean expression, return as-is
  if (t.isBinaryExpression(originalTestNode) ||
      t.isLogicalExpression(originalTestNode) ||
      t.isUnaryExpression(originalTestNode, { operator: '!' })) {
    return csharpTest;
  }

  // For any other expression, wrap with ToBool() helper
  // This will handle strings, numbers, objects, etc. at runtime
  return `MinimactHelpers.ToBool(${csharpTest})`;
}

/**
 * Generate expression for use in boolean context (conditionals, logical operators)
 * Wraps expressions in MObject for JavaScript truthiness semantics
 */
function generateBooleanExpression(expr) {
  const generated = generateCSharpExpression(expr);

  // Check if this is a member expression on dynamic object (like user.isAdmin)
  if (t.isMemberExpression(expr) && !expr.computed && t.isIdentifier(expr.object)) {
    // Wrap dynamic member access in MObject for proper truthiness
    return `new MObject(${generated})`;
  }

  // Check if this is a simple identifier that might be dynamic
  if (t.isIdentifier(expr)) {
    // Wrap in MObject for null/truthiness handling
    return `new MObject(${generated})`;
  }

  // For other expressions (literals, etc.), use as-is
  return generated;
}

/**
 * Generate JSX expression (e.g., {count}, {user.name})
 */
function generateJSXExpression(expr, component, indent) {
  // Analyze dependencies
  const deps = analyzeDependencies(expr, component);
  const zone = classifyNode(deps);

  // For hybrid zones, we need to split
  if (zone === 'hybrid') {
    return generateHybridExpression(expr, component, deps, indent);
  }

  // Add zone attribute if needed
  const zoneAttr = zone === 'client'
    ? 'data-minimact-client-scope'
    : zone === 'server'
      ? 'data-minimact-server-scope'
      : '';

  // Handle special JSX expression types
  if (t.isConditionalExpression(expr)) {
    // Ternary with JSX: condition ? <A/> : <B/>
    // Force runtime helpers for JSX in conditionals
    const condition = generateBooleanExpression(expr.test);
    const consequent = t.isJSXElement(expr.consequent) || t.isJSXFragment(expr.consequent)
      ? generateRuntimeHelperForJSXNode(expr.consequent, component, indent)
      : generateCSharpExpression(expr.consequent, false); // Normal C# expression context

    // Handle alternate - if null literal, use VNull with path
    let alternate;
    if (!expr.alternate || t.isNullLiteral(expr.alternate)) {
      const exprPath = expr.__minimactPath || '';
      alternate = `new VNull("${exprPath}")`;
    } else if (t.isJSXElement(expr.alternate) || t.isJSXFragment(expr.alternate)) {
      alternate = generateRuntimeHelperForJSXNode(expr.alternate, component, indent);
    } else {
      alternate = generateCSharpExpression(expr.alternate, false); // Normal C# expression context
    }

    return `(${condition}) ? ${consequent} : ${alternate}`;
  }

  if (t.isLogicalExpression(expr) && expr.operator === '&&') {
    // Short-circuit with JSX: condition && <Element/>
    // Force runtime helpers for JSX in logical expressions
    const left = generateBooleanExpression(expr.left);
    const right = t.isJSXElement(expr.right) || t.isJSXFragment(expr.right)
      ? generateRuntimeHelperForJSXNode(expr.right, component, indent)
      : generateCSharpExpression(expr.right);
    // Get path for VNull (use the expression container's path)
    const exprPath = expr.__minimactPath || '';
    return `(${left}) ? ${right} : new VNull("${exprPath}")`;
  }

  if (t.isCallExpression(expr) &&
      t.isMemberExpression(expr.callee) &&
      t.isIdentifier(expr.callee.property, { name: 'map' })) {
    // Array.map() with JSX callback
    return generateMapExpression(expr, component, indent);
  }

  // Generate C# expression
  return generateCSharpExpression(expr);
}

/**
 * Generate conditional (ternary)
 */
function generateConditional(node, component, indent) {
  const indentStr = '    '.repeat(indent);
  const condition = generateCSharpExpression(node.test);
  const consequent = generateJSXElement(node.consequent, component, indent);
  const alternate = generateJSXElement(node.alternate, component, indent);

  return `${indentStr}return ${condition}\n${indentStr}    ? ${consequent}\n${indentStr}    : ${alternate};`;
}

/**
 * Generate short-circuit (&&)
 */
function generateShortCircuit(node, component, indent) {
  const indentStr = '    '.repeat(indent);
  const condition = generateCSharpExpression(node.left);
  const element = generateJSXElement(node.right, component, indent);

  return `${indentStr}if (${condition})\n${indentStr}{\n${indentStr}    return ${element};\n${indentStr}}\n${indentStr}return new VText("");`;
}

/**
 * Generate .map() expression
 */
function generateMapExpression(node, component, indent) {
  const indentStr = '    '.repeat(indent);
  const array = node.callee.object;
  const callback = node.arguments[0];

  const arrayName = array.name || generateCSharpExpression(array);
  const itemParam = callback.params[0].name;
  const indexParam = callback.params[1] ? callback.params[1].name : null;
  const body = callback.body;

  // Track map context for event handler closure capture (nested maps)
  const previousMapContext = component ? component.currentMapContext : null;
  const previousParams = previousMapContext ? previousMapContext.params : [];
  const currentParams = indexParam ? [itemParam, indexParam] : [itemParam];
  if (component) {
    component.currentMapContext = { params: [...previousParams, ...currentParams] };
  }

  let itemCode;
  let hasBlockStatements = false;

  if (t.isJSXElement(body)) {
    // Direct JSX return: item => <div>...</div>
    itemCode = generateJSXElement(body, component, indent + 1);
  } else if (t.isBlockStatement(body)) {
    // Block statement: item => { const x = ...; return <div>...</div>; }
    // Need to generate a statement lambda in C#
    hasBlockStatements = true;

    const statements = [];
    let returnJSX = null;

    // Process all statements in the block
    for (const stmt of body.body) {
      if (t.isReturnStatement(stmt) && t.isJSXElement(stmt.argument)) {
        returnJSX = stmt.argument;
        // Don't add return statement to statements array yet
      } else if (t.isVariableDeclaration(stmt)) {
        // Convert variable declarations: const displayValue = item[field];
        for (const decl of stmt.declarations) {
          const varName = decl.id.name;
          const init = decl.init ? generateCSharpExpression(decl.init) : 'null';
          statements.push(`var ${varName} = ${init};`);
        }
      } else {
        // Other statements - convert them
        statements.push(generateCSharpStatement(stmt));
      }
    }

    if (!returnJSX) {
      console.error('[generateMapExpression] Block statement has no JSX return');
      throw new Error('Map callback with block statement must return JSX element');
    }

    const jsxCode = generateJSXElement(returnJSX, component, indent + 1);
    statements.push(`return ${jsxCode};`);

    itemCode = statements.join(' ');
  } else {
    console.error('[generateMapExpression] Unsupported callback body type:', body?.type);
    throw new Error(`Unsupported map callback body type: ${body?.type}`);
  }

  // Restore previous context
  if (component) {
    component.currentMapContext = previousMapContext;
  }

  // Check if array is dynamic (likely from outer .map())
  const needsCast = arrayName.includes('.') && !arrayName.match(/^[A-Z]/); // Property access, not static class
  const castedArray = needsCast ? `((IEnumerable<dynamic>)${arrayName})` : arrayName;

  // C# Select supports (item, index) => ...
  if (hasBlockStatements) {
    // Use statement lambda: item => { statements; return jsx; }
    if (indexParam) {
      const lambdaExpr = `(${itemParam}, ${indexParam}) => { ${itemCode} }`;
      const castedLambda = needsCast ? `(Func<dynamic, int, dynamic>)(${lambdaExpr})` : lambdaExpr;
      return `${castedArray}.Select(${castedLambda}).ToArray()`;
    } else {
      const lambdaExpr = `${itemParam} => { ${itemCode} }`;
      const castedLambda = needsCast ? `(Func<dynamic, dynamic>)(${lambdaExpr})` : lambdaExpr;
      return `${castedArray}.Select(${castedLambda}).ToArray()`;
    }
  } else {
    // Use expression lambda: item => jsx
    if (indexParam) {
      const lambdaExpr = `(${itemParam}, ${indexParam}) => ${itemCode}`;
      const castedLambda = needsCast ? `(Func<dynamic, int, dynamic>)(${lambdaExpr})` : lambdaExpr;
      return `${castedArray}.Select(${castedLambda}).ToArray()`;
    } else {
      const lambdaExpr = `${itemParam} => ${itemCode}`;
      const castedLambda = needsCast ? `(Func<dynamic, dynamic>)(${lambdaExpr})` : lambdaExpr;
      return `${castedArray}.Select(${castedLambda}).ToArray()`;
    }
  }
}

/**
 * Generate C# statement from JavaScript AST node
 */
function generateCSharpStatement(node) {
  if (!node) return '';

  if (t.isExpressionStatement(node)) {
    return generateCSharpExpression(node.expression) + ';';
  }

  if (t.isReturnStatement(node)) {
    // Handle empty return statement: return; (not return null;)
    if (node.argument === null || node.argument === undefined) {
      return 'return;';
    }
    return `return ${generateCSharpExpression(node.argument)};`;
  }

  if (t.isThrowStatement(node)) {
    return `throw ${generateCSharpExpression(node.argument)};`;
  }

  if (t.isVariableDeclaration(node)) {
    const declarations = node.declarations.map(d => {
      const name = d.id.name;
      const value = generateCSharpExpression(d.init);
      return `var ${name} = ${value};`;
    }).join(' ');
    return declarations;
  }

  if (t.isIfStatement(node)) {
    let test = generateCSharpExpression(node.test);

    // Convert string truthy checks to proper C# boolean expressions
    test = convertStringToBool(test, node.test);

    let result = `if (${test}) {\n`;

    // Handle consequent (then branch)
    if (t.isBlockStatement(node.consequent)) {
      for (const stmt of node.consequent.body) {
        result += '    ' + generateCSharpStatement(stmt) + '\n';
      }
    } else {
      result += '    ' + generateCSharpStatement(node.consequent) + '\n';
    }

    result += '}';

    // Handle alternate (else branch) if it exists
    if (node.alternate) {
      result += ' else {\n';
      if (t.isBlockStatement(node.alternate)) {
        for (const stmt of node.alternate.body) {
          result += '    ' + generateCSharpStatement(stmt) + '\n';
        }
      } else if (t.isIfStatement(node.alternate)) {
        // else if
        result += '    ' + generateCSharpStatement(node.alternate) + '\n';
      } else {
        result += '    ' + generateCSharpStatement(node.alternate) + '\n';
      }
      result += '}';
    }

    return result;
  }

  if (t.isTryStatement(node)) {
    let result = 'try {\n';

    // Handle try block
    if (t.isBlockStatement(node.block)) {
      for (const stmt of node.block.body) {
        result += '    ' + generateCSharpStatement(stmt) + '\n';
      }
    }

    result += '}';

    // Handle catch clause
    if (node.handler) {
      const catchParam = node.handler.param ? node.handler.param.name : 'ex';
      result += ` catch (Exception ${catchParam}) {\n`;

      if (t.isBlockStatement(node.handler.body)) {
        for (const stmt of node.handler.body.body) {
          result += '    ' + generateCSharpStatement(stmt) + '\n';
        }
      }

      result += '}';
    }

    // Handle finally block
    if (node.finalizer) {
      result += ' finally {\n';

      if (t.isBlockStatement(node.finalizer)) {
        for (const stmt of node.finalizer.body) {
          result += '    ' + generateCSharpStatement(stmt) + '\n';
        }
      }

      result += '}';
    }

    return result;
  }

  // Fallback: try to convert as expression
  return generateCSharpExpression(node) + ';';
}

/**
 * Generate C# expression from JS expression
 * @param {boolean} inInterpolation - True if this expression will be inside $"{...}"
 */
function generateCSharpExpression(node, inInterpolation = false) {
  if (!node) {
    const nodePath = node?.__minimactPath || '';
    return `new VNull("${nodePath}")`;
  }

  if (t.isStringLiteral(node)) {
    return generateStringLiteral(node, inInterpolation);
  }

  if (t.isNumericLiteral(node)) {
    return generateNumericLiteral(node);
  }

  if (t.isBooleanLiteral(node)) {
    return generateBooleanLiteral(node);
  }

  if (t.isNullLiteral(node)) {
    return generateNullLiteral(node);
  }

  if (t.isIdentifier(node)) {
    return generateIdentifier(node, currentComponent);
  }

  if (t.isAssignmentExpression(node)) {
    return generateAssignmentExpression(node, generateCSharpExpression, inInterpolation);
  }

  if (t.isAwaitExpression(node)) {
    return `await ${generateCSharpExpression(node.argument, inInterpolation)}`;
  }

  // Handle TypeScript type assertions: (e.target as any) → e.target (strip the cast)
  // In C#, we rely on dynamic typing, so type casts are usually unnecessary
  if (t.isTSAsExpression(node)) {
    return generateCSharpExpression(node.expression, inInterpolation);
  }

  // Handle TypeScript type assertions (angle bracket syntax): <any>e.target → e.target
  if (t.isTSTypeAssertion(node)) {
    return generateCSharpExpression(node.expression, inInterpolation);
  }

  // Handle optional chaining: viewModel?.userEmail → viewModel?.UserEmail
  if (t.isOptionalMemberExpression(node)) {
    return generateOptionalMemberExpression(node, generateCSharpExpression, inInterpolation);
  }

  if (t.isMemberExpression(node)) {
    return generateMemberExpression(node, generateCSharpExpression, inInterpolation);
  }

  if (t.isArrayExpression(node)) {
    return generateArrayExpression(node, generateCSharpExpression);
  }

  if (t.isUnaryExpression(node)) {
    return generateUnaryExpression(node, generateCSharpExpression, inInterpolation);
  }

  if (t.isBinaryExpression(node)) {
    return generateBinaryExpression(node, generateCSharpExpression);
  }

  if (t.isLogicalExpression(node)) {
    return generateLogicalExpression(node, generateCSharpExpression);
  }

  if (t.isConditionalExpression(node)) {
    return generateConditionalExpression(node, generateCSharpExpression);
  }

  if (t.isCallExpression(node)) {
    return generateCallExpression(node, generateCSharpExpression, generateCSharpStatement, currentComponent);
  }

  if (t.isOptionalCallExpression(node)) {
    return generateOptionalCallExpression(node, generateCSharpExpression, generateCSharpStatement, currentComponent);
  }

  if (t.isTemplateLiteral(node)) {
    return generateTemplateLiteral(node, generateCSharpExpression);
  }

  if (t.isNewExpression(node)) {
    return generateNewExpression(node, generateCSharpExpression);
  }

  if (t.isObjectExpression(node)) {
    return generateObjectExpression(node, generateCSharpExpression);
  }

  if (t.isArrowFunctionExpression(node) || t.isFunctionExpression(node)) {
    return generateFunctionExpression(node, generateCSharpExpression, generateCSharpStatement);
  }

  // Fallback for unknown node types
  const nodePath = node?.__minimactPath || '';
  return `new VNull("${nodePath}")`;
}

/**
 * Generate attribute value
 */
function generateAttributeValue(value) {
  if (!value) return '""';

  if (t.isStringLiteral(value)) {
    return `"${escapeCSharpString(value.value)}"`;
  }

  if (t.isJSXExpressionContainer(value)) {
    return generateCSharpExpression(value.expression);
  }

  return '""';
}

/**
 * Generate hybrid expression with smart span splitting
 */
function generateHybridExpression(expr, component, deps, indent) {
  // For now, return a simplified version
  // TODO: Implement full AST splitting logic
  return `new VText(${generateCSharpExpression(expr)})`;
}

/**
 * Set the current component context for useState setter detection
 */
function setCurrentComponent(component) {
  currentComponent = component;
}

module.exports = {
  generateAttributeValue,
  generateCSharpExpression,
  generateCSharpStatement,
  generateMapExpression,
  generateConditional,
  generateShortCircuit,
  generateHybridExpression,
  generateJSXExpression,
  generateBooleanExpression,
  setCurrentComponent
};
