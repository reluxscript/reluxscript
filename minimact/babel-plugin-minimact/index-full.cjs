/**
 * Minimact Babel Plugin - Complete Implementation
 *
 * Features:
 * - Dependency tracking for hybrid rendering
 * - Smart span splitting for mixed client/server state
 * - All hooks: useState, useEffect, useRef, useClientState, useMarkdown, useTemplate
 * - Conditional rendering (ternary, &&)
 * - List rendering (.map with key)
 * - Fragment support
 * - Props support
 * - TypeScript interface → C# class conversion
 */

const t = require('@babel/types');
const { traverse } = require('@babel/core');
const fs = require('fs');
const nodePath = require('path');

// Modular imports
const { processComponent } = require('./src/processComponent.cjs');
const { generateCSharpFile } = require('./src/generators/csharpFile.cjs');
const { generateTemplateMapJSON } = require('./src/extractors/templates.cjs');

/**
 * Extract all key attribute values from TSX source code
 *
 * @param {string} sourceCode - TSX source code
 * @returns {Set<string>} - Set of all key values
 */
function extractAllKeysFromSource(sourceCode) {
  const keys = new Set();

  // Match key="value" or key='value' or key={value}
  const keyRegex = /key=(?:"([^"]+)"|'([^']+)'|\{([^}]+)\})/g;
  let match;

  while ((match = keyRegex.exec(sourceCode)) !== null) {
    const keyValue = match[1] || match[2] || match[3];

    // Only include string literal keys (not expressions)
    if (match[1] || match[2]) {
      keys.add(keyValue);
    }
  }

  return keys;
}

