/// Test ref mut pattern

plugin RefMutTest {
    fn test(expr: &mut Expression) {
        if let Expression::JSXElement(ref mut elem) = expr {
            // Mutate elem
            elem.name = "div".to_string();
        }
    }
}
