/// Test: impl blocks for custom structs
///
/// ReluxScript should support impl blocks with methods that take self,
/// &self, and &mut self parameters.

plugin ImplBlocksTest {
    struct StringBuilder {
        lines: Vec<Str>,
        indent_level: i32,
    }

    /// Test basic impl block with self parameters
    impl StringBuilder {
        /// Associated function (no self)
        fn new() -> StringBuilder {
            return StringBuilder {
                lines: vec![],
                indent_level: 0,
            };
        }

        /// Method taking ownership (self)
        fn finish(self) -> Str {
            return self.lines.join("\n");
        }

        /// Immutable method (&self)
        fn get_line_count(&self) -> i32 {
            return self.lines.len();
        }

        /// Mutable method (&mut self)
        fn add_line(&mut self, line: Str) {
            self.lines.push(line);
        }

        /// Mutable method with return
        fn indent(&mut self) {
            self.indent_level += 1;
        }

        /// Method chaining (return Self)
        fn with_line(mut self, line: Str) -> Self {
            self.lines.push(line);
            return self;
        }
    }

    struct Template {
        path: Str,
        content: Str,
    }

    impl Template {
        fn new(path: Str) -> Template {
            return Template {
                path: path,
                content: "",
            };
        }

        fn set_content(&mut self, content: Str) {
            self.content = content;
        }

        fn get_path(&self) -> Str {
            return self.path.clone();
        }
    }

    /// Test calling methods
    pub fn test_method_calls() {
        let mut builder = StringBuilder::new();

        // Mutable method call
        builder.add_line("using System;");
        builder.add_line("namespace MyApp {");
        builder.indent();

        // Immutable method call
        let count = builder.get_line_count();

        // Consuming method call
        let code = builder.finish();
    }

    /// Test method chaining
    pub fn test_chaining() {
        let code = StringBuilder::new()
            .with_line("using System;")
            .with_line("namespace MyApp {")
            .finish();
    }

    /// Test in visitor context
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let mut builder = StringBuilder::new();

        builder.add_line("public void " + &node.id.name + "() {");
        builder.indent();
        builder.add_line("// Implementation");
        builder.add_line("}");

        let code = builder.finish();
    }

    pub fn test_multiple_impls() {
        let mut builder = StringBuilder::new();
        let mut template = Template::new("0.1");

        builder.add_line("test");
        template.set_content("content");

        let path = template.get_path();
        let code = builder.finish();
    }
}
