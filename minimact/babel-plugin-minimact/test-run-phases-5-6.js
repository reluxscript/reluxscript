const babel = require('@babel/core');
const fs = require('fs');
const path = require('path');

// Read the test file
const testFile = path.join(__dirname, 'test-phases-5-6.tsx');
const code = fs.readFileSync(testFile, 'utf-8');

console.log('=== Testing Phase 5 & 6 Template Extraction ===\n');
console.log('Input file:', testFile);
console.log('');

// Transform with Babel
const result = babel.transformSync(code, {
  filename: testFile,
  presets: [
    '@babel/preset-typescript',
    '@babel/preset-react'
  ],
  plugins: [
    './index-full.cjs'
  ]
});

console.log('\n' + '='.repeat(80));
console.log('EXTRACTION SUMMARY');
console.log('='.repeat(80) + '\n');

// We'll capture the console output to analyze
// Since the plugin logs to console, let's just run it and see the output

console.log('\nTest completed! Check the console output above for extracted templates.\n');
console.log('Expected extractions:');
console.log('');
console.log('📋 Phase 5 (Structural Templates):');
console.log('   - UserProfile: isLoggedIn ternary');
console.log('   - LoadingState: isLoading ternary');
console.log('   - ErrorBoundary: error logical AND');
console.log('   - MetricsDashboard: isLoading ternary');
console.log('   - BlogPost: post ternary');
console.log('');
console.log('📋 Phase 6 (Expression Templates):');
console.log('   - PriceDisplay: price.toFixed(2), arithmetic');
console.log('   - StringOperations: toUpperCase(), toLowerCase(), trim()');
console.log('   - ArrayOperations: items.length, items.join()');
console.log('   - MixedExpressions: binary operations (*, +), unary (-count, +count)');
console.log('   - MetricsDashboard: toFixed(), arithmetic');
console.log('   - BlogPost: toUpperCase(), Math.floor()');
console.log('');
