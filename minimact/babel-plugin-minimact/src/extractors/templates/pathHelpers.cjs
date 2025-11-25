/**
 * Build path key for template map
 * Example: div[0].h1[0].text → "div[0].h1[0]"
 */
function buildPathKey(tagName, index, parentPath) {
  const parentKeys = [];
  let currentPath = parentPath;

  // Build parent path from indices
  // This is simplified - in production we'd track tag names
  for (let i = 0; i < currentPath.length; i++) {
    parentKeys.push(`[${currentPath[i]}]`);
  }

  return `${parentKeys.join('.')}.${tagName}[${index}]`.replace(/^\./, '');
}

/**
 * Build attribute path key
 * Example: div[0].@style or div[1].@className
 */
function buildAttributePathKey(tagName, index, parentPath, attrName) {
  const parentKeys = [];
  for (let i = 0; i < parentPath.length; i++) {
    parentKeys.push(`[${parentPath[i]}]`);
  }
  return `${parentKeys.join('.')}.${tagName}[${index}].@${attrName}`.replace(/^\./, '');
}

module.exports = { buildPathKey, buildAttributePathKey };
