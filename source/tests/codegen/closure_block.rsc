// Test closure with block body
plugin ClosureTest {

fn transform_items(items: Vec<Node>) {
    let result = vec![1, 2, 3];
    result.iter().map(|item| {
        let doubled = item * 2;
        doubled
    });
}

}
