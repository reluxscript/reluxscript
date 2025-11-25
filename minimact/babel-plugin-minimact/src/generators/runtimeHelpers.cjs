/**
 * Runtime Helper Generators
 */

const t = require('@babel/types');
const { escapeCSharpString } = require('../utils/helpers.cjs');
const { getPathFromNode } = require('../utils/pathAssignment.cjs');
// Lazy load to avoid circular dependencies with jsx.cjs and expressions.cjs

/**
 * Generate runtime helper call for complex JSX patterns
 * Uses MinimactHelpers.createElement() for dynamic scenarios
 */
function generateRuntimeHelperCall(tagName, attributes, children, component, indent) {
  // Lazy load to avoid circular dependency
  const { generateCSharpExpression } = require('./expressions.cjs');
  const { generateJSXElement } = require('./jsx.cjs');

  const indentStr = '    '.repeat(indent);

  // Build props object
  let propsCode = 'null';
  const regularProps = [];
  const spreadProps = [];

  for (const attr of attributes) {
    if (t.isJSXSpreadAttribute(attr)) {
      // Spread operator: {...props}
      spreadProps.push(generateCSharpExpression(attr.argument));
    } else if (t.isJSXAttribute(attr)) {
      const name = attr.name.name;
      const value = attr.value;

      // Skip 'key' attribute - it's only for hot reload detection in .tsx.keys files
      if (name === 'key') {
        continue;
      }

      // Convert attribute value to C# expression
      let propValue;
      if (t.isStringLiteral(value)) {
        propValue = `"${escapeCSharpString(value.value)}"`;
      } else if (t.isJSXExpressionContainer(value)) {
        // Special handling for style attribute with object expression
        if (name === 'style' && t.isObjectExpression(value.expression)) {
          const { convertStyleObjectToCss } = require('../utils/styleConverter.cjs');
          const cssString = convertStyleObjectToCss(value.expression);
          propValue = `"${cssString}"`;
        } else {
          propValue = generateCSharpExpression(value.expression);
        }
      } else if (value === null) {
        propValue = '"true"'; // Boolean attribute like <input disabled />
      } else {
        propValue = `"${value}"`;
      }

      regularProps.push(`${name} = ${propValue}`);
    }
  }

  // Build props with potential spread merging
  if (regularProps.length > 0 && spreadProps.length > 0) {
    // Need to merge: ((object)new { prop1 = val1 }).MergeWith((object)spreadObj)
    // Cast both to object to avoid dynamic dispatch issues
    const regularPropsObj = `new { ${regularProps.join(', ')} }`;
    propsCode = `((object)${regularPropsObj})`;
    for (const spreadProp of spreadProps) {
      propsCode = `${propsCode}.MergeWith((object)${spreadProp})`;
    }
  } else if (regularProps.length > 0) {
    // Just regular props
    propsCode = `new { ${regularProps.join(', ')} }`;
  } else if (spreadProps.length > 0) {
    // Just spread props
    propsCode = spreadProps[0];
    for (let i = 1; i < spreadProps.length; i++) {
      propsCode = `((object)${propsCode}).MergeWith((object)${spreadProps[i]})`;
    }
  }

  // Build children
  const childrenArgs = [];
  for (const child of children) {
    if (t.isJSXText(child)) {
      const text = child.value.trim();
      if (text) {
        childrenArgs.push(`"${escapeCSharpString(text)}"`);
      }
    } else if (t.isJSXElement(child)) {
      childrenArgs.push(generateJSXElement(child, component, indent + 1));
    } else if (t.isJSXExpressionContainer(child)) {
      const expr = child.expression;

      // Skip JSX comments (empty expressions like {/* comment */})
      if (t.isJSXEmptyExpression(expr)) {
        continue; // Don't add to childrenArgs
      }

      // Handle conditionals with JSX: {condition ? <A/> : <B/>}
      if (t.isConditionalExpression(expr)) {
        const { generateBooleanExpression } = require('./expressions.cjs');
        const condition = generateBooleanExpression(expr.test);
        const consequent = t.isJSXElement(expr.consequent) || t.isJSXFragment(expr.consequent)
          ? generateJSXElement(expr.consequent, component, indent + 1)
          : generateCSharpExpression(expr.consequent);

        // Handle alternate - if null literal, use VNull with path
        let alternate;
        if (!expr.alternate || t.isNullLiteral(expr.alternate)) {
          const exprPath = child.__minimactPath || '';
          alternate = `new VNull("${exprPath}")`;
        } else if (t.isJSXElement(expr.alternate) || t.isJSXFragment(expr.alternate)) {
          alternate = generateJSXElement(expr.alternate, component, indent + 1);
        } else {
          alternate = generateCSharpExpression(expr.alternate);
        }

        childrenArgs.push(`(${condition}) ? ${consequent} : ${alternate}`);
      }
      // Handle logical expressions with JSX: {condition && <Element/>}
      else if (t.isLogicalExpression(expr) && expr.operator === '&&') {
        const { generateBooleanExpression } = require('./expressions.cjs');
        const left = generateBooleanExpression(expr.left);
        const right = t.isJSXElement(expr.right) || t.isJSXFragment(expr.right)
          ? generateJSXElement(expr.right, component, indent + 1)
          : generateCSharpExpression(expr.right);
        const exprPath = child.__minimactPath || '';
        childrenArgs.push(`(${left}) ? ${right} : new VNull("${exprPath}")`);
      }
      // Handle .map() with JSX callback
      else if (t.isCallExpression(expr) &&
               t.isMemberExpression(expr.callee) &&
               t.isIdentifier(expr.callee.property, { name: 'map' })) {
        // Lazy load generateMapExpression
        const { generateMapExpression } = require('./expressions.cjs');
        childrenArgs.push(generateMapExpression(expr, component, indent));
      }
      // Dynamic children (e.g., items.Select(...))
      else {
        childrenArgs.push(generateCSharpExpression(child.expression));
      }
    }
  }

  // Generate the createElement call
  if (childrenArgs.length === 0) {
    return `MinimactHelpers.createElement("${tagName}", ${propsCode})`;
  } else if (childrenArgs.length === 1) {
    return `MinimactHelpers.createElement("${tagName}", ${propsCode}, ${childrenArgs[0]})`;
  } else {
    const childrenStr = childrenArgs.join(', ');
    return `MinimactHelpers.createElement("${tagName}", ${propsCode}, ${childrenStr})`;
  }
}

