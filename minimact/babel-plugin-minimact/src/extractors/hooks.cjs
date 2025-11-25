/**
 * Hook Extractors
 */

const t = require('@babel/types');
const { generateCSharpExpression } = require('../generators/expressions.cjs');
const { inferType, tsTypeToCSharpType } = require('../types/typeConversion.cjs');
const { extractUseStateX } = require('./useStateX.cjs');

/**
 * Extract hook calls (useState, useClientState, etc.)
 */
function extractHook(path, component) {
  const node = path.node;

  if (!t.isIdentifier(node.callee)) return;

  const hookName = node.callee.name;

  // 🔥 NEW: Check if this hook was imported (takes precedence over built-in hooks)
  if (component.importedHookMetadata && component.importedHookMetadata.has(hookName)) {
    console.log(`[Custom Hook] Found imported hook call: ${hookName}`);
    extractCustomHookCall(path, component, hookName);
    return;
  }

  switch (hookName) {
    case 'useState':
      extractUseState(path, component, 'useState');
      break;
    case 'useClientState':
      extractUseState(path, component, 'useClientState');
      break;
    case 'useProtectedState':
      extractUseProtectedState(path, component);
      break;
    case 'useStateX':
      extractUseStateX(path, component);
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
    case 'useRazorMarkdown':
      extractUseRazorMarkdown(path, component);
      break;
    case 'useTemplate':
      extractUseTemplate(path, component);
      break;
    case 'useValidation':
      extractUseValidation(path, component);
      break;
    case 'useModal':
      extractUseModal(path, component);
      break;
    case 'useToggle':
      extractUseToggle(path, component);
      break;
    case 'useDropdown':
      extractUseDropdown(path, component);
      break;
    case 'usePub':
      extractUsePub(path, component);
      break;
    case 'useSub':
      extractUseSub(path, component);
      break;
    case 'useMicroTask':
      extractUseMicroTask(path, component);
      break;
    case 'useMacroTask':
      extractUseMacroTask(path, component);
      break;
    case 'useSignalR':
      extractUseSignalR(path, component);
      break;
    case 'usePredictHint':
      extractUsePredictHint(path, component);
      break;
    case 'useServerTask':
      extractUseServerTask(path, component);
      break;
    case 'usePaginatedServerTask':
      extractUsePaginatedServerTask(path, component);
      break;
    case 'useMvcState':
      extractUseMvcState(path, component);
      break;
    case 'useMvcViewModel':
      extractUseMvcViewModel(path, component);
      break;
    default:
      // Check if this is a custom hook (starts with 'use' and first param is 'namespace')
      if (hookName.startsWith('use')) {
        extractCustomHookCall(path, component, hookName);
      }
      break;
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

  // Handle read-only state (no setter): const [value] = useState(...)
  if (!stateVar) {
    console.log(`[useState] Skipping invalid destructuring (no state variable)`);
    return;
  }

  // Check if there's a generic type parameter (e.g., useState<decimal>(0))
  let explicitType = null;
  if (path.node.typeParameters && path.node.typeParameters.params.length > 0) {
    const typeParam = path.node.typeParameters.params[0];
    explicitType = tsTypeToCSharpType(typeParam);
    console.log(`[useState] Found explicit type parameter for '${stateVar.name}': ${explicitType}`);
  }

  const stateInfo = {
    name: stateVar.name,
    setter: setterVar ? setterVar.name : null, // Setter is optional (read-only state)
    initialValue: generateCSharpExpression(initialValue),
    type: explicitType || inferType(initialValue) // Prefer explicit type over inferred
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
 * Extract useProtectedState (lifted but parent cannot access)
 */
function extractUseProtectedState(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const [stateVar, setterVar] = parent.id.elements;
  const initialValue = path.node.arguments[0];

  // Handle read-only state (no setter): const [value] = useProtectedState(...)
  if (!stateVar) {
    console.log(`[useProtectedState] Skipping invalid destructuring (no state variable)`);
    return;
  }

  // Check if there's a generic type parameter (e.g., useProtectedState<decimal>(0))
  let explicitType = null;
  if (path.node.typeParameters && path.node.typeParameters.params.length > 0) {
    const typeParam = path.node.typeParameters.params[0];
    explicitType = tsTypeToCSharpType(typeParam);
    console.log(`[useProtectedState] Found explicit type parameter for '${stateVar.name}': ${explicitType}`);
  }

  const stateInfo = {
    name: stateVar.name,
    setter: setterVar ? setterVar.name : null,
    initialValue: generateCSharpExpression(initialValue),
    type: explicitType || inferType(initialValue),
    isProtected: true  // ← Key flag marking this as protected
  };

  // Initialize useProtectedState array if it doesn't exist
  if (!component.useProtectedState) {
    component.useProtectedState = [];
  }

  component.useProtectedState.push(stateInfo);

  // Track state type (protected is a special kind of server state)
  component.stateTypes.set(stateVar.name, 'protected');

  console.log(`[useProtectedState] ✅ Extracted protected state: ${stateVar.name} (type: ${stateInfo.type})`);
}

/**
 * Extract useEffect
 */
function extractUseEffect(path, component) {
  const callback = path.node.arguments[0];
  const dependencies = path.node.arguments[1];

  // 1. Server-side C# (existing)
  component.useEffect.push({
    body: callback,
    dependencies: dependencies
  });

  // 2. 🔥 NEW: Client-side JavaScript
  if (!component.clientEffects) {
    component.clientEffects = [];
  }

  const effectIndex = component.clientEffects.length;

  // Analyze what hooks are used in the effect
  const hookCalls = analyzeHookUsage(callback);

  // Transform arrow function to regular function with hook mapping
  const transformedCallback = transformEffectCallback(callback, hookCalls);

  // Generate JavaScript code
  const jsCode = generate(transformedCallback, {
    compact: false,
    retainLines: false
  }).code;

  component.clientEffects.push({
    name: `Effect_${effectIndex}`,
    jsCode: jsCode,
    dependencies: dependencies,
    hookCalls: hookCalls
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

  // Track as markdown state type
  component.stateTypes.set(contentVar.name, 'markdown');
}

/**
 * Extract useRazorMarkdown - markdown with Razor syntax
 */
function extractUseRazorMarkdown(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const [contentVar, setterVar] = parent.id.elements;
  const initialValue = path.node.arguments[0];

  // Initialize useRazorMarkdown array if it doesn't exist
  if (!component.useRazorMarkdown) {
    component.useRazorMarkdown = [];
  }

  // Extract raw markdown string (for Razor conversion)
  let rawMarkdown = '';
  if (t.isStringLiteral(initialValue)) {
    rawMarkdown = initialValue.value;
  } else if (t.isTemplateLiteral(initialValue)) {
    // Template literal - extract raw string
    rawMarkdown = initialValue.quasis.map(q => q.value.raw).join('');
  }

  component.useRazorMarkdown.push({
    name: contentVar.name,
    setter: setterVar.name,
    initialValue: rawMarkdown, // Store raw markdown for Razor conversion
    hasRazorSyntax: true, // Will be determined by Razor detection later
    referencedVariables: [] // Will be populated by Razor variable extraction
  });

  // Track as razor-markdown state type
  component.stateTypes.set(contentVar.name, 'razor-markdown');
}

/**
 * Extract useTemplate
 */
function extractUseTemplate(path, component) {
  const templateName = path.node.arguments[0];
  const templateProps = path.node.arguments[1];

  if (t.isStringLiteral(templateName)) {
    component.useTemplate = {
      name: templateName.value,
      props: {}
    };

    // Extract template props if provided
    if (templateProps && t.isObjectExpression(templateProps)) {
      for (const prop of templateProps.properties) {
        if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
          const propName = prop.key.name;
          let propValue = '';

          if (t.isStringLiteral(prop.value)) {
            propValue = prop.value.value;
          } else if (t.isNumericLiteral(prop.value)) {
            propValue = prop.value.value.toString();
          } else if (t.isBooleanLiteral(prop.value)) {
            propValue = prop.value.value.toString();
          }

          component.useTemplate.props[propName] = propValue;
        }
      }
    }
  }
}

/**
 * Extract useValidation
 */
function extractUseValidation(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;

  const fieldName = parent.id.name;
  const fieldKey = path.node.arguments[0];
  const validationRules = path.node.arguments[1];

  const validationInfo = {
    name: fieldName,
    fieldKey: t.isStringLiteral(fieldKey) ? fieldKey.value : fieldName,
    rules: {}
  };

  // Extract validation rules from the object
  if (validationRules && t.isObjectExpression(validationRules)) {
    for (const prop of validationRules.properties) {
      if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
        const ruleName = prop.key.name;
        let ruleValue = null;

        if (t.isStringLiteral(prop.value)) {
          ruleValue = prop.value.value;
        } else if (t.isNumericLiteral(prop.value)) {
          ruleValue = prop.value.value;
        } else if (t.isBooleanLiteral(prop.value)) {
          ruleValue = prop.value.value;
        } else if (t.isRegExpLiteral(prop.value)) {
          ruleValue = `/${prop.value.pattern}/${prop.value.flags || ''}`;
        }

        validationInfo.rules[ruleName] = ruleValue;
      }
    }
  }

  component.useValidation.push(validationInfo);
}

/**
 * Extract useModal
 */
function extractUseModal(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;

  const modalName = parent.id.name;

  component.useModal.push({
    name: modalName
  });
}

/**
 * Extract useToggle
 */
function extractUseToggle(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const [stateVar, toggleFunc] = parent.id.elements;
  const initialValue = path.node.arguments[0];

  const toggleInfo = {
    name: stateVar.name,
    toggleFunc: toggleFunc.name,
    initialValue: generateCSharpExpression(initialValue)
  };

  component.useToggle.push(toggleInfo);
}

/**
 * Extract useDropdown
 */
function extractUseDropdown(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;

  const dropdownName = parent.id.name;
  const routeArg = path.node.arguments[0];

  let routeReference = null;

  // Try to extract route reference (e.g., Routes.Api.Units.GetAll)
  if (routeArg && t.isMemberExpression(routeArg)) {
    routeReference = generateCSharpExpression(routeArg);
  }

  component.useDropdown.push({
    name: dropdownName,
    route: routeReference
  });
}

/**
 * Extract usePub
 */
function extractUsePub(path, component) {
  const parent = path.parent;
  if (!t.isVariableDeclarator(parent)) return;

  const pubName = parent.id.name;
  const channel = path.node.arguments[0];

  component.usePub = component.usePub || [];
  component.usePub.push({
    name: pubName,
    channel: t.isStringLiteral(channel) ? channel.value : null
  });
}

/**
 * Extract useSub
 */
function extractUseSub(path, component) {
  const parent = path.parent;
  if (!t.isVariableDeclarator(parent)) return;

  const subName = parent.id.name;
  const channel = path.node.arguments[0];
  const callback = path.node.arguments[1];

  component.useSub = component.useSub || [];
  component.useSub.push({
    name: subName,
    channel: t.isStringLiteral(channel) ? channel.value : null,
    hasCallback: !!callback
  });
}

/**
 * Extract useMicroTask
 */
function extractUseMicroTask(path, component) {
  const callback = path.node.arguments[0];

  component.useMicroTask = component.useMicroTask || [];
  component.useMicroTask.push({
    body: callback
  });
}

/**
 * Extract useMacroTask
 */
function extractUseMacroTask(path, component) {
  const callback = path.node.arguments[0];
  const delay = path.node.arguments[1];

  component.useMacroTask = component.useMacroTask || [];
  component.useMacroTask.push({
    body: callback,
    delay: t.isNumericLiteral(delay) ? delay.value : 0
  });
}

/**
 * Extract useSignalR
 */
function extractUseSignalR(path, component) {
  const parent = path.parent;
  if (!t.isVariableDeclarator(parent)) return;

  const signalRName = parent.id.name;
  const hubUrl = path.node.arguments[0];
  const onMessage = path.node.arguments[1];

  component.useSignalR = component.useSignalR || [];
  component.useSignalR.push({
    name: signalRName,
    hubUrl: t.isStringLiteral(hubUrl) ? hubUrl.value : null,
    hasOnMessage: !!onMessage
  });
}

/**
 * Extract usePredictHint
 */
function extractUsePredictHint(path, component) {
  const hintId = path.node.arguments[0];
  const predictedState = path.node.arguments[1];

  component.usePredictHint = component.usePredictHint || [];
  component.usePredictHint.push({
    hintId: t.isStringLiteral(hintId) ? hintId.value : null,
    predictedState: predictedState
  });
}

/**
 * Extract useServerTask
 *
 * Detects: const task = useServerTask(async () => { ... }, options)
 * Transpiles async function → C# async Task<T>
 * Generates [ServerTask] attribute
 */
function extractUseServerTask(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;

  const taskName = parent.id.name;
  const asyncFunction = path.node.arguments[0];
  const options = path.node.arguments[1];

  // Validate async function
  if (!asyncFunction || (!t.isArrowFunctionExpression(asyncFunction) && !t.isFunctionExpression(asyncFunction))) {
    console.warn('[useServerTask] First argument must be an async function');
    return;
  }

  if (!asyncFunction.async) {
    console.warn('[useServerTask] Function must be async');
    return;
  }

  // Check if streaming (async function*)
  const isStreaming = asyncFunction.generator === true;

  // Extract parameters
  const parameters = asyncFunction.params.map(param => {
    if (t.isIdentifier(param)) {
      return {
        name: param.name,
        type: param.typeAnnotation ? extractTypeAnnotation(param.typeAnnotation) : 'object'
      };
    }
    return null;
  }).filter(Boolean);

  // Extract options
  let streamingEnabled = isStreaming;
  let estimatedChunks = null;
  let runtime = 'csharp'; // Default to C#
  let parallel = false;

  if (options && t.isObjectExpression(options)) {
    for (const prop of options.properties) {
      if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
        if (prop.key.name === 'stream' && t.isBooleanLiteral(prop.value)) {
          streamingEnabled = prop.value.value;
        }
        if (prop.key.name === 'estimatedChunks' && t.isNumericLiteral(prop.value)) {
          estimatedChunks = prop.value.value;
        }
        if (prop.key.name === 'runtime' && t.isStringLiteral(prop.value)) {
          runtime = prop.value.value; // 'csharp' | 'rust' | 'auto'
        }
        if (prop.key.name === 'parallel' && t.isBooleanLiteral(prop.value)) {
          parallel = prop.value.value;
        }
      }
    }
  }

  // Initialize component.useServerTask if needed
  component.useServerTask = component.useServerTask || [];

  // Store server task info
  component.useServerTask.push({
    name: taskName,
    asyncFunction: asyncFunction,
    parameters: parameters,
    isStreaming: streamingEnabled,
    estimatedChunks: estimatedChunks,
    returnType: extractReturnType(asyncFunction),
    runtime: runtime, // 'csharp' | 'rust' | 'auto'
    parallel: parallel // Enable Rayon parallel processing
  });
}

/**
 * Extract usePaginatedServerTask hook
 *
 * Detects: const users = usePaginatedServerTask(async ({ page, pageSize, filters }) => { ... }, options)
 * Generates TWO server tasks:
 *   1. Fetch task (with page params)
 *   2. Count task (from getTotalCount option)
 */
function extractUsePaginatedServerTask(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;

  const taskName = parent.id.name;
  const fetchFunction = path.node.arguments[0];
  const options = path.node.arguments[1];

  // Validate fetch function
  if (!fetchFunction || (!t.isArrowFunctionExpression(fetchFunction) && !t.isFunctionExpression(fetchFunction))) {
    console.warn('[usePaginatedServerTask] First argument must be an async function');
    return;
  }

  if (!fetchFunction.async) {
    console.warn('[usePaginatedServerTask] Function must be async');
    return;
  }

  // Extract fetch function parameters
  // Expected: ({ page, pageSize, filters }: PaginationParams<TFilter>) => Promise<T[]>
  const parameters = [
    { name: 'page', type: 'int' },
    { name: 'pageSize', type: 'int' },
    { name: 'filters', type: 'object' }
  ];

  // Extract options
  let runtime = 'csharp'; // Default to C#
  let parallel = false;
  let pageSize = 20;
  let getTotalCountFn = null;

  if (options && t.isObjectExpression(options)) {
    for (const prop of options.properties) {
      if (t.isObjectProperty(prop) && t.isIdentifier(prop.key)) {
        if (prop.key.name === 'runtime' && t.isStringLiteral(prop.value)) {
          runtime = prop.value.value;
        }
        if (prop.key.name === 'parallel' && t.isBooleanLiteral(prop.value)) {
          parallel = prop.value.value;
        }
        if (prop.key.name === 'pageSize' && t.isNumericLiteral(prop.value)) {
          pageSize = prop.value.value;
        }
        if (prop.key.name === 'getTotalCount') {
          getTotalCountFn = prop.value;
        }
      }
    }
  }

  // Initialize component.useServerTask if needed
  component.useServerTask = component.useServerTask || [];
  component.paginatedTasks = component.paginatedTasks || [];

  // 1. Add fetch task
  const fetchTaskName = `${taskName}_fetch`;
  component.useServerTask.push({
    name: fetchTaskName,
    asyncFunction: fetchFunction,
    parameters: parameters,
    isStreaming: false,
    estimatedChunks: null,
    returnType: 'List<object>', // Will be refined by type inference
    runtime: runtime,
    parallel: parallel
  });

  // 2. Add count task (if getTotalCount provided)
  let countTaskName = null;
  if (getTotalCountFn && (t.isArrowFunctionExpression(getTotalCountFn) || t.isFunctionExpression(getTotalCountFn))) {
    countTaskName = `${taskName}_count`;

    const countParameters = [
      { name: 'filters', type: 'object' }
    ];

    component.useServerTask.push({
      name: countTaskName,
      asyncFunction: getTotalCountFn,
      parameters: countParameters,
      isStreaming: false,
      estimatedChunks: null,
      returnType: 'int',
      runtime: runtime,
      parallel: false // Count queries don't need parallelization
    });
  }

  // Store pagination metadata
  component.paginatedTasks.push({
    name: taskName,
    fetchTaskName: fetchTaskName,
    countTaskName: countTaskName,
    pageSize: pageSize,
    runtime: runtime,
    parallel: parallel
  });

  console.log(`[usePaginatedServerTask] Extracted pagination tasks for '${taskName}':`, {
    fetch: fetchTaskName,
    count: countTaskName,
    runtime,
    parallel
  });
}

/**
 * Extract TypeScript type annotation
 */
function extractTypeAnnotation(typeAnnotation) {
  // Strip TSTypeAnnotation wrapper
  const actualType = typeAnnotation.typeAnnotation || typeAnnotation;

  if (t.isTSStringKeyword(actualType)) {
    return 'string';
  }
  if (t.isTSNumberKeyword(actualType)) {
    return 'double';
  }
  if (t.isTSBooleanKeyword(actualType)) {
    return 'bool';
  }
  if (t.isTSArrayType(actualType)) {
    const elementType = extractTypeAnnotation(actualType.elementType);
    return `List<${elementType}>`;
  }
  if (t.isTSTypeReference(actualType) && t.isIdentifier(actualType.typeName)) {
    return actualType.typeName.name; // Use custom type as-is
  }

  return 'object';
}

/**
 * Extract return type from async function
 */
function extractReturnType(asyncFunction) {
  // Check for explicit return type annotation
  if (asyncFunction.returnType) {
    const returnType = asyncFunction.returnType.typeAnnotation;

    // Promise<T> → T
    if (t.isTSTypeReference(returnType) &&
        t.isIdentifier(returnType.typeName) &&
        returnType.typeName.name === 'Promise') {
      if (returnType.typeParameters && returnType.typeParameters.params.length > 0) {
        return extractTypeAnnotation(returnType.typeParameters.params[0]);
      }
    }

    return extractTypeAnnotation(returnType);
  }

  // Try to infer from return statements
  // For now, default to object
  return 'object';
}

/**
 * Extract useMvcState hook
 *
 * Pattern: const [value, setValue] = useMvcState<T>('propertyName', options?)
 *
 * This hook accesses MVC ViewModel properties passed from the controller.
 * The babel plugin treats these as special client-side state that maps
 * to server ViewModel properties.
 */
function extractUseMvcState(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const elements = parent.id.elements;
  const propertyNameArg = path.node.arguments[0];

  // Extract property name (must be string literal)
  if (!t.isStringLiteral(propertyNameArg)) {
    console.warn('[useMvcState] Property name must be a string literal');
    return;
  }

  const propertyName = propertyNameArg.value;

  // useMvcState can return either [value] or [value, setter]
  // depending on mutability
  const stateVar = elements[0];
  const setterVar = elements.length > 1 ? elements[1] : null;

  // Extract TypeScript generic type: useMvcState<string>('name')
  // But prefer the type from the ViewModel interface if available (more reliable)
  const typeParam = path.node.typeParameters?.params[0];
  let csharpType = typeParam ? tsTypeToCSharpType(typeParam) : 'dynamic';

  // Try to find the actual type from the ViewModel interface
  const interfaceType = findViewModelPropertyType(path, propertyName, component);
  if (interfaceType) {
    csharpType = interfaceType;
    console.log(`[useMvcState] Found type for '${propertyName}' from interface: ${interfaceType}`);
  } else {
    console.log(`[useMvcState] Using generic type for '${propertyName}': ${csharpType}`);
  }

  // Initialize useMvcState array if needed
  component.useMvcState = component.useMvcState || [];

  const mvcStateInfo = {
    name: stateVar ? stateVar.name : null,
    setter: setterVar ? setterVar.name : null,
    propertyName: propertyName,
    type: csharpType  // ✅ Use type from interface (preferred) or generic fallback
  };

  component.useMvcState.push(mvcStateInfo);

  // Track as MVC state type
  if (stateVar) {
    component.stateTypes = component.stateTypes || new Map();
    component.stateTypes.set(stateVar.name, 'mvc');
  }
}

/**
 * Extract useMvcViewModel hook
 *
 * Pattern: const viewModel = useMvcViewModel<TViewModel>()
 *
 * This hook provides read-only access to the entire MVC ViewModel.
 * The babel plugin doesn't need to generate C# for this as it's
 * purely client-side access to the embedded ViewModel JSON.
 */
function extractUseMvcViewModel(path, component) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isIdentifier(parent.id)) return;

  const viewModelVarName = parent.id.name;

  // Initialize useMvcViewModel array if needed
  component.useMvcViewModel = component.useMvcViewModel || [];

  component.useMvcViewModel.push({
    name: viewModelVarName
  });

  // Note: This is primarily for documentation/tracking purposes.
  // The actual ViewModel access happens client-side via window.__MINIMACT_VIEWMODEL__
}

