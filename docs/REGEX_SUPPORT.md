# Regex Support Implementation

## Overview

This document describes the design and implementation of **regex pattern matching** in ReluxScript. Following the "vector intersection" principle, ReluxScript supports a **common subset of regex syntax** that works identically in both JavaScript and Rust, providing a unified regex API through the `Regex::` namespace that compiles correctly to both targets.

## Motivation

### The Problem

Many AST transformation tasks require pattern matching:

```javascript
// Common patterns in Babel plugins
if (/^use[A-Z]/.test(name)) {
    // React hook
}

if (/^__\w+__$/.test(prop)) {
    // Dunder property
}

if (/^\d+px$/.test(value)) {
    // CSS pixel value
}
```

Currently, ReluxScript requires either:
1. **Verbose string methods** (limited capabilities):
```reluxscript
if name.starts_with("use") && name.len() > 3 {
    // Incomplete - doesn't check for uppercase
}
```

2. **Platform-specific verbatim blocks** (breaks unification):
```reluxscript
fn is_hook(name: &Str) -> bool {
    babel! {
        return /^use[A-Z]/.test(name);
    }
    swc! {
        use regex::Regex;
        let re = Regex::new(r"^use[A-Z]").unwrap();
        return re.is_match(name);
    }
}
```

### The Solution

ReluxScript provides a **unified regex API** using static methods in the `Regex::` namespace:

```reluxscript
// ReluxScript - works for both targets
fn is_hook(name: &Str) -> bool {
    Regex::matches(name, r"^use[A-Z]")
}

fn extract_hex_digits(s: &Str) -> Option<Str> {
    if let Some(caps) = Regex::captures(s, r"^0x([0-9a-fA-F]+)$") {
        caps.get(1)
    } else {
        None
    }
}
```

**Babel output:**
```javascript
function is_hook(name) {
    return /^use[A-Z]/.test(name);
}

function extract_hex_digits(s) {
    const match = /^0x([0-9a-fA-F]+)$/.exec(s);
    if (match !== null) {
        return match[1];
    } else {
        return null;
    }
}
```

**SWC output:**
```rust
fn is_hook(name: &str) -> bool {
    regex::Regex::new(r"^use[A-Z]").unwrap().is_match(name)
}

fn extract_hex_digits(s: &str) -> Option<String> {
    let re = regex::Regex::new(r"^0x([0-9a-fA-F]+)$").unwrap();
    if let Some(caps) = re.captures(s) {
        Some(caps.get(1).unwrap().as_str().to_string())
    } else {
        None
    }
}
```

## Feature Specification

### 1. Supported Regex Syntax

ReluxScript supports the **intersection** of JavaScript and Rust regex features:

#### 1.1 ✅ Fully Supported

**Character classes:**
```regex
[abc]           Match a, b, or c
[^abc]          Match anything except a, b, c
[a-z]           Match lowercase letters
[A-Z]           Match uppercase letters
[0-9]           Match digits
[a-zA-Z0-9]     Match alphanumeric
```

**Predefined classes:**
```regex
.               Any character (except newline)
\d              Digit [0-9]
\D              Non-digit
\w              Word character [a-zA-Z0-9_]
\W              Non-word character
\s              Whitespace [ \t\n\r\f]
\S              Non-whitespace
```

**Anchors:**
```regex
^               Start of string/line
$               End of string/line
\b              Word boundary
\B              Non-word boundary
```

**Quantifiers:**
```regex
*               0 or more (greedy)
+               1 or more (greedy)
?               0 or 1 (greedy)
{n}             Exactly n times
{n,}            n or more times
{n,m}           Between n and m times

*?              0 or more (lazy)
+?              1 or more (lazy)
??              0 or 1 (lazy)
{n,}?           n or more (lazy)
{n,m}?          Between n and m (lazy)
```

**Groups:**
```regex
(pattern)       Capturing group
(?:pattern)     Non-capturing group
|               Alternation (or)
```

**Escape sequences:**
```regex
\t              Tab
\n              Newline
\r              Carriage return
\\              Backslash
\.              Literal dot
\*              Literal asterisk
(etc.)          Other escaped special characters
```

