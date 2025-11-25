/**
 * Call expression handlers
 */

const t = require('@babel/types');

const { handleMathCalls } = require('./calls/handleMathCalls.cjs');
const { handleEncodeURIComponent, handleSetState, handleFetch, handleAlert, handleStringConstructor } = require('./calls/handleGlobalFunctions.cjs');
const { handlePromiseResolve, handlePromiseReject } = require('./calls/handlePromiseCalls.cjs');
const { handleObjectKeys, handleDateNow, handleConsoleLog } = require('./calls/handleObjectCalls.cjs');
const { handleToFixed, handleToLocaleString, handleToLowerCase, handleToUpperCase, handleTrim, handleSubstring, handlePadStart, handlePadEnd, handleResponseJson } = require('./calls/handleStringMethods.cjs');
const { handleMap, handleStateSetters } = require('./calls/handleArrayMethods.cjs');
const { handleOptionalMap } = require('./calls/handleOptionalMap.cjs');

/**
 * Generate call expression
 */
function generateCallExpression(node, generateCSharpExpression, generateCSharpStatement, currentComponent) {
  // Try each handler in order
  let result;

  result = handleMathCalls(node, generateCSharpExpression);
  if (result) return result;

  result = handleEncodeURIComponent(node, generateCSharpExpression);
  if (result) return result;

  result = handleSetState(node, generateCSharpExpression);
  if (result) return result;

  result = handleFetch(node, generateCSharpExpression);
  if (result) return result;

  result = handlePromiseResolve(node, generateCSharpExpression);
  if (result) return result;

  result = handlePromiseReject(node, generateCSharpExpression);
  if (result) return result;

  result = handleAlert(node, generateCSharpExpression);
  if (result) return result;

  result = handleStringConstructor(node, generateCSharpExpression);
  if (result) return result;

  result = handleObjectKeys(node, generateCSharpExpression);
  if (result) return result;

  result = handleDateNow(node);
  if (result) return result;

  result = handleConsoleLog(node, generateCSharpExpression);
  if (result) return result;

  result = handleResponseJson(node, generateCSharpExpression);
  if (result) return result;

  result = handleToFixed(node, generateCSharpExpression);
  if (result) return result;

  result = handleToLocaleString(node, generateCSharpExpression);
  if (result) return result;

  result = handleToLowerCase(node, generateCSharpExpression);
  if (result) return result;

  result = handleToUpperCase(node, generateCSharpExpression);
  if (result) return result;

  result = handleTrim(node, generateCSharpExpression);
  if (result) return result;

  result = handleSubstring(node, generateCSharpExpression);
  if (result) return result;

  result = handlePadStart(node, generateCSharpExpression);
  if (result) return result;

  result = handlePadEnd(node, generateCSharpExpression);
  if (result) return result;

  result = handleStateSetters(node, generateCSharpExpression, currentComponent);
  if (result) return result;

  result = handleMap(node, generateCSharpExpression, generateCSharpStatement, currentComponent);
  if (result) return result;

  // Generic function call
  const callee = generateCSharpExpression(node.callee);
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
  return `${callee}(${args})`;
}

/**
 * Generate optional call expression
 */
function generateOptionalCallExpression(node, generateCSharpExpression, generateCSharpStatement, currentComponent) {
  // Handle optional map
  const result = handleOptionalMap(node, generateCSharpExpression, generateCSharpStatement, currentComponent);
  if (result) return result;

  // Generic optional call
  const callee = generateCSharpExpression(node.callee);
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
  return `${callee}(${args})`;
}

module.exports = {
  generateCallExpression,
  generateOptionalCallExpression
};
