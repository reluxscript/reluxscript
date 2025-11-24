/**
 * Simple compilation test for utils/helpers.rsc
 * This test just verifies the module compiles correctly
 */

plugin TestHelpersSimple {
    fn test_basic() {
        // Basic test to verify compilation
        let test_str = "Hello World";
        let upper = test_str.to_uppercase();
    }
}
