/// Test: Simple impl block at top level

plugin ImplSimpleTest {
}

struct Builder {
    value: Str,
}

impl Builder {
    fn new() -> Builder {
        return Builder {
            value: "",
        };
    }

    fn get(&self) -> Str {
        return self.value.clone();
    }

    fn set(&mut self, val: Str) {
        self.value = val;
    }
}
