use super::SwcEmitter;
use super::super::decorated_ast::*;
use crate::parser::*;

impl SwcEmitter {
// ========================================================================
    // CUSTOM AST PROPERTIES - INFRASTRUCTURE GENERATION
    // ========================================================================

    pub fn emit_custom_prop_value_enum(&mut self) {
        self.emit_line("#[derive(Clone, Debug)]");
        self.emit_line("enum CustomPropValue {");
        self.indent += 1;
        self.emit_line("Bool(bool),");
        self.emit_line("I32(i32),");
        self.emit_line("I64(i64),");
        self.emit_line("F64(f64),");
        self.emit_line("Str(String),");
        // TODO: Add Vec, Map, and user-defined types if needed
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    pub fn emit_custom_prop_helpers(&mut self) {
        self.emit_line("impl State {");
        self.indent += 1;

        // get_node_id: Generate unique ID for AST nodes
        self.emit_line("fn get_node_id<T>(&self, node: &T) -> usize {");
        self.indent += 1;
        self.emit_line("// Use node memory address as ID");
        self.emit_line("node as *const T as usize");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // set_custom_prop: Set a custom property
        self.emit_line("fn set_custom_prop<T>(&mut self, node: &T, prop: &str, value: CustomPropValue) {");
        self.indent += 1;
        self.emit_line("let node_id = self.get_node_id(node);");
        self.emit_line("self.__custom_props");
        self.indent += 1;
        self.emit_line(".entry(node_id)");
        self.emit_line(".or_insert_with(std::collections::HashMap::new)");
        self.emit_line(".insert(prop.to_string(), value);");
        self.indent -= 1;
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // get_custom_prop: Get a custom property
        self.emit_line("fn get_custom_prop<T>(&self, node: &T, prop: &str) -> Option<&CustomPropValue> {");
        self.indent += 1;
        self.emit_line("let node_id = self.get_node_id(node);");
        self.emit_line("self.__custom_props");
        self.indent += 1;
        self.emit_line(".get(&node_id)");
        self.emit_line(".and_then(|m| m.get(prop))");
        self.indent -= 1;
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // delete_custom_prop: Remove a custom property
        self.emit_line("fn delete_custom_prop<T>(&mut self, node: &T, prop: &str) {");
        self.indent += 1;
        self.emit_line("let node_id = self.get_node_id(node);");
        self.emit_line("if let Some(props) = self.__custom_props.get_mut(&node_id) {");
        self.indent += 1;
        self.emit_line("props.remove(prop);");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    pub fn emit_traverse_stmt(&mut self, _traverse: &Box<DecoratedTraverseStmt>) {
        // Traverse statements should have been transformed by the Hoister stage
        // If we see one here, it's a placeholder that should emit a comment
        self.emit_line("// Traverse statement (should have been hoisted)");
    }

    /// Process a pub use statement - load compiled module code and track for emission
    pub fn process_pub_use(&mut self, use_stmt: &UseStmt) {
        // Only process file-based imports
        if !use_stmt.path.starts_with("./") && !use_stmt.path.starts_with("../") {
            return;
        }

        // Derive module name from path (use last segment only to avoid nested underscores)
        let path_segments: Vec<&str> = use_stmt.path.split('/').collect();
        let module_name = path_segments.last()
            .unwrap_or(&"module")
            .replace("-", "_");

        eprintln!("[EMITTER] Processing pub use: path='{}', base_dir={:?}, module_name='{}'",
            use_stmt.path, self.base_dir, module_name);

        // Try to find the compiled module's lib.rs
        let stripped_path = use_stmt.path.trim_start_matches("./").trim_start_matches("../");
        let module_dir = self.base_dir.join(stripped_path);
        let module_paths = [
            module_dir.join("lib.rs"),  // base/module/lib.rs
            self.base_dir.join(format!("{}.rs", stripped_path)),  // base/module.rs
        ];

        for module_path in &module_paths {
            eprintln!("[EMITTER] Checking path: {:?} (exists: {})", module_path, module_path.exists());
            if module_path.exists() {
                if let Ok(code) = std::fs::read_to_string(module_path) {
                    // Strip the standard SWC headers from the module code since main lib.rs will have them
                    let stripped_code = self.strip_module_headers(&code);

                    // Check for transitive dependencies (mod declarations in the loaded code)
                    let module_parent = module_path.parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| self.base_dir.clone());
                    self.load_transitive_dependencies(&stripped_code, &module_parent);

                    self.imported_modules.push((
                        module_name.clone(),
                        stripped_code,
                        use_stmt.imports.clone(),
                        false,  // Not a transitive dep, directly imported
                    ));
                    eprintln!("[EMITTER] Loaded module '{}' from {:?}", module_name, module_path);
                    return;
                }
            }
        }

        eprintln!("[EMITTER] Warning: Could not find compiled module for '{}' (tried {:?})",
            use_stmt.path, module_paths);
    }

    /// Load transitive dependencies by scanning for `mod xxx;` declarations in module code
    pub fn load_transitive_dependencies(&mut self, code: &str, module_dir: &std::path::Path) {
        // Find all `mod xxx;` declarations (not `mod xxx { ... }` inline modules)
        for line in code.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("mod ") && trimmed.ends_with(';') {
                // Extract module name: "mod foo;" -> "foo"
                let mod_name = trimmed
                    .trim_start_matches("mod ")
                    .trim_end_matches(';')
                    .trim();

                // Skip if we already have this module loaded
                if self.imported_modules.iter().any(|(name, _, _, _)| name == mod_name) {
                    eprintln!("[EMITTER] Transitive dep '{}' already loaded, skipping", mod_name);
                    continue;
                }

                // Try to find the corresponding .rs file
                let dep_path = module_dir.join(format!("{}.rs", mod_name));
                eprintln!("[EMITTER] Looking for transitive dep '{}' at {:?}", mod_name, dep_path);

                if dep_path.exists() {
                    if let Ok(dep_code) = std::fs::read_to_string(&dep_path) {
                        let stripped_dep_code = self.strip_module_headers(&dep_code);

                        // Recursively load this module's transitive dependencies
                        self.load_transitive_dependencies(&stripped_dep_code, module_dir);

                        self.imported_modules.push((
                            mod_name.to_string(),
                            stripped_dep_code,
                            vec![],  // No specific imports for transitive deps
                            true,    // This IS a transitive dep
                        ));
                        eprintln!("[EMITTER] Loaded transitive dep '{}' from {:?}", mod_name, dep_path);
                    }
                } else {
                    eprintln!("[EMITTER] Warning: Transitive dep '{}' not found at {:?}", mod_name, dep_path);
                }
            }
        }
    }

    /// Strip standard headers from module code (since main lib.rs will have them)
    /// Also adds #[path="..."] attributes to mod declarations so they can find sibling files
    pub fn strip_module_headers(&self, code: &str) -> String {
        let mut lines: Vec<&str> = code.lines().collect();
        let mut start_idx = 0;

        // Skip comment headers and use statements
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//")
                || trimmed.starts_with("use swc_")
                || trimmed.starts_with("use std::collections")
                || trimmed.is_empty()
            {
                start_idx = i + 1;
            } else {
                break;
            }
        }

        // Process remaining lines, adding #[path] attributes to mod declarations
        let mut result = Vec::new();
        for line in &lines[start_idx..] {
            let trimmed = line.trim();
            // Check for `mod xxx;` declarations (not inline modules with braces)
            if trimmed.starts_with("mod ") && trimmed.ends_with(';') {
                let mod_name = trimmed
                    .trim_start_matches("mod ")
                    .trim_end_matches(';')
                    .trim();
                // Add path attribute so Rust can find the sibling file
                result.push(format!("#[path = \"{}.rs\"]", mod_name));
            }
            result.push(line.to_string());
        }

        result.join("\n")
    }
}