// Test Issue 2: Some() and None emitted literally
writer TestOption {
    struct State {
        name: Option<Str>,
    }

    fn init() -> State {
        State { name: None }
    }

    fn set_name(name: Str) {
        self.state.name = Some(name);
    }
}
