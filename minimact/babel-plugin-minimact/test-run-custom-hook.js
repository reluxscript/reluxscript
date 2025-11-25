/**
 * Test Script for Custom Hooks
 *
 * Tests hook detection, analysis, and class generation
 */

const fs = require('fs');
const path = require('path');
const { parse } = require('@babel/parser');
const traverse = require('@babel/traverse').default;

const { isCustomHook, getHookName } = require('./src/analyzers/hookDetector.cjs');
const { analyzeHook } = require('./src/analyzers/hookAnalyzer.cjs');
const { generateHookClass } = require('./src/generators/hookClassGenerator.cjs');

// Read test file
const testFile = path.join(__dirname, 'test-custom-hook.tsx');
const code = fs.readFileSync(testFile, 'utf-8');

console.log('='.repeat(80));
console.log('TESTING CUSTOM HOOKS SYSTEM');
console.log('='.repeat(80));
console.log('\n📄 Input File: test-custom-hook.tsx\n');

// Parse TSX
const ast = parse(code, {
  sourceType: 'module',
  plugins: ['jsx', 'typescript']
});

console.log('✅ Parsed TSX successfully\n');

// Find custom hooks
const hooks = [];

traverse(ast, {
  FunctionDeclaration(path) {
    if (isCustomHook(path)) {
      const hookName = getHookName(path);
      console.log(`🎣 Found custom hook: ${hookName}`);
      hooks.push(path);
    }
  },
  VariableDeclarator(path) {
    if (isCustomHook(path)) {
      const hookName = getHookName(path);
      console.log(`🎣 Found custom hook: ${hookName}`);
      hooks.push(path);
    }
  }
});

if (hooks.length === 0) {
  console.log('❌ No custom hooks found!');
  process.exit(1);
}

console.log(`\n✅ Found ${hooks.length} custom hook(s)\n`);
console.log('='.repeat(80));

// Analyze each hook
hooks.forEach((hookPath, index) => {
  console.log(`\n📊 ANALYZING HOOK ${index + 1}`);
  console.log('='.repeat(80));

  const analysis = analyzeHook(hookPath);

  console.log(`\n🏷️  Hook Name: ${analysis.name}`);
  console.log(`📦 Class Name: ${analysis.className}`);

  if (analysis.params.length > 0) {
    console.log(`\n⚙️  Parameters:`);
    analysis.params.forEach(param => {
      console.log(`   - ${param.name}: ${param.type}${param.defaultValue ? ` = ${param.defaultValue}` : ''}`);
    });
  }

  if (analysis.states.length > 0) {
    console.log(`\n📌 State Fields:`);
    analysis.states.forEach(state => {
      console.log(`   - ${state.varName}: ${state.type} = ${state.initialValue}`);
      console.log(`     Setter: ${state.setterName}()`);
    });
  }

  if (analysis.methods.length > 0) {
    console.log(`\n🔧 Methods:`);
    analysis.methods.forEach(method => {
      const params = method.params.map(p => `${p.name}: ${p.type}`).join(', ');
      console.log(`   - ${method.name}(${params}): ${method.returnType}`);
    });
  }

  if (analysis.jsxElements.length > 0) {
    console.log(`\n🎨 JSX Elements: ${analysis.jsxElements.length}`);
    analysis.jsxElements.forEach((jsx, i) => {
      console.log(`   - Element ${i + 1}: ${jsx.type} (has node: ${!!jsx.node})`);
      if (jsx.node) {
        console.log(`     Node type: ${jsx.node.type}`);
      }
    });
  }

  if (analysis.returnValues.length > 0) {
    console.log(`\n↩️  Return Values:`);
    analysis.returnValues.forEach(ret => {
      console.log(`   [${ret.index}] ${ret.name}: ${ret.type}`);
    });
  }

  console.log('\n' + '='.repeat(80));
  console.log('📝 GENERATED C# CLASS');
  console.log('='.repeat(80) + '\n');

  // Create minimal component context
  const componentContext = {
    name: analysis.className,
    eventHandlers: [],
    stateTypes: new Map(),
    pluginUsages: []
  };

  const csharpClass = generateHookClass(analysis, componentContext);
  console.log(csharpClass);

  console.log('='.repeat(80));
});

console.log('\n✅ Custom hooks analysis complete!\n');
