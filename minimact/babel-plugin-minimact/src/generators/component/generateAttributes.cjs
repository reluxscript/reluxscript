/**
 * Attribute generators for component class
 */

/**
 * Generate loop template attributes
 */
function generateLoopTemplateAttributes(component, lines) {
  if (component.loopTemplates && component.loopTemplates.length > 0) {
    for (const loopTemplate of component.loopTemplates) {
      const templateJson = JSON.stringify(loopTemplate)
        .replace(/"/g, '""'); // Escape quotes for C# verbatim string

      lines.push(`[LoopTemplate("${loopTemplate.stateKey}", @"${templateJson}")]`);
    }
  }
}

/**
 * Generate StateX projection attributes
 */
function generateStateXAttributes(component, lines) {
  if (component.useStateX && component.useStateX.length > 0) {
    for (let i = 0; i < component.useStateX.length; i++) {
      const stateX = component.useStateX[i];
      const stateKey = `stateX_${i}`;

      for (const target of stateX.targets) {
        const parts = [];

        // Required: stateKey and selector
        parts.push(`"${stateKey}"`);
        parts.push(`"${target.selector}"`);

        // Optional: Transform (C# lambda)
        if (target.transform) {
          parts.push(`Transform = @"${target.transform}"`);
        }

        // Optional: TransformId (registry reference)
        if (target.transformId) {
          parts.push(`TransformId = "${target.transformId}"`);
        }

        // Optional: ApplyAs mode
        if (target.applyAs && target.applyAs !== 'textContent') {
          parts.push(`ApplyAs = "${target.applyAs}"`);
        }

        // Optional: Property name
        if (target.property) {
          parts.push(`Property = "${target.property}"`);
        }

        // Optional: ApplyIf condition
        if (target.applyIf && target.applyIf.csharpCode) {
          parts.push(`ApplyIf = @"${target.applyIf.csharpCode}"`);
        }

        // Optional: Template hint
        if (target.template) {
          parts.push(`Template = "${target.template}"`);
        }

        lines.push(`[StateXTransform(${parts.join(', ')})]`);
      }
    }
  }
}

module.exports = {
  generateLoopTemplateAttributes,
  generateStateXAttributes
};
