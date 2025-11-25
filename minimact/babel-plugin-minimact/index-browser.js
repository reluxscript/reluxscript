/**
 * Minimact Babel Plugin - Browser Build
 *
 * This is a browser-friendly entry point that expects Babel to be available globally
 */

// Import the core plugin logic (will be bundled by Rollup)
const { processComponent } = require('./src/processComponent.cjs');
const { generateCSharpFile } = require('./src/generators/csharpFile.cjs');

module.exports = function(babel) {
  // Extract types from the babel object
  const t = babel.types;
  const { traverse } = babel;

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
      },

      ExportDefaultDeclaration(path, state) {
        const declaration = path.node.declaration;
        if (declaration.type === 'FunctionDeclaration' ||
            declaration.type === 'ArrowFunctionExpression' ||
            declaration.type === 'FunctionExpression') {
          processComponent(path.get('declaration'), state);
        }
      }
    }
  };
};
