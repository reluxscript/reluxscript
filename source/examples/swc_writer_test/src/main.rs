use swc_common::{sync::Lrc, SourceMap, FileName};
use swc_ecma_parser::{Parser, Syntax, StringInput};
use swc_ecma_visit::{Visit, VisitWith};
use swc_ecma_ast::*;
use std::fs;



#[derive(Clone, Debug)]
struct ComponentMetadata {
    name: String,
    has_state: bool,
    has_effects: bool,
}

pub struct KitchenSinkWriter {
    output: String,
    indent_level: usize,
    components: Vec<ComponentMetadata>,
    current_component: Option<String>,
}

impl Visit for KitchenSinkWriter {}

impl KitchenSinkWriter {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            components: Vec::new(),
            current_component: None,
        }
    }
    
    fn append(&mut self, s: &str) {
        self.output.push_str(s);
    }
    
    fn newline(&mut self) {
        self.output.push('\n');
    }
    
    pub fn to_string(&self) -> String {
        self.output.clone()
    }
    
    fn is_component(name: &String) -> bool {
        if name.is_empty() {
            return false;
        }
        let first = name.chars().next().unwrap();
        first.is_uppercase()
    }
    
    fn sanitize_name(name: &String) -> String {
        if name.is_empty() {
            return name.clone();
        }
        let first = name.chars().next().unwrap();
        format!("{}{}", first.to_uppercase(), &name[1..])
    }
    
    fn get_callee_name(callee: &Expr) -> Option<String> {
        if let Expr::Ident(id) = callee {
            Some(id.sym.to_string())
        } else {
            None
        }
    }
    
    fn visit_mut_fn_decl(&mut self, node: &FnDecl) {
        let name = node.ident.sym.to_string();
        if Self::is_component(&name) {
            let sanitized = Self::sanitize_name(&name);
            self.current_component = Some(name.clone());
            let metadata = ComponentMetadata { name: sanitized.clone(), has_state: false, has_effects: false };
            self.append("public class ");
            self.append(&sanitized);
            self.newline();
            self.append("{");
            self.newline();
            node.visit_children_with(self);
            self.append("}");
            self.newline();
            self.newline();
            self.components.push(metadata);
            self.current_component = None
        }
    }
    
    fn visit_mut_call_expr(&mut self, node: &CallExpr) {
        if let Some(callee_name) = Self::get_callee_name(&node.callee.as_expr().unwrap()) {
            if (callee_name == "useState") {
                self.extract_state_var()
            } else {
                if (callee_name == "useEffect") {
                    self.extract_effect()
                }
            }
        }
        node.visit_children_with(self);
    }
    
    fn visit_mut_ident(&mut self, node: &Ident) {
        let _name = node.sym.to_string();
    }
    
    fn visit_mut_jsx_element(&mut self, node: &JSXElement) {
        self.append("    // JSX element");
        self.newline();
        node.visit_children_with(self);
    }
    
    fn extract_state_var(&mut self) {
        if let Some(component_name) = &self.current_component {
            let sanitized = Self::sanitize_name(component_name);
            let updated_components = self.components.iter().map(|c| {
                if (c.name == sanitized) {
                    ComponentMetadata { name: c.name.clone(), has_state: true, has_effects: c.has_effects }
                } else {
                    c.clone()
                }
            }).collect();
            self.components = updated_components
        }
        self.append("    private object _state;");
        self.newline();
    }
    
    fn extract_effect(&mut self) {
        if let Some(component_name) = &self.current_component {
            let sanitized = Self::sanitize_name(component_name);
            let updated_components = self.components.iter().map(|c| {
                if (c.name == sanitized) {
                    ComponentMetadata { name: c.name.clone(), has_state: c.has_state, has_effects: true }
                } else {
                    c.clone()
                }
            }).collect();
            self.components = updated_components
        }
        self.append("    public void OnInitialized() { }");
        self.newline();
    }
    
    fn count_components_with_state(components: &Vec<ComponentMetadata>) -> i32 {
        let mut count = 0;
        for component in components {
            if component.has_state {
                count += 1
            }
        }
        count
    }
    
    fn get_component_names(components: &Vec<ComponentMetadata>) -> Vec<String> {
        components.iter().map(|c| c.name.clone()).collect()
    }
    
    fn find_component<'a>(components: &'a Vec<ComponentMetadata>, name: &'a String) -> Option<&'a ComponentMetadata> {
        components.iter().find(|c| (c.name == *name))
    }
    
    fn to_camel_case(s: &String) -> String {
        if s.is_empty() {
            return s.clone();
        }
        let first = s.chars().next().unwrap();
        format!("{}{}", first.to_lowercase(), &s[1..])
    }
    
    fn to_snake_case(s: &String) -> String {
        s.to_lowercase().replace(" ", "_")
    }
    
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let input_file = &args[1];
    let output_file = &args[2];

    let source = fs::read_to_string(input_file)?;
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(Lrc::new(FileName::Custom(input_file.clone())), source);

    let syntax = Syntax::Es(Default::default());
    let mut parser = Parser::new(syntax, StringInput::from(&*fm), None);

    let program = parser.parse_program()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    let mut writer = KitchenSinkWriter::new();
    program.visit_with(&mut writer);

    let output = writer.to_string();
    fs::write(output_file, output)?;
    Ok(())
}
