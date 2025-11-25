/**
 * C# File Generator
 */

const { generateComponent } = require('./component.cjs');
const { usesPlugins } = require('./plugin.cjs');

/**
 * Generate C# file from components
 */
function generateCSharpFile(components, state) {
  const lines = [];

  // Check if any component uses plugins
  const hasPlugins = components.some(c => usesPlugins(c));

  // Usings
  lines.push('using Minimact.AspNetCore.Core;');
  lines.push('using Minimact.AspNetCore.Extensions;');
  lines.push('using MinimactHelpers = Minimact.AspNetCore.Core.Minimact;');
  lines.push('using System.Collections.Generic;');
  lines.push('using System.Linq;');
  lines.push('using System.Threading.Tasks;');

  // Add plugin using directives if any component uses plugins
  if (hasPlugins) {
    lines.push('using Minimact.AspNetCore.Plugins;');
  }

  lines.push('');

  // Namespace (extract from file path or use default)
  const namespace = state.opts.namespace || 'Minimact.Components';
  lines.push(`namespace ${namespace};`);
  lines.push('');

  // Generate each component
  for (const component of components) {
    // Check if this is a custom hook with pre-generated code
    if (component.isHook && component.hookData) {
      // Use the pre-generated hook class code
      lines.push(component.hookData);
    } else {
      // Normal component generation
      lines.push(...generateComponent(component));
    }
    lines.push('');
  }

  return lines.join('\n');
}


module.exports = {
  generateCSharpFile
};
