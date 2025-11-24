/**
 * Extract Simple Element Template
 *
 * Extracts simplified element templates without deeply nested state dependencies
 */

/**
 * Property value types
 */
pub enum PropValue {
    Static(Str),
    Binding(Str),
    Expression,
}

/**
 * Simple element template
 */
pub struct SimpleElementTemplate {
    pub template_type: Str,
    pub tag: Str,
    pub props: Option<HashMap<Str, PropValue>>,
    pub children: Option<Vec<TemplateChild>>,
}

/**
 * Template child types
 */
pub enum TemplateChild {
    Element(SimpleElementTemplate),
    Text { content: Str },
}

/**
 * Extract simple element template (without nested state dependencies)
 *
 * For structural templates, we extract a simplified version that captures:
 * - Tag name
 * - Static props
 * - Structure (not deeply nested templates)
 */
pub fn extract_simple_element_template(
    jsx_element: &JSXElement,
    component: &Component
) -> SimpleElementTemplate {
    // Get tag name
    let tag_name = match &jsx_element.opening_element.name {
        JSXElementName::Identifier(ref id) => id.name.clone(),
        _ => String::from("div"),
    };

    let attributes = &jsx_element.opening_element.attributes;

    // Extract static props only (complex props handled separately)
    let mut props = HashMap::new();

    for attr in attributes {
        if let JSXAttributeOrSpread::JSXAttribute(ref jsx_attr) = attr {
            let prop_name = match &jsx_attr.name {
                JSXAttributeName::Identifier(ref id) => id.name.clone(),
                _ => continue,
            };

            let prop_value = match &jsx_attr.value {
                Some(JSXAttributeValue::StringLiteral(ref str_lit)) => {
                    PropValue::Static(str_lit.value.clone())
                }

                Some(JSXAttributeValue::JSXExpressionContainer(ref expr_container)) => {
                    // Mark as dynamic (will be re-evaluated)
                    match &expr_container.expression {
                        Expression::Identifier(ref id) => {
                            PropValue::Binding(id.name.clone())
                        }
                        _ => PropValue::Expression,
                    }
                }

                None => {
                    // Boolean attribute
                    PropValue::Static(String::from("true"))
                }

                _ => continue,
            };

            props.insert(prop_name, prop_value);
        }
    }

    // Extract children (simplified)
    let mut children = vec![];

    for child in &jsx_element.children {
        match child {
            JSXChild::JSXElement(ref child_element) => {
                let child_template = extract_simple_element_template(child_element, component);
                children.push(TemplateChild::Element(child_template));
            }

            JSXChild::JSXText(ref jsx_text) => {
                let text = jsx_text.value.trim();
                if !text.is_empty() {
                    children.push(TemplateChild::Text {
                        content: text.to_string(),
                    });
                }
            }

            _ => {
                // Skip other child types for simple templates
            }
        }
    }

    SimpleElementTemplate {
        template_type: String::from("Element"),
        tag: tag_name,
        props: if props.is_empty() { None } else { Some(props) },
        children: if children.is_empty() { None } else { Some(children) },
    }
}
