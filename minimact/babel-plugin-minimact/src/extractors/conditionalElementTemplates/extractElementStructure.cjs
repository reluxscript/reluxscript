const t = require('@babel/types');
const generate = require('@babel/generator').default;
const { buildMemberPath } = require('./buildMemberPath.cjs');

/**
 * Extract complete element structure including dynamic content
 */
function extractElementStructure(node) {
  if (t.isJSXElement(node)) {
    const tagName = node.openingElement.name.name;
    const hexPath = node.__minimactPath;

    // Extract attributes
    const attributes = {};
    for (const attr of node.openingElement.attributes) {
      if (t.isJSXAttribute(attr)) {
        const attrName = attr.name.name;
        const attrValue = attr.value;

        if (!attrValue) {
          attributes[attrName] = true; // Boolean attribute
        } else if (t.isStringLiteral(attrValue)) {
          attributes[attrName] = attrValue.value;
        } else if (t.isJSXExpressionContainer(attrValue)) {
          // Dynamic attribute
          const expr = attrValue.expression;
          if (t.isIdentifier(expr)) {
            attributes[attrName] = { binding: expr.name };
          } else if (t.isMemberExpression(expr)) {
            attributes[attrName] = { binding: buildMemberPath(expr) };
          } else {
            attributes[attrName] = { expression: generate(expr).code };
          }
        }
      }
    }

    // Extract children
    const children = [];
    for (const child of node.children) {
      if (t.isJSXText(child)) {
        const text = child.value.trim();
        if (text) {
          children.push({
            type: "text",
            value: text,
            hexPath: child.__minimactPath
          });
        }
      } else if (t.isJSXElement(child)) {
        const childStructure = extractElementStructure(child);
        if (childStructure) {
          children.push(childStructure);
        }
      } else if (t.isJSXExpressionContainer(child)) {
        // Dynamic text content
        const expr = child.expression;
        if (t.isIdentifier(expr)) {
          children.push({
            type: "text",
            binding: expr.name,
            hexPath: child.__minimactPath
          });
        } else if (t.isMemberExpression(expr)) {
          children.push({
            type: "text",
            binding: buildMemberPath(expr),
            hexPath: child.__minimactPath
          });
        } else {
          // Complex expression
          children.push({
            type: "text",
            expression: generate(expr).code,
            hexPath: child.__minimactPath
          });
        }
      }
    }

    return {
      type: "element",
      tag: tagName,
      hexPath,
      attributes,
      children
    };
  } else if (t.isJSXFragment(node)) {
    const children = [];
    for (const child of node.children) {
      if (t.isJSXElement(child)) {
        const childStructure = extractElementStructure(child);
        if (childStructure) {
          children.push(childStructure);
        }
      }
    }

    return {
      type: "fragment",
      children
    };
  }

  return null;
}

module.exports = { extractElementStructure };
