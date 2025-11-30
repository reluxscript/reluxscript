function greet(name) {
    console.log("Hello, " + name);
    console.warn("This is a warning");
    return "Hello, " + name;
}
const result = greet("World");
console.error("Done");
console.debug("Debug info");
