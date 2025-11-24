/// Test: Impl blocks with various method signatures
///
/// ReluxScript should support impl blocks with:
/// - Associated functions (no self)
/// - Methods with self, &self, &mut self, mut self

plugin ImplMethodsTest {
    struct Builder {
        code: Str,
        indent: i32,
    }

    impl Builder {
        // Associated function (no self) - acts like static method
        fn new() -> Self {
            Self {
                code: "",
                indent: 0,
            }
        }

        fn with_code(code: Str) -> Self {
            Self {
                code: code,
                indent: 0,
            }
        }

        // Immutable reference - can read but not modify
        fn get_code(&self) -> Str {
            self.code.clone()
        }

        fn get_indent(&self) -> i32 {
            self.indent
        }

        // Mutable reference - can modify in place
        fn append(&mut self, text: Str) {
            self.code.push_str(&text);
        }

        fn increment_indent(&mut self) {
            self.indent += 1;
        }

        // Consuming self - takes ownership, returns modified version
        fn with_indent(mut self, indent: i32) -> Self {
            self.indent = indent;
            self
        }

        // Consuming self - builder pattern
        fn build(self) -> Str {
            self.code
        }
    }

    fn test_usage() {
        // Associated function call
        let mut builder = Builder::new();

        // Mutable method call
        builder.append("hello");
        builder.increment_indent();

        // Immutable method call
        let code = builder.get_code();
        let indent = builder.get_indent();

        // Consuming method (builder pattern)
        let result = Builder::new()
            .with_indent(2)
            .build();
    }
}
