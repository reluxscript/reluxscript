/**
 * Field generators for component class
 */

const { inferCSharpTypeFromInit } = require('./inferTypes.cjs');

/**
 * Generate template properties (from useTemplate)
 */
function generateTemplateProperties(component, lines) {
  if (component.useTemplate && component.useTemplate.props) {
    for (const [propName, propValue] of Object.entries(component.useTemplate.props)) {
      // Capitalize first letter for C# property name
      const csharpPropName = propName.charAt(0).toUpperCase() + propName.slice(1);
      lines.push(`    public override string ${csharpPropName} => "${propValue}";`);
      lines.push('');
    }
  }
}

/**
 * Generate prop fields
 */
function generatePropFields(component, lines) {
  for (const prop of component.props) {
    lines.push(`    [Prop]`);
    lines.push(`    public ${prop.type} ${prop.name} { get; set; }`);
    lines.push('');
  }
}

/**
 * Generate state fields (useState)
 */
function generateStateFields(component, lines) {
  for (const state of component.useState) {
    lines.push(`    [State]`);
    lines.push(`    private ${state.type} ${state.name} = ${state.initialValue};`);
    lines.push('');
  }
}

/**
 * Generate MVC state fields (useMvcState)
 */
function generateMvcStateFields(component, lines) {
  if (component.useMvcState) {
    for (const mvcState of component.useMvcState) {
      const csharpType = mvcState.type || 'dynamic';
      lines.push(`    // MVC State property: ${mvcState.propertyName}`);
      lines.push(`    private ${csharpType} ${mvcState.name} => GetState<${csharpType}>("${mvcState.propertyName}");`);
      lines.push('');
    }
  }
}

/**
 * Generate MVC ViewModel fields (useMvcViewModel)
 */
function generateMvcViewModelFields(component, lines) {
  if (component.useMvcViewModel) {
    for (const viewModel of component.useMvcViewModel) {
      lines.push(`    // useMvcViewModel - read-only access to entire ViewModel`);
      lines.push(`    private dynamic ${viewModel.name} = null;`);
      lines.push('');
    }
  }
}

/**
 * Generate StateX fields
 */
function generateStateXFields(component, lines) {
  for (const stateX of component.useStateX) {
    lines.push(`    [State]`);
    lines.push(`    private ${stateX.initialValueType} ${stateX.varName} = ${stateX.initialValue};`);
    lines.push('');
  }
}

/**
 * Generate ref fields (useRef)
 */
function generateRefFields(component, lines) {
  for (const ref of component.useRef) {
    lines.push(`    [Ref]`);
    lines.push(`    private object ${ref.name} = ${ref.initialValue};`);
    lines.push('');
  }
}

/**
 * Generate markdown fields (useMarkdown)
 */
function generateMarkdownFields(component, lines) {
  for (const md of component.useMarkdown) {
    lines.push(`    [Markdown]`);
    lines.push(`    [State]`);
    lines.push(`    private string ${md.name} = ${md.initialValue};`);
    lines.push('');
  }
}

/**
 * Generate Razor markdown fields (useRazorMarkdown)
 */
function generateRazorMarkdownFields(component, lines) {
  if (component.useRazorMarkdown) {
    for (const md of component.useRazorMarkdown) {
      lines.push(`    [RazorMarkdown]`);
      lines.push(`    [State]`);
      lines.push(`    private string ${md.name} = null!;`);
      lines.push('');
    }
  }
}

/**
 * Generate validation fields (useValidation)
 */
function generateValidationFields(component, lines) {
  for (const validation of component.useValidation) {
    lines.push(`    [Validation]`);
    lines.push(`    private ValidationField ${validation.name} = new ValidationField`);
    lines.push(`    {`);
    lines.push(`        FieldKey = "${validation.fieldKey}",`);

    // Add validation rules
    if (validation.rules.required) {
      lines.push(`        Required = ${validation.rules.required.toString().toLowerCase()},`);
    }
    if (validation.rules.minLength) {
      lines.push(`        MinLength = ${validation.rules.minLength},`);
    }
    if (validation.rules.maxLength) {
      lines.push(`        MaxLength = ${validation.rules.maxLength},`);
    }
    if (validation.rules.pattern) {
      lines.push(`        Pattern = @"${validation.rules.pattern}",`);
    }
    if (validation.rules.message) {
      lines.push(`        Message = "${validation.rules.message}"`);
    }

    lines.push(`    };`);
    lines.push('');
  }
}

/**
 * Generate modal fields (useModal)
 */
function generateModalFields(component, lines) {
  for (const modal of component.useModal) {
    lines.push(`    private ModalState ${modal.name} = new ModalState();`);
    lines.push('');
  }
}

