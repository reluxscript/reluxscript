/**
 * JSX Generators
 */

const t = require('@babel/types');
const { escapeCSharpString } = require('../utils/helpers.cjs');
const { hasSpreadProps, hasDynamicChildren, hasComplexProps } = require('../analyzers/detection.cjs');
const { extractEventHandler } = require('../extractors/eventHandlers.cjs');
const { getPathFromNode } = require('../utils/pathAssignment.cjs');
// Note: generateCSharpExpression, generateRuntimeHelperCall and generateJSXExpression will be lazy-loaded to avoid circular dependencies

/**
 * Generate Fragment
 */
function generateFragment(node, component, indent) {
  const children = generateChildren(node.children, component, indent);
  const childrenArray = children.map(c => c.code).join(', ');
  return `new Fragment(${childrenArray})`;
}

/**
 * Generate C# for JSX element
 */
function generateJSXElement(node, component, indent) {
  // Lazy load to avoid circular dependencies
  const { generateCSharpExpression: _generateCSharpExpression } = require('./expressions.cjs');

  const indentStr = '    '.repeat(indent);

  if (t.isJSXFragment(node)) {
    return generateFragment(node, component, indent);
  }

  // Validate that this is actually a JSXElement
  if (!t.isJSXElement(node)) {
    console.error('[jsx.cjs] generateJSXElement called with non-JSX node:', node?.type || 'undefined');
    throw new Error(`generateJSXElement expects JSXElement or JSXFragment, received: ${node?.type || 'undefined'}`);
  }

  const tagName = node.openingElement.name.name;
  const attributes = node.openingElement.attributes;
  const children = node.children;

  // Get hex path from AST node (assigned by pathAssignment.cjs)
  const hexPath = node.__minimactPath || '';

  // Check if this is a Plugin element
  if (tagName === 'Plugin') {
    const { generatePluginNode } = require('./plugin.cjs');

    // Find the matching plugin metadata from component.pluginUsages
    // Use the plugin index tracker to match plugins in order
    if (!component._pluginRenderIndex) {
      component._pluginRenderIndex = 0;
    }

    const pluginMetadata = component.pluginUsages[component._pluginRenderIndex];
    component._pluginRenderIndex++;

    if (pluginMetadata) {
      return generatePluginNode(pluginMetadata, component);
    } else {
      // Fallback if plugin metadata not found (shouldn't happen)
      console.warn(`[jsx.cjs] Plugin metadata not found for <Plugin> element`);
      return 'new VText("<!-- Plugin not found -->")'
    }
  }

  // Check if this is a Component element (Lifted State Pattern)
  if (tagName === 'Component') {
    return generateComponentWrapper(node, component, indent);
  }

  // Check if this element has markdown attribute and markdown content
  const hasMarkdownAttr = attributes.some(attr =>
    t.isJSXAttribute(attr) && attr.name.name === 'markdown'
  );

  if (hasMarkdownAttr) {
    // Check if child is a markdown state variable
    if (children.length === 1 && t.isJSXExpressionContainer(children[0])) {
      const expr = children[0].expression;
      if (t.isIdentifier(expr)) {
        const varName = expr.name;
        // Check if this is a markdown state variable
        if (component.stateTypes.get(varName) === 'markdown') {
          // Return DivRawHtml with MarkdownHelper.ToHtml()
          return `new DivRawHtml(MarkdownHelper.ToHtml(${varName}))`;
        }
      }
    }
  }

  // Detect if this needs runtime helpers (hybrid approach)
  const needsRuntimeHelper = hasSpreadProps(attributes) ||
                              hasDynamicChildren(children) ||
                              hasComplexProps(attributes);

  if (needsRuntimeHelper) {
    // Lazy load to avoid circular dependency
    const { generateRuntimeHelperCall } = require('./runtimeHelpers.cjs');
    return generateRuntimeHelperCall(tagName, attributes, children, component, indent);
  }

  // Direct VNode construction (compile-time approach)
  // Extract props and event handlers
  const props = [];
  const eventHandlers = [];
  let dataMinimactAttrs = [];

  for (const attr of attributes) {
    if (t.isJSXAttribute(attr)) {
      const name = attr.name.name;
      const value = attr.value;

      // Skip 'key' attribute - it's only for hot reload detection in .tsx.keys files
      if (name === 'key') {
        continue;
      }

      // Convert className to class for HTML compatibility
      const htmlAttrName = name === 'className' ? 'class' : name;

      if (name.startsWith('on')) {
        // Event handler
        const handlerName = extractEventHandler(value, component);
        eventHandlers.push(`["${name.toLowerCase()}"] = "${handlerName}"`);
      } else if (name.startsWith('data-minimact-')) {
        // Keep minimact attributes as-is
        const val = t.isStringLiteral(value) ? value.value : _generateCSharpExpression(value.expression);
        dataMinimactAttrs.push(`["${htmlAttrName}"] = "${val}"`);
      } else {
        // Regular prop
        if (t.isStringLiteral(value)) {
          // String literal - use as-is with quotes
          props.push(`["${htmlAttrName}"] = "${escapeCSharpString(value.value)}"`);
        } else if (t.isJSXExpressionContainer(value)) {
          // Special handling for style attribute with object expression
          if (name === 'style' && t.isObjectExpression(value.expression)) {
            const { convertStyleObjectToCss } = require('../utils/styleConverter.cjs');
            const cssString = convertStyleObjectToCss(value.expression);
            props.push(`["style"] = "${cssString}"`);
          } else if (name === 'ref' && t.isIdentifier(value.expression)) {
            // ref attribute - just use the identifier name as a string, no interpolation
            const refName = value.expression.name;
            props.push(`["ref"] = "${refName}"`);
          } else {
            // Expression - wrap in string interpolation
            const expr = _generateCSharpExpression(value.expression);
            props.push(`["${htmlAttrName}"] = $"{${expr}}"`);
          }
        } else {
          // Fallback
          props.push(`["${htmlAttrName}"] = ""`);
        }
      }
    }
  }

  // Build props dictionary
  const allProps = [...props, ...eventHandlers, ...dataMinimactAttrs];
  const propsStr = allProps.length > 0
    ? `new Dictionary<string, string> { ${allProps.join(', ')} }`
    : 'new Dictionary<string, string>()';

  // Generate children
  const childrenCode = generateChildren(children, component, indent);

  // Build VElement construction with hex path
  if (childrenCode.length === 0) {
    return `new VElement("${tagName}", "${hexPath}", ${propsStr})`;
  } else if (childrenCode.length === 1 && (childrenCode[0].type === 'text' || childrenCode[0].type === 'mixed')) {
    return `new VElement("${tagName}", "${hexPath}", ${propsStr}, ${childrenCode[0].code})`;
  } else {
    // Wrap children appropriately for VNode array
    const childrenArray = childrenCode.map(c => {
      if (c.type === 'text') {
        // Text already has quotes, wrap in VText with path from node
        const textPath = c.node.__minimactPath || '';
        return `new VText(${c.code}, "${textPath}")`;
      } else if (c.type === 'expression') {
        // Expression needs string interpolation wrapper with extra parentheses for complex expressions
        const exprPath = c.node.__minimactPath || '';
        return `new VText($"{(${c.code})}", "${exprPath}")`;
      } else if (c.type === 'mixed') {
        // Mixed content is already an interpolated string, wrap in VText
        // Use path from first child node
        const mixedPath = c.node ? (c.node.__minimactPath || '') : '';
        return `new VText(${c.code}, "${mixedPath}")`;
      } else {
        // Element is already a VNode
        return c.code;
      }
    }).join(',\n' + indentStr + '    ');
    return `new VElement("${tagName}", "${hexPath}", ${propsStr}, new VNode[]\n${indentStr}{\n${indentStr}    ${childrenArray}\n${indentStr}})`;
  }
}

