/**
 * Test with actual minimact plugin to isolate the error
 */

const babel = require('@babel/core');
const path = require('path');
const { pathToFileURL } = require('url');
const minimactPlugin = require('./index-full.cjs');

// Sample TSX code
const tsxCode = `
export function TestComponent() {
  return <div>Hello World</div>;
}
`;

// Test path (Windows format)
const testPath = 'E:\\allocation\\mini-projects\\tyty\\Pages\\Index.tsx';

console.log('Testing with Minimact plugin...\n');
console.log('Test path:', testPath);
console.log('─'.repeat(60));

// Test 1: Raw Windows path WITH PLUGIN
console.log('\n1. Raw Windows path WITH PLUGIN:');
console.log('   Input:', testPath);
try {
  const result = babel.transformSync(tsxCode, {
    filename: testPath,
    presets: ['@babel/preset-react', '@babel/preset-typescript'],
    plugins: [minimactPlugin]
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
  console.log('   Stack:', err.stack);
}

// Test 2: Basename WITH PLUGIN
console.log('\n2. Basename WITH PLUGIN:');
const basename = path.basename(testPath);
console.log('   Input:', basename);
try {
  const result = babel.transformSync(tsxCode, {
    filename: basename,
    presets: ['@babel/preset-react', '@babel/preset-typescript'],
    plugins: [minimactPlugin]
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

// Test 3: pathToFileURL WITH PLUGIN
console.log('\n3. pathToFileURL() WITH PLUGIN:');
const fileUrl = pathToFileURL(testPath).href;
console.log('   Input:', fileUrl);
try {
  const result = babel.transformSync(tsxCode, {
    filename: fileUrl,
    presets: ['@babel/preset-react', '@babel/preset-typescript'],
    plugins: [minimactPlugin]
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

console.log('\n' + '─'.repeat(60));
console.log('Test complete!\n');
