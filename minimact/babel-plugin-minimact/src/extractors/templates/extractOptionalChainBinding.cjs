const { buildOptionalMemberPath } = require('./buildMemberPath.cjs');

/**
 * Extract optional chaining binding
 * Handles: viewModel?.userEmail, obj?.prop1?.prop2
 * Returns: { nullable: true, binding: 'viewModel.userEmail' }
 */
function extractOptionalChainBinding(expr) {
  const path = buildOptionalMemberPath(expr);

  if (!path) {
    return null; // Can't build path
  }

  return {
    nullable: true,
    binding: path
  };
}

module.exports = { extractOptionalChainBinding };
