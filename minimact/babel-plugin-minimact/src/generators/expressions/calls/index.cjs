module.exports = {
  ...require('./handleMathCalls.cjs'),
  ...require('./handleGlobalFunctions.cjs'),
  ...require('./handlePromiseCalls.cjs'),
  ...require('./handleObjectCalls.cjs'),
  ...require('./handleStringMethods.cjs'),
  ...require('./handleArrayMethods.cjs'),
  ...require('./handleOptionalMap.cjs'),
};
