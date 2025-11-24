// Test: String literals should not have .join("")
writer TestStrings {
    fn get_default_value() -> Str {
        let plain_string = "null";
        let from_method = String::new();
        plain_string
    }

    fn conditional_string(flag: bool) -> Str {
        if flag {
            "yes"
        } else {
            "no"
        }
    }
}