#### 1.2 ✅ Supported with Caveats

**Named captures:**
```regex
(?P<name>...)   Named capture group (Rust syntax)
```

**Note:** Uses Rust syntax `(?P<name>...)` which is supported by both:
- JavaScript: Modern browsers support this syntax
- Rust: Native syntax

**Lookahead:**
```regex
(?=...)         Positive lookahead
(?!...)         Negative lookahead
```

**Note:** Supported in both targets, but:
- JavaScript: Native support
- Rust: Requires `regex` crate with lookaround feature (needs verification)

**Unicode:**
```regex
\p{L}           Unicode letter
\p{N}           Unicode number
\p{...}         Other unicode categories
```

**Note:** Both targets support unicode properties, but:
- JavaScript: Uses `\p{...}` with `u` flag
- Rust: Uses `\p{...}` in regex pattern
- Behavior may differ for some edge cases

#### 1.3 ❌ Not Supported

**Lookbehind:**
```regex
(?<=...)        Positive lookbehind
(?<!...)        Negative lookbehind
```

**Reason:** Not supported in Rust's `regex` crate (as of 2024)

**Backreferences:**
```regex
\1, \2          Numbered backreferences
\k<name>        Named backreferences
```

**Reason:** Not supported in Rust's `regex` crate (requires backtracking)

**Workaround for unsupported features:**

```reluxscript
// Use platform-specific verbatim blocks
fn has_lookbehind(text: &Str) -> bool {
    babel! {
        return /(?<=foo)bar/.test(text);
    }
    swc! {
        // Manual implementation
        text.contains("foobar")
    }
}
```

### 2. API Reference

All regex operations are **static methods** on the `Regex::` namespace.

#### 2.1 Regex::matches()

```reluxscript
fn matches(text: &Str, pattern: &str) -> bool
```

Test if the pattern matches anywhere in the text.

**Parameters:**
- `text`: String to search
- `pattern`: Regex pattern (must be string literal)

**Returns:**
- `true` if pattern matches, `false` otherwise

**Example:**
```reluxscript
if Regex::matches(name, r"^use[A-Z]") {
    println!("It's a React hook");
}
```

**Compiles to:**

```javascript
// Babel
if (/^use[A-Z]/.test(name)) {
    console.log("It's a React hook");
}
```

```rust
// SWC
if regex::Regex::new(r"^use[A-Z]").unwrap().is_match(name) {
    println!("It's a React hook");
}
```

#### 2.2 Regex::find()

```reluxscript
fn find(text: &Str, pattern: &str) -> Option<Str>
```

Find the first match of the pattern in the text.

**Parameters:**
- `text`: String to search
- `pattern`: Regex pattern

**Returns:**
- `Some(matched_text)` if match found
- `None` if no match

**Example:**
```reluxscript
if let Some(num) = Regex::find(text, r"\d+") {
    println!("Found number: {}", num);
}
```

**Compiles to:**

```javascript
// Babel
const match = /\d+/.exec(text);
if (match !== null) {
    const num = match[0];
    console.log(`Found number: ${num}`);
}
```

```rust
// SWC
let re = regex::Regex::new(r"\d+").unwrap();
if let Some(m) = re.find(text) {
    let num = m.as_str();
    println!("Found number: {}", num);
}
```

#### 2.3 Regex::find_all()

```reluxscript
fn find_all(text: &Str, pattern: &str) -> Vec<Str>
```

Find all matches of the pattern in the text.

**Parameters:**
- `text`: String to search
- `pattern`: Regex pattern

**Returns:**
- Vector of all matched strings

**Example:**
```reluxscript
let numbers = Regex::find_all(text, r"\d+");
for num in numbers {
    println!("Found: {}", num);
}
```

**Compiles to:**

```javascript
// Babel
const numbers = [];
const re = /\d+/g;
let match;
while ((match = re.exec(text)) !== null) {
    numbers.push(match[0]);
}
for (const num of numbers) {
    console.log(`Found: ${num}`);
}
```

```rust
// SWC
let re = regex::Regex::new(r"\d+").unwrap();
let numbers: Vec<String> = re.find_iter(text)
    .map(|m| m.as_str().to_string())
    .collect();
for num in numbers {
    println!("Found: {}", num);
}
```

