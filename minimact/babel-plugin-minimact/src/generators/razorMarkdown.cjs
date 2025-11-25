/**
 * Razor Markdown to C# Conversion
 *
 * Converts Razor-style syntax in markdown to C# string interpolation.
 *
 * Input (TSX):
 *   `# @name - $@price`
 *
 * Output (C#):
 *   $@"# {name} - ${price}"
 *
 * Supported conversions:
 * - @variable → {variable}
 * - @variable.Property → {variable.Property}
 * - @(expression) → {(expression)}
 * - @if (cond) { ... } else { ... } → {(cond ? @"..." : @"...")}
 * - @foreach (var x in xs) { ... } → {string.Join("\n", xs.Select(x => $@"..."))}
 * - @for (var i = 1; i <= count; i++) { ... } → {string.Join("\n", Enumerable.Range(1, count).Select(i => $@"..."))}
 * - @switch (x) { case ...: ... } → {x switch { ... => @"...", _ => @"..." }}
 */

/**
 * Convert Razor markdown to C# interpolated string
 *
 * @param {string} razorMarkdown - Markdown with Razor syntax
 * @returns {string} C# interpolated string ($@"...")
 */
function convertRazorMarkdownToCSharp(razorMarkdown) {
  if (!razorMarkdown || typeof razorMarkdown !== 'string') {
    return '$@""';
  }

  let markdown = razorMarkdown;

  // Step 1: Convert @if blocks (must come before variable references)
  markdown = convertIfBlocks(markdown);

  // Step 2: Convert @foreach blocks
  markdown = convertForeachBlocks(markdown);

  // Step 3: Convert @for blocks
  markdown = convertForBlocks(markdown);

  // Step 4: Convert @switch blocks
  markdown = convertSwitchBlocks(markdown);

  // Step 5: Convert @(expression)
  markdown = convertInlineExpressions(markdown);

  // Step 6: Convert @variableName (must come last)
  markdown = convertVariableReferences(markdown);

  // Step 7: Escape any remaining unescaped quotes
  // Already handled by nested verbatim strings (@"...")

  // Step 8: Wrap in $@"..."
  return `$@"${markdown}"`;
}

/**
 * Convert @if blocks to C# ternary expressions
 *
 * @if (condition) { body } → {(condition ? @"body" : "")}
 * @if (condition) { body } else { elseBody } → {(condition ? @"body" : @"elseBody")}
 *
 * @param {string} markdown
 * @returns {string}
 */
function convertIfBlocks(markdown) {
  // Pattern: @if \s* ( condition ) \s* { body } [else { elseBody }]
  // Using [\s\S] to match any character including newlines

  const ifPattern = /@if\s*\(([^)]+)\)\s*\{([\s\S]*?)\}(?:\s*else\s*\{([\s\S]*?)\})?/g;

  return markdown.replace(ifPattern, (match, condition, thenBody, elseBody) => {
    const then = thenBody.trim();
    const elsePart = elseBody ? elseBody.trim() : '';

    // Recursively convert nested Razor in the bodies
    const convertedThen = convertNestedRazor(then);
    const convertedElse = elsePart ? convertNestedRazor(elsePart) : '';

    if (convertedElse) {
      return `{(${condition} ? @"${convertedThen}" : @"${convertedElse}")}`;
    } else {
      return `{(${condition} ? @"${convertedThen}" : "")}`;
    }
  });
}

/**
 * Convert @foreach blocks to LINQ Select
 *
 * @foreach (var item in collection) { body } →
 * {string.Join("\n", collection.Select(item => $@"body"))}
 *
 * @param {string} markdown
 * @returns {string}
 */
function convertForeachBlocks(markdown) {
  // Pattern: @foreach \s* ( var itemVar in collection ) \s* { body }
  // Using [\s\S] to match any character including newlines
  const foreachPattern = /@foreach\s*\(\s*var\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+([a-zA-Z_][a-zA-Z0-9_.]*)\)\s*\{([\s\S]*?)\}/g;

  return markdown.replace(foreachPattern, (match, itemVar, collection, body) => {
    const bodyTrimmed = body.trim();
    // Recursively convert nested Razor in body (preserving item variable references)
    const convertedBody = convertNestedRazor(bodyTrimmed, itemVar);

    return `{string.Join("\\n", ${collection}.Select(${itemVar} => $@"${convertedBody}"))}`;
  });
}

/**
 * Convert @for blocks to Enumerable.Range
 *
 * @for (var i = 1; i <= count; i++) { body } →
 * {string.Join("\n", Enumerable.Range(1, count).Select(i => $@"body"))}
 *
 * @param {string} markdown
 * @returns {string}
 */
function convertForBlocks(markdown) {
  // Pattern: @for ( var indexVar = start; indexVar <= end; indexVar++ ) { body }
  // Using [\s\S] to match any character including newlines
  // End can be either a number or a variable name
  const forPattern = /@for\s*\(\s*var\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\d+)\s*;\s*\1\s*<=?\s*([a-zA-Z_0-9][a-zA-Z0-9_.]*)\s*;\s*\1\+\+\s*\)\s*\{([\s\S]*?)\}/g;

  return markdown.replace(forPattern, (match, indexVar, start, end, body) => {
    const bodyTrimmed = body.trim();
    const convertedBody = convertNestedRazor(bodyTrimmed, indexVar);

    // Enumerable.Range(start, count) where count = end - start + 1
    // But if end is a variable, we need: Enumerable.Range(start, end - start + 1)
    const isEndNumeric = /^\d+$/.test(end);
    const count = isEndNumeric
      ? (parseInt(end) - parseInt(start) + 1).toString()
      : `${end} - ${start} + 1`;

    return `{string.Join("\\n", Enumerable.Range(${start}, ${count}).Select(${indexVar} => $@"${convertedBody}"))}`;
  });
}