/**
 * Find the type of a property from the ViewModel interface
 *
 * Searches the AST for an interface named *ViewModel and extracts the property type
 */
function findViewModelPropertyType(path, propertyName, component) {
  // Find the program (top-level) node
  let programPath = path;
  while (programPath && !t.isProgram(programPath.node)) {
    programPath = programPath.parentPath;
  }

  if (!programPath) {
    console.log(`[findViewModelPropertyType] No program path found for ${propertyName}`);
    return null;
  }

  // ⚠️ CRITICAL: Check metadata first (interfaces stored before transformation)
  // The TranspilerService stores interfaces in metadata before @babel/preset-typescript strips them
  let viewModelInterface = null;
  const programNode = programPath.node;

  if (programNode.metadata && programNode.metadata.viewModelInterfaces) {
    const interfaces = programNode.metadata.viewModelInterfaces;
    console.log(`[findViewModelPropertyType] Found ${interfaces.length} interfaces in metadata`);

    for (const iface of interfaces) {
      if (iface.id && iface.id.name && iface.id.name.endsWith('ViewModel')) {
        viewModelInterface = iface;
        console.log(`[findViewModelPropertyType] ✅ Using interface from metadata: ${iface.id.name}`);
        break;
      }
    }
  } else {
    // Fallback: Search program body (won't work if TypeScript preset already ran)
    console.log(`[findViewModelPropertyType] No metadata found, searching program body`);

    if (!programNode || !programNode.body) {
      console.log(`[findViewModelPropertyType] No program body found`);
      return null;
    }

    console.log(`[findViewModelPropertyType] Program body has ${programNode.body.length} statements`);

    // Debug: Log all statement types
    programNode.body.forEach((stmt, idx) => {
      console.log(`[findViewModelPropertyType] Statement ${idx}: ${stmt.type}`);
    });

    // Iterate through top-level statements to find interface declarations
    let interfaceCount = 0;
    for (const statement of programNode.body) {
      if (t.isTSInterfaceDeclaration(statement)) {
        interfaceCount++;
        const interfaceName = statement.id.name;
        console.log(`[findViewModelPropertyType] Found interface #${interfaceCount}: ${interfaceName}`);

        // Look for interfaces ending with "ViewModel"
        if (interfaceName.endsWith('ViewModel')) {
          viewModelInterface = statement;
          console.log(`[findViewModelPropertyType] ✅ Using interface: ${interfaceName}`);
          break; // Use the first matching interface
        }
      }
    }

    console.log(`[findViewModelPropertyType] Total interfaces found: ${interfaceCount}`);
  }

  if (!viewModelInterface) {
    console.log(`[findViewModelPropertyType] ❌ No ViewModel interface found`);
    return null;
  }

  // Find the property in the interface
  for (const member of viewModelInterface.body.body) {
    if (t.isTSPropertySignature(member)) {
      const key = member.key;

      if (t.isIdentifier(key) && key.name === propertyName) {
        // Found the property! Extract its type
        const typeAnnotation = member.typeAnnotation?.typeAnnotation;
        console.log(`[findViewModelPropertyType] Found property ${propertyName}, typeAnnotation:`, typeAnnotation);
        if (typeAnnotation) {
          const csharpType = tsTypeToCSharpType(typeAnnotation);
          console.log(`[findViewModelPropertyType] Mapped ${propertyName} type to: ${csharpType}`);
          return csharpType;
        }
      }
    }
  }

  console.log(`[findViewModelPropertyType] Property ${propertyName} not found in interface`);
  return null;
}

