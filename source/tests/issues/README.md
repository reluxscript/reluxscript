# Remaining 20 Errors in minimact_full.lux

## Error Categories

### 1. Type Narrowing Issues (10 errors)
**Problem:** After `matches!` check, type is not narrowed

- **4× `no field 'elements' on Pat`** (`issue_elements_on_pat.lux`)
  - User does `matches!(binding, ArrayPattern)` then tries `binding.elements`
  - `binding` is still typed as `Pat`, not narrowed to `ArrayPat`
  - **Fix needed:** Type system should narrow after matches! guard, or user should use if-let

- **1× `no field 'properties' on Pat`** (`issue_properties_on_pat.lux`)
  - Same issue but for ObjectPattern
  
- **1× `no field 'name' on Pat`** (`issue_name_on_pat.lux`)
  - Same issue but for IdentifierPattern

- **1× `no field 'name' on Box<Expr>`** (`issue_name_on_box_expr.lux`)
  - Needs dereferencing/unwrapping to get to Ident
  
- **1× `no field 'elements' on ExprOrSpread`** (`issue_elements_on_exprorspread.lux`)
  - Wrong type being accessed
  
- **1× `no field 'render_body' on &'a mut i32`**
  - Completely wrong type inference somewhere

### 2. Missing Type Mapping (1 error)
- **1× `undeclared type 'UserDefined'`** (`issue_userdefined.lux`)
  - `ObjectPatternProperty` doesn't exist in SWC
  - Decorator maps unknown types to `UserDefined`
  - **Fix needed:** Add proper SWC type mapping or error on unknown types

### 3. Atom Conversion Issue (1 error)
- **1× `no method 'as_ref' on Wtf8Atom`** (`issue_atom_as_ref.lux`)
  - Field mapping says to use `.as_ref()` but Wtf8Atom doesn't have it
  - **Fix needed:** Use correct conversion method for Atom/Wtf8Atom

### 4. Type Mismatches (8 errors)
- **8× `mismatched types`** (`issue_mismatched_types.lux`)
  - Various issues: wrong return types, missing returns, wrong assignments
  - **User code errors** - not compiler bugs

### 5. Wrong Arguments (1 error)
- **1× `arguments to this function are incorrect`**
  - Calling function with wrong argument types
  - **User code error**

## Summary

- **Compiler bugs to fix:** ~12 errors (type narrowing, UserDefined, Atom conversion)
- **User code errors:** ~8 errors (type mismatches, wrong arguments)

## Recommended Fixes

1. **Type narrowing after matches!** - Most impactful fix (10 errors)
   - Implement control flow analysis to narrow types after `matches!` guards
   - Or: Better error message suggesting to use `if let` instead

2. **Fix Atom field conversion** - Quick win (1 error)
   - Change `Ident.sym` read_conversion from `.as_ref()` to correct method
   
3. **Handle unknown types gracefully** - Quick win (1 error)  
   - Better error for `ObjectPatternProperty` instead of emitting `UserDefined`

4. **Document user code errors** - No compiler fix needed (9 errors)
   - These are semantic errors in the user's ReluxScript code
