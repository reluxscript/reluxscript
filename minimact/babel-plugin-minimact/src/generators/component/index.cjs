module.exports = {
  ...require('./inferTypes.cjs'),
  ...require('./generateAttributes.cjs'),
  ...require('./generateFields.cjs'),
  ...require('./generateMethods.cjs')
};
