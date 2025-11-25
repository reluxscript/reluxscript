/**
 * Props Extractor
 *
 * Extracts component props from function parameters.
 *
 * Handles:
 * - Destructured props: function Component({ user, loading })
 * - Single object prop: function Component(props)
 * - TypeScript type annotations: ({ user: User, loading: boolean })
 * - Converts TS types to C# types (boolean → bool, etc.)
 *
 * Returns array of prop objects: { name, type }
 */

// TODO: Move props extraction logic from processComponent function
// This is currently embedded in processComponent around lines 80-116

module.exports = {
  // extractProps(params, tsTypeToCSharpType) - to be implemented
};
