/**
 * Path Assignment Pass for Minimact
 *
 * CRITICAL: This is the FIRST PASS that runs before any extraction.
 * It assigns hex paths to every JSX node by mutating the AST.
 *
 * Problem it solves:
 * - Old system: Each extractor recalculated paths independently
 * - Result: Path mismatches between template/attribute/handler extractors
 *
 * Solution:
 * - Single pass assigns paths and stores in node.__minimactPath
 * - All extractors read from node.__minimactPath (no recalculation!)
 *
 * Usage:
 *   const pathGen = new HexPathGenerator();
 *   assignPathsToJSX(jsxRoot, '', pathGen, t);
 *   // Now all JSX nodes have __minimactPath metadata
 */

const { HexPathGenerator } = require('./hexPath.cjs');

/**
 * Assign hex paths to all JSX nodes in tree
 *
 * Mutates AST by adding __minimactPath and __minimactPathSegments to each node.
 * This ensures consistent paths across all subsequent extractors.
 *
 * @param {Object} node - Babel AST node
 * @param {string} parentPath - Parent hex path
 * @param {HexPathGenerator} pathGen - Hex path generator
 * @param {Object} t - Babel types
 * @param {string|null} previousSiblingKey - Previous sibling's key for sort validation
 * @param {string|null} nextSiblingKey - Next sibling's key for sort validation
 * @param {Array} structuralChanges - Array to collect structural changes for hot reload
 * @param {boolean} isHotReload - Whether this is a hot reload (keys file exists)
 */
function assignPathsToJSX(node, parentPath, pathGen, t, previousSiblingKey = null, nextSiblingKey = null, structuralChanges = [], isHotReload = false) {
  if (t.isJSXElement(node)) {
    let currentPath;
    let pathSegments;
    let useExistingKey = false;

    // Check if element already has a key attribute
    if (node.openingElement && node.openingElement.attributes) {
      const keyAttr = node.openingElement.attributes.find(attr =>
        t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name) && attr.name.name === 'key'
      );

      if (keyAttr && t.isStringLiteral(keyAttr.value)) {
        const existingKey = keyAttr.value.value;

        // Validate the key: must be valid hex path format AND in correct sort order
        if (isValidHexPath(existingKey) && isInSortOrder(existingKey, previousSiblingKey, nextSiblingKey)) {
          // Use the existing key as the path
          currentPath = existingKey;
          pathSegments = pathGen.parsePath(currentPath);
          useExistingKey = true;

          // CRITICAL: Synchronize the path generator's counter with this existing key
          // This prevents duplicate keys when auto-generating for the next sibling
          syncPathGeneratorWithKey(currentPath, parentPath, pathGen);

          console.log(`[Path Assignment] ♻️  Using existing key="${currentPath}" for <${node.openingElement.name.name}>`);
        } else if (!isValidHexPath(existingKey)) {
          console.warn(`[Path Assignment] ⚠️  Invalid key format "${existingKey}" - generating new path`);
        } else {
          console.warn(`[Path Assignment] ⚠️  Key "${existingKey}" is out of order (prev: ${previousSiblingKey}, next: ${nextSiblingKey}) - generating half-gap`);
        }
      }
    }

    // If no valid existing key, generate a new one
    if (!useExistingKey) {
      // If we have previous and next siblings, generate a half-gap between them
      if (previousSiblingKey && nextSiblingKey) {
        currentPath = generateHalfGap(previousSiblingKey, nextSiblingKey, parentPath);
        console.log(`[Path Assignment] ⚡ Generated half-gap key="${currentPath}" between "${previousSiblingKey}" and "${nextSiblingKey}"`);
      } else {
        // Normal sequential generation
        const childHex = pathGen.next(parentPath);
        currentPath = pathGen.buildPath(parentPath, childHex);
      }

      // Track insertion ONLY during hot reload (when keys file exists)
      if (isHotReload) {
        console.log(`[Hot Reload] 🆕 Insertion detected at path "${currentPath}"`);
        const vnode = generateVNodeRepresentation(node, currentPath, t);
        if (vnode) {
          structuralChanges.push({
            type: 'insert',
            path: currentPath,
            vnode: vnode
          });
        }
      }

      pathSegments = pathGen.parsePath(currentPath);

      // Add or update key prop to JSX element
      if (node.openingElement && node.openingElement.attributes) {
        // Check if key already exists (but was invalid)
        const existingKeyAttr = node.openingElement.attributes.find(attr =>
          t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name) && attr.name.name === 'key'
        );

        if (existingKeyAttr && t.isStringLiteral(existingKeyAttr.value)) {
          // Update existing key
          existingKeyAttr.value = t.stringLiteral(currentPath);
          console.log(`[Path Assignment] ✅ Replaced invalid key with "${currentPath}" on <${node.openingElement.name.name}>`);
        } else {
          // Add new key
          const keyAttr = t.jsxAttribute(
            t.jsxIdentifier('key'),
            t.stringLiteral(currentPath)
          );
          node.openingElement.attributes.unshift(keyAttr);
          console.log(`[Path Assignment] ✅ Added key="${currentPath}" to <${node.openingElement.name.name}>`);
        }
      }
    }

    // Mutate AST node with path data
    node.__minimactPath = currentPath;
    node.__minimactPathSegments = pathSegments;

    // Process attributes (for @attributeName paths)
    if (node.openingElement && node.openingElement.attributes) {
      for (const attr of node.openingElement.attributes) {
        if (t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name)) {
          const attrName = attr.name.name;
          const attrPath = `${currentPath}.@${attrName}`;

          // Mutate attribute node with path
          attr.__minimactPath = attrPath;
          attr.__minimactPathSegments = [...pathSegments, `@${attrName}`];
        }
      }
    }

    // Recursively assign paths to children
    if (node.children) {
      assignPathsToChildren(node.children, currentPath, pathGen, t, structuralChanges, isHotReload);
    }
  } else if (t.isJSXFragment(node)) {
    // Fragments don't get paths - children become direct siblings
    if (node.children) {
      assignPathsToChildren(node.children, parentPath, pathGen, t, structuralChanges);
    }
  }
}

