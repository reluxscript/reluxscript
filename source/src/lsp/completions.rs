use tower_lsp::lsp_types::*;

/// Generate all keyword completions
pub fn get_keyword_completions() -> Vec<CompletionItem> {
    vec![
        // Top-level declarations
        completion_keyword("plugin", "Define a transformation plugin"),
        completion_keyword("writer", "Define a code generation writer"),
        completion_keyword("module", "Define a module"),
        completion_keyword("interface", "Define an interface"),
        completion_keyword("use", "Import symbols from another module"),

        // Function-related
        completion_keyword("fn", "Define a function"),
        completion_keyword("pub", "Make item public"),
        completion_keyword("pre", "Pre-hook function"),
        completion_keyword("exit", "Exit-hook function"),

        // Variables
        completion_keyword("let", "Declare a variable"),
        completion_keyword("const", "Declare a constant"),
        completion_keyword("mut", "Make binding mutable"),

        // Control flow
        completion_keyword("if", "Conditional statement"),
        completion_keyword("else", "Alternative branch"),
        completion_keyword("match", "Pattern matching"),
        completion_keyword("return", "Return from function"),
        completion_keyword("break", "Break from loop"),
        completion_keyword("continue", "Continue to next iteration"),
        completion_keyword("for", "For loop"),
        completion_keyword("in", "Iterator keyword"),
        completion_keyword("while", "While loop"),

        // Special
        completion_keyword("self", "Reference to current instance"),
        completion_keyword("Self", "Current type"),
        completion_keyword("traverse", "AST traversal"),
        completion_keyword("using", "Import context"),

        // Literals
        completion_keyword("true", "Boolean true"),
        completion_keyword("false", "Boolean false"),
        completion_keyword("None", "Option none variant"),
        completion_keyword("Some", "Option some variant"),
        completion_keyword("Ok", "Result ok variant"),
        completion_keyword("Err", "Result error variant"),

        // Operators (as keywords)
        completion_keyword("and", "Logical AND"),
        completion_keyword("or", "Logical OR"),
        completion_keyword("not", "Logical NOT"),
    ]
}

/// Generate AST node type completions
pub fn get_ast_type_completions() -> Vec<CompletionItem> {
    vec![
        // Expression types
        completion_type("Expression", "Base expression type"),
        completion_type("Identifier", "Variable or property identifier"),
        completion_type("CallExpression", "Function/method call"),
        completion_type("MemberExpression", "Property access (obj.prop)"),
        completion_type("BinaryExpression", "Binary operation (a + b)"),
        completion_type("UnaryExpression", "Unary operation (-x, !x)"),
        completion_type("AssignmentExpression", "Assignment (a = b)"),
        completion_type("UpdateExpression", "Update (a++, --b)"),
        completion_type("LogicalExpression", "Logical operation (a && b)"),
        completion_type("ConditionalExpression", "Ternary (a ? b : c)"),
        completion_type("ArrowFunctionExpression", "Arrow function (() => {})"),
        completion_type("FunctionExpression", "Function expression"),
        completion_type("NewExpression", "Constructor call (new Foo())"),
        completion_type("SequenceExpression", "Comma expression (a, b)"),
        completion_type("ThisExpression", "This reference"),
        completion_type("ArrayExpression", "Array literal ([1, 2, 3])"),
        completion_type("ObjectExpression", "Object literal ({a: 1})"),
        completion_type("TemplateLiteral", "Template string (`foo ${x}`)"),
        completion_type("SpreadElement", "Spread (...args)"),
        completion_type("YieldExpression", "Yield expression"),
        completion_type("AwaitExpression", "Await expression"),

        // Statement types
        completion_type("Statement", "Base statement type"),
        completion_type("Stmt", "Statement (alias)"),
        completion_type("BlockStatement", "Block of statements"),
        completion_type("ExpressionStatement", "Expression as statement"),
        completion_type("IfStatement", "If/else statement"),
        completion_type("SwitchStatement", "Switch statement"),
        completion_type("ForStatement", "For loop"),
        completion_type("WhileStatement", "While loop"),
        completion_type("DoWhileStatement", "Do-while loop"),
        completion_type("ForInStatement", "For-in loop"),
        completion_type("ForOfStatement", "For-of loop"),
        completion_type("BreakStatement", "Break statement"),
        completion_type("ContinueStatement", "Continue statement"),
        completion_type("ReturnStatement", "Return statement"),
        completion_type("ThrowStatement", "Throw statement"),
        completion_type("TryStatement", "Try-catch statement"),
        completion_type("VariableDeclaration", "Variable declaration"),
        completion_type("FunctionDeclaration", "Function declaration"),
        completion_type("ClassDeclaration", "Class declaration"),

        // Pattern types
        completion_type("Pattern", "Base pattern type"),
        completion_type("ObjectPattern", "Object destructuring pattern"),
        completion_type("ArrayPattern", "Array destructuring pattern"),
        completion_type("RestElement", "Rest pattern (...rest)"),
        completion_type("AssignmentPattern", "Default value pattern"),

        // Literal types
        completion_type("Literal", "Base literal type"),
        completion_type("StringLiteral", "String literal"),
        completion_type("NumericLiteral", "Number literal"),
        completion_type("BooleanLiteral", "Boolean literal"),
        completion_type("NullLiteral", "Null literal"),
        completion_type("RegExpLiteral", "Regular expression literal"),

        // Other
        completion_type("Property", "Object property"),
        completion_type("Program", "Root program node"),
        completion_type("File", "File node"),
    ]
}

