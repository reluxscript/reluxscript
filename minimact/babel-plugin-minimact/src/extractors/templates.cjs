/**
 * Template Extractor for Hot Reload
 *
 * Extracts parameterized templates from JSX text nodes for instant hot reload.
 * This enables 100% coverage with minimal memory (2KB vs 100KB per component).
 *
 * Architecture:
 * - Build time: Extract templates with {0}, {1} placeholders
 * - Runtime: Re-hydrate templates with current state values
 * - Hot reload: Send template patches instead of re-rendering
 */

const t = require('@babel/types');
const { getPathFromNode, getPathSegmentsFromNode } = require('../utils/pathAssignment.cjs');

// Import extracted helpers
const { buildMemberPath } = require('./templates/buildMemberPath.cjs');
const { extractTextTemplate } = require('./templates/extractTextTemplate.cjs');
const { extractTemplateLiteral } = require('./templates/extractTemplateLiteral.cjs');
const { extractStyleObjectTemplate } = require('./templates/extractStyleObjectTemplate.cjs');
const { isMapCallExpression } = require('./templates/isMapCallExpression.cjs');

/**
 * Extract all templates from JSX render body
 *
 * Returns a map of node paths to templates:
 * {
 *   "div[0].h1[0].text": {
 *     template: "Count: {0}",
 *     bindings: ["count"],
 *     slots: [7],
 *     path: [0, 0]
 *   }
 * }
 */
