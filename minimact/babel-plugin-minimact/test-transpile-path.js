/**
 * Test different filename formats with Babel to find what works
 */

const babel = require('@babel/core');
const path = require('path');
const { pathToFileURL } = require('url');

// Sample TSX code
const tsxCode = `
export function TestComponent() {
  return <div>Hello World</div>;
}
`;

// Test path (Windows format)
const testPath = 'E:\\allocation\\mini-projects\\tyty\\Pages\\Index.tsx';

console.log('Testing different filename formats with Babel...\n');
console.log('Test path:', testPath);
console.log('─'.repeat(60));

// Test 1: Raw Windows path
console.log('\n1. Raw Windows path:');
console.log('   Input:', testPath);
try {
  const result = babel.transformSync(tsxCode, {
    filename: testPath,
    presets: ['@babel/preset-react', '@babel/preset-typescript']
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

// Test 2: Basename only
console.log('\n2. Basename only:');
const basename = path.basename(testPath);
console.log('   Input:', basename);
try {
  const result = babel.transformSync(tsxCode, {
    filename: basename,
    presets: ['@babel/preset-react', '@babel/preset-typescript']
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

// Test 3: Forward slashes
console.log('\n3. Forward slashes:');
const forwardSlashes = testPath.replace(/\\/g, '/');
console.log('   Input:', forwardSlashes);
try {
  const result = babel.transformSync(tsxCode, {
    filename: forwardSlashes,
    presets: ['@babel/preset-react', '@babel/preset-typescript']
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

// Test 4: file:// URL (manual)
console.log('\n4. file:// URL (manual):');
const manualFileUrl = `file:///${testPath.replace(/\\/g, '/')}`;
console.log('   Input:', manualFileUrl);
try {
  const result = babel.transformSync(tsxCode, {
    filename: manualFileUrl,
    presets: ['@babel/preset-react', '@babel/preset-typescript']
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

// Test 5: pathToFileURL
console.log('\n5. pathToFileURL():');
const fileUrl = pathToFileURL(testPath).href;
console.log('   Input:', fileUrl);
try {
  const result = babel.transformSync(tsxCode, {
    filename: fileUrl,
    presets: ['@babel/preset-react', '@babel/preset-typescript']
  });
  console.log('   ✅ SUCCESS');
} catch (err) {
  console.log('   ❌ FAILED:', err.message);
}

console.log('\n' + '─'.repeat(60));
console.log('Test complete!\n');
