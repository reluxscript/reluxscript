/**
 * Test for utils/helpers.rsc
 *
 * Tests:
 * - escape_csharp_string
 * - get_component_name
 * - is_component_name
 */

use "../../../reluxscript-plugin-minimact/utils/helpers.rsc" {
    escape_csharp_string,
    get_component_name,
    is_component_name
};

plugin TestHelpers {
    fn test_escape_csharp_string() {
        // Test backslash escaping
        let result1 = escape_csharp_string("C:\\path\\to\\file");
        // Should be: "C:\\\\path\\\\to\\\\file"

        // Test quote escaping
        let result2 = escape_csharp_string("He said \"hello\"");
        // Should be: "He said \\\"hello\\\""

        // Test newline escaping
        let result3 = escape_csharp_string("Line1\nLine2");
        // Should be: "Line1\\nLine2"

        // Test multiple escapes
        let result4 = escape_csharp_string("Path: \"C:\\test\"\nDone");
        // Should be: "Path: \\\"C:\\\\test\\\"\\nDone"

        // Test tab and carriage return
        let result5 = escape_csharp_string("Tab:\tReturn:\r");
        // Should be: "Tab:\\tReturn:\\r"
    }

    fn test_is_component_name() {
        // Test uppercase start (component)
        let test1 = is_component_name("MyComponent");
        // Should be: true

        let test2 = is_component_name("Component");
        // Should be: true

        // Test lowercase start (not component)
        let test3 = is_component_name("myFunction");
        // Should be: false

        let test4 = is_component_name("function");
        // Should be: false

        // Test empty string
        let test5 = is_component_name("");
        // Should be: false

        // Test single char
        let test6 = is_component_name("A");
        // Should be: true

        let test7 = is_component_name("a");
        // Should be: false
    }
}
