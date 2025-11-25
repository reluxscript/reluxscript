/**
 * TypeScript → Rust Transpiler
 *
 * Transpiles TypeScript async functions to Rust async functions
 * for useServerTask with runtime: 'rust'
 */

const t = require('@babel/types');

/**
 * Transpile async function body → Rust code
 */
function transpileAsyncFunctionToRust(asyncFunction, options = {}) {
  const body = asyncFunction.body;
  const params = asyncFunction.params;

  let rustCode = '';

  // Transpile body
  if (t.isBlockStatement(body)) {
    rustCode = transpileBlockStatement(body);
  } else {
    // Arrow function with expression body: () => expr
    rustCode = transpileExpression(body);
  }

  return rustCode;
}

/**
 * Transpile TypeScript block statement → Rust code
 */
function transpileBlockStatement(block) {
  let code = '';

  for (const statement of block.body) {
    code += transpileStatement(statement);
  }

  return code;
}

/**
 * Transpile individual TypeScript statement → Rust statement
 */
function transpileStatement(statement) {
  if (t.isVariableDeclaration(statement)) {
    const isMutable = statement.kind === 'let';
    const declarations = statement.declarations.map(decl => {
      const name = decl.id.name;
      const init = decl.init ? transpileExpression(decl.init) : 'Default::default()';
      return `    let ${isMutable ? 'mut ' : ''}${name} = ${init};`;
    });
    return declarations.join('\n') + '\n';
  }

  if (t.isReturnStatement(statement)) {
    const value = statement.argument ? transpileExpression(statement.argument) : '()';
    return `    ${value}\n`;
  }

  if (t.isExpressionStatement(statement)) {
    return `    ${transpileExpression(statement.expression)};\n`;
  }

  if (t.isForStatement(statement)) {
    // for (let i = 0; i < n; i++) → for i in 0..n
    const init = statement.init.declarations[0];
    const varName = init.id.name;
    const start = transpileExpression(init.init);
    const end = transpileExpression(statement.test.right);
    const body = transpileStatement(statement.body);

    return `    for ${varName} in ${start}..${end} {\n${indent(body, 4)}\n    }\n`;
  }

  if (t.isForOfStatement(statement)) {
    const left = t.isVariableDeclaration(statement.left)
      ? statement.left.declarations[0].id.name
      : statement.left.name;
    const right = transpileExpression(statement.right);
    const body = transpileBlockStatement(statement.body);

    // Check if it's for await of
    if (statement.await) {
      return `    while let Some(${left}) = ${right}.next().await {\n${indent(body, 4)}\n    }\n`;
    }

    return `    for ${left} in ${right} {\n${indent(body, 4)}\n    }\n`;
  }

  if (t.isWhileStatement(statement)) {
    const test = transpileExpression(statement.test);
    const body = transpileBlockStatement(statement.body);
    return `    while ${test} {\n${indent(body, 4)}\n    }\n`;
  }

  if (t.isIfStatement(statement)) {
    const test = transpileExpression(statement.test);
    const consequent = transpileBlockStatement(statement.consequent);
    const alternate = statement.alternate
      ? ` else {\n${indent(transpileBlockStatement(statement.alternate), 4)}\n    }`
      : '';
    return `    if ${test} {\n${indent(consequent, 4)}\n    }${alternate}\n`;
  }

  if (t.isBlockStatement(statement)) {
    return transpileBlockStatement(statement);
  }

  if (t.isTryStatement(statement)) {
    const block = transpileBlockStatement(statement.block);
    const handler = statement.handler ? transpileCatchClause(statement.handler) : '';
    const finalizer = statement.finalizer
      ? `\n    // finally block not yet supported`
      : '';
    return `    // try-catch\n${block}${handler}${finalizer}`;
  }

  if (t.isThrowStatement(statement)) {
    return `    panic!("${transpileExpression(statement.argument)}");\n`;
  }

  if (t.isBreakStatement(statement)) {
    return '    break;\n';
  }

  if (t.isContinueStatement(statement)) {
    return '    continue;\n';
  }

  // Default
  return `    // TODO: Transpile ${statement.type}\n`;
}

/**
 * Transpile TypeScript expression → Rust expression
 */