/**
 * Assign paths to JSX children array
 *
 * Handles mixed content: JSXElement, JSXText, JSXExpressionContainer, JSXFragment
 *
 * @param {Array} children - Array of Babel AST nodes
 * @param {string} parentPath - Parent hex path
 * @param {HexPathGenerator} pathGen - Hex path generator
 * @param {Object} t - Babel types
 * @param {Array} structuralChanges - Array to collect structural changes for hot reload
 */
function assignPathsToChildren(children, parentPath, pathGen, t, structuralChanges = [], isHotReload = false) {
  let previousKey = null; // Track previous sibling's key for sort order validation

  for (let i = 0; i < children.length; i++) {
    const child = children[i];

    if (t.isJSXElement(child)) {
      // Look ahead to find next sibling's key (if it exists)
      let nextKey = null;
      for (let j = i + 1; j < children.length; j++) {
        if (t.isJSXElement(children[j])) {
          const nextKeyAttr = children[j].openingElement?.attributes?.find(attr =>
            t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name) && attr.name.name === 'key'
          );
          if (nextKeyAttr && t.isStringLiteral(nextKeyAttr.value)) {
            nextKey = nextKeyAttr.value.value;
            break;
          }
        }
      }

      // Nested JSX element - pass previous and next keys for validation
      assignPathsToJSX(child, parentPath, pathGen, t, previousKey, nextKey, structuralChanges, isHotReload);

      // Update previousKey for next sibling
      if (child.__minimactPath) {
        previousKey = child.__minimactPath;
      }
    } else if (t.isJSXText(child)) {
      // Static text node
      const text = child.value.trim();
      if (text) {
        const textHex = pathGen.next(parentPath);
        const textPath = pathGen.buildPath(parentPath, textHex);
        const textSegments = pathGen.parsePath(textPath);

        // Mutate text node with path
        child.__minimactPath = textPath;
        child.__minimactPathSegments = textSegments;
      }
    } else if (t.isJSXExpressionContainer(child)) {
      // Expression container - assign path and recurse into structural JSX
      const expr = child.expression;

      // Skip JSX comments (empty expressions like {/* comment */})
      if (t.isJSXEmptyExpression(expr)) {
        // Don't assign path, don't increment counter - comments are ignored
        continue;
      }

      // Generate path for the expression container
      const exprHex = pathGen.next(parentPath);
      const exprPath = pathGen.buildPath(parentPath, exprHex);
      const exprSegments = pathGen.parsePath(exprPath);

      // Mutate expression container with path
      child.__minimactPath = exprPath;
      child.__minimactPathSegments = exprSegments;

      // Recurse into structural expressions (conditionals, loops)
      assignPathsToExpression(expr, exprPath, pathGen, t, structuralChanges);
    } else if (t.isJSXFragment(child)) {
      // Fragment - flatten children
      assignPathsToJSX(child, parentPath, pathGen, t, null, null, structuralChanges);
    }
  }
}

