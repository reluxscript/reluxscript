// Test tuple destructuring
plugin TupleTest {

fn unpack_values() {
    let data = vec![1, 2];
    let (x, y) = data;
    let result = x + y;
}

}

