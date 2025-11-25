/**
 * Method generators for component class
 */

const t = require('@babel/types');
const { generateCSharpExpression, generateCSharpStatement } = require('../expressions.cjs');
const { generateRenderBody } = require('../renderBody.cjs');
const { tsTypeToCSharpType } = require('./inferTypes.cjs');

/**
 * Generate render method
 */
function generateRenderMethod(component, lines) {
  const renderMethodName = component.useTemplate ? 'RenderContent' : 'Render';
  lines.push(`    protected override VNode ${renderMethodName}()`);
  lines.push('    {');

  // Only add StateManager sync if NOT using a template
  if (!component.useTemplate) {
    lines.push('        StateManager.SyncMembersToState(this);');
    lines.push('');
  }

  // MVC State local variables
  if (component.useMvcState && component.useMvcState.length > 0) {
    lines.push('        // MVC State - read from State dictionary');
    for (const mvcState of component.useMvcState) {
      const csharpType = mvcState.type !== 'object' ? mvcState.type : 'dynamic';
      lines.push(`        var ${mvcState.name} = GetState<${csharpType}>("${mvcState.propertyName}");`);
    }
    lines.push('');
  }

  // Local variables (exclude client-computed ones)
  const regularLocalVars = component.localVariables.filter(v => !v.isClientComputed);
  for (const localVar of regularLocalVars) {
    lines.push(`        ${localVar.type} ${localVar.name} = ${localVar.initialValue};`);
  }
  if (regularLocalVars.length > 0) {
    lines.push('');
  }

  if (component.renderBody) {
    const renderCode = generateRenderBody(component.renderBody, component, 2);
    lines.push(renderCode);
  } else {
    lines.push('        return new VText("");');
  }

  lines.push('    }');
}

/**
 * Generate effect methods (useEffect)
 */
function generateEffectMethods(component, lines) {
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

    // Generate attributes
    if (deps.length === 0 && effect.dependencies && t.isArrayExpression(effect.dependencies) && effect.dependencies.elements.length === 0) {
      lines.push(`    [OnMounted]`);
    } else if (deps.length > 0) {
      for (const dep of deps) {
        lines.push(`    [OnStateChanged("${dep}")]`);
      }
    }

    lines.push(`    private void Effect_${effectIndex}()`);
    lines.push('    {');

    // Extract and convert effect body
    if (effect.body && t.isArrowFunctionExpression(effect.body)) {
      const body = effect.body.body;
      if (t.isBlockStatement(body)) {
        for (const stmt of body.body) {
          lines.push(`        ${generateCSharpStatement(stmt)}`);
        }
      } else {
        lines.push(`        ${generateCSharpExpression(body)};`);
      }
    }

    lines.push('    }');
    effectIndex++;
  }
}

/**
 * Generate event handlers
 */
function generateEventHandlers(component, lines) {
  for (const handler of component.eventHandlers) {
    lines.push('');

    // Generate parameter list
    const params = handler.params || [];
    let paramList = params.length > 0
      ? params.map(p => t.isIdentifier(p) ? `dynamic ${p.name}` : 'dynamic arg')
      : [];

    // Add captured parameters
    const capturedParams = handler.capturedParams || [];
    if (capturedParams.length > 0) {
      paramList = paramList.concat(capturedParams.map(p => `dynamic ${p}`));
    }

    const paramStr = paramList.join(', ');
    const returnType = handler.isAsync ? 'async Task' : 'void';
    lines.push(`    public ${returnType} ${handler.name}(${paramStr})`);
    lines.push('    {');

    // Check if this is a curried function error
    if (handler.isCurriedError) {
      lines.push(`        throw new InvalidOperationException(`);
      lines.push(`            "Event handler '${handler.name}' returns a function instead of executing an action. " +`);
      lines.push(`            "This is a curried function pattern (e.g., (e) => (id) => action(id)) which is invalid for event handlers. " +`);
      lines.push(`            "The returned function is never called by the event system. " +`);
      lines.push(`            "Fix: Use (e) => action(someValue) or create a properly bound handler."`);
      lines.push(`        );`);
    } else if (handler.body) {
      if (t.isBlockStatement(handler.body)) {
        for (const statement of handler.body.body) {
          const csharpStmt = generateCSharpStatement(statement);
          if (csharpStmt) {
            lines.push(`        ${csharpStmt}`);
          }
        }
      } else {
        const csharpExpr = generateCSharpExpression(handler.body);
        lines.push(`        ${csharpExpr};`);
      }
    }

    lines.push('    }');
  }
}

/**
 * Generate toggle methods
 */