/**
 * Generate children
 */
function generateChildren(children, component, indent) {
  const result = [];

  // Lazy load to avoid circular dependency
  const { generateJSXExpression } = require('./expressions.cjs');

  // First pass: collect all children with their types
  const childList = [];
  for (const child of children) {
    // Skip undefined/null children
    if (!child) {
      console.warn('[jsx.cjs] Skipping undefined child in children array');
      continue;
    }

    if (t.isJSXText(child)) {
      const text = child.value.trim();
      if (text) {
        childList.push({ type: 'text', code: `"${escapeCSharpString(text)}"`, raw: text, node: child });
      }
    } else if (t.isJSXElement(child)) {
      childList.push({ type: 'element', code: generateJSXElement(child, component, indent + 1), node: child });
    } else if (t.isJSXExpressionContainer(child)) {
      const expr = child.expression;

      // Skip JSX comments (empty expressions like {/* comment */})
      if (t.isJSXEmptyExpression(expr)) {
        continue; // Don't add to childList - comments are ignored
      }

      // Check if this is a custom hook UI variable (e.g., {counterUI})
      if (t.isIdentifier(expr) && component.customHooks) {
        const hookInstance = component.customHooks.find(h => h.uiVarName === expr.name);
        if (hookInstance) {
          // This is a custom hook UI! Generate VComponentWrapper instead
          const hexPath = child.__minimactPath || '';
          const wrapperCode = generateCustomHookWrapper(hookInstance, hexPath, component, indent);
          childList.push({ type: 'element', code: wrapperCode, node: child });
          continue; // Skip normal expression handling
        }
      }

      // Skip structural JSX
      const isStructural = t.isJSXElement(expr) ||
                           t.isJSXFragment(expr) ||
                           (t.isLogicalExpression(expr) && (t.isJSXElement(expr.right) || t.isJSXFragment(expr.right))) ||
                           (t.isConditionalExpression(expr) &&
                            (t.isJSXElement(expr.consequent) || t.isJSXElement(expr.alternate) ||
                             t.isJSXFragment(expr.consequent) || t.isJSXFragment(expr.alternate)));

      if (!isStructural) {
        childList.push({ type: 'expression', code: generateJSXExpression(expr, component, indent), node: child });
      } else {
        childList.push({ type: 'element', code: generateJSXExpression(expr, component, indent), node: child });
      }
    } else if (t.isJSXFragment(child)) {
      childList.push({ type: 'element', code: generateFragment(child, component, indent + 1), node: child });
    } else {
      console.warn(`[jsx.cjs] Unknown child type: ${child.type}`);
    }
  }

  // Second pass: merge consecutive text/expression children into mixed content
  let i = 0;
  while (i < childList.length) {
    const current = childList[i];

    // Check if this starts a mixed content sequence (text or expression followed by text or expression)
    if ((current.type === 'text' || current.type === 'expression') && i + 1 < childList.length) {
      const next = childList[i + 1];

      if (next.type === 'text' || next.type === 'expression') {
        // Found mixed content! Merge consecutive text/expression children
        const mixedChildren = [current];
        let j = i + 1;

        while (j < childList.length && (childList[j].type === 'text' || childList[j].type === 'expression')) {
          mixedChildren.push(childList[j]);
          j++;
        }

        // Build a single interpolated string
        let interpolatedCode = '';
        for (const child of mixedChildren) {
          if (child.type === 'text') {
            interpolatedCode += escapeCSharpString(child.raw);
          } else {
            interpolatedCode += `{(${child.code})}`;
          }
        }

        result.push({ type: 'mixed', code: `$"${interpolatedCode}"` });
        i = j; // Skip merged children
        continue;
      }
    }

    // Not mixed content, add as-is
    result.push(current);
    i++;
  }

  return result;
}

