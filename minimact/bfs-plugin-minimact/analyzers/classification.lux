/**
 * Node Classification
 *
 * Classifies JSX nodes as static, dynamic, or hybrid based on dependencies.
 *
 * Classifications:
 * - 'static': No dependencies (can be compile-time VNode)
 * - 'client': All dependencies are client-side
 * - 'server': All dependencies are server-side
 * - 'hybrid': Mixed dependencies (needs runtime helpers)
 *
 * Returns classification string
 */

/**
 * Dependency information
 */
pub struct Dependency {
    pub name: Str,
    pub dep_type: Str,  // 'client' or 'server'
}

/**
 * Classify a JSX node based on dependencies
 *
 * @param deps - Set of dependencies with their types
 * @returns Classification: 'static', 'client', 'server', or 'hybrid'
 */
pub fn classify_node(deps: &HashSet<Dependency>) -> Str {
    // No dependencies means static
    if deps.is_empty() {
        return "static";
    }

    // Collect all unique dependency types
    let mut types = HashSet::new();
    for dep in deps {
        types.insert(dep.dep_type.clone());
    }

    // If all dependencies are of the same type
    if types.len() == 1 {
        if types.contains("client") {
            return "client";
        } else {
            return "server";
        }
    }

    // Mixed dependencies
    "hybrid"
}

/**
 * Check if a node is static (no dependencies)
 */
pub fn is_static(deps: &HashSet<Dependency>) -> bool {
    deps.is_empty()
}

/**
 * Check if a node is hybrid (mixed dependencies)
 */
pub fn is_hybrid(deps: &HashSet<Dependency>) -> bool {
    classify_node(deps) == "hybrid"
}
