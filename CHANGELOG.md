# Changelog

All notable changes to ReluxScript will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2025-12-01

### Added
- **Impl Blocks**: Full support for associated functions using `impl` blocks
  - Methods can be defined on structs for better organization
  - Automatic detection and code generation for both targets
- **JSON Module**: Built-in `json` module with automatic SWC mapping to `serde_json`
- **Traverse Blocks**: Enhanced support for inline visitor generation with proper captured variable transformation
- **Matches! Macro**: Full expansion with proper reference pattern handling
- **Debug Derives**: Automatic `Debug` trait derivation support

### Fixed
- **Type System**: Vec element type extraction now correctly handles nested generics like `Vec<Option<Pat>>`
- **Semantic Types**: Decorator now prefers semantic type checker types over inferred types for better accuracy
- **Type Mappings**: Added mappings for SWC type names (ArrayPat, ObjectPat, etc.) to map to themselves
- **Field Access**: Added `Option<Pat>.name` field mapping for proper unwrap + destructure + sym access
- **Let-Else Statements**: Fixed semicolon generation in `create_pat_destructuring()` for proper Rust syntax
- **Pattern Matching**: Traverse block methods now properly rewritten before hoisting to expand matches! macros
- **Helper Functions**: Fixed `&mut self` parameter detection to only add to visitor methods and explicit self functions

### Changed
- Enhanced type context with bidirectional SWC type mappings
- Improved captured variable transformation in hoisted visitor structs
- Better handling of reference patterns in match expressions

## [0.1.2] - 2025-11-30

### Added
- **Custom AST Properties**: Full support for attaching metadata to AST nodes using `__` prefix
  - Read and write custom properties with type safety
  - Automatic infrastructure generation for both Babel and SWC targets
  - Support for String, bool, i32, i64, f64 types
  - Memory-efficient storage using HashMap (SWC) and WeakMap (Babel)
- **Regex Support**: Unified regex pattern matching via `Regex::` namespace
  - Static methods: `matches()`, `find()`, `find_all()`, `captures()`, `replace()`, `replace_all()`
  - Compile-time pattern validation
  - Support for intersection of JS and Rust regex features
  - Automatic pattern caching optimization for SWC target
- **Documentation**: Added comprehensive spec sections for regex and custom properties

### Fixed
- Parser bug: Custom property access in if-let conditions now correctly parsed as `CustomPropAccess`
- Type persistence: Subsequent assignments to custom properties now use registered type
- Pattern desugaring: Automatic `&` reference wrapping for if-let pattern conditions
- Integration test: console-remover now passes with SWC target

### Changed
- Updated language specification to v0.9.0 with new features documented
- Enhanced CUSTOM_AST_PROPERTIES.md with implementation details
- Enhanced REGEX_SUPPORT.md with full API reference

## [0.1.1] - 2024-11-XX

### Added
- Initial SWC target implementation
- Pattern matching with if-let support
- Visitor pattern infrastructure
- State management for plugins

## [0.1.0] - 2024-11-XX

### Added
- Initial release
- Babel target compilation
- Basic ReluxScript syntax
- Lexer and parser
- Semantic analysis