/**
 * Extract custom hook call (e.g., useCounter('counter1', 0))
 * Custom hooks are treated as child components with lifted state
 */
function extractCustomHookCall(path, component, hookName) {
  const parent = path.parent;

  // Must be: const [x, y, z, ui] = useCounter('namespace', ...params)
  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const args = path.node.arguments;
  if (args.length === 0) return;

  // First argument must be namespace (string literal)
  const namespaceArg = args[0];
  if (!t.isStringLiteral(namespaceArg)) {
    console.warn(`[Custom Hook] ${hookName} first argument must be a string literal (namespace)`);
    return;
  }

  const namespace = namespaceArg.value;
  const hookParams = args.slice(1); // Remaining params become InitialState

  // Extract destructured variables
  const elements = parent.id.elements;

  // 🔥 NEW: Get hook metadata from imported hooks or inline hook
  let hookMetadata = null;

  // Check if this hook was imported
  if (component.importedHookMetadata && component.importedHookMetadata.has(hookName)) {
    hookMetadata = component.importedHookMetadata.get(hookName);
    console.log(`[Custom Hook] Using imported metadata for ${hookName}`);
  } else {
    // TODO: Check if hook is defined inline in same file
    console.log(`[Custom Hook] No metadata found for ${hookName}, assuming last return value is UI`);
  }

  // 🔥 NEW: Use returnValues from metadata to identify UI variable
  let uiVarName = null;
  if (hookMetadata && hookMetadata.returnValues) {
    // Find the JSX return value
    const jsxReturnIndex = hookMetadata.returnValues.findIndex(rv => rv.type === 'jsx');
    if (jsxReturnIndex !== -1 && jsxReturnIndex < elements.length) {
      const uiElement = elements[jsxReturnIndex];
      if (uiElement && t.isIdentifier(uiElement)) {
        uiVarName = uiElement.name;
        console.log(`[Custom Hook] Found UI variable from metadata at index ${jsxReturnIndex}: ${uiVarName}`);
      }
    }
  } else {
    // Fallback: Assume last element is UI (old behavior)
    const lastElement = elements[elements.length - 1];
    if (lastElement && t.isIdentifier(lastElement)) {
      uiVarName = lastElement.name;
      console.log(`[Custom Hook] Using fallback: assuming last element is UI: ${uiVarName}`);
    }
  }

  if (!uiVarName) {
    console.warn(`[Custom Hook] ${hookName} could not identify UI variable`);
    return;
  }

  // Store custom hook instance in component
  if (!component.customHooks) {
    component.customHooks = [];
  }

  const generate = require('@babel/generator').default;

  const className = hookMetadata ? hookMetadata.className : `${capitalize(hookName)}Hook`;

  component.customHooks.push({
    hookName,
    className,
    namespace,
    uiVarName, // Variable name that holds the UI (e.g., 'counterUI')
    params: hookParams.map(p => generate(p).code),
    returnValues: elements.map(e => e ? e.name : null).filter(Boolean),
    metadata: hookMetadata // 🔥 NEW: Store full metadata for later use
  });

  console.log(`[Custom Hook] Found ${hookName}('${namespace}') → UI in {${uiVarName}}`);
}

