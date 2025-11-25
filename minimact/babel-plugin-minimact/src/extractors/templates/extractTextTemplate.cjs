const t = require('@babel/types');
const { extractBinding } = require('./extractBinding.cjs');
const { extractTemplateLiteral } = require('./extractTemplateLiteral.cjs');

/**
 * Extract template from mixed text/expression children
 * Example: <h1>Count: {count}</h1> → "Count: {0}"
 */
function extractTextTemplate(children, currentPath, textIndex, component) {
  let templateStr = '';
  const bindings = [];
  const slots = [];
  let paramIndex = 0;
  let hasExpressions = false;
  let conditionalTemplates = null;
  let transformMetadata = null;
  let nullableMetadata = null;

  for (const child of children) {
    if (t.isJSXText(child)) {
      const text = child.value;
      templateStr += text;
    } else if (t.isJSXExpressionContainer(child)) {
      hasExpressions = true;

      // Special case: Template literal inside JSX expression container
      // Example: {`${(discount * 100).toFixed(0)}%`}
      if (t.isTemplateLiteral(child.expression)) {
        const templateResult = extractTemplateLiteral(child.expression, component);
        if (templateResult) {
          // Merge the template literal's content into the current template
          templateStr += templateResult.template;
          // Add the template literal's bindings
          for (const binding of templateResult.bindings) {
            bindings.push(binding);
          }
          // Store transforms and conditionals if present
          if (templateResult.transforms && templateResult.transforms.length > 0) {
            transformMetadata = templateResult.transforms[0]; // Simplified: take first transform
          }
          if (templateResult.conditionals && templateResult.conditionals.length > 0) {
            conditionalTemplates = {
              true: templateResult.conditionals[0].trueValue,
              false: templateResult.conditionals[0].falseValue
            };
          }
          paramIndex++;
          continue; // Skip normal binding extraction
        }
      }

      const binding = extractBinding(child.expression, component);

      if (binding && typeof binding === 'object' && binding.conditional) {
        // Conditional binding (ternary)
        slots.push(templateStr.length);
        templateStr += `{${paramIndex}}`;
        bindings.push(binding.conditional);

        // Store conditional template values
        conditionalTemplates = {
          true: binding.trueValue,
          false: binding.falseValue
        };

        paramIndex++;
      } else if (binding && typeof binding === 'object' && binding.transform) {
        // Transform binding (method call)
        slots.push(templateStr.length);
        templateStr += `{${paramIndex}}`;
        bindings.push(binding.binding);

        // Store transform metadata
        transformMetadata = {
          method: binding.transform,
          args: binding.args
        };

        paramIndex++;
      } else if (binding && typeof binding === 'object' && binding.nullable) {
        // Nullable binding (optional chaining)
        slots.push(templateStr.length);
        templateStr += `{${paramIndex}}`;
        bindings.push(binding.binding);

        // Mark as nullable
        nullableMetadata = true;

        paramIndex++;
      } else if (binding) {
        // Simple binding (string)
        slots.push(templateStr.length);
        templateStr += `{${paramIndex}}`;
        bindings.push(binding);
        paramIndex++;
      } else {
        // Complex expression - can't template it
        templateStr += `{${paramIndex}}`;
        bindings.push('__complex__');
        paramIndex++;
      }
    }
  }

  // Clean up whitespace
  templateStr = templateStr.trim();

  if (!hasExpressions) return null;

  // Determine template type
  let templateType = 'dynamic';
  if (conditionalTemplates) {
    templateType = 'conditional';
  } else if (transformMetadata) {
    templateType = 'transform';
  } else if (nullableMetadata) {
    templateType = 'nullable';
  }

  const result = {
    template: templateStr,
    bindings,
    slots,
    path: currentPath,
    type: templateType
  };

  // Add conditional template values if present
  if (conditionalTemplates) {
    result.conditionalTemplates = conditionalTemplates;
  }

  // Add transform metadata if present
  if (transformMetadata) {
    result.transform = transformMetadata;
  }

  // Add nullable flag if present
  if (nullableMetadata) {
    result.nullable = true;
  }

  return result;
}

module.exports = { extractTextTemplate };
