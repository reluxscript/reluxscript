/**
 * Minimact Babel Plugin - Complete Implementation
 *
 * Features:
 * - Dependency tracking for hybrid rendering
 * - Smart span splitting for mixed client/server state
 * - All hooks: useState, useEffect, useRef, useClientState, useMarkdown, useTemplate
 * - Conditional rendering (ternary, &&)
 * - List rendering (.map with key)
 * - Fragment support
 * - Props support
 * - TypeScript interface → C# class conversion
 */

const t = require('@babel/types');
const { traverse } = require('@babel/core');

module.exports = function(babel) {
  return {
    name: 'minimact-full',

    visitor: {
      Program: {
        exit(path, state) {
          if (state.file.minimactComponents && state.file.minimactComponents.length > 0) {
            const csharpCode = generateCSharpFile(state.file.minimactComponents, state);

            state.file.metadata = state.file.metadata || {};
            state.file.metadata.minimactCSharp = csharpCode;
          }
        }
      },

      FunctionDeclaration(path, state) {
        processComponent(path, state);
      },

      ArrowFunctionExpression(path, state) {
        if (path.parent.type === 'VariableDeclarator' || path.parent.type === 'ExportNamedDeclaration') {
          processComponent(path, state);
        }
      },

      FunctionExpression(path, state) {
        if (path.parent.type === 'VariableDeclarator') {
          processComponent(path, state);
        }
      }
    }
  };
};

/**
 * Process a component function
 */
function processComponent(path, state) {
  const componentName = getComponentName(path);

  if (!componentName) return;
  if (componentName[0] !== componentName[0].toUpperCase()) return; // Not a component

  state.file.minimactComponents = state.file.minimactComponents || [];

  const component = {
    name: componentName,
    props: [],
    useState: [],
    useClientState: [],
    useEffect: [],
    useRef: [],
    useMarkdown: [],
    useTemplate: null,
    eventHandlers: [],
    localVariables: [], // Local variables (const/let/var) in function body
    renderBody: null,
    stateTypes: new Map(), // Track which hook each state came from
    dependencies: new Map() // Track dependencies per JSX node
  };

  // Extract props from function parameters
  const params = path.node.params;
  if (params.length > 0 && t.isObjectPattern(params[0])) {
    // Destructured props: function Component({ prop1, prop2 })
    // Check if there's a type annotation on the parameter
    const paramTypeAnnotation = params[0].typeAnnotation?.typeAnnotation;

    for (const property of params[0].properties) {
      if (t.isObjectProperty(property) && t.isIdentifier(property.key)) {
        let propType = 'dynamic';

        // Try to extract type from TypeScript annotation
        if (paramTypeAnnotation && t.isTSTypeLiteral(paramTypeAnnotation)) {
          const propName = property.key.name;
          const tsProperty = paramTypeAnnotation.members.find(
            member => t.isTSPropertySignature(member) &&
                     t.isIdentifier(member.key) &&
                     member.key.name === propName
          );
          if (tsProperty && tsProperty.typeAnnotation) {
            propType = tsTypeToCSharpType(tsProperty.typeAnnotation.typeAnnotation);
          }
        }

        component.props.push({
          name: property.key.name,
          type: propType
        });
      }
    }
  } else if (params.length > 0 && t.isIdentifier(params[0])) {
    // Props as single object: function Component(props)
    // Use 'dynamic' to allow property access
    component.props.push({
      name: params[0].name,
      type: 'dynamic'
    });
  }

  // Find function body
  const body = path.node.body.type === 'BlockStatement'
    ? path.node.body
    : t.blockStatement([t.returnStatement(path.node.body)]);

  // Extract hooks and local variables
  path.traverse({
    CallExpression(hookPath) {
      extractHook(hookPath, component);
    },

    VariableDeclaration(varPath) {
      // Only extract local variables at the top level of the function body
      if (varPath.getFunctionParent() === path && varPath.parent.type === 'BlockStatement') {
        extractLocalVariables(varPath, component);
      }
    },

    ReturnStatement(returnPath) {
      if (returnPath.getFunctionParent() === path) {
        component.renderBody = returnPath.node.argument;
        // Replace JSX with null to prevent @babel/preset-react from transforming it
        returnPath.node.argument = t.nullLiteral();
      }
    }
  });

  state.file.minimactComponents.push(component);
}

