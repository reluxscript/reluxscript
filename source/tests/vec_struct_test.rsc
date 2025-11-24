plugin test {

struct Foo {
    x: i32,
}

fn test() {
    let v = vec![
        Foo { x: 1 },
        Foo { x: 2 },
    ];
}

}
