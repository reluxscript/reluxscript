use swc_common::atoms::Atom;

fn test(value: Atom) {
    // Test which works in format!
    println!("{}", &*value);        // This works
    println!("{}", value.as_ref()); // Does this work?
}