/**
 * Capitalize first letter
 */
function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

// ========================================
// 🔥 Client-Side Execution Helpers
// ========================================

const generate = require('@babel/generator').default;
const traverse = require('@babel/traverse').default;

/**
 * Analyze which hooks are used in a function body
 * Returns array like: ["useState", "useRef", "useEffect"]
 */
function analyzeHookUsage(callback) {
  const hooks = new Set();

  // Create a minimal program wrapper to provide proper scope
  const program = t.file(t.program([t.expressionStatement(callback)]));

  // Traverse the program (which provides proper scope)
  traverse(program, {
    CallExpression(path) {
      const callee = path.node.callee;

      // Check for direct hook calls: useState(), useRef(), etc.
      if (t.isIdentifier(callee)) {
        if (callee.name.startsWith('use') && /^use[A-Z]/.test(callee.name)) {
          hooks.add(callee.name);
        }
      }
    }
  });

  return Array.from(hooks);
}

/**
 * Transform effect callback:
 * - Arrow function → Regular function (for .bind() compatibility)
 * - Inject hook mappings at top: const useState = this.useState;
 * - Preserve async if present
 * - Preserve cleanup return value
 */
function transformEffectCallback(callback, hookCalls) {
  if (!t.isArrowFunctionExpression(callback) && !t.isFunctionExpression(callback)) {
    throw new Error('Effect callback must be a function');
  }

  let functionBody = callback.body;

  // If body is an expression, wrap in block statement
  if (!t.isBlockStatement(functionBody)) {
    functionBody = t.blockStatement([
      t.returnStatement(functionBody)
    ]);
  }

  // Build hook mapping statements
  // const useState = this.useState;
  // const useRef = this.useRef;
  const hookMappings = hookCalls.map(hookName => {
    return t.variableDeclaration('const', [
      t.variableDeclarator(
        t.identifier(hookName),
        t.memberExpression(
          t.thisExpression(),
          t.identifier(hookName)
        )
      )
    ]);
  });

  // Prepend hook mappings to function body
  const newBody = t.blockStatement([
    ...hookMappings,
    ...functionBody.body
  ]);

  // Return regular function expression
  return t.functionExpression(
    null,                    // No name (anonymous)
    callback.params,         // Keep original params (usually empty)
    newBody,                 // Body with hook mappings
    false,                   // Not a generator
    callback.async || false  // Preserve async
  );
}

