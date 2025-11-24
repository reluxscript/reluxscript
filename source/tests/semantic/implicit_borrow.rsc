/// Test: Implicit borrow restrictions
///
/// Error: RS001 - Implicit borrow not allowed
/// ReluxScript requires explicit .clone() in some contexts

plugin ImplicitBorrowTest {

    struct HexPathGenerator {
        counter: i32,
    }

    struct State {
        hex_path_gen: HexPathGenerator,
        name: Str,
    }

    /// Test 1: Moving field value (implicit borrow)
    fn test_move_field(state: &State) {
        // This might trigger RS001 - implicit borrow
        let mut hex_gen = state.hex_path_gen;
        hex_gen.counter = 42;
    }

    /// Test 2: Explicit clone (should work)
    fn test_explicit_clone(state: &State) {
        // Explicit clone should avoid RS001
        let mut hex_gen = state.hex_path_gen.clone();
        hex_gen.counter = 42;
    }

    /// Test 3: Passing to function requiring &mut
    fn helper_function(gen: &mut HexPathGenerator) {
        gen.counter += 1;
    }

    fn test_borrow_for_function(state: &State) {
        let mut hex_gen = state.hex_path_gen;
        // Does this trigger implicit borrow error?
        helper_function(&mut hex_gen);
    }

    /// Test 4: Field access without move
    fn test_field_access_only(state: &State) -> i32 {
        // Just accessing, not moving
        state.hex_path_gen.counter
    }

    /// Test 5: Self field access
    fn test_self_field() {
        let state = State {
            hex_path_gen: HexPathGenerator { counter: 0 },
            name: "test".to_string(),
        };

        // Moving from self field
        let gen = state.hex_path_gen;
    }
}
