/**
 * Component Generator
 */

const { setCurrentComponent } = require('./expressions.cjs');
const { generateServerTaskMethods } = require('./serverTask.cjs');
const { generateTimelineAttributes } = require('./timelineGenerator.cjs');

const {
  generateLoopTemplateAttributes,
  generateStateXAttributes
} = require('./component/generateAttributes.cjs');

const {
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
} = require('./component/generateFields.cjs');

const {
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
} = require('./component/generateMethods.cjs');

const { inferCSharpTypeFromInit } = require('./component/inferTypes.cjs');

/**
 * Generate C# class for a component
 */
function generateComponent(component) {
  // Set the current component context for useState setter detection
  setCurrentComponent(component);

  const lines = [];

  // Timeline attributes (for @minimact/timeline)
  if (component.timeline) {
    const timelineAttrs = generateTimelineAttributes(component.timeline);
    timelineAttrs.forEach(attr => lines.push(attr));
  }

  // Loop template attributes
  generateLoopTemplateAttributes(component, lines);

  // StateX projection attributes
  generateStateXAttributes(component, lines);

  // Class declaration
  lines.push('[Component]');

  const baseClass = component.useTemplate
    ? component.useTemplate.name
    : 'MinimactComponent';

  lines.push(`public partial class ${component.name} : ${baseClass}`);
  lines.push('{');

  // Generate all fields
  generateTemplateProperties(component, lines);
  generatePropFields(component, lines);
  generateStateFields(component, lines);
  generateMvcStateFields(component, lines);
  generateMvcViewModelFields(component, lines);
  generateStateXFields(component, lines);
  generateRefFields(component, lines);
  generateMarkdownFields(component, lines);
  generateRazorMarkdownFields(component, lines);
  generateValidationFields(component, lines);
  generateModalFields(component, lines);
  generateToggleFields(component, lines);
  generateDropdownFields(component, lines);
  generatePubSubFields(component, lines);
  generateTaskSchedulingFields(component, lines);
  generateSignalRFields(component, lines);
  generatePredictHintFields(component, lines);
  generateClientComputedProperties(component, lines);

  // Server Task methods (useServerTask)
  const serverTaskMethods = generateServerTaskMethods(component);
  for (const line of serverTaskMethods) {
    lines.push(line);
  }

  // Generate all methods
  generateRenderMethod(component, lines);
  generateEffectMethods(component, lines);
  generateEventHandlers(component, lines);
  generateToggleMethods(component, lines);
  generateClientHandlersMethod(component, lines);
  generateClientEffectsMethod(component, lines);
  generatePubSubMethods(component, lines);
  generateSignalRMethods(component, lines);
  generateMvcStateSetters(component, lines);
  generateOnInitializedMethod(component, lines);
  generateHelperFunctions(component, lines);

  lines.push('}');

  return lines;
}

module.exports = {
  generateComponent,
  inferCSharpTypeFromInit
};
