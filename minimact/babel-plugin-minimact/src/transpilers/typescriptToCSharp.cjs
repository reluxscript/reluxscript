/**
 * TypeScript → C# Transpiler
 *
 * Transpiles TypeScript async functions to C# async Tasks
 * for useServerTask support
 */

const t = require('@babel/types');

/**
 * Transpile async function body → C# code
 */
function transpileAsyncFunctionToCSharp(asyncFunction) {
  const body = asyncFunction.body;
  const params = asyncFunction.params;

  let csharpCode = '';

  // Transpile body
  if (t.isBlockStatement(body)) {
    csharpCode = transpileBlockStatement(body);
  } else {
    // Arrow function with expression body: () => expr
    csharpCode = `return ${transpileExpression(body)};`;
  }

  return csharpCode;
}

/**
 * Transpile TypeScript block statement → C# code
 */
function transpileBlockStatement(block) {
  let code = '';

  for (const statement of block.body) {
    code += transpileStatement(statement) + '\n';
  }

  return code;
}

/**
 * Convert non-boolean expressions to boolean for C# conditions
 * In JavaScript, any value can be truthy/falsy. In C#, we need explicit bools.
 * This wraps non-boolean expressions with a runtime type converter.
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
 * Transpile individual TypeScript statement → C# statement
 */
function transpileStatement(statement) {
  if (t.isVariableDeclaration(statement)) {
    const declarations = statement.declarations.map(decl => {
      const name = decl.id.name;
      const init = decl.init ? transpileExpression(decl.init) : 'null';
      if (name === 'chartData') {
        console.log(`[DEBUG chartData] init type: ${decl.init?.type}, result: ${init}`);
      }
      return `var ${name} = ${init};`;
    });
    return declarations.join('\n');
  }

  if (t.isReturnStatement(statement)) {
    return `return ${transpileExpression(statement.argument)};`;
  }

  if (t.isExpressionStatement(statement)) {
    // Check for yield expression (streaming)
    if (t.isYieldExpression(statement.expression)) {
      return `yield return ${transpileExpression(statement.expression.argument)};`;
    }
    return `${transpileExpression(statement.expression)};`;
  }

  if (t.isForStatement(statement)) {
    const init = statement.init ? transpileStatement(statement.init).replace(/;$/, '') : '';
    const test = statement.test ? transpileExpression(statement.test) : 'true';
    const update = statement.update ? transpileExpression(statement.update) : '';
    const body = transpileStatement(statement.body);
    return `for (${init}; ${test}; ${update})\n{\n${indent(body, 4)}\n}`;
  }

  if (t.isForOfStatement(statement)) {
    const left = t.isVariableDeclaration(statement.left)
      ? statement.left.declarations[0].id.name
      : statement.left.name;
    const right = transpileExpression(statement.right);
    const body = transpileStatement(statement.body);

    // Check if it's for await of (streaming)
    if (statement.await) {
      return `await foreach (var ${left} in ${right})\n{\n${indent(body, 4)}\n}`;
    }

    return `foreach (var ${left} in ${right})\n{\n${indent(body, 4)}\n}`;
  }

  if (t.isWhileStatement(statement)) {
    const test = transpileExpression(statement.test);
    const body = transpileStatement(statement.body);
    return `while (${test})\n{\n${indent(body, 4)}\n}`;
  }

  if (t.isIfStatement(statement)) {
    let test = transpileExpression(statement.test);

    // Convert string truthy checks to proper C# boolean expressions
    test = convertStringToBool(test, statement.test);

    const consequent = transpileStatement(statement.consequent);
    const alternate = statement.alternate
      ? `\nelse\n{\n${indent(transpileStatement(statement.alternate), 4)}\n}`
      : '';
    return `if (${test})\n{\n${indent(consequent, 4)}\n}${alternate}`;
  }

  if (t.isBlockStatement(statement)) {
    return transpileBlockStatement(statement);
  }

  if (t.isTryStatement(statement)) {
    const block = transpileBlockStatement(statement.block);
    const handler = statement.handler ? transpileCatchClause(statement.handler) : '';
    const finalizer = statement.finalizer
      ? `\nfinally\n{\n${indent(transpileBlockStatement(statement.finalizer), 4)}\n}`
      : '';
    return `try\n{\n${indent(block, 4)}\n}${handler}${finalizer}`;
  }

  if (t.isThrowStatement(statement)) {
    return `throw ${transpileExpression(statement.argument)};`;
  }

  if (t.isBreakStatement(statement)) {
    return 'break;';
  }

  if (t.isContinueStatement(statement)) {
    return 'continue;';
  }

  // Default: convert to string (may need refinement)
  return `/* TODO: Transpile ${statement.type} */`;
}

/**
 * Transpile TypeScript expression → C# expression
 */
