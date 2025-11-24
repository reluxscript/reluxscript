/// Test: Self type in return position
///
/// ReluxScript should support using `Self` as a return type in impl blocks
/// and associated functions.

plugin SelfTypeTest {
    struct Builder {
        code: Str,
    }

    /// Test Self in standalone function
    pub fn create_builder() -> Self {
        return Builder {
            code: "",
        };
    }

    /// Test Self in impl block (when impl blocks are supported)
    impl Builder {
        fn new() -> Self {
            return Builder {
                code: "",
            };
        }

        fn with_code(code: Str) -> Self {
            return Builder {
                code: code,
            };
        }
    }
}