#### 2.4 Regex::captures()

```reluxscript
fn captures(text: &Str, pattern: &str) -> Option<Captures>
```

Extract capture groups from the first match.

**Parameters:**
- `text`: String to search
- `pattern`: Regex pattern with capture groups

**Returns:**
- `Some(Captures)` if match found
- `None` if no match

**Captures methods:**
- `caps.get(0)` - Full match
- `caps.get(1)` - First capture group
- `caps.get(n)` - Nth capture group

**Example:**
```reluxscript
if let Some(caps) = Regex::captures(version, r"^(\d+)\.(\d+)\.(\d+)$") {
    let major = caps.get(1);
    let minor = caps.get(2);
    let patch = caps.get(3);
    println!("Version: {}.{}.{}", major, minor, patch);
}
```

**Compiles to:**

```javascript
// Babel
const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
if (match !== null) {
    const major = match[1];
    const minor = match[2];
    const patch = match[3];
    console.log(`Version: ${major}.${minor}.${patch}`);
}
```

```rust
// SWC
let re = regex::Regex::new(r"^(\d+)\.(\d+)\.(\d+)$").unwrap();
if let Some(caps) = re.captures(version) {
    let major = caps.get(1).unwrap().as_str();
    let minor = caps.get(2).unwrap().as_str();
    let patch = caps.get(3).unwrap().as_str();
    println!("Version: {}.{}.{}", major, minor, patch);
}
```

#### 2.5 Regex::replace()

```reluxscript
fn replace(text: &Str, pattern: &str, replacement: &Str) -> Str
```

Replace the first match with the replacement string.

**Parameters:**
- `text`: String to search
- `pattern`: Regex pattern
- `replacement`: Replacement string

**Returns:**
- New string with first match replaced

**Example:**
```reluxscript
let result = Regex::replace(text, r"\d+", "NUM");
// "foo 123 bar 456" -> "foo NUM bar 456"
```

**Compiles to:**

```javascript
// Babel
const result = text.replace(/\d+/, "NUM");
```

```rust
// SWC
let re = regex::Regex::new(r"\d+").unwrap();
let result = re.replace(text, "NUM").to_string();
```

#### 2.6 Regex::replace_all()

```reluxscript
fn replace_all(text: &Str, pattern: &str, replacement: &Str) -> Str
```

Replace all matches with the replacement string.

**Parameters:**
- `text`: String to search
- `pattern`: Regex pattern
- `replacement`: Replacement string

**Returns:**
- New string with all matches replaced

**Example:**
```reluxscript
let result = Regex::replace_all(text, r"\d+", "NUM");
// "foo 123 bar 456" -> "foo NUM bar NUM"
```

**Compiles to:**

```javascript
// Babel
const result = text.replaceAll(/\d+/g, "NUM");
```

```rust
// SWC
let re = regex::Regex::new(r"\d+").unwrap();
let result = re.replace_all(text, "NUM").to_string();
```

### 3. Pattern Literal Requirements

All regex patterns must be **string literals** (compile-time constants):

```reluxscript
// ✅ Valid - string literal
Regex::matches(text, r"^\d+$")

// ✅ Valid - const string
const PATTERN: &str = r"^\d+$";
Regex::matches(text, PATTERN)

// ❌ Invalid - runtime string
let pattern = get_pattern();
Regex::matches(text, pattern)  // Compile error
```

**Rationale:**
- Patterns are validated at compile-time
- Optimizations possible (pattern caching)
- Ensures pattern compatibility across targets

### 4. Type System

#### 4.1 Captures Type

The `Captures` type represents matched capture groups:

```reluxscript
// Returned by Regex::captures()
let caps: Option<Captures> = Regex::captures(text, pattern);
```

**Methods:**
```reluxscript
caps.get(index: i32) -> Str     // Get capture group by index
```

**Compilation:**
- **Babel**: Array wrapper with helper methods
- **SWC**: `regex::Captures` wrapper

## Implementation Design

### 5. Parser Changes

#### 5.1 Regex Call Recognition

Recognize `Regex::` namespace calls:

