/// Test: Match arm returning a reference
///
/// Tests that match arms can return references with &
/// Relates to: hook_detector.rsc line 55

plugin MatchArmReferenceTest {
    struct Item {
        name: Str,
        value: i32,
    }

    fn test_match_arm_reference(opt: &Option<Item>) -> &Str {
        // Match arm returning a reference
        let name = match opt {
            Some(ref item) => &item.name,
            None => &"default".to_string(),
        };
        name
    }
}
