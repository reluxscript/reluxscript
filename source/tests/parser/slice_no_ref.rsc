/// Test slice without ref

plugin SliceNoRefTest {
    fn test() {
        let arr = vec![1, 2, 3];
        let slice = arr[1..];
    }
}