```rust
// ast.rs
pub enum Expr {
    // ... existing variants
    RegexCall(RegexCall),
}

pub struct RegexCall {
    pub method: RegexMethod,     // matches, find, captures, etc.
    pub text_arg: Box<Expr>,     // Text to search
    pub pattern_arg: String,     // Pattern literal
    pub replacement_arg: Option<Box<Expr>>,  // For replace methods
    pub location: Span,
}

pub enum RegexMethod {
    Matches,
    Find,
    FindAll,
    Captures,
    Replace,
    ReplaceAll,
}
```

#### 5.2 Parsing Logic

```rust
// parser.rs
fn parse_call_expr(&mut self) -> Result<Expr> {
    let callee = self.parse_member_expr()?;

    // Check for Regex:: namespace call
    if let Expr::MemberExpression(ref member) = callee {
        if let Expr::Identifier(ref ident) = *member.object {
            if ident.name == "Regex" {
                // Parse Regex::method(args)
                let method = match member.property.as_str() {
                    "matches" => RegexMethod::Matches,
                    "find" => RegexMethod::Find,
                    "find_all" => RegexMethod::FindAll,
                    "captures" => RegexMethod::Captures,
                    "replace" => RegexMethod::Replace,
                    "replace_all" => RegexMethod::ReplaceAll,
                    _ => return Err(Error::UnknownRegexMethod(member.property.clone())),
                };

                self.expect(Token::LParen)?;

                // Parse arguments
                let text_arg = self.parse_expr()?;
                self.expect(Token::Comma)?;

                let pattern_expr = self.parse_expr()?;
                let pattern_arg = self.extract_string_literal(&pattern_expr)?;

                let replacement_arg = if matches!(method, RegexMethod::Replace | RegexMethod::ReplaceAll) {
                    self.expect(Token::Comma)?;
                    Some(Box::new(self.parse_expr()?))
                } else {
                    None
                };

                self.expect(Token::RParen)?;

                return Ok(Expr::RegexCall(RegexCall {
                    method,
                    text_arg: Box::new(text_arg),
                    pattern_arg,
                    replacement_arg,
                    location: self.current_span(),
                }));
            }
        }
    }

    // Regular call expression
    // ...
}

fn extract_string_literal(&self, expr: &Expr) -> Result<String> {
    match expr {
        Expr::StringLiteral(s) => Ok(s.value.clone()),
        _ => Err(Error::RegexPatternMustBeLiteral),
    }
}
```

### 6. Semantic Analysis

#### 6.1 Pattern Validation

Validate patterns at compile-time:

```rust
// semantic/regex_validator.rs
pub struct RegexValidator;

impl RegexValidator {
    pub fn validate(&self, pattern: &str) -> Result<(), RegexError> {
        // 1. Check for unsupported features
        self.check_lookbehind(pattern)?;
        self.check_backreferences(pattern)?;

        // 2. Validate pattern compiles in both targets
        self.validate_javascript(pattern)?;
        self.validate_rust(pattern)?;

        Ok(())
    }

    fn check_lookbehind(&self, pattern: &str) -> Result<(), RegexError> {
        if pattern.contains("(?<=") || pattern.contains("(?<!") {
            return Err(RegexError::UnsupportedLookbehind {
                pattern: pattern.to_string(),
                suggestion: "Lookbehind is not supported in Rust regex. Consider restructuring your pattern.".to_string(),
            });
        }
        Ok(())
    }

    fn check_backreferences(&self, pattern: &str) -> Result<(), RegexError> {
        // Check for \1, \2, etc. or \k<name>
        let backref_pattern = regex::Regex::new(r"\\[1-9]|\\k<").unwrap();
        if backref_pattern.is_match(pattern) {
            return Err(RegexError::UnsupportedBackreference {
                pattern: pattern.to_string(),
                suggestion: "Backreferences are not supported in Rust regex.".to_string(),
            });
        }
        Ok(())
    }

    fn validate_javascript(&self, pattern: &str) -> Result<(), RegexError> {
        // Could spawn Node.js to test pattern compilation
        // For now, trust that if Rust accepts it, JS probably will too
        Ok(())
    }

    fn validate_rust(&self, pattern: &str) -> Result<(), RegexError> {
        match regex::Regex::new(pattern) {
            Ok(_) => Ok(()),
            Err(e) => Err(RegexError::InvalidPattern {
                pattern: pattern.to_string(),
                error: e.to_string(),
            }),
        }
    }
}
```

