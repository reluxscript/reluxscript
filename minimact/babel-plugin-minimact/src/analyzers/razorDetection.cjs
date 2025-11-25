/**
 * Razor Syntax Detection and Variable Extraction
 *
 * Detects Razor-style syntax in markdown strings and extracts referenced variables.
 *
 * Supported Razor syntax:
 * - Variable references: @variableName, @variable.Property
 * - Inline expressions: @(expression)
 * - Conditionals: @if (condition) { ... } else { ... }
 * - Loops: @foreach (var item in items) { ... }
 * - For loops: @for (var i = 0; i < count; i++) { ... }
 * - Switch expressions: @switch (value) { case ...: ... }
 */

/**
 * Check if markdown string contains Razor syntax
 *
 * @param {string} markdown - Markdown content to check
 * @returns {boolean} True if Razor syntax detected
 */
function containsRazorSyntax(markdown) {
  if (!markdown || typeof markdown !== 'string') {
    return false;
  }

  // Razor patterns to detect:
  // 1. @variableName or @variable.Property (but not email addresses)
  // 2. @if, @foreach, @for, @switch, @while
  // 3. @(expression)

  const patterns = [
    /@[a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*/,  // @variable or @variable.property
    /@if\s*\(/,                                               // @if (
    /@foreach\s*\(/,                                          // @foreach (
    /@for\s*\(/,                                              // @for (
    /@switch\s*\(/,                                           // @switch (
    /@while\s*\(/,                                            // @while (
    /@\([^)]+\)/                                              // @(expression)
  ];

  return patterns.some(pattern => pattern.test(markdown));
}

/**
 * Extract all variable references from Razor markdown
 *
 * Returns root variable names (before any '.' property access)
 * Excludes Razor keywords (if, else, foreach, etc.)
 *
 * @param {string} markdown - Markdown content with Razor syntax
 * @returns {Set<string>} Set of variable names referenced
 */
function extractRazorVariables(markdown) {
  if (!markdown || typeof markdown !== 'string') {
    return new Set();
  }

  const variables = new Set();
  const razorKeywords = ['if', 'else', 'foreach', 'for', 'while', 'switch', 'case', 'default', 'break', 'var'];

  // Pattern 1: @variableName or @variable.Property
  const variablePattern = /@([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*)*)/g;
  let match;

  while ((match = variablePattern.exec(markdown)) !== null) {
    const fullPath = match[1]; // e.g., "product.Name" or "price"
    const rootVar = fullPath.split('.')[0]; // Get root variable

    // Skip Razor keywords
    if (!razorKeywords.includes(rootVar)) {
      variables.add(rootVar);
    }
  }

  // Pattern 2: @(expression) - extract identifiers from inside
  const expressionPattern = /@\(([^)]+)\)/g;

  while ((match = expressionPattern.exec(markdown)) !== null) {
    const expression = match[1];
    // Extract all identifiers from the expression
    const identifiers = expression.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g) || [];

    for (const identifier of identifiers) {
      // Skip C# keywords and Razor keywords
      if (!isCSharpKeyword(identifier) && !razorKeywords.includes(identifier)) {
        variables.add(identifier);
      }
    }
  }

  // Pattern 3: @if (condition) - extract from condition
  const ifPattern = /@if\s*\(([^)]+)\)/g;

  while ((match = ifPattern.exec(markdown)) !== null) {
    const condition = match[1];
    const identifiers = condition.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g) || [];

    for (const identifier of identifiers) {
      if (!isCSharpKeyword(identifier) && !razorKeywords.includes(identifier)) {
        variables.add(identifier);
      }
    }
  }

  // Pattern 4: @foreach (var item in collection)
  const foreachPattern = /@foreach\s*\(\s*var\s+[a-zA-Z_][a-zA-Z0-9_]*\s+in\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s*\)/g;

  while ((match = foreachPattern.exec(markdown)) !== null) {
    const collection = match[1];
    const rootVar = collection.split('.')[0];

    if (!razorKeywords.includes(rootVar)) {
      variables.add(rootVar);
    }
  }

  // Pattern 5: @for (var i = 0; i < count; i++)
  const forPattern = /@for\s*\([^)]+\)/g;

  while ((match = forPattern.exec(markdown)) !== null) {
    const forExpression = match[0];
    const identifiers = forExpression.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g) || [];

    for (const identifier of identifiers) {
      if (!isCSharpKeyword(identifier) && !razorKeywords.includes(identifier) && identifier !== 'i' && identifier !== 'j' && identifier !== 'k') {
        variables.add(identifier);
      }
    }
  }

  // Pattern 6: @switch (expression)
  const switchPattern = /@switch\s*\(([^)]+)\)/g;

  while ((match = switchPattern.exec(markdown)) !== null) {
    const expression = match[1];
    const identifiers = expression.match(/\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g) || [];

    for (const identifier of identifiers) {
      if (!isCSharpKeyword(identifier) && !razorKeywords.includes(identifier)) {
        variables.add(identifier);
      }
    }
  }

  return variables;
}

/**
 * Check if identifier is a C# keyword
 * @param {string} identifier
 * @returns {boolean}
 */
function isCSharpKeyword(identifier) {
  const keywords = [
    'abstract', 'as', 'base', 'bool', 'break', 'byte', 'case', 'catch', 'char',
    'checked', 'class', 'const', 'continue', 'decimal', 'default', 'delegate',
    'do', 'double', 'else', 'enum', 'event', 'explicit', 'extern', 'false',
    'finally', 'fixed', 'float', 'for', 'foreach', 'goto', 'if', 'implicit',
    'in', 'int', 'interface', 'internal', 'is', 'lock', 'long', 'namespace',
    'new', 'null', 'object', 'operator', 'out', 'override', 'params', 'private',
    'protected', 'public', 'readonly', 'ref', 'return', 'sbyte', 'sealed',
    'short', 'sizeof', 'stackalloc', 'static', 'string', 'struct', 'switch',
    'this', 'throw', 'true', 'try', 'typeof', 'uint', 'ulong', 'unchecked',
    'unsafe', 'ushort', 'using', 'virtual', 'void', 'volatile', 'while',
    // C# contextual keywords
    'add', 'and', 'alias', 'ascending', 'async', 'await', 'by', 'descending',
    'dynamic', 'equals', 'from', 'get', 'global', 'group', 'into', 'join',
    'let', 'nameof', 'not', 'notnull', 'on', 'or', 'orderby', 'partial',
    'record', 'remove', 'select', 'set', 'unmanaged', 'value', 'var', 'when',
    'where', 'with', 'yield'
  ];

  return keywords.includes(identifier);
}

module.exports = {
  containsRazorSyntax,
  extractRazorVariables,
  isCSharpKeyword
};
