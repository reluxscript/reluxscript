# Compiling to Babel

Learn how to compile your ReluxScript plugins to Babel (JavaScript) plugins.

## Basic Compilation

Compile a `.lux` file to Babel:

```bash
relux build my-plugin.lux --target babel
```

This generates `dist/index.js` - a CommonJS module that exports a Babel plugin.

## Output Format

The generated Babel plugin follows the standard Babel plugin format:

```javascript
module.exports = function({ types: t }) {
  return {
    visitor: {
      CallExpression(path) {
        // Your transformation logic
      }
    }
  };
};
```

## Using with Babel

### With babel.config.js

```javascript
module.exports = {
  plugins: [
    './dist/index.js'
  ]
};
```

### With .babelrc

```json
{
  "plugins": ["./dist/index.js"]
}
```

### Programmatically

```javascript
const babel = require('@babel/core');

const result = babel.transformSync(code, {
  plugins: [require('./dist/index.js')]
});

console.log(result.code);
```

## Custom Output Directory

Specify a different output directory:

```bash
relux build my-plugin.lux --target babel --output build
```

This generates `build/index.js` instead of `dist/index.js`.

## Next Steps

- [Compile to SWC](/v0.1/guide/compiling-swc)
- [View Examples](/v0.1/examples/)
