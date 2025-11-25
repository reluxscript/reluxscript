/**
 * Hook Import Analyzer
 *
 * Handles cross-file custom hook imports
 * When a component imports a custom hook from another file,
 * we need to analyze that file to understand the hook's signature
 */

const t = require('@babel/types');
const fs = require('fs');
const path = require('path');
const { isCustomHook, getHookName } = require('./hookDetector.cjs');
const { analyzeHook } = require('./hookAnalyzer.cjs');

/**
 * Analyze imported hooks from relative imports
 *
 * @param {NodePath} filePath - Path to the component file
 * @param {Object} state - Babel state
 * @returns {Map<string, Object>} - Map of hook name to hook metadata
 */
function analyzeImportedHooks(filePath, state) {
  const importedHooks = new Map();
  const currentFilePath = state.file.opts.filename;
  const currentDir = path.dirname(currentFilePath);

  console.log(`[DEBUG hookImports] Current file: ${currentFilePath}`);
  console.log(`[DEBUG hookImports] Current dir: ${currentDir}`);

  // Find all import declarations
  filePath.traverse({
    ImportDeclaration(importPath) {
      const source = importPath.node.source.value;
      console.log(`[DEBUG hookImports] Found import: ${source}`);

      // Only process relative imports (potential hook files)
      if (!source.startsWith('./') && !source.startsWith('../')) {
        console.log(`[DEBUG hookImports] Skipping non-relative import: ${source}`);
        return;
      }

      console.log(`[DEBUG hookImports] Processing relative import: ${source}`);

      // Resolve the absolute path to the imported file
      const resolvedPath = resolveImportPath(source, currentDir);
      console.log(`[DEBUG hookImports] Resolved path: ${resolvedPath}`);
      if (!resolvedPath || !fs.existsSync(resolvedPath)) {
        console.log(`[DEBUG hookImports] File not found: ${resolvedPath}`);
        return;
      }
      console.log(`[DEBUG hookImports] File exists, reading...`);

      // Read and parse the imported file
      try {
        const importedCode = fs.readFileSync(resolvedPath, 'utf-8');
        const babel = require('@babel/core');

        // Parse the imported file
        const ast = babel.parseSync(importedCode, {
          filename: resolvedPath,
          presets: ['@babel/preset-typescript'],
          plugins: []
        });

        if (!ast) return;

        // Check each export in the file
        babel.traverse(ast, {
          // Handle: export default function useCounter(namespace, start) { ... }
          ExportDefaultDeclaration(exportPath) {
            const declaration = exportPath.node.declaration;

            if (t.isFunctionDeclaration(declaration) && isCustomHook(exportPath.get('declaration'))) {
              const hookName = getHookName(exportPath.get('declaration'));
              const hookMetadata = analyzeHook(exportPath.get('declaration'));

              // Store with the imported name
              const importedAs = getImportedName(importPath.node, true); // default import
              if (importedAs) {
                importedHooks.set(importedAs, {
                  ...hookMetadata,
                  originalName: hookName,
                  filePath: resolvedPath
                });
                console.log(`[Hook Import] Found default export: ${hookName} imported as ${importedAs}`);
              }
            }
          },

          // Handle: export function useCounter(namespace, start) { ... }
          ExportNamedDeclaration(exportPath) {
            const declaration = exportPath.node.declaration;

            if (t.isFunctionDeclaration(declaration) && isCustomHook(exportPath.get('declaration'))) {
              const hookName = getHookName(exportPath.get('declaration'));
              const hookMetadata = analyzeHook(exportPath.get('declaration'));

              // Store with the imported name (might be renamed)
              const importedAs = getImportedName(importPath.node, false, hookName);
              if (importedAs) {
                importedHooks.set(importedAs, {
                  ...hookMetadata,
                  originalName: hookName,
                  filePath: resolvedPath
                });
                console.log(`[Hook Import] Found named export: ${hookName} imported as ${importedAs}`);
              }
            }
          },

          // Handle: function useCounter(...) { ... }; export default useCounter;
          FunctionDeclaration(funcPath) {
            // Skip if already handled by export
            if (funcPath.parent.type === 'ExportDefaultDeclaration' ||
                funcPath.parent.type === 'ExportNamedDeclaration') {
              return;
            }

            if (isCustomHook(funcPath)) {
              const hookName = getHookName(funcPath);

              // Check if this function is exported later
              const isExported = isExportedLater(ast, hookName);
              if (isExported) {
                const hookMetadata = analyzeHook(funcPath);
                const importedAs = getImportedName(importPath.node, isExported === 'default', hookName);

                if (importedAs) {
                  importedHooks.set(importedAs, {
                    ...hookMetadata,
                    originalName: hookName,
                    filePath: resolvedPath
                  });
                  console.log(`[Hook Import] Found ${isExported} export: ${hookName} imported as ${importedAs}`);
                }
              }
            }
          }
        });
      } catch (err) {
        console.error(`[Hook Import] Error analyzing ${resolvedPath}:`, err.message);
      }
    }
  });

  return importedHooks;
}

/**
 * Resolve relative import path to absolute path
 *
 * @param {string} importSource - Import source (e.g., './useToggle')
 * @param {string} currentDir - Directory of the importing file
 * @returns {string|null} - Absolute path or null
 */
function resolveImportPath(importSource, currentDir) {
  const extensions = ['.tsx', '.ts', '.jsx', '.js'];

  // Try each extension
  for (const ext of extensions) {
    const withExt = importSource.endsWith(ext) ? importSource : importSource + ext;
    const resolved = path.resolve(currentDir, withExt);

    if (fs.existsSync(resolved)) {
      return resolved;
    }
  }

  return null;
}

/**
 * Get the imported name from an import declaration
 *
 * @param {Node} importNode - ImportDeclaration node
 * @param {boolean} isDefault - Whether this is a default import
 * @param {string} originalName - Original export name (for named imports)
 * @returns {string|null} - Imported name or null
 */
function getImportedName(importNode, isDefault, originalName = null) {
  for (const spec of importNode.specifiers) {
    if (isDefault && t.isImportDefaultSpecifier(spec)) {
      return spec.local.name;
    }
    if (!isDefault && t.isImportSpecifier(spec)) {
      // Check if this is the right named import
      const importedName = spec.imported.name;
      if (importedName === originalName) {
        return spec.local.name; // Might be renamed
      }
    }
  }

  return null;
}

/**
 * Check if a function is exported later in the file
 *
 * @param {Node} ast - File AST
 * @param {string} funcName - Function name to check
 * @returns {string|null} - 'default' or 'named' or null
 */
function isExportedLater(ast, funcName) {
  let exportType = null;

  babel.traverse(ast, {
    ExportDefaultDeclaration(path) {
      if (t.isIdentifier(path.node.declaration) &&
          path.node.declaration.name === funcName) {
        exportType = 'default';
      }
    },
    ExportNamedDeclaration(path) {
      if (path.node.specifiers) {
        for (const spec of path.node.specifiers) {
          if (t.isExportSpecifier(spec) &&
              t.isIdentifier(spec.exported) &&
              spec.exported.name === funcName) {
            exportType = 'named';
          }
        }
      }
    }
  });

  return exportType;
}

module.exports = {
  analyzeImportedHooks
};