/**
 * Extract hook calls (useState, useClientState, etc.)
 */
function extractHook(path, component) {
  const node = path.node;

  if (!t.isIdentifier(node.callee)) return;

  const hookName = node.callee.name;

  switch (hookName) {
    case 'useState':
      extractUseState(path, component, 'useState');
      break;
    case 'useClientState':
      extractUseState(path, component, 'useClientState');
      break;
    case 'useEffect':
      extractUseEffect(path, component);
      break;
    case 'useRef':
      extractUseRef(path, component);
      break;
    case 'useMarkdown':
      extractUseMarkdown(path, component);
      break;
    case 'useTemplate':
      extractUseTemplate(path, component);
      break;
  }
}

/**
 * Extract local variables (const/let/var) from function body
 */
function extractLocalVariables(path, component) {
  const declarations = path.node.declarations;

  for (const declarator of declarations) {
    // Skip if it's a hook call (already handled)
    if (t.isCallExpression(declarator.init)) {
      const callee = declarator.init.callee;
      if (t.isIdentifier(callee) && callee.name.startsWith('use')) {
        continue; // Skip hook calls
      }
    }

    // Extract variable name and initial value
    if (t.isIdentifier(declarator.id) && declarator.init) {
      const varName = declarator.id.name;
      const initValue = generateCSharpExpression(declarator.init);

      // Try to infer type from TypeScript annotation or initial value
      let varType = 'var'; // C# var for type inference
      if (declarator.id.typeAnnotation?.typeAnnotation) {
        varType = tsTypeToCSharpType(declarator.id.typeAnnotation.typeAnnotation);
      }

      component.localVariables.push({
        name: varName,
        type: varType,
        initialValue: initValue
      });
    }
  }
}

/**
 * Extract useState or useClientState
 */
function extractUseState(path, component, hookType) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const [stateVar, setterVar] = parent.id.elements;
  const initialValue = path.node.arguments[0];

  const stateInfo = {
    name: stateVar.name,
    setter: setterVar.name,
    initialValue: generateCSharpExpression(initialValue),
    type: inferType(initialValue)
  };

  if (hookType === 'useState') {
    component.useState.push(stateInfo);
    component.stateTypes.set(stateVar.name, 'server');
  } else {
    component.useClientState.push(stateInfo);
    component.stateTypes.set(stateVar.name, 'client');
  }
}

/**
 * Extract useEffect
 */
function extractUseEffect(path, component) {
  const callback = path.node.arguments[0];
  const dependencies = path.node.arguments[1];

  component.useEffect.push({
    body: callback,
    dependencies: dependencies
  });
}

/**
 * Extract useRef
 */
function extractUseRef(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;

  const refName = parent.id.name;
  const initialValue = path.node.arguments[0];

  component.useRef.push({
    name: refName,
    initialValue: generateCSharpExpression(initialValue)
  });
}

/**
 * Extract useMarkdown
 */
function extractUseMarkdown(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const [contentVar, setterVar] = parent.id.elements;
  const initialValue = path.node.arguments[0];

  component.useMarkdown.push({
    name: contentVar.name,
    setter: setterVar.name,
    initialValue: generateCSharpExpression(initialValue)
  });
}

/**
 * Extract useTemplate
 */
function extractUseTemplate(path, component) {
  const templateName = path.node.arguments[0];

  if (t.isStringLiteral(templateName)) {
    component.useTemplate = templateName.value;
  }
}

/**
 * Analyze dependencies in JSX expressions
 * Walk the AST manually to find identifier dependencies
 */
function analyzeDependencies(jsxExpr, component) {
  const deps = new Set();

  function walk(node) {
    if (!node) return;

    // Check if this is an identifier that's a state variable
    if (t.isIdentifier(node)) {
      const name = node.name;
      if (component.stateTypes.has(name)) {
        deps.add({
          name: name,
          type: component.stateTypes.get(name) // 'client' or 'server'
        });
      }
    }

    // Recursively walk the tree
    if (t.isConditionalExpression(node)) {
      walk(node.test);
      walk(node.consequent);
      walk(node.alternate);
    } else if (t.isLogicalExpression(node)) {
      walk(node.left);
      walk(node.right);
    } else if (t.isMemberExpression(node)) {
      walk(node.object);
      walk(node.property);
    } else if (t.isCallExpression(node)) {
      walk(node.callee);
      node.arguments.forEach(walk);
    } else if (t.isBinaryExpression(node)) {
      walk(node.left);
      walk(node.right);
    } else if (t.isUnaryExpression(node)) {
      walk(node.argument);
    } else if (t.isArrowFunctionExpression(node) || t.isFunctionExpression(node)) {
      walk(node.body);
    }
  }

  walk(jsxExpr);
  return deps;
}