/**
 * Convert @switch blocks to C# switch expressions
 *
 * @switch (expr) { case "x": body break; default: defaultBody break; } →
 * {expr switch { "x" => @"body", _ => @"defaultBody" }}
 *
 * @param {string} markdown
 * @returns {string}
 */
function convertSwitchBlocks(markdown) {
  const switchPattern = /@switch\s*\(([^)]+)\)\s*\{([\s\S]*?)\}/g;

  return markdown.replace(switchPattern, (match, expr, cases) => {
    const switchCases = [];

    // Match case patterns: case pattern: body break;
    const casePattern = /case\s+(.*?):([\s\S]*?)(?=break;)/g;
    const caseMatches = [...cases.matchAll(casePattern)];

    for (const caseMatch of caseMatches) {
      const pattern = caseMatch[1].trim();
      const body = caseMatch[2].trim();

      // Recursively convert nested Razor in body
      const convertedBody = convertNestedRazor(body);

      // Check if pattern contains 'var' (pattern guard)
      // e.g., "var q when q < 5"
      if (pattern.startsWith('var ')) {
        // Pattern guard - use $@"..." for interpolation
        switchCases.push(`${pattern} => $@"${convertedBody}"`);
      } else {
        // Simple pattern - use @"..." (no interpolation needed unless body has @)
        switchCases.push(`${pattern} => @"${convertedBody}"`);
      }
    }

    // Match default case: default: body break;
    const defaultMatch = cases.match(/default:([\s\S]*?)(?=break;)/);
    if (defaultMatch) {
      const body = defaultMatch[1].trim();
      const convertedBody = convertNestedRazor(body);
      switchCases.push(`_ => @"${convertedBody}"`);
    }

    return `{${expr} switch { ${switchCases.join(', ')} }}`;
  });
}

/**
 * Convert @(expression) to {(expression)}
 *
 * @param {string} markdown
 * @returns {string}
 */
function convertInlineExpressions(markdown) {
  // Convert @(expression) → {(expression)}
  return markdown.replace(/@\(([^)]+)\)/g, '{($1)}');
}

/**
 * Convert @variableName to {variableName}
 *
 * @param {string} markdown
 * @returns {string}
 */
function convertVariableReferences(markdown) {
  // Convert @variableName → {variableName}
  // Convert @variable.Property → {variable.Property}
  // Convert @variable.Method() → {variable.Method()}

  // Pattern: @ followed by identifier, with optional property/method chain
  // But skip Razor keywords (already converted)
  const keywords = ['if', 'else', 'foreach', 'for', 'while', 'switch'];

  return markdown.replace(/@([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*|\([^)]*\))*)/g, (match, varPath) => {
    const rootVar = varPath.split(/[.(]/)[0];

    // Skip Razor keywords (shouldn't happen - already converted)
    if (keywords.includes(rootVar)) {
      return match;
    }

    return `{${varPath}}`;
  });
}

/**
 * Recursively convert nested Razor syntax within bodies
 *
 * Used for converting Razor inside @if, @foreach, @for, @switch bodies
 *
 * @param {string} body - Body text that may contain nested Razor
 * @param {string} [itemVar] - Loop item variable to preserve (for @foreach, @for)
 * @returns {string} Body with Razor converted to C# interpolation placeholders
 */
function convertNestedRazor(body, itemVar = null) {
  let result = body;

  // Step 1: Convert @(expression)
  result = result.replace(/@\(([^)]+)\)/g, '{($1)}');

  // Step 2: If itemVar provided, convert @itemVar references
  if (itemVar) {
    // Convert @itemVar.property or @itemVar
    const itemPattern = new RegExp(`@${itemVar}(\\.[a-zA-Z_][a-zA-Z0-9_]*|\\([^)]*\\))*`, 'g');
    result = result.replace(itemPattern, (match) => {
      return `{${match.substring(1)}}`; // Remove @ and wrap in {}
    });
  }

  // Step 3: Convert other @variable references
  result = result.replace(/@([a-zA-Z_][a-zA-Z0-9_]*(?:\.[a-zA-Z_][a-zA-Z0-9_]*|\([^)]*\))*)/g, (match, varPath) => {
    // Don't double-convert itemVar (already done above)
    if (itemVar && varPath.startsWith(itemVar)) {
      return match;
    }

    return `{${varPath}}`;
  });

  // Step 4: Escape quotes in the body for C# verbatim strings
  // Replace " with "" for @"..." strings
  result = result.replace(/"/g, '""');

  return result;
}

module.exports = {
  convertRazorMarkdownToCSharp,
  convertIfBlocks,
  convertForeachBlocks,
  convertForBlocks,
  convertSwitchBlocks,
  convertInlineExpressions,
  convertVariableReferences,
  convertNestedRazor
};
