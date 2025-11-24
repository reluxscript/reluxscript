// Test file for newly added parser/codegen features
plugin NewFeaturesTest {

fn test_unit_literal() -> Result<(), Str> {
    let x = ();
    Ok(())
}

fn test_try_operator(value: Option<Str>) -> Result<Str, Str> {
    let unwrapped = value?;
    Ok(unwrapped)
}

fn test_tuple_destructuring() {
    let pair = get_pair();
    let (a, b) = pair;
}

fn test_block_expression() {
    let result = {
        let x = 5;
        let y = 10;
        x + y
    };
}

fn test_closure_with_block() {
    let add = |x, y| {
        let sum = x + y;
        sum
    };
}

// Helper function
fn get_pair() -> (i32, i32) {
    (1, 2)
}

}