function transpileExpression(expr) {
  if (!expr) return 'Default::default()';

  if (t.isStringLiteral(expr)) {
    return `"${escapeString(expr.value)}".to_string()`;
  }

  if (t.isNumericLiteral(expr)) {
    return expr.value.toString();
  }

  if (t.isBooleanLiteral(expr)) {
    return expr.value ? 'true' : 'false';
  }

  if (t.isNullLiteral(expr)) {
    return 'None';
  }

  if (t.isIdentifier(expr)) {
    // Special handling for progress parameter
    if (expr.name === 'progress') {
      return 'progress';
    }
    // Special handling for cancellation token
    if (expr.name === 'cancellationToken' || expr.name === 'cancel') {
      return 'cancellation_token';
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
    return transpileMemberExpression(fullExpr, object, property, expr.object);
  }

  if (t.isCallExpression(expr)) {
    const callee = transpileExpression(expr.callee);
    const args = expr.arguments.map(arg => transpileExpression(arg)).join(', ');

    // Handle special method calls
    return transpileMethodCall(callee, args, expr);
  }

  if (t.isAwaitExpression(expr)) {
    return `${transpileExpression(expr.argument)}.await`;
  }

  if (t.isArrayExpression(expr)) {
    const elements = expr.elements.map(el => transpileExpression(el)).join(', ');
    return `vec![${elements}]`;
  }

  if (t.isObjectExpression(expr)) {
    // For now, serialize as JSON value
    return `serde_json::json!({})`;
  }

  if (t.isArrowFunctionExpression(expr)) {
    const params = expr.params.map(p => {
      if (t.isIdentifier(p)) {
        return p.name;
      }
      return '_';
    }).join(', ');

    const body = t.isBlockStatement(expr.body)
      ? `{ ${transpileBlockStatement(expr.body)} }`
      : transpileExpression(expr.body);

    return `|${params}| ${body}`;
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
    return `if ${test} { ${consequent} } else { ${alternate} }`;
  }

  if (t.isTemplateLiteral(expr)) {
    return transpileTemplateLiteral(expr);
  }

  if (t.isNewExpression(expr)) {
    const callee = transpileExpression(expr.callee);
    const args = expr.arguments.map(arg => transpileExpression(arg)).join(', ');
    return `${callee}::new(${args})`;
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

    if (operator === '++') {
      return expr.prefix ? `{ ${argument} += 1; ${argument} }` : `{ let temp = ${argument}; ${argument} += 1; temp }`;
    } else if (operator === '--') {
      return expr.prefix ? `{ ${argument} -= 1; ${argument} }` : `{ let temp = ${argument}; ${argument} -= 1; temp }`;
    }

    return argument;
  }

  return `/* TODO: ${expr.type} */`;
}

/**
 * Transpile member expression (handle special cases)
 */
function transpileMemberExpression(fullExpr, object, property, objectNode) {
  // progress.report() → progress sender
  if (object === 'progress' && property === '.report') {
    return 'progress.send';
  }

  // Handle iterator method chaining
  // Only add .iter() if the object is NOT already an iterator method call
  const iteratorMethods = ['.map', '.filter', '.reduce', '.find', '.some', '.every'];

  if (iteratorMethods.includes(property)) {
    // Check if object already ends with an iterator method
    const needsIter = !object.includes('.iter()') &&
                      !object.endsWith('.map') &&
                      !object.endsWith('.filter') &&
                      !object.endsWith('.reduce') &&
                      !object.endsWith('.find') &&
                      !object.endsWith('.some') &&
                      !object.endsWith('.every');

    if (needsIter) {
      return `${object}.iter()${property}`;
    }
  }

  return fullExpr;
}

/**
 * Transpile method call (handle special methods)
 */
function transpileMethodCall(callee, args, expr) {
  // Handle .reduce() → .fold() with special argument order
  if (callee.includes('.reduce')) {
    if (expr.arguments.length >= 2) {
      const fn = transpileExpression(expr.arguments[0]);
      const init = transpileExpression(expr.arguments[1]);
      const base = callee.replace('.reduce', '.fold');
      return `${base}(${init}, ${fn})`;
    }
  }

  // Array methods mappings
  const mappings = {
    'Math.floor': 'f64::floor',
    'Math.ceil': 'f64::ceil',
    'Math.round': 'f64::round',
    'Math.abs': 'f64::abs',
    'Math.max': 'f64::max',
    'Math.min': 'f64::min',
    'Math.sqrt': 'f64::sqrt',
    'Math.pow': 'f64::powf',
    'console.log': 'println!',
    'console.error': 'eprintln!',
    'console.warn': 'println!',
    'JSON.stringify': 'serde_json::to_string',
    'JSON.parse': 'serde_json::from_str'
  };

  for (const [ts, rust] of Object.entries(mappings)) {
    if (callee.includes(ts)) {
      const transpiledCallee = callee.replace(ts, rust);
      return `${transpiledCallee}(${args})`;
    }
  }

  // Special handling for .toFixed()
  if (callee.endsWith('.toFixed')) {
    const obj = callee.replace('.toFixed', '');
    return `format!("{:.${args}}", ${obj})`;
  }

  // Special handling for .split()
  if (callee.endsWith('.split')) {
    const obj = callee.replace('.split', '');
    return `${obj}.split(${args}).collect::<Vec<_>>()`;
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
  const param = handler.param ? handler.param.name : 'err';
  const body = transpileBlockStatement(handler.body);
  return `.or_else(|${param}| { ${body} })`;
}

/**
 * Transpile template literal → Rust format! macro
 */
function transpileTemplateLiteral(expr) {
  let formatStr = '';
  const formatArgs = [];

  for (let i = 0; i < expr.quasis.length; i++) {
    formatStr += expr.quasis[i].value.cooked;

    if (i < expr.expressions.length) {
      formatStr += '{}';
      formatArgs.push(transpileExpression(expr.expressions[i]));
    }
  }

  if (formatArgs.length === 0) {
    return `"${formatStr}".to_string()`;
  }

  return `format!("${formatStr}", ${formatArgs.join(', ')})`;
}

/**
 * Escape string for Rust
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
 * Indent code
 */
function indent(code, spaces) {
  const prefix = ' '.repeat(spaces);
  return code.split('\n')
    .map(line => line ? prefix + line : '')
    .join('\n');
}

module.exports = {
  transpileAsyncFunctionToRust,
  transpileExpression,
  transpileStatement,
  transpileBlockStatement
};