### 7. Code Generation - Babel

#### 7.1 Regex::matches()

```rust
// codegen/babel.rs
impl BabelCodegen {
    fn gen_regex_call(&mut self, call: &RegexCall) {
        match call.method {
            RegexMethod::Matches => {
                // Regex::matches(text, pattern) -> /pattern/.test(text)
                self.emit("/");
                self.emit(&call.pattern_arg);
                self.emit("/.test(");
                self.gen_expr(&call.text_arg);
                self.emit(")");
            }
            // ... other methods
        }
    }
}
```

**Output:**
```javascript
/^use[A-Z]/.test(name)
```

#### 7.2 Regex::find()

```rust
RegexMethod::Find => {
    // Regex::find(text, pattern) -> /pattern/.exec(text)?.[0] ?? null
    self.emit("(/");
    self.emit(&call.pattern_arg);
    self.emit("/.exec(");
    self.gen_expr(&call.text_arg);
    self.emit(")?.[0] ?? null");
}
```

**Output:**
```javascript
(/\d+/.exec(text)?.[0] ?? null)
```

#### 7.3 Regex::find_all()

```rust
RegexMethod::FindAll => {
    // Needs helper function
    self.emit("__regex_find_all(");
    self.gen_expr(&call.text_arg);
    self.emit(", /");
    self.emit(&call.pattern_arg);
    self.emit("/)");
}
```

**Helper injected:**
```javascript
function __regex_find_all(text, pattern) {
    const matches = [];
    const globalPattern = new RegExp(pattern.source, 'g');
    let match;
    while ((match = globalPattern.exec(text)) !== null) {
        matches.push(match[0]);
    }
    return matches;
}
```

#### 7.4 Regex::captures()

```rust
RegexMethod::Captures => {
    // Regex::captures(text, pattern) -> __regex_captures(text, /pattern/)
    self.emit("__regex_captures(");
    self.gen_expr(&call.text_arg);
    self.emit(", /");
    self.emit(&call.pattern_arg);
    self.emit("/)");
}
```

**Helper injected:**
```javascript
function __regex_captures(text, pattern) {
    const match = pattern.exec(text);
    if (match === null) return null;
    return {
        get: (index) => match[index] ?? null,
    };
}
```

#### 7.5 Regex::replace() / replace_all()

```rust
RegexMethod::Replace => {
    // Regex::replace(text, pattern, repl) -> text.replace(/pattern/, repl)
    self.gen_expr(&call.text_arg);
    self.emit(".replace(/");
    self.emit(&call.pattern_arg);
    self.emit("/, ");
    self.gen_expr(call.replacement_arg.as_ref().unwrap());
    self.emit(")");
}

RegexMethod::ReplaceAll => {
    // Regex::replace_all(text, pattern, repl) -> text.replaceAll(/pattern/g, repl)
    self.gen_expr(&call.text_arg);
    self.emit(".replaceAll(/");
    self.emit(&call.pattern_arg);
    self.emit("/g, ");
    self.gen_expr(call.replacement_arg.as_ref().unwrap());
    self.emit(")");
}
```

### 8. Code Generation - SWC

#### 8.1 Dependencies

Auto-inject regex crate:

```rust
// codegen/swc.rs
impl SwcCodegen {
    fn gen_imports(&mut self) {
        if self.uses_regex {
            self.emit("use regex::Regex;\n");
        }
    }
}
```

#### 8.2 Regex::matches()

```rust
fn gen_regex_call(&mut self, call: &RegexCall) {
    match call.method {
        RegexMethod::Matches => {
            // Regex::matches(text, pattern) -> Regex::new(r"pattern").unwrap().is_match(text)
            self.emit("Regex::new(r\"");
            self.emit(&call.pattern_arg);
            self.emit("\").unwrap().is_match(");
            self.gen_expr(&call.text_arg);
            self.emit(")");
        }
        // ... other methods
    }
}
```

