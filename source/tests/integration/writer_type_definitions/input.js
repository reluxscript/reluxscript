// Regular function declaration
function calculateSum(a, b) {
    return a + b;
}

// Function expression
const multiply = function(x, y) {
    return x * y;
};

// Arrow function
const greet = (name) => {
    return `Hello, ${name}!`;
};

// Multi-parameter arrow function
const processData = (data, options, callback) => {
    callback(data);
};

// No parameters
function reset() {
    console.log("reset");
}
