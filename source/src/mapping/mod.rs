//! AST Mapping Tables for ReluxScript
//!
//! This module provides comprehensive mappings between:
//! - ReluxScript unified AST types
//! - Babel (ESTree) AST types
//! - SWC (swc_ecma_ast) types
//!
//! These mappings are used by code generators to emit correct platform-specific code.

mod nodes;
mod fields;
mod helpers;
mod patterns;
mod ts_helpers;

pub use nodes::{NodeMapping, NODE_MAPPINGS, get_node_mapping, get_node_mapping_by_visitor};
pub use fields::{FieldMapping, get_field_mapping};
pub use helpers::{HelperMapping, get_helper_for_field};
pub use patterns::{PatternMapping, get_pattern_check};
pub use ts_helpers::{TsHelperMapping, get_ts_helper, gen_ts_helper_babel, gen_ts_helper_swc};
