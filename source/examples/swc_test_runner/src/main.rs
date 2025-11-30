use swc_common::{sync::Lrc, SourceMap, FileName, DUMMY_SP};
use swc_ecma_parser::{Parser, Syntax, EsConfig, StringInput};
use swc_ecma_codegen::{Emitter, text_writer::JsWriter, Config as CodegenConfig};
use swc_ecma_visit::VisitMutWith;
use std::fs;

mod plugin;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_file = std::env::args().nth(1).expect("Input file required");
    let output_file = std::env::args().nth(2).expect("Output file required");

    // Read input
    let source = fs::read_to_string(&input_file)?;

    // Parse
    let cm = Lrc::new(SourceMap::default());
    let fm = cm.new_source_file(FileName::Custom(input_file.clone()), source);

    let syntax = Syntax::Es(EsConfig {
        jsx: true,
        ..Default::default()
    });

    let mut parser = Parser::new(syntax, StringInput::from(&*fm), None);
    let mut program = parser.parse_program()
        .map_err(|e| format!("Parse error: {:?}", e))?;

    // Transform
    let mut visitor = plugin::ConsoleRemover::default();
    program.visit_mut_with(&mut visitor);

    // Generate output
    let mut buf = vec![];
    {
        let mut emitter = Emitter {
            cfg: CodegenConfig::default(),
            cm: cm.clone(),
            comments: None,
            wr: Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)),
        };
        program.emit_with(&mut emitter)?;
    }

    let output = String::from_utf8(buf)?;
    fs::write(output_file, output)?;

    Ok(())
}
