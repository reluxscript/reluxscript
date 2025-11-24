/**
 * Extract Element Structure
 *
 * Extracts complete element structure including dynamic content
 */

use codegen;
use "./build_member_path.rsc" { build_member_path };

/**
 * Attribute value types
 */
pub enum AttributeValue {
    Boolean(bool),
    Static(Str),
    Binding(Str),
    Expression(Str),
}

/**
 * Child structure types
 */
pub enum ChildStructure {
    Text {
        value: Option<Str>,
        binding: Option<Str>,
        expression: Option<Str>,
        hex_path: Option<Str>,
    },
    Element(ElementStructure),
}

/**
 * Element structure result
 */
pub struct ElementStructure {
    pub element_type: Str,
    pub tag: Option<Str>,
    pub hex_path: Option<Str>,
    pub attributes: HashMap<Str, AttributeValue>,
    pub children: Vec<ChildStructure>,
}

/**
 * Extract complete element structure including dynamic content
 */
pub fn extract_element_structure(node: &JSXElementOrFragment) -> Option<ElementStructure> {
    match node {
        JSXElementOrFragment::JSXElement(ref element) => {
            extract_jsx_element(element)
        }

        JSXElementOrFragment::JSXFragment(ref fragment) => {
            extract_jsx_fragment(fragment)
        }
    }
}

/**
 * Extract JSX element structure
 */
fn extract_jsx_element(node: &JSXElement) -> Option<ElementStructure> {
    // Get tag name
    let tag_name = match &node.opening_element.name {
        JSXElementName::Identifier(ref id) => id.name.clone(),
        _ => return None,
    };

    // Get hex path if available (custom property set by plugin)
    let hex_path = node.get_custom_property("__minimactPath");

    // Extract attributes
    let mut attributes = HashMap::new();

    for attr in &node.opening_element.attributes {
        if let JSXAttributeOrSpread::JSXAttribute(ref jsx_attr) = attr {
            let attr_name = match &jsx_attr.name {
                JSXAttributeName::Identifier(ref id) => id.name.clone(),
                _ => continue,
            };

            let attr_value = match &jsx_attr.value {
                None => {
                    // Boolean attribute
                    AttributeValue::Boolean(true)
                }

                Some(JSXAttributeValue::StringLiteral(ref str_lit)) => {
                    AttributeValue::Static(str_lit.value.clone())
                }

                Some(JSXAttributeValue::JSXExpressionContainer(ref expr_container)) => {
                    // Dynamic attribute
                    match &expr_container.expression {
                        Expression::Identifier(ref id) => {
                            AttributeValue::Binding(id.name.clone())
                        }

                        Expression::MemberExpression(_) => {
                            if let Some(path) = build_member_path(&expr_container.expression) {
                                AttributeValue::Binding(path)
                            } else {
                                let code = codegen::generate(&expr_container.expression);
                                AttributeValue::Expression(code)
                            }
                        }

                        _ => {
                            let code = codegen::generate(&expr_container.expression);
                            AttributeValue::Expression(code)
                        }
                    }
                }

                _ => continue,
            };

            attributes.insert(attr_name, attr_value);
        }
    }

    // Extract children
    let mut children = vec![];

    for child in &node.children {
        match child {
            JSXChild::JSXText(ref jsx_text) => {
                let text = jsx_text.value.trim();
                if !text.is_empty() {
                    let child_hex_path = jsx_text.get_custom_property("__minimactPath");
                    children.push(ChildStructure::Text {
                        value: Some(text.to_string()),
                        binding: None,
                        expression: None,
                        hex_path: child_hex_path,
                    });
                }
            }

            JSXChild::JSXElement(ref child_element) => {
                if let Some(child_structure) = extract_jsx_element(child_element) {
                    children.push(ChildStructure::Element(child_structure));
                }
            }

            JSXChild::JSXExpressionContainer(ref expr_container) => {
                // Dynamic text content
                let child_hex_path = expr_container.get_custom_property("__minimactPath");

                let child_struct = match &expr_container.expression {
                    Expression::Identifier(ref id) => {
                        ChildStructure::Text {
                            value: None,
                            binding: Some(id.name.clone()),
                            expression: None,
                            hex_path: child_hex_path,
                        }
                    }

                    Expression::MemberExpression(_) => {
                        if let Some(path) = build_member_path(&expr_container.expression) {
                            ChildStructure::Text {
                                value: None,
                                binding: Some(path),
                                expression: None,
                                hex_path: child_hex_path,
                            }
                        } else {
                            let code = codegen::generate(&expr_container.expression);
                            ChildStructure::Text {
                                value: None,
                                binding: None,
                                expression: Some(code),
                                hex_path: child_hex_path,
                            }
                        }
                    }

                    _ => {
                        // Complex expression
                        let code = codegen::generate(&expr_container.expression);
                        ChildStructure::Text {
                            value: None,
                            binding: None,
                            expression: Some(code),
                            hex_path: child_hex_path,
                        }
                    }
                };

                children.push(child_struct);
            }

            _ => {
                // Other child types
            }
        }
    }

    Some(ElementStructure {
        element_type: String::from("element"),
        tag: Some(tag_name),
        hex_path,
        attributes,
        children,
    })
}

/**
 * Extract JSX fragment structure
 */
fn extract_jsx_fragment(node: &JSXFragment) -> Option<ElementStructure> {
    let mut children = vec![];

    for child in &node.children {
        if let JSXChild::JSXElement(ref child_element) = child {
            if let Some(child_structure) = extract_jsx_element(child_element) {
                children.push(ChildStructure::Element(child_structure));
            }
        }
    }

    Some(ElementStructure {
        element_type: String::from("fragment"),
        tag: None,
        hex_path: None,
        attributes: HashMap::new(),
        children,
    })
}