function transpileExpression(expr) {
  if (!expr) return 'null';

  if (t.isStringLiteral(expr)) {
    return `"${escapeString(expr.value)}"`;
  }

  if (t.isNumericLiteral(expr)) {
    return expr.value.toString();
  }

  if (t.isBooleanLiteral(expr)) {
    return expr.value ? 'true' : 'false';
  }

  if (t.isNullLiteral(expr)) {
    return 'null';
  }

  if (t.isIdentifier(expr)) {
    // Special handling for progress parameter
    if (expr.name === 'progress') {
      return 'progress';
    }
    // Special handling for cancellation token
    if (expr.name === 'cancellationToken' || expr.name === 'cancel') {
      return 'cancellationToken';
    }
    return expr.name;
  }

  if (t.isMemberExpression(expr)) {
    const object = transpileExpression(expr.object);
    const property = expr.computed
      ? `[${transpileExpression(expr.property)}]`
      : `.${expr.property.name}`;

    // Handle special member expressions
    const fullExpr = `${object}${property}`;
    return transpileMemberExpression(fullExpr, object, property);
  }

  if (t.isOptionalMemberExpression(expr)) {
    const object = transpileExpression(expr.object);
    const property = expr.computed
      ? `[${transpileExpression(expr.property)}]`
      : `.${expr.property.name}`;

    // In C#, optional chaining (?.) is just ?.
    const fullExpr = `${object}?${property}`;
    return transpileMemberExpression(fullExpr, object, property);
  }

  if (t.isCallExpression(expr)) {
    const callee = transpileExpression(expr.callee);
    const args = expr.arguments.map(arg => transpileExpression(arg)).join(', ');

    // Handle special method calls
    return transpileMethodCall(callee, args);
  }

  if (t.isOptionalCallExpression(expr)) {
    const callee = transpileExpression(expr.callee);
    const args = expr.arguments.map(arg => transpileExpression(arg)).join(', ');

    // In C#, optional call (?.) is handled via null-conditional operator
    // The callee should already have ? from OptionalMemberExpression
    return transpileMethodCall(callee, args);
  }

  if (t.isAwaitExpression(expr)) {
    return `await ${transpileExpression(expr.argument)}`;
  }

  if (t.isArrayExpression(expr)) {
    const elements = expr.elements.map(el => transpileExpression(el)).join(', ');
    return `new[] { ${elements} }`;
  }

  if (t.isObjectExpression(expr)) {
    const props = expr.properties.map(prop => {
      if (t.isObjectProperty(prop)) {
        const key = t.isIdentifier(prop.key) ? prop.key.name : transpileExpression(prop.key);
        const value = transpileExpression(prop.value);
        return `${capitalize(key)} = ${value}`;
      }
      if (t.isSpreadElement(prop)) {
        // C# object spread using with expression (C# 9+)
        return `/* spread: ${transpileExpression(prop.argument)} */`;
      }
      return '';
    }).filter(Boolean).join(', ');
    return `new { ${props} }`;
  }

  if (t.isArrowFunctionExpression(expr)) {
    const params = expr.params.map(p => p.name).join(', ');
    const body = t.isBlockStatement(expr.body)
      ? `{\n${indent(transpileBlockStatement(expr.body), 4)}\n}`
      : transpileExpression(expr.body);
    return `(${params}) => ${body}`;
  }

  if (t.isParenthesizedExpression(expr)) {
    // Unwrap parentheses - just transpile the inner expression
    return transpileExpression(expr.expression);
  }

  if (t.isBinaryExpression(expr)) {
    const left = transpileExpression(expr.left);
    const right = transpileExpression(expr.right);
    const operator = transpileOperator(expr.operator);
    return `(${left} ${operator} ${right})`;
  }

  if (t.isLogicalExpression(expr)) {
    const left = transpileExpression(expr.left);
    const right = transpileExpression(expr.right);
    const operator = transpileOperator(expr.operator);
    return `(${left} ${operator} ${right})`;
  }

  if (t.isUnaryExpression(expr)) {
    const operator = transpileOperator(expr.operator);
    const argument = transpileExpression(expr.argument);
    return expr.prefix ? `${operator}${argument}` : `${argument}${operator}`;
  }

  if (t.isConditionalExpression(expr)) {
    const test = transpileExpression(expr.test);
    const consequent = transpileExpression(expr.consequent);
    const alternate = transpileExpression(expr.alternate);
    return `(${test} ? ${consequent} : ${alternate})`;
  }

  if (t.isTemplateLiteral(expr)) {
    // Convert template literal to C# interpolated string
    return transpileTemplateLiteral(expr);
  }

  if (t.isNewExpression(expr)) {
    const callee = transpileExpression(expr.callee);
    const args = expr.arguments.map(arg => transpileExpression(arg)).join(', ');
    return `new ${callee}(${args})`;
  }

  if (t.isAssignmentExpression(expr)) {
    const left = transpileExpression(expr.left);
    const right = transpileExpression(expr.right);
    const operator = transpileOperator(expr.operator);
    return `${left} ${operator} ${right}`;
  }

  if (t.isUpdateExpression(expr)) {
    const argument = transpileExpression(expr.argument);
    const operator = expr.operator;
    return expr.prefix ? `${operator}${argument}` : `${argument}${operator}`;
  }

  console.warn(`[transpileExpression] Unknown expression type: ${expr.type}`);
  return `/* TODO: ${expr.type} */`;
}

