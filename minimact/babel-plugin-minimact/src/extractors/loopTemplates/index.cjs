// Resolve circular dependency: extractChildrenTemplates needs extractElementTemplate
const { extractElementTemplate } = require('./extractElementTemplate.cjs');
const { setExtractElementTemplate } = require('./extractChildrenTemplates.cjs');
setExtractElementTemplate(extractElementTemplate);

module.exports = {
  ...require('./buildMemberExpressionPath.cjs'),
  ...require('./extractLiteralValue.cjs'),
  ...require('./extractLoopIdentifiers.cjs'),
  ...require('./extractLoopExpressions.cjs'),
  ...require('./buildBindingPath.cjs'),
  ...require('./extractConditionalTemplate.cjs'),
  ...require('./extractTemplateFromTemplateLiteral.cjs'),
  ...require('./extractTextTemplate.cjs'),
  ...require('./extractPropTemplates.cjs'),
  ...require('./extractChildrenTemplates.cjs'),
  ...require('./extractElementTemplate.cjs'),
  ...require('./extractKeyBinding.cjs'),
  ...require('./extractArrayBinding.cjs'),
  ...require('./extractJSXFromCallback.cjs'),
  ...require('./extractLoopTemplate.cjs'),
  ...require('./findMapExpressions.cjs'),
  ...require('./traverseJSX.cjs'),
};
