/// Test: Field Type Mappings
///
/// Tests for proper field type handling:
/// 1. ArrayPat.elems are Option<Pat> not Expr
/// 2. CallExpr.args are Vec<ExprOrSpread> not Vec<Expr>

plugin FieldTypesTest {
    /// Test ArrayPat.elems - elements are Option<Pat>
    /// In SWC: ArrayPat { elems: Vec<Option<Pat>>, ... }
    pub fn test_array_pattern_elements(pat: &ArrayPattern) {
        // elems is Vec<Option<Pat>>, not Vec<Expr>
        for elem in &pat.elements {
            // elem is Option<Pat>, need to unwrap
            if let Some(p) = elem {
                // p is Pat, not Expr
                if matches!(p, Identifier) {
                    let name = p.name.clone();
                }
            }
        }
    }

    /// Test extracting useState destructuring
    /// const [value, setValue] = useState(initial)
    pub fn extract_use_state(decl: &VariableDeclarator) {
        if matches!(decl.id, ArrayPattern) {
            let arr = decl.id.clone();

            // Get first element (state variable)
            if arr.elements.len() > 0 {
                let first_elem = &arr.elements[0];
                // first_elem is Option<Pat>
                if let Some(pat) = first_elem {
                    if matches!(pat, Identifier) {
                        let state_var = pat.name.clone();
                    }
                }
            }

            // Get second element (setter function)
            if arr.elements.len() > 1 {
                let second_elem = &arr.elements[1];
                if let Some(pat) = second_elem {
                    if matches!(pat, Identifier) {
                        let setter_var = pat.name.clone();
                    }
                }
            }
        }
    }

    /// Test CallExpr.args - arguments are ExprOrSpread
    /// In SWC: CallExpr { args: Vec<ExprOrSpread>, ... }
    /// ExprOrSpread { spread: Option<Span>, expr: Box<Expr> }
    pub fn test_call_args(call: &CallExpression) {
        // args is Vec<ExprOrSpread>, not Vec<Expr>
        for arg in &call.arguments {
            // arg is ExprOrSpread, need to access .expr
            let expr = &arg.expr;

            // Now expr is the actual expression
            if matches!(expr, StringLiteral) {
                let value = expr.value.clone();
            }
        }
    }

    /// Test extracting useState initial value
    /// useState("initial")
    pub fn extract_initial_value(call: &CallExpression) -> Str {
        if call.arguments.len() > 0 {
            let first_arg = &call.arguments[0];
            // first_arg is ExprOrSpread
            let expr = &first_arg.expr;

            if matches!(expr, StringLiteral) {
                return expr.value.clone();
            } else if matches!(expr, NumericLiteral) {
                return expr.value.to_string();
            }
        }
        return "null";
    }

    /// Test extracting useEffect dependencies
    /// useEffect(() => {}, [dep1, dep2])
    pub fn extract_effect_deps(call: &CallExpression) -> Vec<Str> {
        let mut deps: Vec<Str> = vec![];

        if call.arguments.len() > 1 {
            let deps_arg = &call.arguments[1];
            // deps_arg is ExprOrSpread
            let expr = &deps_arg.expr;

            if matches!(expr, ArrayExpression) {
                // expr.elements is Vec<Option<ExprOrSpread>>
                for elem in &expr.elements {
                    if let Some(item) = elem {
                        // item is ExprOrSpread
                        let inner_expr = &item.expr;
                        if matches!(inner_expr, Identifier) {
                            deps.push(inner_expr.name.clone());
                        }
                    }
                }
            }
        }

        return deps;
    }

    /// Combined test - full useState extraction
    pub fn visit_variable_declarator(decl: &VariableDeclarator) {
        if let Some(init) = &decl.init {
            if matches!(init, CallExpression) {
                if matches!(init.callee, Identifier) {
                    let callee_name = init.callee.name.clone();

                    if callee_name == "useState" {
                        // Extract from ArrayPattern
                        if matches!(decl.id, ArrayPattern) {
                            let arr = decl.id.clone();

                            // Get state var name
                            if arr.elements.len() > 0 {
                                if let Some(pat) = &arr.elements[0] {
                                    if matches!(pat, Identifier) {
                                        let state_name = pat.name.clone();
                                    }
                                }
                            }

                            // Get initial value
                            if init.arguments.len() > 0 {
                                let arg = &init.arguments[0];
                                let value_expr = &arg.expr;
                                // Process value_expr...
                            }
                        }
                    }
                }
            }
        }
    }
}
