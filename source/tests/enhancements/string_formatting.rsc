/// Test: String Formatting and Concatenation
///
/// ReluxScript should support format! macro and string concatenation
/// for code generation tasks.

plugin StringFormattingTest {
    struct Prop {
        name: Str,
        prop_type: Str,
    }

    struct Component {
        name: Str,
        props: Vec<Prop>,
    }

    /// Test format! macro with single argument
    pub fn test_format_single() -> Str {
        let name = "MyComponent";
        let result = format!("public class {}", name);
        return result;
    }

    /// Test format! macro with multiple arguments
    pub fn test_format_multiple() -> Str {
        let prop_type = "string";
        let prop_name = "title";
        let result = format!("public {} {} {{ get; set; }}", prop_type, prop_name);
        return result;
    }

    /// Test format! with expressions
    pub fn test_format_expressions() -> Str {
        let count = 5;
        let name = "items";
        let result = format!("Found {} {} in list", count, name);
        return result;
    }

    /// Test multi-line format
    pub fn test_format_multiline() -> Str {
        let class_name = "Counter";
        let code = format!(
            "using System;\n\npublic class {} {{\n    // Implementation\n}}",
            class_name
        );
        return code;
    }

    /// Test string concatenation with +
    pub fn test_concat_basic() -> Str {
        let prefix = "Hello";
        let suffix = "World";
        let result = prefix + ", " + &suffix + "!";
        return result;
    }

    /// Test concatenation with to_string()
    pub fn test_concat_numbers() -> Str {
        let index = 42;
        let path = "items[" + &index.to_string() + "]";
        return path;
    }

    /// Test String::new() and push_str
    pub fn test_string_builder() -> Str {
        let mut code = String::new();

        code.push_str("using System;\n");
        code.push_str("using System.Collections.Generic;\n");
        code.push_str("\n");
        code.push_str("namespace MyApp {\n");
        code.push_str("}\n");

        return code;
    }

    /// Test combined format and push_str
    pub fn generate_csharp_class(component: &Component) -> Str {
        let mut code = String::new();

        // Using statements
        code.push_str("using System;\n\n");

        // Class declaration
        code.push_str(&format!("public class {} {{\n", component.name));

        // Properties
        for prop in &component.props {
            let property_line = format!(
                "    public {} {} {{ get; set; }}\n",
                prop.prop_type,
                prop.name
            );
            code.push_str(&property_line);
        }

        // Close class
        code.push_str("}\n");

        return code;
    }

    /// Test format in visitor context
    pub fn visit_function_declaration(node: &FunctionDeclaration) -> Str {
        let func_name = node.id.name.clone();

        // Build parameter list
        let mut params: Vec<Str> = vec![];
        for param in &node.params {
            if matches!(param, Identifier) {
                let param_str = format!("dynamic {}", param.name);
                params.push(param_str);
            }
        }

        // Generate method signature
        let param_list = params.join(", ");
        let signature = format!("public void {}({})", func_name, param_list);

        return signature;
    }

    /// Test path building with format
    pub fn build_jsx_path(indices: &Vec<i32>) -> Str {
        let mut parts: Vec<Str> = vec![];
        for idx in &indices {
            let s = format!("index: {}", idx);
            parts.push(s);
        }
        return parts.join(".");
    }

    /// Test error message formatting
    pub fn format_error(line: i32, col: i32, message: &Str) -> Str {
        return format!("Error at {}:{}: {}", line, col, message);
    }

    /// Test conditional formatting
    pub fn format_type(is_nullable: bool, base_type: &Str) -> Str {
        if is_nullable {
            return format!("{}?", base_type);
        } else {
            return base_type.clone();
        }
    }
}
