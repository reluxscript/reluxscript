// Test parser module usage
use parser;

plugin ParserTest {

fn analyze_file(file_path: &Str) -> Result<(), Str> {
    let ast = parser::parse_file(file_path)?;
    Ok(())
}

fn parse_code_snippet(code: &Str) -> Result<(), Str> {
    let ast = parser::parse(code)?;
    Ok(())
}

fn parse_with_custom_syntax(code: &Str) -> Result<(), Str> {
    let ast = parser::parse_with_syntax(code, "TypeScript")?;
    Ok(())
}

}