function extractTemplates(renderBody, component) {
  if (!renderBody) return {};

  const templates = {};

  // Build path stack for tracking node positions
  const pathStack = [];

  /**
   * Traverse JSX tree and extract text templates
   */
  function traverseJSX(node, parentPath = [], siblingCounts = {}) {
    if (t.isJSXElement(node)) {
      const tagName = node.openingElement.name.name;

      // Use pre-assigned hex path
      const pathKey = node.__minimactPath || null;
      if (!pathKey) {
        throw new Error(`[Template Extractor] No __minimactPath found on <${tagName}>. Did assignPathsToJSX run first?`);
      }

      // For backward compatibility with attribute extraction that expects array paths
      const currentPath = getPathSegmentsFromNode(node);

      pathStack.push({ tag: tagName, path: pathKey });

      // Process children
      let textNodeIndex = 0;

      // First pass: Identify text/expression children and check for mixed content
      const textChildren = [];
      let hasTextNodes = false;
      let hasExpressionNodes = false;

      for (const child of node.children) {
        if (t.isJSXText(child)) {
          const text = child.value.trim();
          if (text) {
            textChildren.push(child);
            hasTextNodes = true;
          }
        } else if (t.isJSXExpressionContainer(child)) {
          const expr = child.expression;

          // Skip structural JSX
          const isStructural = t.isJSXElement(expr) ||
                               t.isJSXFragment(expr) ||
                               t.isJSXEmptyExpression(expr) ||
                               (t.isLogicalExpression(expr) &&
                                (t.isJSXElement(expr.right) || t.isJSXFragment(expr.right))) ||
                               (t.isConditionalExpression(expr) &&
                                (t.isJSXElement(expr.consequent) || t.isJSXElement(expr.alternate) ||
                                 t.isJSXFragment(expr.consequent) || t.isJSXFragment(expr.alternate))) ||
                               isMapCallExpression(expr);

          if (!isStructural) {
            textChildren.push(child);
            hasExpressionNodes = true;
          }
        }
      }

      // Second pass: Process text content
      if (textChildren.length > 0) {
        // Check if this is mixed content (text + expressions together)
        const isMixedContent = hasTextNodes && hasExpressionNodes;

        if (isMixedContent) {
          // Mixed content: process all children together as one template
          const firstTextChild = textChildren[0];
          const textPath = firstTextChild.__minimactPath || `${pathKey}.text[${textNodeIndex}]`;

          const template = extractTextTemplate(node.children, currentPath, textNodeIndex, component);
          if (template) {
            console.log(`[Template Extractor] Found mixed content in <${tagName}>: "${template.template.substring(0, 50)}" (path: ${textPath})`);
            templates[textPath] = template;
            textNodeIndex++;
          }
        } else {
          // Pure text or pure expressions: process each separately
          for (const child of textChildren) {
            if (t.isJSXText(child)) {
              const text = child.value.trim();
              if (text) {
                const textPath = child.__minimactPath || `${pathKey}.text[${textNodeIndex}]`;
                console.log(`[Template Extractor] Found static text in <${tagName}>: "${text}" (path: ${textPath})`);
                templates[textPath] = {
                  template: text,
                  bindings: [],
                  slots: [],
                  path: getPathSegmentsFromNode(child),
                  type: 'static'
                };
                textNodeIndex++;
              }
            } else if (t.isJSXExpressionContainer(child)) {
              const exprPath = child.__minimactPath || `${pathKey}.text[${textNodeIndex}]`;

              const template = extractTextTemplate([child], currentPath, textNodeIndex, component);
              if (template) {
                console.log(`[Template Extractor] Found dynamic expression in <${tagName}>: "${template.template}" (path: ${exprPath})`);
                templates[exprPath] = template;
                textNodeIndex++;
              }
            }
          }
        }
      }

      // Third pass: Traverse JSXElement children
      const childSiblingCounts = {};
      for (const child of node.children) {
        if (t.isJSXElement(child)) {
          traverseJSX(child, currentPath, childSiblingCounts);
        } else if (t.isJSXExpressionContainer(child)) {
          const expr = child.expression;

          // Traverse conditional JSX branches
          if (t.isLogicalExpression(expr) && expr.operator === '&&') {
            if (t.isJSXElement(expr.right)) {
              console.log(`[Template Extractor] Traversing conditional branch (&&) in <${tagName}>`);
              traverseJSX(expr.right, currentPath, childSiblingCounts);
            }
          } else if (t.isConditionalExpression(expr)) {
            if (t.isJSXElement(expr.consequent)) {
              console.log(`[Template Extractor] Traversing conditional branch (? consequent) in <${tagName}>`);
              traverseJSX(expr.consequent, currentPath, childSiblingCounts);
            }
            if (t.isJSXElement(expr.alternate)) {
              console.log(`[Template Extractor] Traversing conditional branch (? alternate) in <${tagName}>`);
              traverseJSX(expr.alternate, currentPath, childSiblingCounts);
            }
          }
        }
      }

      pathStack.pop();
    } else if (t.isJSXFragment(node)) {
      // Handle fragments
      const childSiblingCounts = {};
      for (const child of node.children) {
        if (t.isJSXElement(child)) {
          traverseJSX(child, parentPath, childSiblingCounts);
        }
      }
    }
  }

  // Start traversal
  traverseJSX(renderBody);

  return templates;
}

/**
 * Extract templates for attributes (props)
 * Supports:
 * - Template literals: className={`count-${count}`}
 * - Style objects: style={{ fontSize: '32px', color: isActive ? 'red' : 'blue' }}
 * - Static string attributes: className="btn-primary"
 */
