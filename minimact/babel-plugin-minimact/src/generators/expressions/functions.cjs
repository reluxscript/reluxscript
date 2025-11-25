/**
 * Function expression handlers
 */

const t = require('@babel/types');

/**
 * Generate arrow function or function expression
 */
function generateFunctionExpression(node, generateCSharpExpression, generateCSharpStatement) {
  // Arrow function: (x) => x * 2  →  x => x * 2
  // Function expression: function(x) { return x * 2; }  →  x => x * 2
  const params = node.params.map(p => {
    if (t.isIdentifier(p)) return p.name;
    if (t.isObjectPattern(p)) return '{...}'; // Destructuring - simplified
    return 'param';
  }).join(', ');

  // Wrap params in parentheses if multiple or none
  const paramsString = node.params.length === 1 ? params : `(${params})`;

  // Generate function body
  let body;
  if (t.isBlockStatement(node.body)) {
    // Block body: (x) => { return x * 2; }
    const statements = node.body.body.map(stmt => generateCSharpStatement(stmt)).join(' ');
    body = `{ ${statements} }`;
  } else {
    // Expression body: (x) => x * 2
    body = generateCSharpExpression(node.body);
  }

  return `${paramsString} => ${body}`;
}

module.exports = {
  generateFunctionExpression
};