/**
 * Classify a JSX node based on dependencies
 */
function classifyNode(deps) {
  if (deps.size === 0) {
    return 'static';
  }

  const types = new Set([...deps].map(d => d.type));

  if (types.size === 1) {
    return types.has('client') ? 'client' : 'server';
  }

  return 'hybrid'; // Mixed dependencies
}

/**
 * Generate C# file from components
 */
function generateCSharpFile(components, state) {
  const lines = [];

  // Usings
  lines.push('using Minimact.AspNetCore.Core;');
  lines.push('using Minimact.AspNetCore.Extensions;');
  lines.push('using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;');
  lines.push('using System.Collections.Generic;');
  lines.push('using System.Linq;');
  lines.push('using System.Threading.Tasks;');
  lines.push('');

  // Namespace (extract from file path or use default)
  const namespace = state.opts.namespace || 'Minimact.Components';
  lines.push(`namespace ${namespace};`);
  lines.push('');

  // Generate each component
  for (const component of components) {
    lines.push(...generateComponent(component));
    lines.push('');
  }

  return lines.join('\n');
}

/**
 * Generate C# class for a component
 */
function generateComponent(component) {
  const lines = [];

  // Class declaration
  lines.push('[Component]');

  const baseClass = component.useTemplate
    ? `${component.useTemplate}Base`
    : 'MinimactComponent';

  lines.push(`public partial class ${component.name} : ${baseClass}`);
  lines.push('{');

  // Prop fields (from function parameters)
  for (const prop of component.props) {
    lines.push(`    [Prop]`);
    lines.push(`    public ${prop.type} ${prop.name} { get; set; }`);
    lines.push('');
  }

  // State fields (useState)
  for (const state of component.useState) {
    lines.push(`    [State]`);
    lines.push(`    private ${state.type} ${state.name} = ${state.initialValue};`);
    lines.push('');
  }

  // Ref fields (useRef)
  for (const ref of component.useRef) {
    lines.push(`    [Ref]`);
    lines.push(`    private object ${ref.name} = ${ref.initialValue};`);
    lines.push('');
  }

  // Markdown fields (useMarkdown)
  for (const md of component.useMarkdown) {
    lines.push(`    [State]`);
    lines.push(`    private string ${md.name} = ${md.initialValue};`);
    lines.push('');
  }

  // Render method
  lines.push('    protected override VNode Render()');
  lines.push('    {');
  lines.push('        StateManager.SyncMembersToState(this);');
  lines.push('');

  // Local variables
  for (const localVar of component.localVariables) {
    lines.push(`        ${localVar.type} ${localVar.name} = ${localVar.initialValue};`);
  }
  if (component.localVariables.length > 0) {
    lines.push('');
  }

  if (component.renderBody) {
    const renderCode = generateRenderBody(component.renderBody, component, 2);
    lines.push(renderCode);
  } else {
    lines.push('        return new VText("");');
  }

  lines.push('    }');

  // Effect methods (useEffect)
  let effectIndex = 0;
  for (const effect of component.useEffect) {
    lines.push('');

    // Extract dependency names from array
    const deps = [];
    if (effect.dependencies && t.isArrayExpression(effect.dependencies)) {
      for (const dep of effect.dependencies.elements) {
        if (t.isIdentifier(dep)) {
          deps.push(dep.name);
        }
      }
    }

    // Generate [OnStateChanged] for each dependency
    for (const dep of deps) {
      lines.push(`    [OnStateChanged("${dep}")]`);
    }

    lines.push(`    private void Effect_${effectIndex}()`);
    lines.push('    {');

    // Extract and convert effect body
    if (effect.body && t.isArrowFunctionExpression(effect.body)) {
      const body = effect.body.body;
      if (t.isBlockStatement(body)) {
        // Multi-statement effect
        for (const stmt of body.body) {
          lines.push(`        ${generateCSharpStatement(stmt)}`);
        }
      } else {
        // Single expression effect
        lines.push(`        ${generateCSharpExpression(body)};`);
      }
    }

    lines.push('    }');
    effectIndex++;
  }

  // Event handlers
  for (const handler of component.eventHandlers) {
    lines.push('');
    lines.push(`    private void ${handler.name}()`);
    lines.push('    {');
    lines.push(`        // TODO: Implement ${handler.name}`);
    lines.push('    }');
  }

  lines.push('}');

  return lines;
}

