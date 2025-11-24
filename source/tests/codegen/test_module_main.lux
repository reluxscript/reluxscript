// Test importing from another module
use "./test_module_helper.rsc" { add, multiply, Point };

plugin ModuleTest {

fn test_imports() -> i32 {
    let result1 = add(5, 3);
    let result2 = multiply(4, 7);
    let point = Point { x: 10, y: 20 };
    result1 + result2
}

}
