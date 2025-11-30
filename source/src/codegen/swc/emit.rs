//! Output Emission Utilities
//!
//! This module provides low-level string emission and indentation management
//! for the SWC code generator.

use super::SwcGenerator;

impl SwcGenerator {
    pub(super) fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    pub(super) fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    pub(super) fn emit_line(&mut self, s: &str) {
        self.emit_indent();
        self.emit(s);
        self.emit("\n");
    }
}