/**
 * Assign paths to expressions containing JSX
 *
 * Handles:
 * - Logical AND: {isVisible && <Modal />}
 * - Ternary: {isAdmin ? <AdminPanel /> : <UserPanel />}
 * - Array.map: {items.map(item => <li>{item}</li>)}
 *
 * @param {Object} expr - Babel expression node
 * @param {string} parentPath - Parent hex path
 * @param {HexPathGenerator} pathGen - Hex path generator
 * @param {Object} t - Babel types
 * @param {Array} structuralChanges - Array to collect structural changes for hot reload
 */
function assignPathsToExpression(expr, parentPath, pathGen, t, structuralChanges = []) {
  if (!expr) return;

  if (t.isLogicalExpression(expr) && expr.operator === '&&') {
    // Logical AND: {isAdmin && <div>Admin Panel</div>}
    if (t.isJSXElement(expr.right)) {
      assignPathsToJSX(expr.right, parentPath, pathGen, t, null, null, structuralChanges);
    } else if (t.isJSXExpressionContainer(expr.right)) {
      assignPathsToExpression(expr.right.expression, parentPath, pathGen, t, structuralChanges);
    }
  } else if (t.isConditionalExpression(expr)) {
    // Ternary: {isAdmin ? <AdminPanel/> : <UserPanel/>}

    // Assign paths to consequent (true branch)
    if (t.isJSXElement(expr.consequent)) {
      assignPathsToJSX(expr.consequent, parentPath, pathGen, t, null, null, structuralChanges);
    } else if (t.isJSXExpressionContainer(expr.consequent)) {
      assignPathsToExpression(expr.consequent.expression, parentPath, pathGen, t, structuralChanges);
    }

    // Assign paths to alternate (false branch)
    if (expr.alternate) {
      if (t.isJSXElement(expr.alternate)) {
        assignPathsToJSX(expr.alternate, parentPath, pathGen, t, null, null, structuralChanges);
      } else if (t.isJSXExpressionContainer(expr.alternate)) {
        assignPathsToExpression(expr.alternate.expression, parentPath, pathGen, t, structuralChanges);
      }
    }
  } else if (t.isCallExpression(expr) &&
             t.isMemberExpression(expr.callee) &&
             t.isIdentifier(expr.callee.property) &&
             expr.callee.property.name === 'map') {
    // Array.map: {items.map(item => <li>{item}</li>)}

    const callback = expr.arguments[0];
    if (t.isArrowFunctionExpression(callback) || t.isFunctionExpression(callback)) {
      const body = callback.body;

      if (t.isJSXElement(body)) {
        // Arrow function with JSX body: item => <li>{item}</li>
        assignPathsToJSX(body, parentPath, pathGen, t, null, null, structuralChanges);
      } else if (t.isBlockStatement(body)) {
        // Arrow function with block: item => { return <li>{item}</li>; }
        const returnStmt = body.body.find(stmt => t.isReturnStatement(stmt));
        if (returnStmt && t.isJSXElement(returnStmt.argument)) {
          assignPathsToJSX(returnStmt.argument, parentPath, pathGen, t, null, null, structuralChanges);
        }
      }
    }
  } else if (t.isJSXFragment(expr)) {
    // Fragment
    assignPathsToJSX(expr, parentPath, pathGen, t, null, null, structuralChanges);
  } else if (t.isJSXElement(expr)) {
    // Direct JSX element
    assignPathsToJSX(expr, parentPath, pathGen, t, null, null, structuralChanges);
  }
}

