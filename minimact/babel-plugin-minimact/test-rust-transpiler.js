/**
 * Test TypeScript → Rust Transpiler
 */

const babel = require('@babel/core');
const { generateRustTaskFiles, generateCargoToml, generateLibRs } = require('./src/generators/rustTask.cjs');
const path = require('path');

// Test TypeScript code
const testCode = `
function TestComponent() {
  const crunch = useServerTask(async (numbers) => {
    return numbers
      .map(x => x * x)
      .filter(x => x > 100)
      .reduce((sum, x) => sum + x, 0);
  }, { runtime: 'rust' });

  const parallel = useServerTask(async (data) => {
    const results = data.map(x => Math.sqrt(x));
    return results.filter(x => x > 10);
  }, { runtime: 'rust', parallel: true });

  return <div>Test</div>;
}
`;

console.log('🦀 Testing TypeScript → Rust Transpiler\n');

// Parse with Babel
const result = babel.transformSync(testCode, {
  plugins: [
    '@babel/plugin-syntax-jsx',
    './src/processComponent.cjs'
  ],
  filename: 'test.tsx'
});

console.log('✅ Babel parsing complete');
console.log('Result:', result);

// Test individual transpilation
const { transpileAsyncFunctionToRust } = require('./src/transpilers/typescriptToRust.cjs');

const simpleTest = `async (x) => x * 2`;
const parsedSimple = babel.parse(`(${simpleTest})`, {
  plugins: ['@babel/plugin-syntax-jsx']
});

const arrowFn = parsedSimple.program.body[0].expression;

console.log('\n🔧 Testing simple arrow function transpilation:');
console.log('TypeScript:', simpleTest);

try {
  const rustCode = transpileAsyncFunctionToRust(arrowFn);
  console.log('Rust:', rustCode);
} catch (error) {
  console.error('Error:', error.message);
}

// Test array operations
const arrayTest = `async (numbers) => {
  return numbers
    .map(x => x * x)
    .filter(x => x > 100)
    .reduce((sum, x) => sum + x, 0);
}`;

const parsedArray = babel.parse(`(${arrayTest})`, {
  plugins: ['@babel/plugin-syntax-jsx']
});

const arrayFn = parsedArray.program.body[0].expression;

console.log('\n🔧 Testing array operations transpilation:');
console.log('TypeScript:');
console.log(arrayTest);

try {
  const rustCode = transpileAsyncFunctionToRust(arrayFn);
  console.log('\nRust:');
  console.log(rustCode);
} catch (error) {
  console.error('Error:', error.message);
}

console.log('\n✅ Transpiler test complete!');
