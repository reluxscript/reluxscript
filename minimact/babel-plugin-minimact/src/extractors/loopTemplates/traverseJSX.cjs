const t = require('@babel/types');
const { findMapExpressions } = require('./findMapExpressions.cjs');

/**
 * Traverse JSX tree looking for .map() call expressions
 */
function traverseJSX(node, loopTemplates) {
  if (t.isJSXElement(node)) {
    // Check attributes for .map() expressions
    for (const attr of node.openingElement.attributes) {
      if (t.isJSXAttribute(attr) && t.isJSXExpressionContainer(attr.value)) {
        findMapExpressions(attr.value.expression, loopTemplates);
      }
    }

    // Check children for .map() expressions
    for (const child of node.children) {
      if (t.isJSXExpressionContainer(child)) {
        findMapExpressions(child.expression, loopTemplates);
      } else if (t.isJSXElement(child)) {
        traverseJSX(child, loopTemplates);
      } else if (t.isJSXFragment(child)) {
        for (const fragmentChild of child.children) {
          if (t.isJSXElement(fragmentChild)) {
            traverseJSX(fragmentChild, loopTemplates);
          }
        }
      }
    }
  } else if (t.isJSXFragment(node)) {
    for (const child of node.children) {
      if (t.isJSXElement(child)) {
        traverseJSX(child, loopTemplates);
      }
    }
  }
}

module.exports = { traverseJSX };
