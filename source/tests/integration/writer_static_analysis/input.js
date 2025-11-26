// Exported but used
export function usedFunction() {
    return 42;
}

// Exported but never used
export function unusedFunction() {
    return "never called";
}

// Exported variable, used
export const API_KEY = "secret";

// Exported variable, not used
export const UNUSED_CONSTANT = 100;

// Complex function with branches
export function complexFunction(x) {
    if (x > 0) {
        return "positive";
    }
    if (x < 0) {
        return "negative";
    }
    if (x === 0) {
        return "zero";
    }
}

// Simple function
export function simpleFunction() {
    return true;
}

// Internal usage
const result = usedFunction();
const key = API_KEY;
