/// Test: Method chaining on custom types
///
/// ReluxScript should support calling methods on custom types using the .
/// operator, and chaining multiple method calls.

plugin MethodChainingTest {
    struct StringBuilder {
        parts: Vec<Str>,
    }

    impl StringBuilder {
        fn new() -> StringBuilder {
            return StringBuilder {
                parts: vec![],
            };
        }

        /// Builder pattern - return self for chaining
        fn append(mut self, text: Str) -> Self {
            self.parts.push(text);
            return self;
        }

        fn append_line(mut self, line: Str) -> Self {
            self.parts.push(line);
            self.parts.push("\n");
            return self;
        }

        fn build(self) -> Str {
            return self.parts.join("");
        }
    }

    /// Test simple chaining
    pub fn test_simple_chain() -> Str {
        let result = StringBuilder::new()
            .append("Hello")
            .append(" ")
            .append("World")
            .build();

        return result;
    }

    /// Test longer chains
    pub fn test_long_chain() -> Str {
        let code = StringBuilder::new()
            .append_line("using System;")
            .append_line("using System.Collections.Generic;")
            .append_line("")
            .append_line("public class MyClass {")
            .append_line("    // Implementation")
            .append_line("}")
            .build();

        return code;
    }

    /// Test in visitor
    pub fn visit_function_declaration(node: &FunctionDeclaration) -> Str {
        let signature = StringBuilder::new()
            .append("public void ")
            .append(&node.id.name)
            .append("()")
            .build();

        return signature;
    }

    /// Test mixed mutable and immutable methods
    struct Counter {
        value: i32,
    }

    impl Counter {
        fn new() -> Counter {
            return Counter { value: 0 };
        }

        fn increment(&mut self) {
            self.value += 1;
        }

        fn get(&self) -> i32 {
            return self.value;
        }

        fn reset(&mut self) {
            self.value = 0;
        }
    }

    pub fn test_mutable_methods() {
        let mut counter = Counter::new();

        // Call methods with .
        counter.increment();
        counter.increment();
        let value = counter.get();
        counter.reset();
    }

    /// Test method calls on fields
    struct Component {
        builder: StringBuilder,
    }

    pub fn test_field_method_call() {
        let mut comp = Component {
            builder: StringBuilder::new(),
        };

        // Method call on field
        let result = comp.builder
            .append("test")
            .build();
    }
}
