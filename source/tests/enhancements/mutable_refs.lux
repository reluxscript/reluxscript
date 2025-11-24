/// Test: Mutable References in Visitor Callbacks
///
/// ReluxScript should support &mut references being captured and modified
/// within traverse blocks.

plugin MutableRefsTest {
    struct Component {
        name: Str,
        state_vars: Vec<Str>,
        effect_deps: Vec<Vec<Str>>,
    }

    /// Test mutable reference capture in traverse
    pub fn visit_function_declaration(node: &FunctionDeclaration) -> Component {
        let mut component = Component {
            name: node.id.name.clone(),
            state_vars: vec![],
            effect_deps: vec![],
        };

        if let Some(body) = &node.body {
            // component should be mutably borrowed here
            traverse(body) {
                fn visit_variable_declarator(decl: &VariableDeclarator) {
                    if let Some(init) = &decl.init {
                        if matches!(init, CallExpression) {
                            let call = init.clone();
                            if matches!(call.callee, Identifier) {
                                let callee_name = call.callee.name.clone();
                                if callee_name == "useState" {
                                    // Mutate captured component
                                    if matches!(decl.id, ArrayPattern) {
                                        let arr = decl.id.clone();
                                        if arr.elements.len() > 0 {
                                            let var_name = arr.elements[0].name.clone();
                                            component.state_vars.push(var_name);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                fn visit_call_expression(call: &CallExpression) {
                    if matches!(call.callee, Identifier) {
                        let callee_name = call.callee.name.clone();
                        if callee_name == "useEffect" {
                            // Extract dependencies
                            if call.arguments.len() > 1 {
                                let deps_arg = &call.arguments[1];
                                if matches!(deps_arg, ArrayExpression) {
                                    let mut deps: Vec<Str> = vec![];
                                    for elem in &deps_arg.elements {
                                        if matches!(elem, Identifier) {
                                            deps.push(elem.name.clone());
                                        }
                                    }
                                    component.effect_deps.push(deps);
                                }
                            }
                        }
                    }
                }
            }
        }

        return component;
    }

    /// Test multiple mutable references
    pub fn analyze_hooks(body: &BlockStatement) {
        let mut state_count = 0;
        let mut effect_count = 0;
        let mut ref_count = 0;

        traverse(body) capturing [&mut state_count, &mut effect_count, &mut ref_count] {
            fn visit_call_expression(call: &CallExpression) {
                if matches!(call.callee, Identifier) {
                    let name = call.callee.name.clone();
                    if name == "useState" {
                        state_count += 1;
                    } else if name == "useEffect" {
                        effect_count += 1;
                    } else if name == "useRef" {
                        ref_count += 1;
                    }
                }
            }
        }

        // All counters should be accessible and modified
        let total = state_count + effect_count + ref_count;
    }
}
