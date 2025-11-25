import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';
import replace from '@rollup/plugin-replace';

export default {
  input: 'index-full.cjs',
  output: [
    {
      file: 'dist/minimact-babel-plugin.js',
      format: 'iife',
      name: 'MinimactBabelPlugin',
      banner: `(function() {
  // Inject Babel types and core as globals for the plugin to use
  if (typeof window !== 'undefined' && window.Babel) {
    // @babel/standalone exposes types via packages.types, not directly as .types
    globalThis.__BABEL_TYPES__ = window.Babel.packages?.types || window.Babel.types;
    globalThis.__BABEL_CORE__ = window.Babel;
  }
})();`,
      sourcemap: true
    },
    {
      file: 'dist/minimact-babel-plugin.esm.js',
      format: 'es',
      sourcemap: true
    }
  ],
  plugins: [
    replace({
      preventAssignment: true,
      delimiters: ['', ''],
      values: {
        "require('@babel/types')": "globalThis.__BABEL_TYPES__",
        "require('@babel/core')": "globalThis.__BABEL_CORE__",
        'require("@babel/types")': "globalThis.__BABEL_TYPES__",
        'require("@babel/core")': "globalThis.__BABEL_CORE__"
      }
    }),
    resolve({
      preferBuiltins: false,
      browser: true
    }),
    commonjs({
      sourceMap: true
    }),
    json()
  ]
};