function extractAttributeTemplates(renderBody, component) {
  const templates = {};

  // Traverse JSX tree using pre-assigned hex paths
  function traverseJSX(node) {
    if (t.isJSXElement(node)) {
      const tagName = node.openingElement.name.name;

      const elementPath = node.__minimactPath;
      if (!elementPath) {
        throw new Error(`[Attribute Extractor] No __minimactPath found on <${tagName}>. Did assignPathsToJSX run first?`);
      }

      const currentPath = getPathSegmentsFromNode(node);

      // Check attributes for template expressions
      for (const attr of node.openingElement.attributes) {
        if (t.isJSXAttribute(attr)) {
          const attrName = attr.name.name;
          const attrValue = attr.value;

          const attrPath = attr.__minimactPath || `${elementPath}.@${attrName}`;

          // 1. Template literal: className={`count-${count}`}
          if (t.isJSXExpressionContainer(attrValue) && t.isTemplateLiteral(attrValue.expression)) {
            const template = extractTemplateLiteral(attrValue.expression, component);
            if (template) {
              console.log(`[Attribute Template] Found template literal in ${attrName}: "${template.template}" (path: ${attrPath})`);
              templates[attrPath] = {
                ...template,
                path: currentPath,
                attribute: attrName,
                type: template.bindings.length > 0 ? 'attribute-dynamic' : 'attribute-static'
              };
            }
          }
          // 2. Style object: style={{ fontSize: '32px', opacity: isVisible ? 1 : 0.5 }}
          else if (attrName === 'style' && t.isJSXExpressionContainer(attrValue) && t.isObjectExpression(attrValue.expression)) {
            const styleTemplate = extractStyleObjectTemplate(attrValue.expression, tagName, null, null, currentPath, component);
            if (styleTemplate) {
              console.log(`[Attribute Template] Found style object: "${styleTemplate.template.substring(0, 60)}..." (path: ${attrPath})`);
              templates[attrPath] = styleTemplate;
            }
          }
          // 3. Static string attribute: className="btn-primary", placeholder="Enter name"
          else if (t.isStringLiteral(attrValue)) {
            console.log(`[Attribute Template] Found static attribute ${attrName}: "${attrValue.value}" (path: ${attrPath})`);
            templates[attrPath] = {
              template: attrValue.value,
              bindings: [],
              slots: [],
              path: currentPath,
              attribute: attrName,
              type: 'attribute-static'
            };
          }
          // 4. Simple expression (for future dynamic attribute support)
          else if (t.isJSXExpressionContainer(attrValue)) {
            const expr = attrValue.expression;
            if (t.isIdentifier(expr) || t.isMemberExpression(expr)) {
              const binding = t.isIdentifier(expr) ? expr.name : buildMemberPath(expr);
              console.log(`[Attribute Template] Found dynamic attribute ${attrName}: binding="${binding}" (path: ${attrPath})`);
              templates[attrPath] = {
                template: '{0}',
                bindings: [binding],
                slots: [0],
                path: currentPath,
                attribute: attrName,
                type: 'attribute-dynamic'
              };
            }
          }
        }
      }

      // Traverse children
      for (const child of node.children) {
        if (t.isJSXElement(child)) {
          traverseJSX(child);
        }
      }
    }
  }

  if (renderBody) {
    traverseJSX(renderBody);
  }

  return templates;
}

/**
 * Generate template map JSON file content
 */
function generateTemplateMapJSON(componentName, templates, attributeTemplates, conditionalElementTemplates = {}) {
  const allTemplates = {
    ...templates,
    ...attributeTemplates
  };

  const result = {
    component: componentName,
    version: '1.0',
    generatedAt: Date.now(),
    templates: Object.entries(allTemplates).reduce((acc, [path, template]) => {
      acc[path] = {
        template: template.template,
        bindings: template.bindings,
        slots: template.slots,
        path: template.path,
        type: template.type
      };

      if (template.conditionalTemplates) {
        acc[path].conditionalTemplates = template.conditionalTemplates;
      }

      if (template.transform) {
        acc[path].transform = template.transform;
      }

      if (template.nullable) {
        acc[path].nullable = template.nullable;
      }

      return acc;
    }, {})
  };

  if (Object.keys(conditionalElementTemplates).length > 0) {
    result.conditionalElements = conditionalElementTemplates;
  }

  return result;
}

/**
 * Add template metadata to component for C# code generation
 */
function addTemplateMetadata(component, templates) {
  component.templates = templates;

  // Add template bindings to track which state affects which templates
  component.templateBindings = new Map();

  for (const [path, template] of Object.entries(templates)) {
    for (const binding of template.bindings) {
      if (!component.templateBindings.has(binding)) {
        component.templateBindings.set(binding, []);
      }
      component.templateBindings.get(binding).push(path);
    }
  }
}

module.exports = {
  extractTemplates,
  extractAttributeTemplates,
  generateTemplateMapJSON,
  addTemplateMetadata
};