/**
 * Get path from AST node (helper for extractors)
 *
 * Reads __minimactPath metadata assigned by this pass.
 * Throws error if path wasn't assigned (indicates bug).
 *
 * @param {Object} node - Babel AST node
 * @returns {string} - Hex path
 */
function getPathFromNode(node) {
  if (!node.__minimactPath) {
    throw new Error('[Minimact] Path not assigned to node! Did you forget to run assignPathsToJSX?');
  }
  return node.__minimactPath;
}

/**
 * Get path segments from AST node (helper for extractors)
 *
 * @param {Object} node - Babel AST node
 * @returns {string[]} - Path segments array
 */
function getPathSegmentsFromNode(node) {
  if (!node.__minimactPathSegments) {
    throw new Error('[Minimact] Path segments not assigned to node! Did you forget to run assignPathsToJSX?');
  }
  return node.__minimactPathSegments;
}

/**
 * Validate if a string is a valid hex path
 *
 * Valid formats:
 * - "1" (single hex segment)
 * - "1.1" (dot-separated hex segments)
 * - "a.b.c" (multiple segments)
 *
 * Invalid formats:
 * - "foo" (non-hex characters)
 * - "1..2" (empty segments)
 * - ".1" (leading dot)
 * - "1." (trailing dot)
 *
 * @param {string} key - The key to validate
 * @returns {boolean} - True if valid hex path format
 */
function isValidHexPath(key) {
  if (!key || typeof key !== 'string') {
    return false;
  }

  // Must not start or end with dot
  if (key.startsWith('.') || key.endsWith('.')) {
    return false;
  }

  // Split by dots and validate each segment
  const segments = key.split('.');

  for (const segment of segments) {
    // Each segment must be non-empty and valid hex (0-9, a-f)
    if (!segment || !/^[0-9a-f]+$/i.test(segment)) {
      return false;
    }
  }

  return true;
}

/**
 * Check if a key is in correct lexicographic sort order
 *
 * @param {string} key - The key to check
 * @param {string|null} previousKey - Previous sibling's key (must be less than key)
 * @param {string|null} nextKey - Next sibling's key (must be greater than key)
 * @returns {boolean} - True if key is in correct sort order
 */
function isInSortOrder(key, previousKey, nextKey) {
  // If there's a previous key, current must be greater
  if (previousKey && key <= previousKey) {
    return false;
  }

  // If there's a next key, current must be less
  if (nextKey && key >= nextKey) {
    return false;
  }

  return true;
}

/**
 * Synchronize the path generator's counter with an existing key
 *
 * When we use an existing key instead of generating a new one, we need to
 * update the path generator's internal counter so it doesn't reuse that key.
 *
 * @param {string} existingKey - The existing key we're using (e.g., "1.2")
 * @param {string} parentPath - Parent path (e.g., "1")
 * @param {HexPathGenerator} pathGen - Path generator to synchronize
 */
function syncPathGeneratorWithKey(existingKey, parentPath, pathGen) {
  // Extract the child segment from the existing key
  const parentPrefix = parentPath ? parentPath + '.' : '';
  const childHex = existingKey.startsWith(parentPrefix)
    ? existingKey.slice(parentPrefix.length)
    : existingKey;

  // Convert to decimal to get the numeric value
  const childNum = parseInt(childHex, 16);

  // Update the counter for this parent path to be at least one past this key
  // This ensures the next call to pathGen.next() won't reuse this key
  if (!pathGen.counters[parentPath]) {
    pathGen.counters[parentPath] = 0;
  }

  // Set counter to the max of current counter or the existing key's value
  pathGen.counters[parentPath] = Math.max(pathGen.counters[parentPath], childNum);
}

