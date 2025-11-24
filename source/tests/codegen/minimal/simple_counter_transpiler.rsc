/// Ultra-Simple Counter Transpiler
/// Goal: Generate basic C# structure from Counter.tsx

writer SimpleCounterTranspiler {
    struct JSXNode {
        tag: Str,
        attributes: Vec<JSXAttr>,
        children: Vec<JSXChild>,
    }

    struct JSXAttr {
        name: Str,
        value: Str,
    }

    enum JSXChild {
        Element(JSXNode),
        Text(Str),
        Expression(Str),
    }

    struct ComponentInfo {
        name: Str,
        state_fields: Vec<StateField>,
        ref_fields: Vec<RefField>,
        effect_methods: Vec<EffectMethod>,
        event_methods: Vec<EventMethod>,
        render_jsx: Option<JSXNode>,
    }

    struct StateField {
        name: Str,
        initial_value: Str,
        csharp_type: Str,
    }

    struct RefField {
        name: Str,
    }

    struct EffectMethod {
        index: i32,
        dependencies: Vec<Str>,
    }

    struct EventMethod {
        name: Str,
    }

    struct State {
        current_component: Option<ComponentInfo>,
    }

    struct TranspilerOutput {
        csharp: Str,
    }

    fn init() -> State {
        State {
            current_component: None,
        }
    }

    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let component_name = if let Some(id) = &node.id {
            id.name.clone()
        } else {
            return;
        };

        // Only process PascalCase components
        if !component_name.chars().next().unwrap().is_uppercase() {
            return;
        }

        let mut component = ComponentInfo {
            name: component_name,
            state_fields: vec![],
            ref_fields: vec![],
            effect_methods: vec![],
            event_methods: vec![],
            render_jsx: None,
        };

        // Extract hooks using traverse
        if let Some(body) = &node.body {
            let mut effect_counter = 0;
            let mut jsx_node: Option<JSXNode> = None;

            traverse(body) capturing [&mut component, &mut effect_counter, &mut jsx_node] {
                fn visit_variable_declarator(decl: &VariableDeclarator) {
                    if let Some(init) = &decl.init {
                        if matches!(init, Expression::CallExpression(_)) {
                            let call = init;

                            // Check if it's a useState call
                            if matches!(call.callee, Expression::Identifier(_)) {
                                let callee_name = call.callee.name.clone();

                                if callee_name == "useState" {
                                    // Extract: const [count, setCount] = useState(0);
                                    if matches!(decl.id, Pattern::ArrayPattern(_)) {
                                        let arr = &decl.id;
                                        if arr.elements.len() >= 1 {
                                            if let Some(Pattern::Identifier(state_id)) = &arr.elements[0] {
                                                let state_name = state_id.name.clone();

                                                let initial_val = if call.arguments.len() > 0 {
                                                    Self::expr_to_csharp_value(&call.arguments[0])
                                                } else {
                                                    "null".to_string()
                                                };

                                                let csharp_type = if call.arguments.len() > 0 {
                                                    Self::infer_csharp_type_from_expr(&call.arguments[0])
                                                } else {
                                                    "dynamic".to_string()
                                                };

                                                component.state_fields.push(StateField {
                                                    name: state_name,
                                                    initial_value: initial_val,
                                                    csharp_type,
                                                });
                                            }
                                        }
                                    }
                                } else if callee_name == "useRef" {
                                    // Extract: const buttonRef = useRef(null);
                                    if matches!(decl.id, Pattern::Identifier(_)) {
                                        let ref_name = decl.id.name.clone();
                                        component.ref_fields.push(RefField {
                                            name: ref_name,
                                        });
                                    }
                                } else if callee_name == "useEffect" {
                                    // Extract dependencies from second argument
                                    let mut deps = vec![];
                                    if call.arguments.len() > 1 {
                                        if matches!(call.arguments[1], Expression::ArrayExpression(_)) {
                                            let arr = &call.arguments[1];
                                            for elem in &arr.elements {
                                                if let Some(Expression::Identifier(dep_id)) = elem {
                                                    deps.push(dep_id.name.clone());
                                                }
                                            }
                                        }
                                    }

                                    component.effect_methods.push(EffectMethod {
                                        index: effect_counter,
                                        dependencies: deps,
                                    });
                                    effect_counter += 1;
                                }
                            }
                        }
                    }
                }

                fn visit_function_declaration(func: &FunctionDeclaration) {
                    // Extract event handler functions
                    if let Some(id) = &func.id {
                        let method_name = Self::capitalize_first(&id.name);
                        component.event_methods.push(EventMethod {
                            name: method_name,
                        });
                    }
                }

                fn visit_return_statement(ret: &ReturnStatement) {
                    // Extract JSX from return statement
                    if let Some(arg) = &ret.argument {
                        if matches!(arg, Expression::JSXElement(_)) {
                            let jsx_elem = arg;
                            jsx_node = Some(Self::extract_jsx_node(jsx_elem));
                        }
                    }
                }
            }

            // Rebuild component with extracted JSX
            component = ComponentInfo {
                name: component.name.clone(),
                state_fields: component.state_fields.clone(),
                ref_fields: component.ref_fields.clone(),
                effect_methods: component.effect_methods.clone(),
                event_methods: component.event_methods.clone(),
                render_jsx: jsx_node,
            };
        }

        // Store component (rebuild state instead of direct mutation)
        self = State {
            current_component: Some(component),
        };
    }

    fn capitalize_first(s: &str) -> Str {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().chain(chars).collect(),
        }
    }

    fn expr_to_csharp_value(expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(n) => n.value.to_string(),
            Expression::StringLiteral(s) => format!("\"{}\"", s.value),
            Expression::BooleanLiteral(b) => if b.value { "true".to_string() } else { "false".to_string() },
            Expression::NullLiteral(_) => "null".to_string(),
            Expression::Identifier(id) => id.name.clone(),
            _ => "null".to_string(),
        }
    }

    fn infer_csharp_type_from_expr(expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(n) => {
                if n.value == n.value.floor() {
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

    fn extract_jsx_node(jsx: &JSXElement) -> JSXNode {
        // Extract tag name
        let tag = match &jsx.opening_element.name {
            JSXElementName::Identifier(id) => id.name.clone(),
            _ => "div".to_string(),
        };

        // Extract attributes
        let mut attributes = vec![];
        for attr in &jsx.opening_element.attributes {
            if matches!(attr, JSXAttribute::JSXAttribute(_)) {
                let attr_name = match &attr.name {
                    JSXAttributeName::Identifier(id) => id.name.clone(),
                    _ => continue,
                };

                let attr_value = if let Some(val) = &attr.value {
                    match val {
                        JSXAttributeValue::StringLiteral(s) => s.value.clone(),
                        JSXAttributeValue::JSXExpressionContainer(container) => {
                            if matches!(container.expression, JSXExpression::Expression(_)) {
                                let expr = &container.expression;
                                format!("{{expr:{}}}", Self::expr_to_csharp_value(expr))
                            } else {
                                String::new()
                            }
                        },
                        _ => String::new(),
                    }
                } else {
                    "true".to_string()
                };

                attributes.push(JSXAttr {
                    name: attr_name,
                    value: attr_value,
                });
            }
        }

        // Extract children
        let mut children = vec![];
        for child in &jsx.children {
            match child {
                JSXChild::JSXElement(elem) => {
                    children.push(JSXChild::Element(Self::extract_jsx_node(elem)));
                },
                JSXChild::JSXText(text) => {
                    let trimmed = text.value.trim();
                    if !trimmed.is_empty() {
                        children.push(JSXChild::Text(trimmed.to_string()));
                    }
                },
                JSXChild::JSXExpressionContainer(container) => {
                    if matches!(container.expression, JSXExpression::Expression(_)) {
                        let expr = &container.expression;
                        let expr_str = Self::expr_to_csharp_value(expr);
                        children.push(JSXChild::Expression(expr_str));
                    }
                },
                _ => {}
            }
        }

        JSXNode {
            tag,
            attributes,
            children,
        }
    }

    fn jsx_node_to_vnode_code(node: &JSXNode) -> Str {
        // Build attributes
        let mut attrs_code = vec![];
        for attr in &node.attributes {
            let value_code = if attr.value.starts_with("{expr:") {
                // It's an expression
                attr.value.trim_start_matches("{expr:").trim_end_matches("}").to_string()
            } else {
                // It's a string literal
                format!("\"{}\"", attr.value)
            };

            attrs_code.push(format!("                [\"{}\"] = {}", attr.name, value_code));
        }

        // Build children
        let mut children_code = vec![];
        for child in &node.children {
            match child {
                JSXChild::Element(elem) => {
                    children_code.push(Self::jsx_node_to_vnode_code(elem));
                },
                JSXChild::Text(text) => {
                    children_code.push(format!("\"{}\"", text));
                },
                JSXChild::Expression(expr) => {
                    children_code.push(format!("$\"{{{}}}\"", expr));
                },
            }
        }

        // Generate VElement code
        let mut result = format!("new VElement(\"{}\", ", node.tag);

        if attrs_code.is_empty() && children_code.is_empty() {
            result.push_str("null, null)");
        } else if attrs_code.is_empty() {
            result.push_str("null, ");
            if children_code.len() == 1 {
                result.push_str(&children_code[0]);
            } else {
                result.push_str("new VNode[]\n            {\n                ");
                result.push_str(&children_code.join(",\n                "));
                result.push_str("\n            }");
            }
            result.push(')');
        } else {
            result.push_str("new Dictionary<string, string>\n            {\n");
            result.push_str(&attrs_code.join(",\n"));
            result.push_str("\n            }, ");

            if children_code.len() == 0 {
                result.push_str("null");
            } else if children_code.len() == 1 {
                result.push_str(&children_code[0]);
            } else {
                result.push_str("new VNode[]\n            {\n                ");
                result.push_str(&children_code.join(",\n                "));
                result.push_str("\n            }");
            }
            result.push(')');
        }

        result
    }

    fn finish(&self) -> TranspilerOutput {
        let mut lines = vec![];

        if let Some(component) = &self.current_component {
            // Usings
            lines.push("using Minimact;".to_string());
            lines.push("using System;".to_string());
            lines.push("using System.Collections.Generic;".to_string());
            lines.push("".to_string());

            // Namespace
            lines.push("namespace Generated.Components".to_string());
            lines.push("{".to_string());

            // Class
            lines.push("    [MinimactComponent]".to_string());
            lines.push(format!("    public class {} : MinimactComponent", component.name));
            lines.push("    {".to_string());

            // useState fields
            for field in &component.state_fields {
                lines.push(format!("        [UseState({})]", field.initial_value));
                lines.push(format!("        private {} {};", field.csharp_type, field.name));
                lines.push("".to_string());
            }

            // useRef fields
            for ref_field in &component.ref_fields {
                lines.push("        [UseRef(null)]".to_string());
                lines.push(format!("        private ElementRef {};", ref_field.name));
                lines.push("".to_string());
            }

            // useEffect methods
            for effect in &component.effect_methods {
                let deps_str = if effect.dependencies.is_empty() {
                    "".to_string()
                } else {
                    format!("\"{}\"", effect.dependencies.join("\", \""))
                };
                lines.push(format!("        [UseEffect({})]", deps_str));
                lines.push(format!("        private void Effect_{}()", effect.index));
                lines.push("        {".to_string());
                lines.push("            // Effect body would go here".to_string());
                lines.push("        }".to_string());
                lines.push("".to_string());
            }

            // Render method
            lines.push("        protected override VNode Render()".to_string());
            lines.push("        {".to_string());
            if let Some(jsx) = &component.render_jsx {
                let vnode_code = Self::jsx_node_to_vnode_code(jsx);
                lines.push(format!("            return {};", vnode_code));
            } else {
                lines.push("            return new VElement(\"div\", null, null);".to_string());
            }
            lines.push("        }".to_string());
            lines.push("".to_string());

            // Event handlers
            for method in &component.event_methods {
                lines.push(format!("        private void {}()", method.name));
                lines.push("        {".to_string());
                lines.push("            // Method body would go here".to_string());
                lines.push("        }".to_string());
                lines.push("".to_string());
            }

            lines.push("    }".to_string());
            lines.push("}".to_string());
        }

        TranspilerOutput {
            csharp: lines.join("\n"),
        }
    }
}