/**
 * Generate toggle fields (useToggle)
 */
function generateToggleFields(component, lines) {
  for (const toggle of component.useToggle) {
    lines.push(`    [State]`);
    lines.push(`    private bool ${toggle.name} = ${toggle.initialValue};`);
    lines.push('');
  }
}

/**
 * Generate dropdown fields (useDropdown)
 */
function generateDropdownFields(component, lines) {
  for (const dropdown of component.useDropdown) {
    lines.push(`    private DropdownState ${dropdown.name} = new DropdownState();`);
    lines.push('');
  }
}

/**
 * Generate pub/sub fields
 */
function generatePubSubFields(component, lines) {
  // Pub fields
  if (component.usePub) {
    for (const pub of component.usePub) {
      const channelStr = pub.channel ? `"${pub.channel}"` : 'null';
      lines.push(`    // usePub: ${pub.name}`);
      lines.push(`    private string ${pub.name}_channel = ${channelStr};`);
      lines.push('');
    }
  }

  // Sub fields
  if (component.useSub) {
    for (const sub of component.useSub) {
      const channelStr = sub.channel ? `"${sub.channel}"` : 'null';
      lines.push(`    // useSub: ${sub.name}`);
      lines.push(`    private string ${sub.name}_channel = ${channelStr};`);
      lines.push(`    private dynamic ${sub.name}_value = null;`);
      lines.push('');
    }
  }
}

/**
 * Generate task scheduling fields
 */
function generateTaskSchedulingFields(component, lines) {
  // MicroTask fields
  if (component.useMicroTask) {
    for (let i = 0; i < component.useMicroTask.length; i++) {
      lines.push(`    // useMicroTask ${i}`);
      lines.push(`    private bool _microTaskScheduled_${i} = false;`);
      lines.push('');
    }
  }

  // MacroTask fields
  if (component.useMacroTask) {
    for (let i = 0; i < component.useMacroTask.length; i++) {
      const task = component.useMacroTask[i];
      lines.push(`    // useMacroTask ${i} (delay: ${task.delay}ms)`);
      lines.push(`    private bool _macroTaskScheduled_${i} = false;`);
      lines.push('');
    }
  }
}

/**
 * Generate SignalR fields
 */
function generateSignalRFields(component, lines) {
  if (component.useSignalR) {
    for (const signalR of component.useSignalR) {
      const hubUrlStr = signalR.hubUrl ? `"${signalR.hubUrl}"` : 'null';
      lines.push(`    // useSignalR: ${signalR.name}`);
      lines.push(`    private string ${signalR.name}_hubUrl = ${hubUrlStr};`);
      lines.push(`    private bool ${signalR.name}_connected = false;`);
      lines.push(`    private string ${signalR.name}_connectionId = null;`);
      lines.push(`    private string ${signalR.name}_error = null;`);
      lines.push('');
    }
  }
}

/**
 * Generate predict hint fields
 */
function generatePredictHintFields(component, lines) {
  if (component.usePredictHint) {
    for (let i = 0; i < component.usePredictHint.length; i++) {
      const hint = component.usePredictHint[i];
      const hintIdStr = hint.hintId ? `"${hint.hintId}"` : `"hint_${i}"`;
      lines.push(`    // usePredictHint: ${hintIdStr}`);
      lines.push(`    private string _hintId_${i} = ${hintIdStr};`);
      lines.push('');
    }
  }
}

/**
 * Generate client-computed properties
 */
function generateClientComputedProperties(component, lines) {
  const clientComputedVars = component.localVariables.filter(v => v.isClientComputed);
  if (clientComputedVars.length > 0) {
    lines.push('    // Client-computed properties (external libraries)');
    for (const clientVar of clientComputedVars) {
      const csharpType = inferCSharpTypeFromInit(clientVar.init);
      lines.push(`    [ClientComputed("${clientVar.name}")]`);
      lines.push(`    private ${csharpType} ${clientVar.name} => GetClientState<${csharpType}>("${clientVar.name}", default);`);
      lines.push('');
    }
  }
}

module.exports = {
  generateTemplateProperties,
  generatePropFields,
  generateStateFields,
  generateMvcStateFields,
  generateMvcViewModelFields,
  generateStateXFields,
  generateRefFields,
  generateMarkdownFields,
  generateRazorMarkdownFields,
  generateValidationFields,
  generateModalFields,
  generateToggleFields,
  generateDropdownFields,
  generatePubSubFields,
  generateTaskSchedulingFields,
  generateSignalRFields,
  generatePredictHintFields,
  generateClientComputedProperties
};
