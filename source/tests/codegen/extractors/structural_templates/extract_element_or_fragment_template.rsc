/**
 * Extract Element Or Fragment Template
 *
 * Extracts template from JSX element or fragment
 */

use "./extract_simple_element_template.rsc" { extract_simple_element_template, SimpleElementTemplate, TemplateChild };

/**
 * Element or fragment template result
 */
pub enum ElementOrFragmentTemplate {
    Element(SimpleElementTemplate),
    Fragment {
        template_type: Str,
        children: Vec<TemplateChild>,
    },
}

/**
 * Extract element or fragment template
 */
pub fn extract_element_or_fragment_template(
    node: &JSXElementOrFragment,
    component: &Component
) -> Option<ElementOrFragmentTemplate> {
    match node {
        JSXElementOrFragment::JSXElement(ref element) => {
            let template = extract_simple_element_template(element, component);
            Some(ElementOrFragmentTemplate::Element(template))
        }

        JSXElementOrFragment::JSXFragment(ref fragment) => {
            let mut children = vec![];

            for child in &fragment.children {
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

                    _ => {}
                }
            }

            Some(ElementOrFragmentTemplate::Fragment {
                template_type: String::from("Fragment"),
                children,
            })
        }
    }
}