/**
 * Generate C# code for render body
 */
function generateRenderBody(node, component, indent) {
  const indentStr = '    '.repeat(indent);

  if (!node) {
    return `${indentStr}return new VText("");`;
  }

  // Handle different node types
  if (t.isJSXElement(node) || t.isJSXFragment(node)) {
    return `${indentStr}return ${generateJSXElement(node, component, indent)};`;
  }

  if (t.isConditionalExpression(node)) {
    // Ternary: condition ? a : b
    return generateConditional(node, component, indent);
  }

  if (t.isLogicalExpression(node) && node.operator === '&&') {
    // Short-circuit: condition && <Element>
    return generateShortCircuit(node, component, indent);
  }

  if (t.isCallExpression(node) && t.isMemberExpression(node.callee) && node.callee.property.name === 'map') {
    // Array.map()
    return generateMapExpression(node, component, indent);
  }

  // Fallback
  return `${indentStr}return new VText("${node.type}");`;
}

/**
 * Detect if attributes contain spread operators
 */
function hasSpreadProps(attributes) {
  return attributes.some(attr => t.isJSXSpreadAttribute(attr));
}

/**
 * Detect if children contain dynamic patterns (like .map())
 */
function hasDynamicChildren(children) {
  return children.some(child => {
    if (!t.isJSXExpressionContainer(child)) return false;
    const expr = child.expression;

    // Check for .map() calls
    if (t.isCallExpression(expr) &&
        t.isMemberExpression(expr.callee) &&
        t.isIdentifier(expr.callee.property, { name: 'map' })) {
      return true;
    }

    // Check for array expressions from LINQ/Select
    if (t.isCallExpression(expr) &&
        t.isMemberExpression(expr.callee) &&
        (t.isIdentifier(expr.callee.property, { name: 'Select' }) ||
         t.isIdentifier(expr.callee.property, { name: 'ToArray' }))) {
      return true;
    }

    // Check for conditionals with JSX: {condition ? <A/> : <B/>}
    if (t.isConditionalExpression(expr)) {
      if (t.isJSXElement(expr.consequent) || t.isJSXFragment(expr.consequent) ||
          t.isJSXElement(expr.alternate) || t.isJSXFragment(expr.alternate)) {
        return true;
      }
    }

    // Check for logical expressions with JSX: {condition && <Element/>}
    if (t.isLogicalExpression(expr)) {
      if (t.isJSXElement(expr.right) || t.isJSXFragment(expr.right)) {
        return true;
      }
    }

    return false;
  });
}

/**
 * Detect if props contain complex expressions
 */
function hasComplexProps(attributes) {
  return attributes.some(attr => {
    if (!t.isJSXAttribute(attr)) return false;
    const value = attr.value;

    if (!t.isJSXExpressionContainer(value)) return false;
    const expr = value.expression;

    // Check for conditional spread: {...(condition && { prop: value })}
    if (t.isConditionalExpression(expr) || t.isLogicalExpression(expr)) {
      return true;
    }

    return false;
  });
}

/**
 * Force runtime helper generation for a JSX node (used in conditionals/logical expressions)
 */
function generateRuntimeHelperForJSXNode(node, component, indent) {
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

  return 'null';
}

/**
 * Generate runtime helper call for complex JSX patterns
 * Uses MinimactHelpers.createElement() for dynamic scenarios
 */
