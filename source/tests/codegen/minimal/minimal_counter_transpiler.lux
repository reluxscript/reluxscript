/// Minimal Minimact TSX to C# Transpiler
/// Goal: Successfully generate C# for Counter.tsx

writer MinimalCounterTranspiler {
    struct State {
        component_name: Str,
        use_state: Vec<UseStateInfo>,
        use_effect: Vec<UseEffectInfo>,
        use_ref: Vec<UseRefInfo>,
        event_handlers: Vec<EventHandler>,
        render_jsx: Option<JSXElement>,
    }

    struct UseStateInfo {
        name: Str,
        setter: Str,
        initial_value: Str,  // C# expression
        csharp_type: Str,    // "int", "string", etc.
    }

    struct UseEffectInfo {
        index: i32,
        body: Str,            // C# code
        dependencies: Vec<Str>,
    }

    struct UseRefInfo {
        name: Str,
        initial_value: Str,
    }

    struct EventHandler {
        name: Str,
        body: Str,  // C# code
    }

    fn init() -> State {
        State {
            component_name: String::new(),
            use_state: vec![],
            use_effect: vec![],
            use_ref: vec![],
            event_handlers: vec![],
            render_jsx: None,
        }
    }

    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        // Get component name
        if let Some(id) = &node.id {
            self.component_name = id.name.clone();
        } else {
            return;
        }

        // Extract hooks and JSX from function body
        if let Some(body) = &node.body {
            self.process_function_body(body);
        }
    }

    fn process_function_body(&mut self, body: &BlockStatement) {
        let mut effect_index = 0;

        for stmt in &body.body {
            // Extract variable declarations (hooks)
            match stmt {
                Statement::VariableDeclaration(var_decl) => {
                    for declarator in &var_decl.declarations {
                        if let Some(init) = &declarator.init {
                            match init {
                                Expression::CallExpression(call) => {
                                    match &call.callee {
                                        Expression::Identifier(callee) => {
                                            match callee.name.as_str() {
                                                "useState" => self.extract_use_state(declarator, call),
                                                "useEffect" => {
                                                    self.extract_use_effect(call, effect_index);
                                                    effect_index += 1;
                                                },
                                                "useRef" => self.extract_use_ref(declarator, call),
                                                _ => {}
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                        }
                    }
                },
                // Extract return statement (JSX)
                Statement::ReturnStatement(ret) => {
                    if let Some(arg) = &ret.argument {
                        match arg {
                            Expression::JSXElement(jsx) => {
                                self.render_jsx = Some(jsx.clone());
                            },
                            _ => {}
                        }
                    }
                },
                // Extract function declarations (event handlers)
                Statement::FunctionDeclaration(func) => {
                    if let Some(id) = &func.id {
                        let handler_name = self.to_csharp_method_name(&id.name);
                        let body = self.function_body_to_csharp(&func.body);

                        self.event_handlers.push(EventHandler {
                            name: handler_name,
                            body,
                        });
                    }
                },
                _ => {}
            }
        }
    }

    fn extract_use_state(&mut self, declarator: &VariableDeclarator, call: &CallExpression) {
        // Extract: const [count, setCount] = useState(0);
        match &declarator.id {
            Pattern::ArrayPattern(array_pattern) => {
                if array_pattern.elements.len() >= 2 {
                    let state_name = self.get_pattern_name(&array_pattern.elements[0]);
                    let setter_name = self.get_pattern_name(&array_pattern.elements[1]);

                    // Get initial value
                    let initial_value = if call.arguments.len() > 0 {
                        self.expr_to_csharp(&call.arguments[0])
                    } else {
                        "null".to_string()
                    };

                    // Infer C# type from initial value
                    let csharp_type = if call.arguments.len() > 0 {
                        self.infer_csharp_type(&call.arguments[0])
                    } else {
                        "dynamic".to_string()
                    };

                    self.use_state.push(UseStateInfo {
                        name: state_name,
                        setter: setter_name,
                        initial_value,
                        csharp_type,
                    });
                }
            },
            _ => {}
        }
    }

    fn extract_use_effect(&mut self, call: &CallExpression, index: i32) {
        // Extract: useEffect(() => { console.log(...) }, [count]);
        let mut body = String::new();
        let mut dependencies = vec![];

        // Get callback function (first argument)
        if call.arguments.len() > 0 {
            match &call.arguments[0] {
                Expression::ArrowFunctionExpression(arrow) => {
                    body = self.function_body_to_csharp(&arrow.body);
                },
                _ => {}
            }
        }

        // Get dependencies array (second argument)
        if call.arguments.len() > 1 {
            match &call.arguments[1] {
                Expression::ArrayExpression(arr) => {
                    for elem in &arr.elements {
                        if let Some(expr) = elem {
                            match expr {
                                Expression::Identifier(id) => {
                                    dependencies.push(id.name.clone());
                                },
                                _ => {}
                            }
                        }
                    }
                },
                _ => {}
            }
        }

        self.use_effect.push(UseEffectInfo {
            index,
            body,
            dependencies,
        });
    }

    fn extract_use_ref(&mut self, declarator: &VariableDeclarator, call: &CallExpression) {
        // Extract: const buttonRef = useRef(null);
        match &declarator.id {
            Pattern::Identifier(id) => {
                let initial_value = if call.arguments.len() > 0 {
                    self.expr_to_csharp(&call.arguments[0])
                } else {
                    "null".to_string()
                };

                self.use_ref.push(UseRefInfo {
                    name: id.name.clone(),
                    initial_value,
                });
            },
            _ => {}
        }
    }

    fn get_pattern_name(&self, pattern: &Option<Pattern>) -> Str {
        if let Some(pat) = pattern {
            match pat {
                Pattern::Identifier(id) => id.name.clone(),
                _ => String::new()
            }
        } else {
            String::new()
        }
    }

    fn expr_to_csharp(&self, expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(num) => num.value.to_string(),
            Expression::StringLiteral(s) => format!("\"{}\"", s.value),
            Expression::BooleanLiteral(b) => if b.value { "true" } else { "false" }.to_string(),
            Expression::NullLiteral(_) => "null".to_string(),
            Expression::Identifier(id) => id.name.clone(),
            _ => "null".to_string(),
        }
    }

    fn infer_csharp_type(&self, expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(num) => {
                if num.value == num.value.floor() {
                    "int".to_string()
                } else {
                    "double".to_string()
                }
            },
            Expression::StringLiteral(_) => "string".to_string(),
            Expression::BooleanLiteral(_) => "bool".to_string(),
            Expression::ArrayExpression(_) => "List<dynamic>".to_string(),
            Expression::ObjectExpression(_) => "Dictionary<string, dynamic>".to_string(),
            _ => "dynamic".to_string(),
        }
    }

    fn function_body_to_csharp(&self, body: &FunctionBody) -> Str {
        // Simplified: just convert console.log to Console.WriteLine
        let mut result = String::new();

        match body {
            FunctionBody::BlockStatement(block) => {
                for stmt in &block.body {
                    match stmt {
                        Statement::ExpressionStatement(expr_stmt) => {
                            match &expr_stmt.expression {
                                Expression::CallExpression(call) => {
                                    match &call.callee {
                                        Expression::MemberExpression(member) => {
                                            // Check for console.log
                                            let is_log = self.is_console_log(member);
                                            if is_log {
                                                result.push_str("        Console.WriteLine(");
                                                if call.arguments.len() > 0 {
                                                    result.push_str(&self.template_literal_to_csharp(&call.arguments[0]));
                                                }
                                                result.push_str(");\n");
                                            }
                                        },
                                        _ => {}
                                    }
                                },
                                _ => {}
                            }
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }

        result
    }

    fn is_console_log(&self, member: &MemberExpression) -> bool {
        match &member.object {
            Expression::Identifier(obj) => {
                match &member.property {
                    MemberProperty::Identifier(prop) => {
                        obj.name == "console" && prop.name == "log"
                    },
                    _ => false
                }
            },
            _ => false
        }
    }

    fn template_literal_to_csharp(&self, expr: &Expression) -> Str {
        match expr {
            Expression::TemplateLiteral(tmpl) => {
                let mut result = String::from("$\"");
                for (i, quasi) in tmpl.quasis.iter().enumerate() {
                    result.push_str(&quasi.value.cooked);
                    if i < tmpl.expressions.len() {
                        result.push('{');
                        result.push_str(&self.expr_to_csharp(&tmpl.expressions[i]));
                        result.push('}');
                    }
                }
                result.push('"');
                result
            },
            _ => self.expr_to_csharp(expr),
        }
    }

    fn to_csharp_method_name(&self, js_name: &str) -> Str {
        // Convert camelCase to PascalCase
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in js_name.chars() {
            if capitalize_next {
                result.push(ch.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn jsx_to_vnode(&self, jsx: &JSXElement) -> Str {
        let tag = self.get_jsx_tag_name(&jsx.opening_element.name);

        // Build attributes dictionary
        let mut attrs = vec![];
        for attr in &jsx.opening_element.attributes {
            match attr {
                JSXAttribute::JSXAttribute(attr_node) => {
                    let name = self.get_jsx_attr_name(&attr_node.name);
                    let value = if let Some(val) = &attr_node.value {
                        self.jsx_attr_value_to_csharp(val)
                    } else {
                        "\"true\"".to_string()
                    };

                    // Special handling for event handlers
                    if name.starts_with("on") {
                        // Convert to method name (e.g., onClick -> "Increment")
                        attrs.push(format!("                [\"{}n\"] = \"{}\"", name, value.replace("\"", "")));
                    } else if name == "ref" {
                        attrs.push(format!("                [\"ref\"] = \"{}\"", value.replace("\"", "")));
                    } else {
                        attrs.push(format!("                [\"{}\"] = {}", name, value));
                    }
                },
                _ => {}
            }
        }

        // Build children
        let mut children = vec![];
        for child in &jsx.children {
            match child {
                JSXChild::JSXElement(elem) => {
                    children.push(self.jsx_to_vnode(elem));
                },
                JSXChild::JSXText(text) => {
                    let trimmed = text.value.trim();
                    if !trimmed.is_empty() {
                        children.push(format!("\"{}\"", trimmed));
                    }
                },
                JSXChild::JSXExpressionContainer(container) => {
                    match &container.expression {
                        JSXExpression::Expression(expr) => {
                            children.push(format!("$\"{{{}}\"", self.expr_to_csharp(expr)));
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        // Generate VElement
        let mut result = format!("new VElement(\"{}\", ", tag);

        if attrs.is_empty() && children.is_empty() {
            result.push_str("null, null)");
        } else if attrs.is_empty() {
            result.push_str("null, ");
            if children.len() == 1 {
                result.push_str(&children[0]);
            } else {
                result.push_str("new VNode[]\n            {\n                ");
                result.push_str(&children.join(",\n                "));
                result.push_str("\n            }");
            }
            result.push(')');
        } else {
            result.push_str("new Dictionary<string, string>\n            {\n");
            result.push_str(&attrs.join(",\n"));
            result.push_str("\n            }, ");

            if children.len() == 1 {
                result.push_str(&children[0]);
            } else if children.len() > 1 {
                result.push_str("new VNode[]\n            {\n                ");
                result.push_str(&children.join(",\n                "));
                result.push_str("\n            }");
            } else {
                result.push_str("null");
            }
            result.push(')');
        }

        result
    }

    fn get_jsx_tag_name(&self, name: &JSXElementName) -> Str {
        match name {
            JSXElementName::Identifier(id) => id.name.clone(),
            _ => "div".to_string(),
        }
    }

    fn get_jsx_attr_name(&self, name: &JSXAttributeName) -> Str {
        match name {
            JSXAttributeName::Identifier(id) => id.name.clone(),
            _ => "unknown".to_string(),
        }
    }

    fn jsx_attr_value_to_csharp(&self, value: &JSXAttributeValue) -> Str {
        match value {
            JSXAttributeValue::StringLiteral(s) => format!("\"{}\"", s.value),
            JSXAttributeValue::JSXExpressionContainer(container) => {
                match &container.expression {
                    JSXExpression::Expression(expr) => self.expr_to_csharp(expr),
                    _ => "null".to_string()
                }
            },
            _ => "null".to_string(),
        }
    }

    fn finish(&self) -> TranspilerOutput {
        let mut lines = vec![];

        // Usings
        lines.push("using Minimact;".to_string());
        lines.push("using System;".to_string());
        lines.push("using System.Collections.Generic;".to_string());
        lines.push("using System.Linq;".to_string());
        lines.push("".to_string());

        // Namespace
        lines.push("namespace Generated.Components".to_string());
        lines.push("{".to_string());

        // Class declaration
        lines.push("    [MinimactComponent]".to_string());
        lines.push(format!("    public class {} : MinimactComponent", self.component_name));
        lines.push("    {".to_string());

        // useState fields
        for state in &self.use_state {
            lines.push(format!("        [UseState({})]", state.initial_value));
            lines.push(format!("        private {} {};", state.csharp_type, state.name));
            lines.push("".to_string());
        }

        // useRef fields
        for ref_info in &self.use_ref {
            lines.push(format!("        [UseRef({})]", ref_info.initial_value));
            lines.push(format!("        private ElementRef {};", ref_info.name));
            lines.push("".to_string());
        }

        // useEffect methods
        for effect in &self.use_effect {
            let deps = if effect.dependencies.is_empty() {
                String::new()
            } else {
                format!("\"{}\"", effect.dependencies.join("\", \""))
            };
            lines.push(format!("        [UseEffect({})]", deps));
            lines.push(format!("        private void Effect_{}()", effect.index));
            lines.push("        {".to_string());
            lines.push(effect.body.clone());
            lines.push("        }".to_string());
            lines.push("".to_string());
        }

        // Render method
        lines.push("        protected override VNode Render()".to_string());
        lines.push("        {".to_string());
        if let Some(jsx) = &self.render_jsx {
            lines.push(format!("            return {};", self.jsx_to_vnode(jsx)));
        } else {
            lines.push("            return VNull();".to_string());
        }
        lines.push("        }".to_string());
        lines.push("".to_string());

        // Event handler methods
        for handler in &self.event_handlers {
            lines.push(format!("        private void {}()", handler.name));
            lines.push("        {".to_string());
            lines.push(handler.body.clone());
            lines.push("        }".to_string());
        }

        lines.push("    }".to_string());
        lines.push("}".to_string());

        TranspilerOutput {
            csharp: lines.join("\n"),
        }
    }

    struct TranspilerOutput {
        csharp: Str,
    }
}
