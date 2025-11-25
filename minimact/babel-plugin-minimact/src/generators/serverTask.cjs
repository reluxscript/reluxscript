/**
 * Server Task Generator
 *
 * Generates C# async Task methods from useServerTask calls
 */

const { transpileAsyncFunctionToCSharp } = require('../transpilers/typescriptToCSharp.cjs');

/**
 * Generate C# server task methods
 */
function generateServerTaskMethods(component) {
  if (!component.useServerTask || component.useServerTask.length === 0) {
    return [];
  }

  const lines = [];

  for (let i = 0; i < component.useServerTask.length; i++) {
    const task = component.useServerTask[i];
    const taskId = `serverTask_${i}`;

    // Generate method
    lines.push('');
    lines.push(`    [ServerTask("${taskId}"${task.isStreaming ? ', Streaming = true' : ''})]`);

    // Method signature
    const returnType = task.isStreaming
      ? `IAsyncEnumerable<${task.returnType}>`
      : `Task<${task.returnType}>`;

    const params = [];

    // Add user parameters
    for (const param of task.parameters) {
      params.push(`${param.type} ${param.name}`);
    }

    // Add progress parameter (non-streaming only)
    if (!task.isStreaming) {
      params.push('IProgress<double> progress');
    }

    // Add cancellation token
    if (task.isStreaming) {
      params.push('[EnumeratorCancellation] CancellationToken cancellationToken = default');
    } else {
      params.push('CancellationToken cancellationToken');
    }

    const methodName = capitalize(taskId);
    const paramsList = params.join(', ');

    lines.push(`    private async ${returnType} ${methodName}(${paramsList})`);
    lines.push(`    {`);

    // Transpile function body
    const csharpBody = transpileAsyncFunctionToCSharp(task.asyncFunction);
    const indentedBody = indent(csharpBody, 8);

    lines.push(indentedBody);
    lines.push(`    }`);
  }

  return lines;
}

/**
 * Capitalize first letter
 */
function capitalize(str) {
  if (!str) return '';
  return str.charAt(0).toUpperCase() + str.slice(1);
}

/**
 * Indent code
 */
function indent(code, spaces) {
  const prefix = ' '.repeat(spaces);
  return code.split('\n').map(line => line ? prefix + line : '').join('\n');
}

module.exports = {
  generateServerTaskMethods
};
