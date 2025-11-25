/**
 * Hook Signature Extractor
 *
 * Detects structural changes in hook usage (additions/removals/reordering)
 * for hot reload instance replacement.
 */

const fs = require('fs');
const path = require('path');

/**
 * Extract hook signature from component
 *
 * Returns array of hook metadata for structural change detection
 */
function extractHookSignature(component) {
  const hooks = [];
  let index = 0;

  // Extract useState
  for (const stateInfo of component.useState) {
    hooks.push({
      type: 'useState',
      varName: stateInfo.name,
      index: index++
    });
  }

  // Extract useClientState
  for (const stateInfo of component.useClientState) {
    hooks.push({
      type: 'useClientState',
      varName: stateInfo.name,
      index: index++
    });
  }

  // Extract useStateX (declarative state projections)
  if (component.useStateX) {
    for (const stateInfo of component.useStateX) {
      hooks.push({
        type: 'useStateX',
        varName: stateInfo.name,
        index: index++
      });
    }
  }

  // Extract useEffect
  for (const effect of component.useEffect) {
    const depsCount = effect.dependencies
      ? (effect.dependencies.elements ? effect.dependencies.elements.length : -1)
      : -1; // -1 = no deps array (runs every render)

    hooks.push({
      type: 'useEffect',
      depsCount: depsCount,
      index: index++
    });
  }

  // Extract useRef
  for (const refInfo of component.useRef) {
    hooks.push({
      type: 'useRef',
      varName: refInfo.name,
      index: index++
    });
  }

  // Extract useMarkdown
  for (const markdownInfo of component.useMarkdown) {
    hooks.push({
      type: 'useMarkdown',
      varName: markdownInfo.name,
      index: index++
    });
  }

  // Extract useRazorMarkdown
  if (component.useRazorMarkdown) {
    for (const razorInfo of component.useRazorMarkdown) {
      hooks.push({
        type: 'useRazorMarkdown',
        varName: razorInfo.name,
        index: index++
      });
    }
  }

  // Extract useTemplate
  if (component.useTemplate) {
    hooks.push({
      type: 'useTemplate',
      templateName: component.useTemplate.name,
      index: index++
    });
  }

  // Extract useValidation
  for (const validation of component.useValidation) {
    hooks.push({
      type: 'useValidation',
      varName: validation.name,
      fieldKey: validation.fieldKey,
      index: index++
    });
  }

  // Extract useModal
  for (const modal of component.useModal) {
    hooks.push({
      type: 'useModal',
      varName: modal.name,
      index: index++
    });
  }

  // Extract useToggle
  for (const toggle of component.useToggle) {
    hooks.push({
      type: 'useToggle',
      varName: toggle.name,
      index: index++
    });
  }

  // Extract useDropdown
  for (const dropdown of component.useDropdown) {
    hooks.push({
      type: 'useDropdown',
      varName: dropdown.name,
      index: index++
    });
  }

  // Extract usePub
  if (component.usePub) {
    for (const pub of component.usePub) {
      hooks.push({
        type: 'usePub',
        varName: pub.name,
        channel: pub.channel,
        index: index++
      });
    }
  }

  // Extract useSub
  if (component.useSub) {
    for (const sub of component.useSub) {
      hooks.push({
        type: 'useSub',
        varName: sub.name,
        channel: sub.channel,
        index: index++
      });
    }
  }

  // Extract useMicroTask
  if (component.useMicroTask) {
    for (const _ of component.useMicroTask) {
      hooks.push({
        type: 'useMicroTask',
        index: index++
      });
    }
  }

  // Extract useMacroTask
  if (component.useMacroTask) {
    for (const task of component.useMacroTask) {
      hooks.push({
        type: 'useMacroTask',
        delay: task.delay,
        index: index++
      });
    }
  }

  // Extract useSignalR
  if (component.useSignalR) {
    for (const signalR of component.useSignalR) {
      hooks.push({
        type: 'useSignalR',
        varName: signalR.name,
        hubUrl: signalR.hubUrl,
        index: index++
      });
    }
  }

  // Extract usePredictHint
  if (component.usePredictHint) {
    for (const hint of component.usePredictHint) {
      hooks.push({
        type: 'usePredictHint',
        hintId: hint.hintId,
        index: index++
      });
    }
  }

  // Extract useServerTask
  if (component.useServerTask) {
    for (const task of component.useServerTask) {
      hooks.push({
        type: 'useServerTask',
        varName: task.name,
        runtime: task.runtime,
        isStreaming: task.isStreaming,
        index: index++
      });
    }
  }

  // Extract usePaginatedServerTask (tracked via paginatedTasks)
  if (component.paginatedTasks) {
    for (const pagTask of component.paginatedTasks) {
      hooks.push({
        type: 'usePaginatedServerTask',
        varName: pagTask.name,
        runtime: pagTask.runtime,
        index: index++
      });
    }
  }

  // Extract useMvcState
  if (component.useMvcState) {
    for (const mvcState of component.useMvcState) {
      hooks.push({
        type: 'useMvcState',
        varName: mvcState.name,
        propertyName: mvcState.propertyName,
        index: index++
      });
    }
  }

  // Extract useMvcViewModel
  if (component.useMvcViewModel) {
    for (const mvcViewModel of component.useMvcViewModel) {
      hooks.push({
        type: 'useMvcViewModel',
        varName: mvcViewModel.name,
        index: index++
      });
    }
  }

  return hooks;
}

