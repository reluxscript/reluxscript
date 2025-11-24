/// Test return statement in match arms

plugin ReturnInMatchTest {
    fn test_return_in_match(opt: &Option<i32>) -> i32 {
        let value = match opt {
            Some(x) => x,
            None => return 0,  // return as match arm body
        };
        value + 1
    }
}