/**
 * Generate a half-gap hex path between two keys
 *
 * Takes the average of two hex paths to create a value that sorts between them.
 *
 * @param {string} prevKey - Previous sibling's key (e.g., "1.1")
 * @param {string} nextKey - Next sibling's key (e.g., "1.2")
 * @param {string} parentPath - Parent path (e.g., "1")
 * @returns {string} - Half-gap hex path (e.g., "1.15")
 */
function generateHalfGap(prevKey, nextKey, parentPath) {
  // Remove parent path prefix to get just the child segment
  const parentPrefix = parentPath ? parentPath + '.' : '';
  const prevChild = prevKey.startsWith(parentPrefix) ? prevKey.slice(parentPrefix.length) : prevKey;
  const nextChild = nextKey.startsWith(parentPrefix) ? nextKey.slice(parentPrefix.length) : nextKey;

  // Convert child hex segments to decimal
  const prevNum = parseInt(prevChild, 16);
  const nextNum = parseInt(nextChild, 16);

  // Calculate midpoint
  const midNum = Math.floor((prevNum + nextNum) / 2);

  // If midpoint equals prevNum, we need more precision (add a fractional hex digit)
  if (midNum === prevNum) {
    // Generate a value between prevNum and nextNum by adding .8 (half of 0x10)
    const midHex = prevChild + '8';
    return parentPath ? `${parentPath}.${midHex}` : midHex;
  }

  // Convert back to hex (without padding to keep it compact)
  const midHex = midNum.toString(16);

  // Build full path
  return parentPath ? `${parentPath}.${midHex}` : midHex;
}

/**
 * Convert a Babel JSX AST node to VNode JSON representation for hot reload
 *
 * @param {Object} node - Babel JSX element node
 * @param {string} path - Hex path for this node
 * @param {Object} t - Babel types
 * @returns {Object} - VNode representation for C#
 */
function generateVNodeRepresentation(node, path, t) {
  if (!t.isJSXElement(node)) {
    return null;
  }

  const tagName = node.openingElement.name.name;
  const attributes = {};

  // Extract attributes
  for (const attr of node.openingElement.attributes) {
    if (t.isJSXAttribute(attr) && t.isJSXIdentifier(attr.name)) {
      const attrName = attr.name.name;

      if (attrName === 'key') continue; // Skip key attribute

      if (t.isStringLiteral(attr.value)) {
        attributes[attrName] = attr.value.value;
      } else if (t.isJSXExpressionContainer(attr.value)) {
        // For expressions, mark as dynamic
        attributes[attrName] = '__DYNAMIC__';
      } else if (attr.value === null) {
        // Boolean attribute (e.g., <input disabled />)
        attributes[attrName] = true;
      }
    }
  }

  // Extract children (simplified - only static content and basic structure)
  const children = [];
  if (node.children) {
    for (const child of node.children) {
      if (t.isJSXText(child)) {
        const text = child.value.trim();
        if (text) {
          children.push({
            type: 'text',
            path: child.__minimactPath || `${path}.${children.length + 1}`,
            value: text
          });
        }
      } else if (t.isJSXElement(child)) {
        // Nested element - include path and tag
        children.push({
          type: 'element',
          path: child.__minimactPath || `${path}.${children.length + 1}`,
          tag: child.openingElement.name.name
        });
      } else if (t.isJSXExpressionContainer(child)) {
        // Expression - mark as dynamic
        children.push({
          type: 'expression',
          path: child.__minimactPath || `${path}.${children.length + 1}`,
          value: '__DYNAMIC__'
        });
      }
    }
  }

  return {
    type: 'element',
    tag: tagName,
    path: path,
    attributes: attributes,
    children: children
  };
}

module.exports = {
  assignPathsToJSX,
  assignPathsToChildren,
  assignPathsToExpression,
  getPathFromNode,
  getPathSegmentsFromNode,
  isValidHexPath,
  generateVNodeRepresentation
};
