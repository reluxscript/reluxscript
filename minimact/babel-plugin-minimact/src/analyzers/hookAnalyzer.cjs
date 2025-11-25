/**
 * Hook Analyzer Module
 *
 * Analyzes a custom hook to extract:
 * - useState calls → [State] fields
 * - Methods (arrow functions, function declarations)
 * - JSX elements → Render() method
 * - Return values → API surface
 */

const t = require('@babel/types');
const { getHookName, getHookParameters, getHookBody, extractTypeString } = require('./hookDetector.cjs');
const { generateCSharpExpression } = require('../generators/expressions.cjs');
const { transpileBlockStatement, transpileExpression } = require('../transpilers/typescriptToCSharp.cjs');

/**
 * Analyze a custom hook and extract all relevant information
 *
 * @param {NodePath} hookPath - Path to hook function
 * @returns {Object} - Hook analysis result
 */
function analyzeHook(hookPath) {
  const name = getHookName(hookPath);
  const params = getHookParameters(hookPath);
  const body = getHookBody(hookPath);

  const analysis = {
    name: name,
    className: `${capitalize(name)}Hook`,
    params: params,
    states: [],
    methods: [],
    jsxElements: [],
    returnValues: [],
    eventHandlers: []
  };

  if (!body) {
    return analysis;
  }

  // Extract useState calls
  hookPath.traverse({
    CallExpression(path) {
      if (t.isIdentifier(path.node.callee) &&
          path.node.callee.name === 'useState') {
        const state = extractStateFromUseState(path);
        if (state) {
          analysis.states.push(state);
        }
      }
    }
  });

  // Extract methods (arrow functions, function declarations in body)
  if (t.isBlockStatement(body)) {
    body.body.forEach(statement => {
      // const method = () => { ... }
      if (t.isVariableDeclaration(statement)) {
        statement.declarations.forEach(declarator => {
          if (t.isVariableDeclarator(declarator)) {
            const method = extractMethod(declarator);
            if (method) {
              analysis.methods.push(method);
            }
          }
        });
      }
      // function method() { ... }
      else if (t.isFunctionDeclaration(statement)) {
        const method = extractFunctionDeclaration(statement);
        if (method) {
          analysis.methods.push(method);
        }
      }
    });
  }

  // Extract JSX elements and return values
  hookPath.traverse({
    ReturnStatement(path) {
      const returnNode = path.node.argument;

      // Find JSX in return
      if (returnNode) {
        const jsx = extractJSXFromReturn(returnNode, analysis, hookPath);
        if (jsx) {
          analysis.jsxElements.push(jsx);
        }

        // Extract return values (usually array destructuring pattern)
        const returnVals = extractReturnValues(returnNode);
        analysis.returnValues = returnVals;
      }
    }
  });

  // Second pass: Improve return value type inference with context
  if (analysis.returnValues.length > 0) {
    analysis.returnValues = analysis.returnValues.map(ret => ({
      ...ret,
      type: inferReturnValueType(ret.name, analysis)
    }));
  }

  return analysis;
}

/**
 * Extract state information from useState call
 *
 * @param {NodePath} path - Path to useState CallExpression
 * @returns {Object|null} - State info or null
 */
function extractStateFromUseState(path) {
  const parent = path.parent;

  // Must be: const [value, setValue] = useState(initial);
  if (!t.isVariableDeclarator(parent)) return null;
  if (!t.isArrayPattern(parent.id)) return null;

  const elements = parent.id.elements;
  if (elements.length !== 2) return null;

  const valueVar = elements[0];
  const setterVar = elements[1];

  if (!t.isIdentifier(valueVar) || !t.isIdentifier(setterVar)) return null;

  const valueName = valueVar.name;
  const setterName = setterVar.name;

  // Get initial value
  const initialValue = path.node.arguments[0];
  let initialValueCode = null;
  let inferredType = 'any';

  if (initialValue) {
    initialValueCode = generateExpressionCode(initialValue);
    inferredType = inferType(initialValue);
  }

  return {
    varName: valueName,
    setterName: setterName,
    type: inferredType,
    initialValue: initialValueCode
  };
}

/**
 * Extract method from variable declarator
 *
 * @param {Node} declarator - VariableDeclarator node
 * @returns {Object|null} - Method info or null
 */
