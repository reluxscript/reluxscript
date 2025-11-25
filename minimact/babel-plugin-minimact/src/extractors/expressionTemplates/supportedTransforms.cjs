/**
 * Supported transformation types
 */
const SUPPORTED_TRANSFORMS = {
  // Number formatting
  'toFixed': { type: 'numberFormat', safe: true },
  'toPrecision': { type: 'numberFormat', safe: true },
  'toExponential': { type: 'numberFormat', safe: true },

  // String operations
  'toUpperCase': { type: 'stringTransform', safe: true },
  'toLowerCase': { type: 'stringTransform', safe: true },
  'trim': { type: 'stringTransform', safe: true },
  'substring': { type: 'stringTransform', safe: true },
  'substr': { type: 'stringTransform', safe: true },
  'slice': { type: 'stringTransform', safe: true },

  // Array operations
  'length': { type: 'property', safe: true },
  'join': { type: 'arrayTransform', safe: true },

  // Math operations (handled separately via binary expressions)
  // +, -, *, /, %
};

module.exports = { SUPPORTED_TRANSFORMS };