**Output:**
```rust
Regex::new(r"^use[A-Z]").unwrap().is_match(name)
```

#### 8.3 Regex::find()

```rust
RegexMethod::Find => {
    // Regex::find(text, pattern) -> Regex::new(r"pattern").unwrap().find(text).map(|m| m.as_str().to_string())
    self.emit("Regex::new(r\"");
    self.emit(&call.pattern_arg);
    self.emit("\").unwrap().find(");
    self.gen_expr(&call.text_arg);
    self.emit(").map(|m| m.as_str().to_string())");
}
```

#### 8.4 Regex::find_all()

```rust
RegexMethod::FindAll => {
    // Regex::find_all(text, pattern) -> Regex::new(r"pattern").unwrap().find_iter(text).map(|m| m.as_str().to_string()).collect()
    self.emit("Regex::new(r\"");
    self.emit(&call.pattern_arg);
    self.emit("\").unwrap().find_iter(");
    self.gen_expr(&call.text_arg);
    self.emit(").map(|m| m.as_str().to_string()).collect::<Vec<String>>()");
}
```

#### 8.5 Regex::captures()

```rust
RegexMethod::Captures => {
    // Regex::captures(text, pattern) -> __regex_captures(text, r"pattern")
    self.emit("__regex_captures(");
    self.gen_expr(&call.text_arg);
    self.emit(", r\"");
    self.emit(&call.pattern_arg);
    self.emit("\")");
}
```

**Helper function generated:**
```rust
fn __regex_captures(text: &str, pattern: &str) -> Option<__Captures> {
    let re = Regex::new(pattern).unwrap();
    re.captures(text).map(|caps| __Captures { inner: caps })
}

struct __Captures {
    inner: regex::Captures<'static>,
}

impl __Captures {
    fn get(&self, index: usize) -> String {
        self.inner.get(index)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default()
    }
}
```

#### 8.6 Pattern Caching Optimization

For frequently-used patterns, generate lazy_static:

```rust
fn gen_regex_optimizations(&mut self) {
    if self.regex_patterns.len() > 0 {
        self.emit("lazy_static::lazy_static! {\n");

        for (id, pattern) in &self.regex_patterns {
            self.emit(&format!(
                "    static ref REGEX_{}: Regex = Regex::new(r\"{}\").unwrap();\n",
                id, pattern
            ));
        }

        self.emit("}\n\n");
    }
}

fn gen_regex_call_optimized(&mut self, call: &RegexCall) {
    // If pattern is used multiple times, use cached version
    if let Some(id) = self.get_cached_pattern(&call.pattern_arg) {
        match call.method {
            RegexMethod::Matches => {
                self.emit(&format!("REGEX_{}.is_match(", id));
                self.gen_expr(&call.text_arg);
                self.emit(")");
            }
            // ... other methods
        }
    } else {
        // Inline version
        self.gen_regex_call_inline(call);
    }
}
```

### 9. Error Handling

#### 9.1 Compile-Time Errors

**Invalid pattern:**
```reluxscript
Regex::matches(text, r"[invalid")  // Unclosed bracket
```

Error:
```
error: invalid regex pattern
  --> plugin.lux:5:26
   |
5  |     Regex::matches(text, r"[invalid")
   |                            ^^^^^^^^^^ unclosed character class
   |
   = note: regex error: unclosed character class at position 8
```

**Unsupported feature (lookbehind):**
```reluxscript
Regex::matches(text, r"(?<=foo)bar")
```

Error:
```
error: unsupported regex feature: lookbehind
  --> plugin.lux:5:26
   |
5  |     Regex::matches(text, r"(?<=foo)bar")
   |                            ^^^^^^^^^^^^^ lookbehind not supported
   |
   = note: lookbehind (?<=...) is not supported in Rust's regex crate
   = help: consider restructuring your pattern or use verbatim blocks:
           babel! { /(?<=foo)bar/.test(text) }
           swc! { /* manual implementation */ }
```

**Non-literal pattern:**
```reluxscript
let p = get_pattern();
Regex::matches(text, p)  // Not a literal
```