function extractMethod(declarator) {
  if (!t.isIdentifier(declarator.id)) return null;

  const init = declarator.init;

  // Must be arrow function or function expression
  if (!t.isArrowFunctionExpression(init) && !t.isFunctionExpression(init)) {
    return null;
  }

  // Skip if this is a JSX element assignment (const ui = <div>...</div>)
  if (t.isJSXElement(init) || t.isJSXFragment(init)) {
    return null;
  }

  const name = declarator.id.name;
  const params = init.params.map(p => ({
    name: p.name || 'arg',
    type: extractTypeFromParam(p)
  }));

  const body = init.body;
  let bodyCode = '';

  if (t.isBlockStatement(body)) {
    // Use existing transpiler
    bodyCode = transpileBlockStatement(body).trim();
  } else {
    // Expression body: const add = (a, b) => a + b
    try {
      bodyCode = `return ${transpileExpression(body)};`;
    } catch (e) {
      bodyCode = `return /* TODO: Transpile expression */;`;
    }
  }

  return {
    name: name,
    params: params,
    returnType: 'void', // TODO: Infer from return statement
    body: bodyCode,
    astNode: init
  };
}

/**
 * Extract function declaration as method
 *
 * @param {Node} node - FunctionDeclaration node
 * @returns {Object|null} - Method info or null
 */
function extractFunctionDeclaration(node) {
  if (!node.id) return null;

  const name = node.id.name;
  const params = node.params.map(p => ({
    name: p.name || 'arg',
    type: extractTypeFromParam(p)
  }));

  const bodyCode = generateBlockCode(node.body);

  return {
    name: name,
    params: params,
    returnType: 'void',
    body: bodyCode
  };
}

/**
 * Extract JSX from return statement and find the actual JSX node
 *
 * @param {Node} returnArg - Return argument node
 * @param {Object} analysis - Current hook analysis (to lookup variables)
 * @param {NodePath} hookPath - Path to hook function (to traverse and find JSX)
 * @returns {Object|null} - JSX info { type, node, varName } or null
 */
function extractJSXFromReturn(returnArg, analysis, hookPath) {
  // Pattern 1: return ui; (where ui = <div>...</div>)
  if (t.isIdentifier(returnArg)) {
    const varName = returnArg.name;
    // Find the JSX node by traversing the hook body
    const jsxNode = findJSXVariable(varName, hookPath);
    return jsxNode ? { type: 'variable', varName: varName, node: jsxNode } : null;
  }

  // Pattern 2: const ui = <div>...</div>; return [value, ui];
  if (t.isArrayExpression(returnArg)) {
    for (const element of returnArg.elements) {
      if (t.isJSXElement(element) || t.isJSXFragment(element)) {
        return { type: 'inline', node: element };
      }
      if (t.isIdentifier(element)) {
        const varName = element.name;
        const jsxNode = findJSXVariable(varName, hookPath);
        if (jsxNode) {
          return { type: 'variable', varName: varName, node: jsxNode };
        }
      }
      // Pattern: loading && <div>...</div>
      if (t.isLogicalExpression(element)) {
        const right = element.right;
        if (t.isJSXElement(right) || t.isJSXFragment(right)) {
          return { type: 'conditional', node: element };
        }
        // Find JSX in variable: loading && ui
        if (t.isIdentifier(right)) {
          const jsxNode = findJSXVariable(right.name, hookPath);
          if (jsxNode) {
            return { type: 'conditional', node: element };
          }
        }
      }
    }
  }

  // Pattern 3: Conditional JSX: isOpen && <div>...</div>
  if (t.isLogicalExpression(returnArg)) {
    const right = returnArg.right;
    if (t.isJSXElement(right) || t.isJSXFragment(right)) {
      return { type: 'conditional', node: returnArg };
    }
    // loading && ui (where ui is JSX variable)
    if (t.isIdentifier(right)) {
      const jsxNode = findJSXVariable(right.name, hookPath);
      if (jsxNode) {
        return { type: 'conditional', node: returnArg };
      }
    }
  }

  // Pattern 4: Direct JSX: return <div>...</div>
  if (t.isJSXElement(returnArg) || t.isJSXFragment(returnArg)) {
    return { type: 'inline', node: returnArg };
  }

  return null;
}

/**
 * Find JSX node assigned to a variable
 *
 * @param {string} varName - Variable name
 * @param {NodePath} hookPath - Path to hook function
 * @returns {Node|null} - JSX node or null
 */
function findJSXVariable(varName, hookPath) {
  let foundNode = null;

  const body = hookPath.node.body;
  if (!t.isBlockStatement(body)) return null;

  // Traverse the hook body to find: const varName = <JSX>;
  for (const statement of body.body) {
    if (t.isVariableDeclaration(statement)) {
      for (const declarator of statement.declarations) {
        if (t.isIdentifier(declarator.id) && declarator.id.name === varName) {
          const init = declarator.init;
          if (t.isJSXElement(init) || t.isJSXFragment(init)) {
            foundNode = init;
            break;
          }
          // Conditional: const ui = loading && <div>;
          if (t.isLogicalExpression(init)) {
            const right = init.right;
            if (t.isJSXElement(right) || t.isJSXFragment(right)) {
              foundNode = init; // Return the whole logical expression
              break;
            }
          }
          // Parenthesized expression
          if (t.isParenthesizedExpression(init)) {
            const inner = init.expression;
            if (t.isJSXElement(inner) || t.isJSXFragment(inner)) {
              foundNode = inner;
              break;
            }
            if (t.isLogicalExpression(inner)) {
              foundNode = inner;
              break;
            }
          }
        }
      }
    }
    if (foundNode) break;
  }

  return foundNode;
}

