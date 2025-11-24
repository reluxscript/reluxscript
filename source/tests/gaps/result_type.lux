/// Test: Result type and ? operator
///
/// ReluxScript should support Result<T, E> for error handling
/// and the ? operator for early return on errors.

plugin ResultTypeTest {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Function that might fail
        let result = process_node(node);

        // Handle result explicitly
        match result {
            Ok(value) => {
                // use value
            }
            Err(error) => {
                // handle error
            }
        }

        // Or use if let
        if let Ok(value) = process_node(node) {
            // use value
        }
    }

    fn validate_and_transform(node: &mut FunctionDeclaration) -> Result<(), Str> {
        // Use ? operator for early return on error
        let name = get_function_name(node)?;
        let params = validate_params(&node.params)?;

        // If we get here, everything succeeded
        transform_node(node, &name, &params)?;

        Ok(())
    }
}

// Function returning Result
fn get_function_name(node: &FunctionDeclaration) -> Result<Str, Str> {
    if node.id.name.is_empty() {
        Err("Function name cannot be empty")
    } else {
        Ok(node.id.name.clone())
    }
}

fn validate_params(params: &Vec<Param>) -> Result<Vec<Str>, Str> {
    if params.is_empty() {
        Err("Function must have at least one parameter")
    } else {
        Ok(params.iter().map(|p| p.name.clone()).collect())
    }
}

fn transform_node(node: &mut FunctionDeclaration, name: &Str, params: &Vec<Str>) -> Result<(), Str> {
    // Perform transformation
    Ok(())
}

fn process_node(node: &FunctionDeclaration) -> Result<Str, Str> {
    Ok("processed")
}