Error:
```
error: regex pattern must be a string literal
  --> plugin.lux:6:22
   |
6  |     Regex::matches(text, p)
   |                          ^ not a string literal
   |
   = note: patterns must be compile-time constants for validation
   = help: use a string literal: Regex::matches(text, r"pattern")
```

### 10. Performance Considerations

#### 10.1 Babel Performance

- Regex literals are compiled by JavaScript engine at parse time
- Very fast: < 1μs for simple patterns
- Native browser/Node.js optimization

#### 10.2 SWC Performance

**Without optimization:**
- Pattern compiled on every call: ~10-100μs

**With lazy_static optimization:**
- Pattern compiled once: amortized to ~0μs
- Matching: ~500ns-2μs depending on complexity

**Heuristic for caching:**
```rust
// Cache pattern if:
// 1. Used more than once in the plugin
// 2. Used inside a loop
// 3. Used in a visitor method (called many times)

fn should_cache_pattern(&self, pattern: &str) -> bool {
    let usage_count = self.pattern_usage.get(pattern).unwrap_or(&0);
    *usage_count > 1 || self.is_in_loop() || self.is_in_visitor()
}
```

### 11. Examples

#### 11.1 Hook Detection

```reluxscript
plugin HookDetector {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if let Callee::Identifier(ref ident) = node.callee {
            if Regex::matches(&ident.name, r"^use[A-Z]\w*$") {
                println!("Found React hook: {}", ident.name);
            }
        }
    }
}
```

#### 11.2 Hex Path Validation

```reluxscript
plugin HexPathValidator {
    fn validate_hex_path(path: &Str) -> bool {
        Regex::matches(path, r"^0x[0-9a-fA-F]+$")
    }

    fn extract_hex_value(path: &Str) -> Option<Str> {
        if let Some(caps) = Regex::captures(path, r"^0x([0-9a-fA-F]+)$") {
            Some(caps.get(1))
        } else {
            None
        }
    }
}
```

#### 11.3 CSS Value Parsing

```reluxscript
plugin CSSParser {
    fn parse_pixel_value(value: &Str) -> Option<f64> {
        if let Some(caps) = Regex::captures(value, r"^(\d+(?:\.\d+)?)px$") {
            caps.get(1).parse::<f64>().ok()
        } else {
            None
        }
    }

    fn visit_string_literal(node: &mut StringLiteral, ctx: &Context) {
        if let Some(pixels) = parse_pixel_value(&node.value) {
            println!("Found pixel value: {}px", pixels);
        }
    }
}
```

#### 11.4 Identifier Sanitization

```reluxscript
plugin IdentifierSanitizer {
    fn sanitize_identifier(name: &Str) -> Str {
        // Remove invalid characters, replace with underscore
        Regex::replace_all(name, r"[^a-zA-Z0-9_]", "_")
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        let sanitized = sanitize_identifier(&node.name);
        if sanitized != node.name {
            node.name = sanitized;
        }
    }
}
```

#### 11.5 Version Extraction

```reluxscript
plugin VersionExtractor {
    fn extract_version(text: &Str) -> Option<(i32, i32, i32)> {
        if let Some(caps) = Regex::captures(text, r"^v?(\d+)\.(\d+)\.(\d+)") {
            let major = caps.get(1).parse::<i32>().ok()?;
            let minor = caps.get(2).parse::<i32>().ok()?;
            let patch = caps.get(3).parse::<i32>().ok()?;
            Some((major, minor, patch))
        } else {
            None
        }
    }

    fn visit_string_literal(node: &mut StringLiteral, ctx: &Context) {
        if let Some((maj, min, pat)) = extract_version(&node.value) {
            println!("Version: {}.{}.{}", maj, min, pat);
        }
    }
}
```

#### 11.6 Extract All Numbers

```reluxscript
plugin NumberExtractor {
    fn visit_string_literal(node: &mut StringLiteral, ctx: &Context) {
        let numbers = Regex::find_all(&node.value, r"\d+");

        for num in numbers {
            println!("Found number: {}", num);
        }
    }
}
```

### 12. Testing Strategy

#### 12.1 Pattern Validation Tests