/**
 * Extract return values from return statement
 *
 * @param {Node} returnArg - Return argument node
 * @returns {Array} - Array of return value descriptors
 */
function extractReturnValues(returnArg) {
  const values = [];

  // Pattern 1: return [value, setValue, ui];
  if (t.isArrayExpression(returnArg)) {
    returnArg.elements.forEach((element, index) => {
      if (t.isIdentifier(element)) {
        values.push({
          index: index,
          name: element.name,
          type: inferReturnValueType(element.name)
        });
      } else if (t.isJSXElement(element) || t.isJSXFragment(element)) {
        values.push({
          index: index,
          name: `ui_${index}`,
          type: 'jsx'
        });
      } else if (t.isLogicalExpression(element)) {
        values.push({
          index: index,
          name: `ui_${index}`,
          type: 'jsx'
        });
      }
    });
  }
  // Pattern 2: return { value, setValue, ui };
  else if (t.isObjectExpression(returnArg)) {
    returnArg.properties.forEach((prop, index) => {
      if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
        values.push({
          index: index,
          name: prop.key.name,
          type: inferReturnValueType(prop.key.name)
        });
      }
    });
  }
  // Pattern 3: return value; (single value)
  else if (t.isIdentifier(returnArg)) {
    values.push({
      index: 0,
      name: returnArg.name,
      type: 'unknown'
    });
  }

  return values;
}

/**
 * Infer return value type from name and context
 *
 * @param {string} name - Variable name
 * @param {Object} analysis - Current hook analysis
 * @returns {string} - Inferred type
 */
function inferReturnValueType(name, analysis) {
  // Check if it's a setter (from useState)
  if (analysis && analysis.states) {
    const isStateSetter = analysis.states.some(s => s.setterName === name);
    if (isStateSetter) return 'setter';

    const isStateValue = analysis.states.some(s => s.varName === name);
    if (isStateValue) return 'state';
  }

  // Check if it's a method
  if (analysis && analysis.methods) {
    const isMethod = analysis.methods.some(m => m.name === name);
    if (isMethod) return 'method';
  }

  // Fallback to name-based inference
  if (name.startsWith('set')) return 'setter';
  if (name === 'ui' || name.endsWith('UI')) return 'jsx';
  if (name.includes('handle') || name.includes('on')) return 'method';

  return 'state';
}

/**
 * Capitalize first letter
 *
 * @param {string} str - Input string
 * @returns {string} - Capitalized string
 */
function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Infer type from AST node
 *
 * @param {Node} node - AST node
 * @returns {string} - Inferred type
 */
function inferType(node) {
  if (t.isNumericLiteral(node)) return 'number';
  if (t.isStringLiteral(node)) return 'string';
  if (t.isBooleanLiteral(node)) return 'boolean';
  if (t.isArrayExpression(node)) return 'any[]';
  if (t.isObjectExpression(node)) return 'object';
  if (t.isNullLiteral(node)) return 'object';
  return 'any';
}

/**
 * Extract type from parameter
 *
 * @param {Node} param - Parameter node
 * @returns {string} - Type as string
 */
function extractTypeFromParam(param) {
  if (param.typeAnnotation && t.isTSTypeAnnotation(param.typeAnnotation)) {
    return extractTypeString(param.typeAnnotation.typeAnnotation);
  }
  return 'any';
}

/**
 * Generate C# code from expression node
 * (Simplified - will use existing generators)
 *
 * @param {Node} node - Expression node
 * @returns {string} - C# code
 */
function generateExpressionCode(node) {
  if (t.isNumericLiteral(node)) return node.value.toString();
  if (t.isStringLiteral(node)) return `"${node.value}"`;
  if (t.isBooleanLiteral(node)) return node.value ? 'true' : 'false';
  if (t.isNullLiteral(node)) return 'null';
  if (t.isIdentifier(node)) return node.name;
  if (t.isArrayExpression(node)) {
    const elements = node.elements.map(e => generateExpressionCode(e)).join(', ');
    return `new[] { ${elements} }`;
  }
  // TODO: Handle more complex expressions
  return 'null';
}

module.exports = {
  analyzeHook,
  extractStateFromUseState,
  extractMethod,
  extractJSXFromReturn,
  extractReturnValues
};
