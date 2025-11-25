const t = require('@babel/types');

/**
 * Convert camelCase to kebab-case
 */
function camelToKebab(str) {
  return str.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`);
}

/**
 * Convert style value to CSS string
 */
function convertStyleValue(value) {
  if (t.isStringLiteral(value)) {
    return value.value;
  } else if (t.isNumericLiteral(value)) {
    return `${value.value}px`;
  } else if (t.isIdentifier(value)) {
    return value.name;
  }
  return String(value);
}

module.exports = { camelToKebab, convertStyleValue };