module.exports = function(babel) {
  const generate = require('@babel/generator').default;

  return {
    name: 'minimact-full',

    pre(file) {
      // Save the original code BEFORE React preset transforms JSX
      // This allows us to generate .tsx.keys with real JSX syntax
      file.originalCode = file.code;
      console.log(`[Minimact Keys] pre() hook - Saved original code (${file.code ? file.code.length : 0} chars)`);
    },

    visitor: {
      Program: {
        enter(path, state) {
          // Initialize minimactComponents array
          state.file.minimactComponents = [];

          // Collect all top-level function declarations for potential inclusion as helpers
          state.file.topLevelFunctions = [];

          path.traverse({
            FunctionDeclaration(funcPath) {
              // Only collect top-level functions (not nested inside components)
              if (funcPath.parent.type === 'Program' || funcPath.parent.type === 'ExportNamedDeclaration') {
                const funcName = funcPath.node.id ? funcPath.node.id.name : null;
                // Skip if it's a component (starts with uppercase)
                if (funcName && funcName[0] === funcName[0].toLowerCase()) {
                  state.file.topLevelFunctions.push({
                    name: funcName,
                    node: funcPath.node,
                    path: funcPath
                  });
                }
              }
            }
          });
        },

        exit(programPath, state) {
          // 🔥 Generate .tsx.keys FIRST - from original JSX source with keys added
          // This must happen BEFORE JSX is replaced with null!
          const inputFilePath = state.file.opts.filename;
          console.log(`[Minimact Keys] inputFilePath: ${inputFilePath}, originalCode exists: ${!!state.file.originalCode}`);
          if (inputFilePath && state.file.originalCode) {
            const recast = require('recast');
            const babelParser = require('@babel/parser');
            const babelTypes = require('@babel/types');
            const { HexPathGenerator } = require('./src/utils/hexPath.cjs');
            const { assignPathsToJSX } = require('./src/utils/pathAssignment.cjs');

            try {
              // Parse with Recast using Babel parser
              // Recast preserves original formatting, whitespace, and comments
              const originalAst = recast.parse(state.file.originalCode, {
                parser: require('recast/parsers/babel-ts')
              });

              // Now add keys to this AST using Recast's traverse
              recast.visit(originalAst, {
                visitFunctionDeclaration(funcPath) {
                  // Find components (must have JSX return)
                  this.traverse(funcPath);

                  recast.visit(funcPath, {
                    visitReturnStatement(returnPath) {
                      // Check if this return is directly in the component function
                      if (returnPath.parent && returnPath.parent.value.type === 'BlockStatement') {
                        const returnNode = returnPath.value;
                        if (returnNode.argument && babelTypes.isJSXElement(returnNode.argument)) {
                          // This is a component! Add keys to its JSX
                          const pathGen = new HexPathGenerator();
                          assignPathsToJSX(returnNode.argument, '', pathGen, babelTypes);
                        }
                      }
                      return false; // Don't traverse deeper
                    }
                  });

                  return false; // Don't traverse deeper
                }
              });

              // Generate code with Recast - preserves original formatting!
              const output = recast.print(originalAst, {
                tabWidth: 2,
                useTabs: false,
                quote: 'single',
                trailingComma: false
              });

              const keysFilePath = inputFilePath + '.keys';
              fs.writeFileSync(keysFilePath, output.code);
              console.log(`[Minimact Keys] ✅ Generated ${nodePath.basename(keysFilePath)} with preserved formatting`);
            } catch (error) {
              console.error(`[Minimact Keys] ❌ Failed to generate .tsx.keys:`, error);
            }
          }

          // 🔥 STEP 2: NOW nullify JSX in all components (after .tsx.keys generation)
          const t = require('@babel/types');
          if (state.file.componentPathsToNullify) {
            for (const componentPath of state.file.componentPathsToNullify) {
              componentPath.traverse({
                ReturnStatement(returnPath) {
                  if (returnPath.getFunctionParent() === componentPath) {
                    returnPath.node.argument = t.nullLiteral();
                  }
                }
              });
            }
            console.log(`[Minimact] ✅ Nullified JSX in ${state.file.componentPathsToNullify.length} components`);
          }

          // 🔥 STEP 3: Generate C# code (now with nullified JSX)
          if (state.file.minimactComponents && state.file.minimactComponents.length > 0) {
            const csharpCode = generateCSharpFile(state.file.minimactComponents, state);

            state.file.metadata = state.file.metadata || {};
            state.file.metadata.minimactCSharp = csharpCode;

            // 🔥 CRITICAL: Write C# file BEFORE structural-changes.json
            // This ensures the StructuralChangeManager can find and compile the C# file
            if (inputFilePath) {
              const outputDir = nodePath.dirname(inputFilePath);
              const csFilePath = nodePath.join(outputDir, `${state.file.minimactComponents[0].name}.cs`);

              try {
                fs.writeFileSync(csFilePath, csharpCode);
                console.log(`[Minimact C#] ✅ Generated ${csFilePath}`);
              } catch (error) {
                console.error(`[Minimact C#] Failed to write ${csFilePath}:`, error);
              }
            }

            // Generate .templates.json files for hot reload
            const inputFilePath2 = state.file.opts.filename;
            if (inputFilePath) {
              for (const component of state.file.minimactComponents) {
                if (component.templates && Object.keys(component.templates).length > 0) {
                  const templateMapJSON = generateTemplateMapJSON(
                    component.name,
                    component.templates,
                    {}, // Attribute templates already included in component.templates
                    component.conditionalElementTemplates || {}
                  );

                  // Write to .templates.json file
                  const outputDir = nodePath.dirname(inputFilePath);
                  const templateFilePath = nodePath.join(outputDir, `${component.name}.templates.json`);

                  try {
                    fs.writeFileSync(templateFilePath, JSON.stringify(templateMapJSON, null, 2));
                    console.log(`[Minimact Templates] Generated ${templateFilePath}`);
                  } catch (error) {
                    console.error(`[Minimact Templates] Failed to write ${templateFilePath}:`, error);
                  }
                }

                // Generate timeline metadata file if timeline exists
                if (component.timeline) {
                  const { generateTimelineMetadataFile } = require('./src/generators/timelineGenerator.cjs');
                  const timelineMetadata = generateTimelineMetadataFile(
                    component.name,
                    component.timeline,
                    component.templates || {}
                  );

                  const outputDir = nodePath.dirname(inputFilePath);
                  const timelineFilePath = nodePath.join(outputDir, `${component.name}.timeline-templates.json`);

                  try {
                    fs.writeFileSync(timelineFilePath, JSON.stringify(timelineMetadata, null, 2));
                    console.log(`[Minimact Timeline] Generated ${timelineFilePath}`);
                  } catch (error) {
                    console.error(`[Minimact Timeline] Failed to write ${timelineFilePath}:`, error);
                  }
                }

                // 🔥 HOOK CHANGE DETECTION
                // Extract hook signature and compare with previous to detect hook changes
                const {
                  extractHookSignature,
                  writeHookSignature,
                  readPreviousHookSignature,
                  compareHookSignatures
                } = require('./src/extractors/hookSignature.cjs');

                // Extract current hook signature
                const currentHooks = extractHookSignature(component);

                // Read previous signature BEFORE writing new one
                const previousHooks = readPreviousHookSignature(component.name, inputFilePath);

                // Write current signature to file (for next comparison)
                writeHookSignature(component.name, currentHooks, inputFilePath);

                // Compare signatures and detect hook changes
                let hookChanges = [];
                if (previousHooks) {
                  hookChanges = compareHookSignatures(previousHooks, currentHooks);
                  if (hookChanges.length > 0) {
                    console.log(`[Hook Changes] Detected ${hookChanges.length} hook change(s) in ${component.name}`);
                  }
                } else {
                  console.log(`[Hook Signature] No previous signature found for ${component.name} (first transpilation)`);
                }

                // 🔥 JSX STRUCTURAL CHANGE DETECTION
                // Combine JSX changes + hook changes into single structural changes array
                const jsxChanges = component.structuralChanges || [];

                // Read previous .tsx.keys to detect JSX deletions
                const keysFilePath = inputFilePath + '.keys';
                let previousKeys = new Set();

                if (fs.existsSync(keysFilePath)) {
                  try {
                    const previousSource = fs.readFileSync(keysFilePath, 'utf-8');
                    previousKeys = extractAllKeysFromSource(previousSource);
                    console.log(`[Hot Reload] Read ${previousKeys.size} keys from previous transpilation`);
                  } catch (error) {
                    console.error(`[Hot Reload] Failed to read ${keysFilePath}:`, error);
                  }
                }

                // Collect current keys from the newly generated .tsx.keys file
                const currentKeys = new Set();
                const newKeysFilePath = inputFilePath + '.keys';
                if (fs.existsSync(newKeysFilePath)) {
                  try {
                    const currentSource = fs.readFileSync(newKeysFilePath, 'utf-8');
                    const extractedKeys = extractAllKeysFromSource(currentSource);
                    extractedKeys.forEach(key => currentKeys.add(key));
                    console.log(`[Hot Reload] Read ${currentKeys.size} keys from current transpilation`);
                  } catch (error) {
                    console.error(`[Hot Reload] Failed to read current keys:`, error);
                  }
                }

                // Detect JSX deletions
                const jsxDeletions = [];
                for (const prevKey of previousKeys) {
                  if (!currentKeys.has(prevKey)) {
                    console.log(`[Hot Reload] 🗑️  JSX deletion detected at path "${prevKey}"`);
                    jsxDeletions.push({
                      type: 'delete',
                      path: prevKey
                    });
                  }
                }

                // Combine ALL structural changes (JSX insertions + JSX deletions + hook changes)
                const allChanges = [...jsxChanges, ...jsxDeletions, ...hookChanges];

                // Write structural changes file if there are any changes
                if (allChanges.length > 0) {
                  const structuralChangesJSON = {
                    componentName: component.name,
                    timestamp: new Date().toISOString(),
                    sourceFile: inputFilePath,
                    changes: allChanges
                  };

                  const outputDir = nodePath.dirname(inputFilePath);
                  const changesFilePath = nodePath.join(outputDir, `${component.name}.structural-changes.json`);

                  try {
                    fs.writeFileSync(changesFilePath, JSON.stringify(structuralChangesJSON, null, 2));
                    console.log(`[Hot Reload] ✅ Generated ${changesFilePath} with ${allChanges.length} changes (${jsxChanges.length} JSX insertions, ${jsxDeletions.length} JSX deletions, ${hookChanges.length} hook changes)`);
                  } catch (error) {
                    console.error(`[Hot Reload] Failed to write ${changesFilePath}:`, error);
                  }
                }
              }
            }
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
      }
    }
  };
};