/// Generate built-in type completions (Rust types)
pub fn get_builtin_type_completions() -> Vec<CompletionItem> {
    vec![
        completion_type("Vec", "Dynamic array (Vec<T>)"),
        completion_type("Option", "Optional value (Option<T>)"),
        completion_type("Result", "Result type (Result<T, E>)"),
        completion_type("HashMap", "Hash map (HashMap<K, V>)"),
        completion_type("HashSet", "Hash set (HashSet<T>)"),
        completion_type("String", "Owned string"),
        completion_type("str", "String slice"),
        completion_type("bool", "Boolean type"),
        completion_type("i32", "32-bit signed integer"),
        completion_type("u32", "32-bit unsigned integer"),
        completion_type("f64", "64-bit floating point"),
        completion_type("usize", "Pointer-sized unsigned integer"),
    ]
}

/// Generate function snippet completions
pub fn get_snippet_completions() -> Vec<CompletionItem> {
    vec![
        completion_snippet(
            "visit_call_expression",
            "fn visit_call_expression(node: &mut CallExpression) {\n    $0\n}",
            "Visitor for call expressions"
        ),
        completion_snippet(
            "visit_identifier",
            "fn visit_identifier(node: &mut Identifier) {\n    $0\n}",
            "Visitor for identifiers"
        ),
        completion_snippet(
            "visit_member_expression",
            "fn visit_member_expression(node: &mut MemberExpression) {\n    $0\n}",
            "Visitor for member expressions"
        ),
        completion_snippet(
            "visit_binary_expression",
            "fn visit_binary_expression(node: &mut BinaryExpression) {\n    $0\n}",
            "Visitor for binary expressions"
        ),
        completion_snippet(
            "if-let",
            "if let ${1:pattern} = ${2:expr} {\n    $0\n}",
            "If-let pattern match"
        ),
        completion_snippet(
            "match",
            "match ${1:expr} {\n    ${2:pattern} => ${3:expr},\n    $0\n}",
            "Match expression"
        ),
        completion_snippet(
            "for-in",
            "for ${1:item} in ${2:collection} {\n    $0\n}",
            "For-in loop"
        ),
    ]
}

/// Helper to create keyword completion
fn completion_keyword(label: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::KEYWORD),
        detail: Some(detail.to_string()),
        documentation: None,
        ..Default::default()
    }
}

/// Helper to create type completion
fn completion_type(label: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::CLASS),
        detail: Some(detail.to_string()),
        documentation: None,
        ..Default::default()
    }
}

/// Helper to create snippet completion
fn completion_snippet(label: &str, snippet: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::SNIPPET),
        detail: Some(detail.to_string()),
        insert_text: Some(snippet.to_string()),
        insert_text_format: Some(InsertTextFormat::SNIPPET),
        documentation: None,
        ..Default::default()
    }
}

