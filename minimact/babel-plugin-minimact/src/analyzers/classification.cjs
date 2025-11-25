/**
 * Node Classification
 *
 * Classifies JSX nodes as static, dynamic, or hybrid based on dependencies.
 *
 * Function to move:
 * - classifyNode(deps) - Classifies based on dependency set
 *
 * Classifications:
 * - 'static': No dependencies (can be compile-time VNode)
 * - 'dynamic': All dependencies are from same zone (state or props)
 * - 'hybrid': Mixed dependencies (needs runtime helpers)
 *
 * Currently returns 'hybrid' for any dependencies as a conservative approach.
 *
 * Returns classification string
 */

// TODO: Move classifyNode function here

/**
 * Classify a JSX node based on dependencies
 */
function classifyNode(deps) {
  if (deps.size === 0) {
    return 'static';
  }

  const types = new Set([...deps].map(d => d.type));

  if (types.size === 1) {
    return types.has('client') ? 'client' : 'server';
  }

  return 'hybrid'; // Mixed dependencies
}

module.exports = {
  classifyNode
};