/**
 * Write hook signature to file
 */
function writeHookSignature(componentName, hooks, inputFilePath) {
  const signature = {
    componentName: componentName,
    timestamp: new Date().toISOString(),
    hooks: hooks
  };

  const outputDir = path.dirname(inputFilePath);
  const signatureFilePath = path.join(outputDir, `${componentName}.hooks.json`);

  try {
    fs.writeFileSync(signatureFilePath, JSON.stringify(signature, null, 2));
    console.log(`[Hook Signature] ✅ Wrote ${path.basename(signatureFilePath)} with ${hooks.length} hooks`);
  } catch (error) {
    console.error(`[Hook Signature] Failed to write ${signatureFilePath}:`, error);
  }
}

/**
 * Read previous hook signature from file
 */
function readPreviousHookSignature(componentName, inputFilePath) {
  const outputDir = path.dirname(inputFilePath);
  const signatureFilePath = path.join(outputDir, `${componentName}.hooks.json`);

  if (!fs.existsSync(signatureFilePath)) {
    return null; // First transpilation
  }

  try {
    const json = fs.readFileSync(signatureFilePath, 'utf-8');
    const signature = JSON.parse(json);
    console.log(`[Hook Signature] 📖 Read ${path.basename(signatureFilePath)} with ${signature.hooks.length} hooks`);
    return signature.hooks;
  } catch (error) {
    console.error(`[Hook Signature] Failed to read ${signatureFilePath}:`, error);
    return null;
  }
}

/**
 * Compare two hook signatures and detect changes
 */
