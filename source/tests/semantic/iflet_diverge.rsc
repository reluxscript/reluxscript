/// Test diverging with if-let specifically

plugin IfLetDivergeTest {
    fn test() -> Str {
        let opt = Some("value".to_string());

        // if-let with return in else
        let result: Str = if let Some(val) = opt {
            val
        } else {
            return "early".to_string();
        };

        result
    }
}
