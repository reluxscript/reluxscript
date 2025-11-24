/**
 * Extract Bindings From Condition
 *
 * Extracts all state bindings from a condition expression
 */

use "./build_member_path.rsc" { build_member_path };

/**
 * Extract all state bindings from a condition expression
 * Example: myState1 && !myState2 → ["myState1", "myState2"]
 */
pub fn extract_bindings_from_condition(expr: &Expression) -> Vec<Str> {
    let mut bindings = HashSet::new();

    traverse_expr(expr, &mut bindings);

    bindings.into_iter().collect()
}

/**
 * Internal: Traverse expression tree and collect bindings
 */
fn traverse_expr(node: &Expression, bindings: &mut HashSet<Str>) {
    match node {
        Expression::Identifier(ref id) => {
            bindings.insert(id.name.clone());
        }

        Expression::LogicalExpression(ref logical) => {
            traverse_expr(&logical.left, bindings);
            traverse_expr(&logical.right, bindings);
        }

        Expression::UnaryExpression(ref unary) => {
            traverse_expr(&unary.argument, bindings);
        }

        Expression::BinaryExpression(ref binary) => {
            traverse_expr(&binary.left, bindings);
            traverse_expr(&binary.right, bindings);
        }

        Expression::MemberExpression(_) => {
            if let Some(path) = build_member_path(node) {
                bindings.insert(path);
            }
        }

        _ => {
            // Other expression types don't contribute bindings
        }
    }
}
