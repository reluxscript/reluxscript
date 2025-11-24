// Test writer hooks generate proper Babel plugin structure
writer TestWriterHooks {
    fn pre(file: &File) {
        babel! {
            builder._filename = file.opts.filename || "unknown";
        }
    }

    fn visit_identifier(node: &Identifier) {
        self.builder.append(node.name);
        self.builder.append(" ");
    }

    fn exit(program: &Program, state: &PluginState, builder: &CodeBuilder) {
        babel! {
            const output = builder.toString();
            state.file.metadata.collectedIdentifiers = output.trim();
        }
    }
}
