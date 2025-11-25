/**
 * Extract element template from JSX element
 *
 * Returns template in format compatible with loop rendering:
 * {
 *   type: "Element",
 *   tag: "li",
 *   props_templates: { className: { template: "{0}", bindings: ["item.done"], ... } },
 *   children_templates: [ ... ],
 * }
 */

use "./extract_prop_templates.rsc" { extract_prop_templates, PropTemplate };
use "./extract_children_templates.rsc" { extract_children_templates, ChildTemplate };

/**
 * Element template structure
 */
pub struct ElementTemplate {
    pub template_type: Str,
    pub tag: Str,
    pub props_templates: Option<HashMap<Str, PropTemplate>>,
    pub children_templates: Option<Vec<ChildTemplate>>,
}

/**
 * Extract element template from JSX element
 *
 * @param jsx_element - The JSX element to extract from
 * @param item_var - The loop item variable name
 * @param index_var - The loop index variable name
 * @returns ElementTemplate
 */
pub fn extract_element_template(
    jsx_element: &JSXElement,
    item_var: &Str,
    index_var: Option<&Str>
) -> ElementTemplate {
    // Get tag name
    let tag_name = match &jsx_element.opening_element.name {
        JSXElementName::Identifier(ref id) => id.name.clone(),
        JSXElementName::MemberExpression(ref member) => {
            // Handle namespaced tags like <React.Fragment>
            "Fragment".to_string() // Simplified for now
        }
        _ => "div".to_string(), // Fallback
    };

    // Extract prop templates
    let props_templates = extract_prop_templates(
        &jsx_element.opening_element.attributes,
        item_var,
        index_var
    );

    // Extract children templates
    let children_templates = extract_children_templates(
        &jsx_element.children,
        item_var,
        index_var
    );

    ElementTemplate {
        template_type: "Element".to_string(),
        tag: tag_name,
        props_templates: if props_templates.is_empty() {
            None
        } else {
            Some(props_templates)
        },
        children_templates: if children_templates.is_empty() {
            None
        } else {
            Some(children_templates)
        },
    }
}
