// Test codegen::generate() basic usage
use codegen;

plugin CodegenTest {

fn transform_expr(expr: &Expr) {
    let code = codegen::generate(expr);
    let formatted = format!("Generated: {}", code);
}

}
