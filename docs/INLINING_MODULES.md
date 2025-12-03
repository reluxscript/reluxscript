\# Inlining Modules in SWC Codegen



This document describes how to properly implement module inlining for SWC codegen without breaking the code generation pipeline.



\## Background



When compiling a ReluxScript plugin with multiple `.lux` files, we have two options:

1\. \*\*Multi-file output\*\*: Generate separate `.rs` files and use Rust's module system

2\. \*\*Inline output\*\*: Generate a single `lib.rs` with all code inlined



The inline approach is simpler because it avoids:

\- `mod` declaration complexity

\- `pub` visibility requirements

\- `use crate::module::...` path issues

\- Rust's module file location rules



\## The Pipeline



The SWC codegen has a 4-stage pipeline:



```

Source (.lux) → Decorate → Rewrite → Hoist → Emit → Output (.rs)

```



Each stage is critical:



1\. \*\*Decorate\*\* (`SwcDecorator`): Adds metadata to AST nodes (SWC types, patterns, field mappings)

2\. \*\*Rewrite\*\* (`SwcRewriter`): Transforms patterns, desugars syntax

3\. \*\*Hoist\*\* (`SwcHoister`): Moves declarations, handles deferred loading

4\. \*\*Emit\*\* (`SwcEmitter`): Generates Rust code string



\## Critical: The `detect\_imports` Function



Before emitting, `SwcEmitter::emit\_program` calls `detect\_imports(program)`. This function:



1\. Scans the entire decorated AST

2\. Detects what features are used (HashMap, CodeBuilder, regex, etc.)

3\. Sets flags that control:

&nbsp;  - What `use` statements to emit

&nbsp;  - What helper structs/functions to generate

&nbsp;  - How certain patterns are emitted



\*\*NEVER skip `detect\_imports`\*\*. Without it, the emitter doesn't know:

\- What types are being used

\- What helpers to generate

\- How to transform certain patterns



\## Wrong Approach (What NOT to Do)



```rust

// BAD: This skips detect\_imports!

pub fn emit\_program(\&mut self, program: \&DecoratedProgram) -> String {

&nbsp;   if self.is\_inline {

&nbsp;       // WRONG: Jumps straight to emitting without detection

&nbsp;       self.emit\_top\_level\_decl(\&program.decl);

&nbsp;       return std::mem::take(\&mut self.output);

&nbsp;   }



&nbsp;   self.detect\_imports(program);  // Skipped for inline mode!

&nbsp;   // ...

}

```



This causes:

\- Missing type mappings

\- Wrong pattern generation

\- Missing helper code

\- 40+ compilation errors



\## Correct Approach



\### Option A: Skip Output, Not Detection



```rust

pub fn emit\_program(\&mut self, program: \&DecoratedProgram) -> String {

&nbsp;   // ALWAYS run detection - this populates critical state

&nbsp;   self.detect\_imports(program);



&nbsp;   // Only skip the OUTPUT of headers/imports for inline mode

&nbsp;   if !self.is\_inline {

&nbsp;       self.emit\_header();

&nbsp;       self.emit\_user\_imports(\&program.uses);

&nbsp;   }



&nbsp;   // Always emit the main code

&nbsp;   self.emit\_top\_level\_decl(\&program.decl);



&nbsp;   // Only emit helpers for non-inline (main file emits them once)

&nbsp;   if !self.is\_inline {

&nbsp;       self.emit\_helpers\_if\_needed();

&nbsp;   }



&nbsp;   std::mem::take(\&mut self.output)

}

```



\### Option B: Separate Detection from Emission



```rust

impl SwcEmitter {

&nbsp;   /// Run detection phase (call this for ALL modules)

&nbsp;   pub fn detect(\&mut self, program: \&DecoratedProgram) {

&nbsp;       self.detect\_imports(program);

&nbsp;   }



&nbsp;   /// Emit just the function bodies (for inlined modules)

&nbsp;   pub fn emit\_body\_only(\&mut self, program: \&DecoratedProgram) -> String {

&nbsp;       self.emit\_top\_level\_decl(\&program.decl);

&nbsp;       std::mem::take(\&mut self.output)

&nbsp;   }



&nbsp;   /// Emit full program with headers (for main lib.rs)

&nbsp;   pub fn emit\_full(\&mut self, program: \&DecoratedProgram) -> String {

&nbsp;       self.emit\_header();

&nbsp;       self.emit\_user\_imports(\&program.uses);

&nbsp;       self.emit\_top\_level\_decl(\&program.decl);

&nbsp;       self.emit\_helpers\_if\_needed();

&nbsp;       std::mem::take(\&mut self.output)

&nbsp;   }

}

```



\### Option C: Aggregate Detection Across All Modules



For proper inlining, you need to detect imports across ALL modules first:



```rust

pub fn generate\_inlined(

&nbsp;   main\_program: \&DecoratedProgram,

&nbsp;   module\_programs: \&\[DecoratedProgram],

) -> String {

&nbsp;   let mut emitter = SwcEmitter::new();



&nbsp;   // Step 1: Detect imports from ALL programs

&nbsp;   emitter.detect\_imports(main\_program);

&nbsp;   for module in module\_programs {

&nbsp;       emitter.detect\_imports(module);

&nbsp;   }



&nbsp;   // Step 2: Emit header once (with all detected imports)

&nbsp;   emitter.emit\_header();



&nbsp;   // Step 3: Emit main program body (skip file-based use statements)

&nbsp;   emitter.emit\_top\_level\_decl(\&main\_program.decl);



&nbsp;   // Step 4: Emit each module's body

&nbsp;   for module in module\_programs {

&nbsp;       emitter.emit\_top\_level\_decl(\&module.decl);

&nbsp;   }



&nbsp;   // Step 5: Emit helpers once at the end

&nbsp;   emitter.emit\_helpers\_if\_needed();



&nbsp;   emitter.output

}

```



\## Handling File-Based Imports



When inlining, skip `use ./module.lux::...` statements since the code is directly available:



```rust

fn emit\_user\_imports(\&mut self, uses: \&\[UseStmt]) {

&nbsp;   for use\_stmt in uses {

&nbsp;       let is\_file\_module = use\_stmt.path.starts\_with("./")

&nbsp;                         || use\_stmt.path.starts\_with("../");



&nbsp;       // Skip file imports when inlining - code is already present

&nbsp;       if is\_file\_module \&\& self.is\_inlining {

&nbsp;           continue;

&nbsp;       }



&nbsp;       // Emit non-file imports normally

&nbsp;       // ...

&nbsp;   }

}

```



Also skip `pub use ./module.lux::...` re-exports in `emit\_plugin\_item`.



\## Summary



1\. \*\*Always run `detect\_imports`\*\* on every program being compiled

2\. \*\*Only skip the output\*\* of headers/imports, not the detection

3\. \*\*Aggregate detection\*\* across all modules before emitting anything

4\. \*\*Emit helpers once\*\* at the end, not per-module

5\. \*\*Skip file-based imports\*\* in output, but process everything else normally



The detection phase populates critical emitter state. Skipping it breaks everything.



