/**
 * Simple Test TypeScript → Rust Transpiler
 */

const babel = require('@babel/core');
const { transpileAsyncFunctionToRust } = require('./src/transpilers/typescriptToRust.cjs');

console.log('🦀 Testing TypeScript → Rust Transpiler\n');

// Test 1: Simple arrow function
console.log('Test 1: Simple arrow function');
console.log('─'.repeat(50));

const test1Code = `async (x) => x * 2`;
const parsed1 = babel.parse(`(${test1Code})`, {
  plugins: ['@babel/plugin-syntax-jsx']
});

const arrowFn1 = parsed1.program.body[0].expression;

console.log('TypeScript:', test1Code);
const rust1 = transpileAsyncFunctionToRust(arrowFn1);
console.log('Rust:');
console.log(rust1);
console.log('');

// Test 2: Array operations
console.log('Test 2: Array operations');
console.log('─'.repeat(50));

const test2Code = `async (numbers) => {
  return numbers
    .map(x => x * x)
    .filter(x => x > 100)
    .reduce((sum, x) => sum + x, 0);
}`;

const parsed2 = babel.parse(`(${test2Code})`, {
  plugins: ['@babel/plugin-syntax-jsx']
});

const arrowFn2 = parsed2.program.body[0].expression;

console.log('TypeScript:');
console.log(test2Code);
console.log('\nRust:');
const rust2 = transpileAsyncFunctionToRust(arrowFn2);
console.log(rust2);
console.log('');

// Test 3: With variables
console.log('Test 3: With variables');
console.log('─'.repeat(50));

const test3Code = `async (data) => {
  const filtered = data.filter(x => x > 10);
  const squared = filtered.map(x => x * x);
  return squared;
}`;

const parsed3 = babel.parse(`(${test3Code})`, {
  plugins: ['@babel/plugin-syntax-jsx']
});

const arrowFn3 = parsed3.program.body[0].expression;

console.log('TypeScript:');
console.log(test3Code);
console.log('\nRust:');
const rust3 = transpileAsyncFunctionToRust(arrowFn3);
console.log(rust3);
console.log('');

console.log('✅ All tests complete!');
