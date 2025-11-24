// Test Issue 10: Missing semicolons
writer TestSemicolons {
    fn append_lines() {
        let mut lines = Vec::new();
        lines.push("line1".to_string());
        lines.push("line2".to_string());
        lines.push("line3".to_string())
    }
}