/// Get field completions for a specific AST node type
pub fn get_field_completions_for_type(type_name: &str) -> Vec<CompletionItem> {
    match type_name {
        "CallExpression" => vec![
            completion_field("callee", "Expression", "The function being called"),
            completion_field("arguments", "Vec<Expression>", "The arguments"),
            completion_field("optional", "bool", "Optional chaining (?.())"),
        ],
        "MemberExpression" => vec![
            completion_field("object", "Expression", "The object being accessed"),
            completion_field("property", "Expression", "The property being accessed"),
            completion_field("computed", "bool", "Computed (obj[x]) vs static (obj.x)"),
            completion_field("optional", "bool", "Optional chaining (?.)"),
        ],
        "BinaryExpression" => vec![
            completion_field("left", "Expression", "Left operand"),
            completion_field("right", "Expression", "Right operand"),
            completion_field("operator", "String", "Binary operator (+, -, *, etc.)"),
        ],
        "UnaryExpression" => vec![
            completion_field("argument", "Expression", "The operand"),
            completion_field("operator", "String", "Unary operator (-, !, ~, etc.)"),
            completion_field("prefix", "bool", "Prefix vs postfix"),
        ],
        "Identifier" => vec![
            completion_field("name", "String", "The identifier name"),
            completion_field("sym", "String", "The symbol (SWC)"),
        ],
        "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression" => vec![
            completion_field("params", "Vec<Pattern>", "Function parameters"),
            completion_field("body", "BlockStatement", "Function body"),
            completion_field("async", "bool", "Is async function"),
            completion_field("generator", "bool", "Is generator function"),
        ],
        "IfStatement" => vec![
            completion_field("test", "Expression", "The condition"),
            completion_field("consequent", "Statement", "Then branch"),
            completion_field("alternate", "Option<Statement>", "Else branch"),
        ],
        "VariableDeclaration" => vec![
            completion_field("declarations", "Vec<VariableDeclarator>", "Variable declarators"),
            completion_field("kind", "String", "var, let, or const"),
        ],
        _ => vec![],
    }
}

/// Helper to create field completion
fn completion_field(label: &str, type_name: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::FIELD),
        detail: Some(format!("{}: {}", label, type_name)),
        documentation: Some(Documentation::String(detail.to_string())),
        ..Default::default()
    }
}

/// Get method completions (built-in methods on types)
pub fn get_method_completions_for_type(type_name: &str) -> Vec<CompletionItem> {
    match type_name {
        "String" | "str" => vec![
            completion_method("starts_with", "(prefix: &str) -> bool", "Check if starts with prefix"),
            completion_method("ends_with", "(suffix: &str) -> bool", "Check if ends with suffix"),
            completion_method("contains", "(pattern: &str) -> bool", "Check if contains pattern"),
            completion_method("trim", "() -> String", "Remove leading/trailing whitespace"),
            completion_method("to_lowercase", "() -> String", "Convert to lowercase"),
            completion_method("to_uppercase", "() -> String", "Convert to uppercase"),
            completion_method("split", "(sep: &str) -> Vec<String>", "Split by separator"),
            completion_method("replace", "(from: &str, to: &str) -> String", "Replace substring"),
        ],
        "Vec" => vec![
            completion_method("push", "(item: T)", "Add item to end"),
            completion_method("pop", "() -> Option<T>", "Remove and return last item"),
            completion_method("len", "() -> usize", "Get length"),
            completion_method("is_empty", "() -> bool", "Check if empty"),
            completion_method("clear", "()", "Remove all items"),
            completion_method("contains", "(item: &T) -> bool", "Check if contains item"),
        ],
        "Option" => vec![
            completion_method("is_some", "() -> bool", "Check if Some"),
            completion_method("is_none", "() -> bool", "Check if None"),
            completion_method("unwrap", "() -> T", "Get value (panics if None)"),
            completion_method("unwrap_or", "(default: T) -> T", "Get value or default"),
        ],
        _ => vec![],
    }
}