/**
 * Generate VComponentWrapper for custom hook UI (e.g., {counterUI})
 *
 * Example:
 *   const [count, increment, , , counterUI] = useCounter('counter1', 0);
 *   {counterUI}
 *
 * Generates:
 *   new VComponentWrapper {
 *     ComponentName = "counter1",
 *     ComponentType = "UseCounterHook",
 *     HexPath = "1.2.4",
 *     InitialState = new Dictionary<string, object> { ["_config.start"] = 0 }
 *   }
 */
function generateCustomHookWrapper(hookInstance, hexPath, component, indent) {
  const { namespace, className, params } = hookInstance;

  // Build InitialState dictionary from hook params
  let stateCode = 'new Dictionary<string, object>()';

  if (params && params.length > 0) {
    const stateEntries = params.map((paramCode, index) => {
      // Hook params become _config.param0, _config.param1, etc.
      return `["_config.param${index}"] = ${paramCode}`;
    });
    stateCode = `new Dictionary<string, object> { ${stateEntries.join(', ')} }`;
  }

  const indentStr = '  '.repeat(indent);

  return `new VComponentWrapper
${indentStr}{
${indentStr}  ComponentName = "${namespace}",
${indentStr}  ComponentType = "${className}",
${indentStr}  HexPath = "${hexPath}",
${indentStr}  InitialState = ${stateCode}
${indentStr}}`;
}

/**
 * Generate VComponentWrapper for <Component> element (Lifted State Pattern)
 *
 * Example:
 *   <Component name="Counter" state={{ count: 0 }}>
 *     <Counter />
 *   </Component>
 *
 * Generates:
 *   new VComponentWrapper {
 *     ComponentName = "Counter",
 *     ComponentType = "Counter",
 *     HexPath = "1.2",
 *     InitialState = new Dictionary<string, object> { ["count"] = 0 }
 *   }
 */