function generateRuntimeHelperCall(tagName, attributes, children, component, indent) {
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

      // Convert attribute value to C# expression
      let propValue;
      if (t.isStringLiteral(value)) {
        propValue = `"${escapeCSharpString(value.value)}"`;
      } else if (t.isJSXExpressionContainer(value)) {
        propValue = generateCSharpExpression(value.expression);
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
    // Need to merge: new { prop1 = val1 }.MergeWith(spreadObj)
    const regularPropsObj = `new { ${regularProps.join(', ')} }`;
    propsCode = regularPropsObj;
    for (const spreadProp of spreadProps) {
      propsCode = `${propsCode}.MergeWith(${spreadProp})`;
    }
  } else if (regularProps.length > 0) {
    // Just regular props
    propsCode = `new { ${regularProps.join(', ')} }`;
  } else if (spreadProps.length > 0) {
    // Just spread props
    propsCode = spreadProps[0];
    for (let i = 1; i < spreadProps.length; i++) {
      propsCode = `${propsCode}.MergeWith(${spreadProps[i]})`;
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

      // Handle conditionals with JSX: {condition ? <A/> : <B/>}
      if (t.isConditionalExpression(expr)) {
        const condition = generateCSharpExpression(expr.test);
        const consequent = t.isJSXElement(expr.consequent) || t.isJSXFragment(expr.consequent)
          ? generateJSXElement(expr.consequent, component, indent + 1)
          : generateCSharpExpression(expr.consequent);
        const alternate = t.isJSXElement(expr.alternate) || t.isJSXFragment(expr.alternate)
          ? generateJSXElement(expr.alternate, component, indent + 1)
          : generateCSharpExpression(expr.alternate);
        childrenArgs.push(`(${condition}) ? ${consequent} : ${alternate}`);
      }
      // Handle logical expressions with JSX: {condition && <Element/>}
      else if (t.isLogicalExpression(expr) && expr.operator === '&&') {
        const left = generateCSharpExpression(expr.left);
        const right = t.isJSXElement(expr.right) || t.isJSXFragment(expr.right)
          ? generateJSXElement(expr.right, component, indent + 1)
          : generateCSharpExpression(expr.right);
        childrenArgs.push(`(${left}) ? ${right} : null`);
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
 * Generate C# for JSX element
 */
function generateJSXElement(node, component, indent) {
  const indentStr = '    '.repeat(indent);

  if (t.isJSXFragment(node)) {
    return generateFragment(node, component, indent);
  }

  const tagName = node.openingElement.name.name;
  const attributes = node.openingElement.attributes;
  const children = node.children;

  // Detect if this needs runtime helpers (hybrid approach)
  const needsRuntimeHelper = hasSpreadProps(attributes) ||
                              hasDynamicChildren(children) ||
                              hasComplexProps(attributes);

  if (needsRuntimeHelper) {
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

      if (name.startsWith('on')) {
        // Event handler
        const handlerName = extractEventHandler(value, component);
        eventHandlers.push(`["${name.toLowerCase()}"] = "${handlerName}"`);
      } else if (name.startsWith('data-minimact-')) {
        // Keep minimact attributes as-is
        const val = t.isStringLiteral(value) ? value.value : generateCSharpExpression(value.expression);
        dataMinimactAttrs.push(`["${name}"] = "${val}"`);
      } else {
        // Regular prop
        if (t.isStringLiteral(value)) {
          // String literal - use as-is with quotes
          props.push(`["${name}"] = "${escapeCSharpString(value.value)}"`);
        } else if (t.isJSXExpressionContainer(value)) {
          // Expression - wrap in string interpolation
          const expr = generateCSharpExpression(value.expression);
          props.push(`["${name}"] = $"{${expr}}"`);
        } else {
          // Fallback
          props.push(`["${name}"] = ""`);
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

  // Build VElement construction
  if (childrenCode.length === 0) {
    return `new VElement("${tagName}", ${propsStr})`;
  } else if (childrenCode.length === 1 && childrenCode[0].type === 'text') {
    return `new VElement("${tagName}", ${propsStr}, ${childrenCode[0].code})`;
  } else {
    // Wrap children appropriately for VNode array
    const childrenArray = childrenCode.map(c => {
      if (c.type === 'text') {
        // Text already has quotes, wrap in VText
        return `new VText(${c.code})`;
      } else if (c.type === 'expression') {
        // Expression needs string interpolation wrapper
        return `new VText($"{${c.code}}")`;
      } else {
        // Element is already a VNode
        return c.code;
      }
    }).join(',\n' + indentStr + '    ');
    return `new VElement("${tagName}", ${propsStr}, new VNode[]\n${indentStr}{\n${indentStr}    ${childrenArray}\n${indentStr}})`;
  }
}

/**
 * Generate children
 */
function generateChildren(children, component, indent) {
  const result = [];

  for (const child of children) {
    if (t.isJSXText(child)) {
      const text = child.value.trim();
      if (text) {
        result.push({ type: 'text', code: `"${escapeCSharpString(text)}"` });
      }
    } else if (t.isJSXElement(child)) {
      result.push({ type: 'element', code: generateJSXElement(child, component, indent + 1) });
    } else if (t.isJSXExpressionContainer(child)) {
      result.push({ type: 'expression', code: generateJSXExpression(child.expression, component, indent) });
    }
  }

  return result;
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
    const condition = generateCSharpExpression(expr.test);
    const consequent = t.isJSXElement(expr.consequent) || t.isJSXFragment(expr.consequent)
      ? generateRuntimeHelperForJSXNode(expr.consequent, component, indent)
      : generateCSharpExpression(expr.consequent);
    const alternate = t.isJSXElement(expr.alternate) || t.isJSXFragment(expr.alternate)
      ? generateRuntimeHelperForJSXNode(expr.alternate, component, indent)
      : generateCSharpExpression(expr.alternate);
    return `(${condition}) ? ${consequent} : ${alternate}`;
  }

  if (t.isLogicalExpression(expr) && expr.operator === '&&') {
    // Short-circuit with JSX: condition && <Element/>
    // Force runtime helpers for JSX in logical expressions
    const left = generateCSharpExpression(expr.left);
    const right = t.isJSXElement(expr.right) || t.isJSXFragment(expr.right)
      ? generateRuntimeHelperForJSXNode(expr.right, component, indent)
      : generateCSharpExpression(expr.right);
    // Use != null for truthy check (works for bool, object, int, etc.)
    return `(${left}) ? ${right} : null`;
  }

  // Generate C# expression
  return generateCSharpExpression(expr);
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
 * Generate Fragment
 */
function generateFragment(node, component, indent) {
  const children = generateChildren(node.children, component, indent);
  const childrenArray = children.map(c => c.code).join(', ');
  return `new Fragment(${childrenArray})`;
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
  const body = callback.body;

  const itemCode = t.isJSXElement(body)
    ? generateJSXElement(body, component, indent + 1)
    : generateJSXElement(body.body, component, indent + 1);

  return `${arrayName}.Select(${itemParam} => ${itemCode}).ToArray()`;
}

/**
 * Extract event handler name
 */
function extractEventHandler(value, component) {
  if (t.isStringLiteral(value)) {
    return value.value;
  }

  if (t.isJSXExpressionContainer(value)) {
    const expr = value.expression;

    if (t.isArrowFunctionExpression(expr) || t.isFunctionExpression(expr)) {
      // Inline arrow function - extract to named method
      const handlerName = `Handle${component.eventHandlers.length}`;
      component.eventHandlers.push({ name: handlerName, body: expr.body });
      return handlerName;
    }

    if (t.isIdentifier(expr)) {
      return expr.name;
    }

    if (t.isCallExpression(expr)) {
      // () => someMethod() - extract
      const handlerName = `Handle${component.eventHandlers.length}`;
      component.eventHandlers.push({ name: handlerName, body: expr });
      return handlerName;
    }
  }

  return 'UnknownHandler';
}

/**
 * Generate C# expression from JS expression
 */
/**
 * Generate C# statement from JavaScript AST node
 */
function generateCSharpStatement(node) {
  if (!node) return '';

  if (t.isExpressionStatement(node)) {
    return generateCSharpExpression(node.expression) + ';';
  }

  if (t.isReturnStatement(node)) {
    return `return ${generateCSharpExpression(node.argument)};`;
  }

  if (t.isVariableDeclaration(node)) {
    const declarations = node.declarations.map(d => {
      const name = d.id.name;
      const value = generateCSharpExpression(d.init);
      return `var ${name} = ${value};`;
    }).join(' ');
    return declarations;
  }

  // Fallback: try to convert as expression
  return generateCSharpExpression(node) + ';';
}

function generateCSharpExpression(node) {
  if (!node) return 'null';

  if (t.isStringLiteral(node)) {
    return `"${escapeCSharpString(node.value)}"`;
  }

  if (t.isNumericLiteral(node)) {
    return String(node.value);
  }

  if (t.isBooleanLiteral(node)) {
    return node.value ? 'true' : 'false';
  }

  if (t.isNullLiteral(node)) {
    return 'null';
  }

  if (t.isIdentifier(node)) {
    return node.name;
  }

  if (t.isMemberExpression(node)) {
    const object = generateCSharpExpression(node.object);
    const property = node.computed
      ? `[${generateCSharpExpression(node.property)}]`
      : `.${node.property.name}`;
    return `${object}${property}`;
  }

  if (t.isArrayExpression(node)) {
    const elements = node.elements.map(e => generateCSharpExpression(e)).join(', ');
    return `new List<object> { ${elements} }`;
  }

  if (t.isBinaryExpression(node)) {
    const left = generateCSharpExpression(node.left);
    const right = generateCSharpExpression(node.right);
    return `${left} ${node.operator} ${right}`;
  }

  if (t.isCallExpression(node)) {
    // Handle console.log → Console.WriteLine
    if (t.isMemberExpression(node.callee) &&
        t.isIdentifier(node.callee.object, { name: 'console' }) &&
        t.isIdentifier(node.callee.property, { name: 'log' })) {
      const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(' + ');
      return `Console.WriteLine(${args})`;
    }

    // Generic function call
    const callee = generateCSharpExpression(node.callee);
    const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
    return `${callee}(${args})`;
  }

  if (t.isTemplateLiteral(node)) {
    // Convert template literal to C# string interpolation
    let result = '$"';
    for (let i = 0; i < node.quasis.length; i++) {
      result += node.quasis[i].value.raw;
      if (i < node.expressions.length) {
        result += '{' + generateCSharpExpression(node.expressions[i]) + '}';
      }
    }
    result += '"';
    return result;
  }

  if (t.isObjectExpression(node)) {
    // Convert JS object literal to C# anonymous object
    // { className: 'container', id: 'main' } → new { className = "container", id = "main" }
    const properties = node.properties.map(prop => {
      if (t.isObjectProperty(prop)) {
        const key = t.isIdentifier(prop.key) ? prop.key.name : prop.key.value;
        const value = generateCSharpExpression(prop.value);
        return `${key} = ${value}`;
      }
      return '';
    }).filter(p => p !== '');

    if (properties.length === 0) return 'null';
    return `new { ${properties.join(', ')} }`;
  }

  return 'null';
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
 * Convert TypeScript type annotation to C# type
 */
function tsTypeToCSharpType(tsType) {
  if (!tsType) return 'dynamic';

  // TSStringKeyword -> string
  if (t.isTSStringKeyword(tsType)) return 'string';

  // TSNumberKeyword -> double
  if (t.isTSNumberKeyword(tsType)) return 'double';

  // TSBooleanKeyword -> bool
  if (t.isTSBooleanKeyword(tsType)) return 'bool';

  // TSAnyKeyword -> dynamic
  if (t.isTSAnyKeyword(tsType)) return 'dynamic';

  // TSArrayType -> List<T>
  if (t.isTSArrayType(tsType)) {
    const elementType = tsTypeToCSharpType(tsType.elementType);
    return `List<${elementType}>`;
  }

  // TSTypeLiteral (object type) -> dynamic
  if (t.isTSTypeLiteral(tsType)) return 'dynamic';

  // TSTypeReference (custom types, interfaces) -> dynamic
  if (t.isTSTypeReference(tsType)) return 'dynamic';

  // Default to dynamic for full JSX semantics
  return 'dynamic';
}

/**
 * Infer C# type from initial value
 */
function inferType(node) {
  if (!node) return 'dynamic';

  if (t.isStringLiteral(node)) return 'string';
  if (t.isNumericLiteral(node)) return 'int';
  if (t.isBooleanLiteral(node)) return 'bool';
  if (t.isNullLiteral(node)) return 'dynamic';
  if (t.isArrayExpression(node)) return 'List<dynamic>';
  if (t.isObjectExpression(node)) return 'dynamic';

  return 'dynamic';
}

/**
 * Get component name from path
 */
function getComponentName(path) {
  if (path.node.id) {
    return path.node.id.name;
  }

  if (path.parent.type === 'VariableDeclarator') {
    return path.parent.id.name;
  }

  if (path.parent.type === 'ExportNamedDeclaration') {
    return path.node.id ? path.node.id.name : null;
  }

  return null;
}

/**
 * Escape C# string
 */
function escapeCSharpString(str) {
  return str
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\n/g, '\\n')
    .replace(/\r/g, '\\r')
    .replace(/\t/g, '\\t');
}