/**
 * Transform event handler:
 * - Arrow function → Regular function
 * - Inject hook mappings at top
 * - Preserve event parameter (e)
 * - Preserve async if present
 */
function transformHandlerFunction(body, params, hookCalls) {
  let functionBody = body;

  // If body is expression, wrap in block
  if (!t.isBlockStatement(functionBody)) {
    functionBody = t.blockStatement([
      t.expressionStatement(functionBody)
    ]);
  }

  // Build hook mappings
  const hookMappings = hookCalls.map(hookName => {
    return t.variableDeclaration('const', [
      t.variableDeclarator(
        t.identifier(hookName),
        t.memberExpression(t.thisExpression(), t.identifier(hookName))
      )
    ]);
  });

  // Prepend hook mappings
  const newBody = t.blockStatement([
    ...hookMappings,
    ...functionBody.body
  ]);

  // Return regular function
  return t.functionExpression(
    null,
    params,        // Keep event parameter: (e) => ...
    newBody,
    false,
    false          // Handlers are typically not async (unless await inside)
  );
}

module.exports = {
  extractHook,
  extractUseState,
  extractUseProtectedState,
  extractUseEffect,
  extractUseRef,
  extractUseMarkdown,
  extractUseRazorMarkdown,
  extractUseTemplate,
  extractUseValidation,
  extractUseModal,
  extractUseToggle,
  extractUseDropdown,
  extractUsePub,
  extractUseSub,
  extractUseMicroTask,
  extractUseMacroTask,
  extractUseSignalR,
  extractUsePredictHint,
  extractUseServerTask,
  extractUseMvcState,
  extractUseMvcViewModel,
  // 🔥 Export new helpers
  analyzeHookUsage,
  transformEffectCallback,
  transformHandlerFunction
};