function generateComponentWrapper(node, parentComponent, indent) {
  const generate = require('@babel/generator').default;

  const attributes = node.openingElement.attributes;
  const hexPath = node.__minimactPath || '';

  // Extract name="..." attribute
  const nameAttr = attributes.find(attr => {
    if (!t.isJSXAttribute(attr)) return false;
    // attr.name is JSXIdentifier, not regular Identifier
    const attrName = t.isJSXIdentifier(attr.name) ? attr.name.name : null;
    return attrName === 'name';
  });

  if (!nameAttr) {
    throw new Error('[Lifted State] <Component> element must have a "name" attribute');
  }

  const componentName = t.isStringLiteral(nameAttr.value)
    ? nameAttr.value.value
    : null;

  if (!componentName) {
    throw new Error('[Lifted State] <Component> name attribute must be a string literal');
  }

  // Extract state={...} attribute
  const stateAttr = attributes.find(attr => {
    if (!t.isJSXAttribute(attr)) return false;
    const attrName = t.isJSXIdentifier(attr.name) ? attr.name.name : null;
    return attrName === 'state';
  });

  let stateCode = 'new Dictionary<string, object>()';

  if (stateAttr && t.isJSXExpressionContainer(stateAttr.value)) {
    const stateExpr = stateAttr.value.expression;

    if (t.isObjectExpression(stateExpr)) {
      // Track lifted state keys in parent component
      if (!parentComponent.liftedComponentState) {
        parentComponent.liftedComponentState = [];
      }

      for (const prop of stateExpr.properties) {
        if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
          const localKey = prop.key.name;
          const namespacedKey = `${componentName}.${localKey}`;
          const initialValue = generate(prop.value).code;

          parentComponent.liftedComponentState.push({
            componentName,
            localKey,
            namespacedKey,
            initialValue
          });
        }
      }

      // Generate C# dictionary initializer
      // Force Dictionary format for VComponentWrapper.InitialState
      const { generateCSharpExpression } = require('./expressions.cjs');
      const properties = stateExpr.properties.map(prop => {
        if (t.isObjectProperty(prop)) {
          const key = t.isIdentifier(prop.key) ? prop.key.name : prop.key.value;
          const value = generateCSharpExpression(prop.value, parentComponent, 0);
          return `["${key}"] = ${value}`;
        }
        return '';
      }).filter(p => p !== '');

      if (properties.length === 0) {
        stateCode = 'new Dictionary<string, object>()';
      } else {
        stateCode = `new Dictionary<string, object> { ${properties.join(', ')} }`;
      }
    }
  }

  // Extract child component JSX (should be single element)
  const childComponents = node.children.filter(c => t.isJSXElement(c));

  if (childComponents.length === 0) {
    throw new Error(`[Lifted State] <Component name="${componentName}"> must have exactly one child element`);
  }

  if (childComponents.length > 1) {
    throw new Error(`[Lifted State] <Component name="${componentName}"> must have exactly one child element, found ${childComponents.length}`);
  }

  const childComponent = childComponents[0];
  const childTagName = childComponent.openingElement.name.name;

  console.log(`[Lifted State] ✅ Detected <Component name="${componentName}"> wrapping <${childTagName} />`);

  // Detect protected state keys (from child component's useProtectedState)
  // We need to find the child component in the processed components
  const protectedKeys = [];

  // Look through processed components for a component matching childTagName
  if (parentComponent._childComponents) {
    const childComp = parentComponent._childComponents.find(c => c.name === childTagName);

    if (childComp && childComp.useProtectedState) {
      for (const protectedState of childComp.useProtectedState) {
        protectedKeys.push(protectedState.name);
        console.log(`[Lifted State] 🔒 Protected state detected: ${protectedState.name} in ${childTagName}`);
      }
    }
  }

  // Generate ProtectedKeys code
  const protectedKeysCode = protectedKeys.length > 0
    ? `    ProtectedKeys = new HashSet<string> { ${protectedKeys.map(k => `"${k}"`).join(', ')} },`
    : '';

  // Generate VComponentWrapper instantiation
  return `new VComponentWrapper
{
    ComponentName = "${componentName}",
    ComponentType = "${childTagName}",
    HexPath = "${hexPath}",
    InitialState = ${stateCode},
${protectedKeysCode}
    ParentComponent = this
}`;
}

module.exports = {
  generateFragment,
  generateJSXElement,
  generateChildren
};
