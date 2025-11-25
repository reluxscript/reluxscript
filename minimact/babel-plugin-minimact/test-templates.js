/**
 * Test template extraction from babel-plugin-minimact
 */

const fs = require('fs');
const path = require('path');
const babel = require('@babel/core');
const minimactPlugin = require('./index-full.cjs');

// Read test input
const inputPath = path.join(__dirname, 'test', 'Counter.input.jsx');
const inputCode = fs.readFileSync(inputPath, 'utf-8');

console.log('Input file:', inputPath);
console.log('Input code length:', inputCode.length, 'bytes');
console.log('');

// Transform with babel
const result = babel.transformSync(inputCode, {
  filename: inputPath,
  presets: [
    ['@babel/preset-react', { runtime: 'classic' }]
  ],
  plugins: [minimactPlugin]
});

console.log('✅ Babel transformation complete');
console.log('');

// Check if .templates.json was generated
const templateFilePath = path.join(__dirname, 'test', 'Counter.templates.json');

if (fs.existsSync(templateFilePath)) {
  const templateJson = fs.readFileSync(templateFilePath, 'utf-8');
  const templateMap = JSON.parse(templateJson);

  console.log('✅ Template file generated:', templateFilePath);
  console.log('');
  console.log('Template Map:');
  console.log(JSON.stringify(templateMap, null, 2));
  console.log('');
  console.log('Template Count:', Object.keys(templateMap.templates || {}).length);
  console.log('');

  // Display each template
  for (const [path, template] of Object.entries(templateMap.templates || {})) {
    console.log(`Template: ${path}`);
    console.log(`  Template: "${template.template}"`);
    console.log(`  Bindings: [${template.bindings.join(', ')}]`);
    console.log(`  Slots: [${template.slots.join(', ')}]`);
    console.log(`  Type: ${template.type}`);
    console.log('');
  }
} else {
  console.log('❌ Template file not generated');
  console.log('Expected:', templateFilePath);
}

console.log('Test complete!');
