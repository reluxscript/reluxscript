plugin RefInTupleTest {
    fn test() {
        let pair = (1, 2);
        let (ref x, y) = pair;
    }
}