/// Helper to create method completion
fn completion_method(label: &str, signature: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::METHOD),
        detail: Some(signature.to_string()),
        documentation: Some(Documentation::String(detail.to_string())),
        ..Default::default()
    }
}

/// Get pattern variant completions for specific types
pub fn get_pattern_completions_for_type(type_name: &str) -> Vec<CompletionItem> {
    match type_name {
        "Expression" => vec![
            completion_pattern("Identifier(id)", "Identifier pattern"),
            completion_pattern("CallExpression(call)", "Call expression pattern"),
            completion_pattern("MemberExpression(member)", "Member expression pattern"),
            completion_pattern("BinaryExpression(bin)", "Binary expression pattern"),
            completion_pattern("UnaryExpression(unary)", "Unary expression pattern"),
            completion_pattern("Literal(lit)", "Literal pattern"),
            completion_pattern("ArrayExpression(arr)", "Array expression pattern"),
            completion_pattern("ObjectExpression(obj)", "Object expression pattern"),
            completion_pattern("ArrowFunctionExpression(arrow)", "Arrow function pattern"),
            completion_pattern("FunctionExpression(func)", "Function expression pattern"),
            completion_pattern("ThisExpression", "This expression pattern"),
            completion_pattern("NewExpression(new_expr)", "New expression pattern"),
            completion_pattern("ConditionalExpression(cond)", "Conditional expression pattern"),
            completion_pattern("AssignmentExpression(assign)", "Assignment expression pattern"),
            completion_pattern("UpdateExpression(update)", "Update expression pattern"),
            completion_pattern("LogicalExpression(logical)", "Logical expression pattern"),
        ],
        "Statement" | "Stmt" => vec![
            completion_pattern("ExpressionStatement(expr_stmt)", "Expression statement pattern"),
            completion_pattern("BlockStatement(block)", "Block statement pattern"),
            completion_pattern("IfStatement(if_stmt)", "If statement pattern"),
            completion_pattern("ReturnStatement(ret)", "Return statement pattern"),
            completion_pattern("VariableDeclaration(var_decl)", "Variable declaration pattern"),
            completion_pattern("FunctionDeclaration(fn_decl)", "Function declaration pattern"),
            completion_pattern("ForStatement(for_stmt)", "For statement pattern"),
            completion_pattern("WhileStatement(while_stmt)", "While statement pattern"),
            completion_pattern("BreakStatement", "Break statement pattern"),
            completion_pattern("ContinueStatement", "Continue statement pattern"),
            completion_pattern("ThrowStatement(throw)", "Throw statement pattern"),
            completion_pattern("TryStatement(try_stmt)", "Try statement pattern"),
        ],
        "Pattern" => vec![
            completion_pattern("Identifier(id)", "Identifier pattern"),
            completion_pattern("ObjectPattern(obj_pat)", "Object pattern"),
            completion_pattern("ArrayPattern(arr_pat)", "Array pattern"),
            completion_pattern("RestElement(rest)", "Rest element pattern"),
            completion_pattern("AssignmentPattern(assign_pat)", "Assignment pattern"),
        ],
        "MemberProperty" | "MemberProp" => vec![
            completion_pattern("Identifier(id)", "Identifier property"),
            completion_pattern("Computed(expr)", "Computed property"),
        ],
        "Callee" => vec![
            completion_pattern("Expression(expr)", "Expression callee"),
            completion_pattern("Super", "Super callee"),
            completion_pattern("Import", "Import callee"),
        ],
        _ => vec![],
    }
}

/// Helper to create pattern completion
fn completion_pattern(label: &str, detail: &str) -> CompletionItem {
    CompletionItem {
        label: label.to_string(),
        kind: Some(CompletionItemKind::ENUM_MEMBER),
        detail: Some(detail.to_string()),
        insert_text: Some(label.to_string()),
        documentation: None,
        ..Default::default()
    }
}

/// Get all common pattern completions (used in match expressions)
pub fn get_common_pattern_completions() -> Vec<CompletionItem> {
    let mut completions = Vec::new();
    completions.extend(get_pattern_completions_for_type("Expression"));
    completions.extend(get_pattern_completions_for_type("Statement"));
    completions.extend(get_pattern_completions_for_type("Pattern"));
    completions
}
