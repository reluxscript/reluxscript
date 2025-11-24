plugin IfLetExpr {
    fn test(opt: Option<i32>) -> i32 {
        let x = if let Some(ref val) = opt {
            *val
        } else {
            0
        };
        x
    }
}