/**
 * Transpile member expression (handle special cases)
 */
function transpileMemberExpression(fullExpr, object, property) {
  // progress.report() → progress.Report()
  if (object === 'progress' && property === '.report') {
    return 'progress.Report';
  }

  // cancellationToken.requested → cancellationToken.IsCancellationRequested
  if ((object === 'cancellationToken' || object === 'cancel') && property === '.requested') {
    return 'cancellationToken.IsCancellationRequested';
  }

  return fullExpr;
}

/**
 * Transpile method call (handle special methods)
 */
function transpileMethodCall(callee, args) {
  // Array methods: .map → .Select, .filter → .Where, etc.
  const mappings = {
    '.map': '.Select',
    '.filter': '.Where',
    '.reduce': '.Aggregate',
    '.find': '.FirstOrDefault',
    '.findIndex': '.FindIndex',
    '.some': '.Any',
    '.every': '.All',
    '.includes': '.Contains',
    '.sort': '.OrderBy',
    '.reverse': '.Reverse',
    '.slice': '.Skip',
    '.concat': '.Concat',
    '.join': '.Join',
    'console.log': 'Console.WriteLine',
    'console.error': 'Console.Error.WriteLine',
    'console.warn': 'Console.WriteLine',
    'Math.floor': 'Math.Floor',
    'Math.ceil': 'Math.Ceiling',
    'Math.round': 'Math.Round',
    'Math.abs': 'Math.Abs',
    'Math.max': 'Math.Max',
    'Math.min': 'Math.Min',
    'Math.sqrt': 'Math.Sqrt',
    'Math.pow': 'Math.Pow',
    'JSON.stringify': 'JsonSerializer.Serialize',
    'JSON.parse': 'JsonSerializer.Deserialize'
  };

  for (const [ts, csharp] of Object.entries(mappings)) {
    if (callee.includes(ts)) {
      const transpiledCallee = callee.replace(ts, csharp);
      return `${transpiledCallee}(${args})`;
    }
  }

  // Special handling for .toFixed()
  if (callee.endsWith('.toFixed')) {
    const obj = callee.replace('.toFixed', '');
    return `${obj}.ToString("F" + ${args})`;
  }

  // Special handling for .split()
  if (callee.endsWith('.split')) {
    const obj = callee.replace('.split', '');
    return `${obj}.Split(${args})`;
  }

  // Special handling for .trim() → .Trim()
  if (callee.endsWith('.trim')) {
    const obj = callee.replace('.trim', '');
    return `${obj}.Trim(${args})`;
  }

  // Special handling for .toLowerCase() → .ToLower()
  if (callee.endsWith('.toLowerCase')) {
    const obj = callee.replace('.toLowerCase', '');
    return `${obj}.ToLower(${args})`;
  }

  // Special handling for .toUpperCase() → .ToUpper()
  if (callee.endsWith('.toUpperCase')) {
    const obj = callee.replace('.toUpperCase', '');
    return `${obj}.ToUpper(${args})`;
  }

  // Special handling for fetch (convert to HttpClient call)
  if (callee === 'fetch') {
    return `await _httpClient.GetStringAsync(${args})`;
  }

  return `${callee}(${args})`;
}

/**
 * Transpile operator
 */
function transpileOperator(op) {
  const mappings = {
    '===': '==',
    '!==': '!=',
    '&&': '&&',
    '||': '||',
    '!': '!',
    '+': '+',
    '-': '-',
    '*': '*',
    '/': '/',
    '%': '%',
    '<': '<',
    '>': '>',
    '<=': '<=',
    '>=': '>=',
    '=': '=',
    '+=': '+=',
    '-=': '-=',
    '*=': '*=',
    '/=': '/=',
    '++': '++',
    '--': '--'
  };
  return mappings[op] || op;
}

/**
 * Transpile catch clause
 */
function transpileCatchClause(handler) {
  const param = handler.param ? handler.param.name : 'ex';
  const body = transpileBlockStatement(handler.body);
  return `\ncatch (Exception ${param})\n{\n${indent(body, 4)}\n}`;
}

/**
 * Transpile template literal → C# interpolated string
 */
function transpileTemplateLiteral(expr) {
  let result = '$"';

  for (let i = 0; i < expr.quasis.length; i++) {
    result += expr.quasis[i].value.cooked;

    if (i < expr.expressions.length) {
      result += `{${transpileExpression(expr.expressions[i])}}`;
    }
  }

  result += '"';
  return result;
}

/**
 * Escape string for C#
 */
function escapeString(str) {
  return str
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}

/**
 * Capitalize first letter
 */
function capitalize(str) {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Indent code
 */
function indent(code, spaces) {
  const prefix = ' '.repeat(spaces);
  return code.split('\n').map(line => prefix + line).join('\n');
}

module.exports = {
  transpileAsyncFunctionToCSharp,
  transpileExpression,
  transpileStatement,
  transpileBlockStatement
};
