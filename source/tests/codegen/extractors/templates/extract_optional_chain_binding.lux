/**
 * Extract Optional Chain Binding
 *
 * Extracts binding from optional chaining expressions
 */

use "./build_member_path.rsc" { build_optional_member_path };

/**
 * Optional chain binding result
 */
pub struct OptionalChainBinding {
    pub nullable: bool,
    pub binding: Str,
}

/**
 * Extract optional chaining binding
 * Handles: viewModel?.userEmail, obj?.prop1?.prop2
 * Returns: { nullable: true, binding: 'viewModel.userEmail' }
 */
pub fn extract_optional_chain_binding(expr: &Expression) -> Option<OptionalChainBinding> {
    let path = build_optional_member_path(expr)?;

    Some(OptionalChainBinding {
        nullable: true,
        binding: path,
    })
}