function compareHookSignatures(previousHooks, currentHooks) {
  const changes = [];

  // Check if hook count changed
  if (previousHooks.length !== currentHooks.length) {
    console.log(`[Hook Changes] Hook count changed: ${previousHooks.length} → ${currentHooks.length}`);
  }

  // Compare each hook by index
  const maxLength = Math.max(previousHooks.length, currentHooks.length);

  for (let i = 0; i < maxLength; i++) {
    const prevHook = previousHooks[i];
    const currHook = currentHooks[i];

    if (!prevHook && currHook) {
      // Hook added
      const hookDesc = getHookDescription(currHook);
      console.log(`[Hook Changes] 🆕 Hook added at index ${i}: ${hookDesc}`);
      changes.push({
        type: 'hook-added',
        hookType: currHook.type,
        varName: currHook.varName,
        index: i
      });
    } else if (prevHook && !currHook) {
      // Hook removed
      const hookDesc = getHookDescription(prevHook);
      console.log(`[Hook Changes] 🗑️  Hook removed at index ${i}: ${hookDesc}`);
      changes.push({
        type: 'hook-removed',
        hookType: prevHook.type,
        varName: prevHook.varName,
        index: i
      });
    } else if (prevHook && currHook) {
      // Check if hook type changed
      if (prevHook.type !== currHook.type) {
        console.log(`[Hook Changes] 🔄 Hook type changed at index ${i}: ${prevHook.type} → ${currHook.type}`);
        changes.push({
          type: 'hook-type-changed',
          oldHookType: prevHook.type,
          newHookType: currHook.type,
          index: i
        });
      }

      // Check if variable name changed (for hooks with variables)
      if (prevHook.varName && currHook.varName && prevHook.varName !== currHook.varName) {
        console.log(`[Hook Changes] 🔄 Hook variable changed at index ${i}: ${prevHook.varName} → ${currHook.varName}`);
        changes.push({
          type: 'hook-variable-changed',
          hookType: currHook.type,
          oldVarName: prevHook.varName,
          newVarName: currHook.varName,
          index: i
        });
      }

      // Check if property name changed (for useMvcState)
      if (prevHook.propertyName && currHook.propertyName && prevHook.propertyName !== currHook.propertyName) {
        console.log(`[Hook Changes] 🔄 useMvcState property changed at index ${i}: ${prevHook.propertyName} → ${currHook.propertyName}`);
        changes.push({
          type: 'hook-property-changed',
          hookType: 'useMvcState',
          oldPropertyName: prevHook.propertyName,
          newPropertyName: currHook.propertyName,
          index: i
        });
      }

      // Check if channel changed (for usePub/useSub)
      if (prevHook.channel !== undefined && currHook.channel !== undefined && prevHook.channel !== currHook.channel) {
        console.log(`[Hook Changes] 🔄 ${currHook.type} channel changed at index ${i}: ${prevHook.channel} → ${currHook.channel}`);
        // Note: Channel change is NOT structural (doesn't affect C# fields), so we don't add it to changes
        // But we log it for visibility
      }

      // Check if runtime changed (for useServerTask/usePaginatedServerTask)
      if (prevHook.runtime && currHook.runtime && prevHook.runtime !== currHook.runtime) {
        console.log(`[Hook Changes] 🔄 ${currHook.type} runtime changed at index ${i}: ${prevHook.runtime} → ${currHook.runtime}`);
        changes.push({
          type: 'hook-runtime-changed',
          hookType: currHook.type,
          oldRuntime: prevHook.runtime,
          newRuntime: currHook.runtime,
          index: i
        });
      }

      // Check if deps count changed (for useEffect)
      // NOTE: Deps count change is NOT a structural change (doesn't affect C# fields)
      // The effect body and registration stay the same, only execution timing changes
      // So we log it but don't add to structural changes
      if (prevHook.depsCount !== undefined &&
          currHook.depsCount !== undefined &&
          prevHook.depsCount !== currHook.depsCount) {
        console.log(`[Hook Changes] ℹ️  useEffect deps count changed at index ${i}: ${prevHook.depsCount} → ${currHook.depsCount} deps (NOT structural)`);
      }

      // Check if streaming changed (for useServerTask)
      if (prevHook.isStreaming !== undefined && currHook.isStreaming !== undefined && prevHook.isStreaming !== currHook.isStreaming) {
        console.log(`[Hook Changes] 🔄 useServerTask streaming changed at index ${i}: ${prevHook.isStreaming} → ${currHook.isStreaming}`);
        changes.push({
          type: 'hook-streaming-changed',
          hookType: 'useServerTask',
          oldStreaming: prevHook.isStreaming,
          newStreaming: currHook.isStreaming,
          index: i
        });
      }
    }
  }

  return changes;
}

/**
 * Get a human-readable description of a hook
 */
function getHookDescription(hook) {
  if (hook.varName) {
    return `${hook.type} (${hook.varName})`;
  }
  if (hook.templateName) {
    return `${hook.type} (${hook.templateName})`;
  }
  if (hook.hintId) {
    return `${hook.type} (${hook.hintId})`;
  }
  if (hook.fieldKey) {
    return `${hook.type} (${hook.fieldKey})`;
  }
  if (hook.propertyName) {
    return `${hook.type} (${hook.propertyName})`;
  }
  if (hook.channel) {
    return `${hook.type} (${hook.channel})`;
  }
  return hook.type;
}

module.exports = {
  extractHookSignature,
  writeHookSignature,
  readPreviousHookSignature,
  compareHookSignatures
};