function generateToggleMethods(component, lines) {
  for (const toggle of component.useToggle) {
    lines.push('');
    lines.push(`    private void ${toggle.toggleFunc}()`);
    lines.push('    {');
    lines.push(`        ${toggle.name} = !${toggle.name};`);
    lines.push(`        SetState("${toggle.name}", ${toggle.name});`);
    lines.push('    }');
  }
}

/**
 * Generate client handlers method
 */
function generateClientHandlersMethod(component, lines) {
  if (component.clientHandlers && component.clientHandlers.length > 0) {
    lines.push('');
    lines.push('    /// <summary>');
    lines.push('    /// Returns JavaScript event handlers for client-side execution');
    lines.push('    /// These execute in the browser with bound hook context');
    lines.push('    /// </summary>');
    lines.push('    protected override Dictionary<string, string> GetClientHandlers()');
    lines.push('    {');
    lines.push('        return new Dictionary<string, string>');
    lines.push('        {');

    for (let i = 0; i < component.clientHandlers.length; i++) {
      const handler = component.clientHandlers[i];
      const escapedJs = handler.jsCode
        .replace(/\\/g, '\\\\')
        .replace(/"/g, '\\"')
        .replace(/\n/g, '\\n')
        .replace(/\r/g, '');

      const comma = i < component.clientHandlers.length - 1 ? ',' : '';
      lines.push(`            ["${handler.name}"] = @"${escapedJs}"${comma}`);
    }

    lines.push('        };');
    lines.push('    }');
  }
}

/**
 * Generate client effects method
 */
function generateClientEffectsMethod(component, lines) {
  if (component.clientEffects && component.clientEffects.length > 0) {
    lines.push('');
    lines.push('    /// <summary>');
    lines.push('    /// Returns JavaScript callbacks for useEffect hooks');
    lines.push('    /// These execute in the browser with bound hook context');
    lines.push('    /// </summary>');
    lines.push('    protected override Dictionary<string, EffectDefinition> GetClientEffects()');
    lines.push('    {');
    lines.push('        return new Dictionary<string, EffectDefinition>');
    lines.push('        {');

    for (let i = 0; i < component.clientEffects.length; i++) {
      const effect = component.clientEffects[i];

      const escapedJs = effect.jsCode
        .replace(/\\/g, '\\\\')
        .replace(/"/g, '\\"')
        .replace(/\n/g, '\\n')
        .replace(/\r/g, '');

      const deps = [];
      if (effect.dependencies && effect.dependencies.elements) {
        for (const dep of effect.dependencies.elements) {
          if (dep && dep.name) {
            deps.push(`"${dep.name}"`);
          }
        }
      }
      const depsArray = deps.length > 0 ? deps.join(', ') : '';

      const comma = i < component.clientEffects.length - 1 ? ',' : '';

      lines.push(`            ["${effect.name}"] = new EffectDefinition`);
      lines.push(`            {`);
      lines.push(`                Callback = @"${escapedJs}",`);
      lines.push(`                Dependencies = new[] { ${depsArray} }`);
      lines.push(`            }${comma}`);
    }

    lines.push('        };');
    lines.push('    }');
  }
}

/**
 * Generate pub/sub methods
 */
function generatePubSubMethods(component, lines) {
  // Pub methods
  if (component.usePub) {
    for (const pub of component.usePub) {
      lines.push('');
      lines.push(`    // Publish to ${pub.name}_channel`);
      lines.push(`    private void ${pub.name}(dynamic value, PubSubOptions? options = null)`);
      lines.push('    {');
      lines.push(`        EventAggregator.Instance.Publish(${pub.name}_channel, value, options);`);
      lines.push('    }');
    }
  }

  // Sub methods
  if (component.useSub) {
    for (const sub of component.useSub) {
      lines.push('');
      lines.push(`    // Subscribe to ${sub.name}_channel`);
      lines.push(`    protected override void OnInitialized()`);
      lines.push('    {');
      lines.push(`        base.OnInitialized();`);
      lines.push(`        `);
      lines.push(`        // Subscribe to ${sub.name}_channel`);
      lines.push(`        EventAggregator.Instance.Subscribe(${sub.name}_channel, (msg) => {`);
      lines.push(`            ${sub.name}_value = msg.Value;`);
      lines.push(`            SetState("${sub.name}_value", ${sub.name}_value);`);
      lines.push(`        });`);
      lines.push('    }');
    }
  }
}

/**
 * Generate SignalR methods
 */
function generateSignalRMethods(component, lines) {
  if (component.useSignalR) {
    for (const signalR of component.useSignalR) {
      lines.push('');
      lines.push(`    // SignalR send method for ${signalR.name}`);
      lines.push(`    // Note: useSignalR is primarily client-side.`);
      lines.push(`    // Server-side SignalR invocation can use HubContext directly if needed.`);
      lines.push(`    private async Task ${signalR.name}_send(string methodName, params object[] args)`);
      lines.push('    {');
      lines.push(`        if (HubContext != null && ConnectionId != null)`);
      lines.push(`        {`);
      lines.push(`            // Send message to specific client connection`);
      lines.push(`            await HubContext.Clients.Client(ConnectionId).SendAsync(methodName, args);`);
      lines.push(`        }`);
      lines.push('    }');
    }
  }
}

/**
 * Generate MVC state setters
 */
function generateMvcStateSetters(component, lines) {
  if (component.useMvcState) {
    for (const mvcState of component.useMvcState) {
      if (mvcState.setter) {
        const csharpType = mvcState.type !== 'object' ? mvcState.type : 'dynamic';
        lines.push('');
        lines.push(`    private void ${mvcState.setter}(${csharpType} value)`);
        lines.push('    {');
        lines.push(`        SetState("${mvcState.propertyName}", value);`);
        lines.push('    }');
      }
    }
  }
}

/**
 * Generate OnInitialized method for Razor Markdown
 */
function generateOnInitializedMethod(component, lines) {
  if (component.useRazorMarkdown && component.useRazorMarkdown.length > 0) {
    const { convertRazorMarkdownToCSharp } = require('../razorMarkdown.cjs');

    lines.push('');
    lines.push('    protected override void OnInitialized()');
    lines.push('    {');
    lines.push('        base.OnInitialized();');
    lines.push('');

    for (const md of component.useRazorMarkdown) {
      const csharpMarkdown = convertRazorMarkdownToCSharp(md.initialValue);
      lines.push(`        ${md.name} = ${csharpMarkdown};`);
    }

    lines.push('    }');
  }
}

/**
 * Generate helper functions
 */
function generateHelperFunctions(component, lines) {
  // In-component helper functions
  if (component.helperFunctions && component.helperFunctions.length > 0) {
    for (const func of component.helperFunctions) {
      // Skip custom hooks
      if (func.name && func.name.startsWith('use') && func.params && func.params.length > 0) {
        const firstParam = func.params[0];
        if (firstParam.name === 'namespace') {
          continue;
        }
      }

      lines.push('');

      const returnType = func.isAsync
        ? (func.returnType === 'void' ? 'async Task' : `async Task<${func.returnType}>`)
        : func.returnType;

      const params = (func.params || []).map(p => `${p.type} ${p.name}`).join(', ');

      lines.push(`    private ${returnType} ${func.name}(${params})`);
      lines.push('    {');

      if (func.body && t.isBlockStatement(func.body)) {
        for (const statement of func.body.body) {
          const stmtCode = generateCSharpStatement(statement, 2);
          lines.push(stmtCode);
        }
      }

      lines.push('    }');
    }
  }

  // Top-level helper functions
  if (component.topLevelHelperFunctions && component.topLevelHelperFunctions.length > 0) {
    for (const helper of component.topLevelHelperFunctions) {
      // Skip custom hooks
      if (helper.name && helper.name.startsWith('use') && helper.node && helper.node.params && helper.node.params.length > 0) {
        const firstParam = helper.node.params[0];
        if (firstParam && firstParam.name === 'namespace') {
          continue;
        }
      }

      lines.push('');
      lines.push(`    // Helper function: ${helper.name}`);

      const func = helper.node;
      const params = (func.params || []).map(p => {
        let paramType = 'dynamic';
        if (p.typeAnnotation && p.typeAnnotation.typeAnnotation) {
          paramType = tsTypeToCSharpType(p.typeAnnotation.typeAnnotation);
        }
        return `${paramType} ${p.name}`;
      }).join(', ');

      let returnType = 'dynamic';
      if (func.returnType && func.returnType.typeAnnotation) {
        returnType = tsTypeToCSharpType(func.returnType.typeAnnotation);
      }

      lines.push(`    private static ${returnType} ${helper.name}(${params})`);
      lines.push('    {');

      if (t.isBlockStatement(func.body)) {
        for (const statement of func.body.body) {
          const csharpStmt = generateCSharpStatement(statement);
          if (csharpStmt) {
            lines.push(`        ${csharpStmt}`);
          }
        }
      } else {
        const csharpExpr = generateCSharpExpression(func.body);
        lines.push(`        return ${csharpExpr};`);
      }

      lines.push('    }');
    }
  }
}

module.exports = {
  generateRenderMethod,
  generateEffectMethods,
  generateEventHandlers,
  generateToggleMethods,
  generateClientHandlersMethod,
  generateClientEffectsMethod,
  generatePubSubMethods,
  generateSignalRMethods,
  generateMvcStateSetters,
  generateOnInitializedMethod,
  generateHelperFunctions
};
