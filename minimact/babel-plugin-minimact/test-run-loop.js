const babel = require('@babel/core');
const fs = require('fs');
const path = require('path');

// Read the test file
const testFile = path.join(__dirname, 'test-loop-templates.tsx');
const code = fs.readFileSync(testFile, 'utf-8');

console.log('=== Testing Loop Template Extraction ===\n');
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

// Check different locations where components might be stored
let components = null;

if (result && result.metadata) {
  console.log('Metadata keys:', Object.keys(result.metadata));

  // Try different metadata locations
  components = result.metadata.minimactComponents ||
               result.metadata.components ||
               result.file?.minimactComponents;
}

if (components) {

  console.log(`Found ${components.length} components:\n`);

  components.forEach(component => {
    console.log(`\n${'='.repeat(60)}`);
    console.log(`Component: ${component.name}`);
    console.log(`${'='.repeat(60)}`);

    if (component.loopTemplates && component.loopTemplates.length > 0) {
      console.log(`\nLoop Templates (${component.loopTemplates.length}):`);
      component.loopTemplates.forEach((lt, i) => {
        console.log(`\n  ${i + 1}. ${lt.stateKey}.map(${lt.itemVar}${lt.indexVar ? ', ' + lt.indexVar : ''} => ...)`);
        console.log(`     Array Binding: ${lt.arrayBinding}`);
        console.log(`     Key Binding: ${lt.keyBinding || '(none)'}`);

        if (lt.itemTemplate) {
          console.log(`\n     Item Template:`);
          console.log(`       Tag: <${lt.itemTemplate.tag}>`);

          if (lt.itemTemplate.propsTemplates) {
            console.log(`       Props Templates:`);
            Object.entries(lt.itemTemplate.propsTemplates).forEach(([propName, template]) => {
              console.log(`         ${propName}:`);
              console.log(`           template: "${template.template}"`);
              console.log(`           bindings: [${template.bindings.join(', ')}]`);
              if (template.conditionalTemplates) {
                console.log(`           conditionals:`, template.conditionalTemplates);
              }
            });
          }

          if (lt.itemTemplate.childrenTemplates) {
            console.log(`       Children Templates:`);
            lt.itemTemplate.childrenTemplates.forEach((child, idx) => {
              console.log(`         [${idx}] ${child.type}:`);
              console.log(`             template: "${child.template}"`);
              console.log(`             bindings: [${child.bindings.join(', ')}]`);
              if (child.conditionalTemplates) {
                console.log(`             conditionals:`, child.conditionalTemplates);
              }
            });
          }
        }
      });
    } else {
      console.log('\n  (No loop templates extracted)');
    }

    console.log('');
  });

  console.log('\n' + '='.repeat(60));
  console.log('Test completed!');
} else {
  console.log('ERROR: No components extracted!');
  console.log('Result metadata:', result.metadata);
}
