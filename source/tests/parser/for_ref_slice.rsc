/// Test for loop with ref slice

plugin ForRefSliceTest {
    fn test() {
        let params = vec![1, 2, 3, 4];
        for item in &params[1..] {
            // use item
        }
    }
}
