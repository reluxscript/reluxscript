// Test Issue 4: Early returns in loops
writer TestLoopReturn {
    fn process_items(items: Vec<Str>) -> i32 {
        let mut count = 0;
        for item in items {
            match item.as_str() {
                "skip" => {
                    count += 0;
                }
                "stop" => {
                    count += 1;
                }
                _ => {
                    count += 2;
                }
            }
        }
        count
    }
}