/**
 * Force runtime helper generation for a JSX node (used in conditionals/logical expressions)
 */
function generateRuntimeHelperForJSXNode(node, component, indent) {
  // Lazy load to avoid circular dependency
  const { generateCSharpExpression } = require('./expressions.cjs');

  if (t.isJSXFragment(node)) {
    // Handle fragments
    const children = node.children;
    const childrenArgs = [];
    for (const child of children) {
      if (t.isJSXText(child)) {
        const text = child.value.trim();
        if (text) {
          childrenArgs.push(`"${escapeCSharpString(text)}"`);
        }
      } else if (t.isJSXElement(child)) {
        childrenArgs.push(generateRuntimeHelperForJSXNode(child, component, indent + 1));
      } else if (t.isJSXExpressionContainer(child)) {
        // Skip JSX comments (empty expressions like {/* comment */})
        if (t.isJSXEmptyExpression(child.expression)) {
          continue; // Don't add to childrenArgs
        }
        childrenArgs.push(generateCSharpExpression(child.expression));
      }
    }
    if (childrenArgs.length === 0) {
      return 'MinimactHelpers.Fragment()';
    }
    return `MinimactHelpers.Fragment(${childrenArgs.join(', ')})`;
  }

  if (t.isJSXElement(node)) {
    const tagName = node.openingElement.name.name;
    const attributes = node.openingElement.attributes;
    const children = node.children;
    return generateRuntimeHelperCall(tagName, attributes, children, component, indent);
  }

  // Fallback for null/undefined nodes
  const nodePath = node.__minimactPath || '';
  return `new VNull("${nodePath}")`;
}




module.exports = {
  generateRuntimeHelperCall,
  generateRuntimeHelperForJSXNode
};
