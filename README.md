# reluxscript

<p align="center">
  <img src="./assets/lux-image-4.png" alt="ReluxScript Logo" width="250">
</p>

ReluxScript is a domain-specific language for writing AST transformation plugins that compile to both Babel (JavaScript) and SWC (Rust). It enforces a strict visitor pattern with explicit ownership semantics that map cleanly to both garbage-collected and borrow-checked runtimes.
