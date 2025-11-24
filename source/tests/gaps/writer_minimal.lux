/// Minimal writer test

writer TestWriter {
    struct State {
        count: i32,
    }

    fn init() -> State {
        return State { count: 0 };
    }

    pub fn visit_program(node: &Program) {
        self.count += 1;
    }

    fn finish(&self) -> i32 {
        return self.count;
    }
}
