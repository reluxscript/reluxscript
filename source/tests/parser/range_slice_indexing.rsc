/// Test: Range slice indexing
///
/// Tests that array/string slicing with ranges works:
/// - arr[0..5]
/// - str[start..end]
/// - arr[..end]
/// - arr[start..]

plugin RangeSliceTest {

    /// Test 1: Basic range slice
    fn test_basic_slice() -> Str {
        let s = "hello world";
        s[0..5].to_string()  // "hello"
    }

    /// Test 2: Range slice with variables
    fn test_variable_range(text: &Str, start: i32, end: i32) -> Str {
        text[start..end].to_string()
    }

    /// Test 3: Open-ended range (start..)
    fn test_open_end_range(text: &Str, start: i32) -> Str {
        text[start..].to_string()
    }

    /// Test 4: Open-start range (..end)
    fn test_open_start_range(text: &Str, end: i32) -> Str {
        text[..end].to_string()
    }

    /// Test 5: Range slice in assignment
    fn test_slice_assignment() -> Str {
        let full = "0123456789";
        let part = full[3..7];
        part.to_string()
    }

    /// Test 6: Range slice with complex expression
    fn test_complex_slice(path: &Str) -> Str {
        if let Some(last_dot) = path.rfind('.') {
            path[0..last_dot].to_string()  // This is the pattern from hex_path.rsc
        } else {
            path.clone()
        }
    }
}