```rust
#[test]
fn test_supported_patterns() {
    let patterns = vec![
        r"^\w+$",
        r"[a-zA-Z0-9]+",
        r"\d+\.\d+",
        r"^use[A-Z]\w*$",
        r"(?:foo|bar)",
        r"(?P<name>\w+)",  // Named capture
        r"foo(?=bar)",     // Lookahead
    ];

    for pattern in patterns {
        assert!(validate_regex_pattern(pattern).is_ok());
    }
}

#[test]
fn test_unsupported_patterns() {
    let patterns = vec![
        r"(?<=foo)bar",     // Lookbehind
        r"foo\1bar",        // Backreference
    ];

    for pattern in patterns {
        assert!(validate_regex_pattern(pattern).is_err());
    }
}
```

#### 12.2 Codegen Tests

```rust
#[test]
fn test_babel_matches() {
    let src = r#"Regex::matches(name, r"^use[A-Z]")"#;
    let output = compile_to_babel(src).unwrap();
    assert_eq!(output, r#"/^use[A-Z]/.test(name)"#);
}

#[test]
fn test_swc_matches() {
    let src = r#"Regex::matches(name, r"^use[A-Z]")"#;
    let output = compile_to_swc(src).unwrap();
    assert!(output.contains(r#"Regex::new(r"^use[A-Z]").unwrap().is_match(name)"#));
}

#[test]
fn test_babel_captures() {
    let src = r#"Regex::captures(text, r"(\d+)")"#;
    let output = compile_to_babel(src).unwrap();
    assert!(output.contains("__regex_captures"));
}
```

### 13. Migration Guide

#### 13.1 From String Methods

**Before:**
```reluxscript
fn is_hook(name: &Str) -> bool {
    name.starts_with("use") && name.len() > 3
}
```

**After:**
```reluxscript
fn is_hook(name: &Str) -> bool {
    Regex::matches(name, r"^use[A-Z]\w*$")
}
```

#### 13.2 From Verbatim Blocks

**Before:**
```reluxscript
fn extract_version(s: &Str) -> Option<(i32, i32, i32)> {
    babel! {
        const match = s.match(/^(\d+)\.(\d+)\.(\d+)$/);
        if (match) {
            return [parseInt(match[1]), parseInt(match[2]), parseInt(match[3])];
        }
        return null;
    }
    swc! {
        // ... manual Rust implementation
    }
}
```

**After:**
```reluxscript
fn extract_version(s: &Str) -> Option<(i32, i32, i32)> {
    if let Some(caps) = Regex::captures(s, r"^(\d+)\.(\d+)\.(\d+)$") {
        let major = caps.get(1).parse::<i32>().ok()?;
        let minor = caps.get(2).parse::<i32>().ok()?;
        let patch = caps.get(3).parse::<i32>().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}
```

## Summary

Regex support in ReluxScript provides:

✅ **Unified API**: Static methods in `Regex::` namespace
✅ **Type-safe**: Compile-time pattern validation
✅ **Compatible**: Supports intersection of JS and Rust features
✅ **Performant**: Native regex engines + optimization
✅ **Ergonomic**: Clean, functional API design

**Supported:**
- Character classes, predefined classes, anchors
- Quantifiers (greedy and lazy)
- Capturing and non-capturing groups
- Named captures `(?P<name>...)`
- Lookahead `(?=...)`, `(?!...)`
- Unicode properties `\p{...}` (with caveats)

**Not supported:**
- Lookbehind `(?<=...)` (Rust limitation)
- Backreferences `\1`, `\k<name>` (Rust limitation)

**API:**
- `Regex::matches(text, pattern)` - Test match
- `Regex::find(text, pattern)` - Find first match
- `Regex::find_all(text, pattern)` - Find all matches
- `Regex::captures(text, pattern)` - Extract groups
- `Regex::replace(text, pattern, repl)` - Replace first
- `Regex::replace_all(text, pattern, repl)` - Replace all

**Implementation priority:**
1. Parser support (Regex:: namespace recognition)
2. Pattern validation (unsupported feature detection)
3. Babel codegen (regex literal generation)
4. SWC codegen (regex crate calls + optimization)
5. Runtime helpers (Captures wrapper, find_all)
6. Testing and documentation
