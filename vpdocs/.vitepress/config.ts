import { defineConfig } from 'vitepress'

const currentVersion = '0.1'

export default defineConfig({
  title: 'ReluxScript',
  description: 'Write AST transformations once. Compile to Babel, SWC, and beyond.',
  ignoreDeadLinks: true,

  themeConfig: {
    logo: '/logo.png',

    nav: [
      { text: 'Guide', link: '/v0.1/guide/getting-started' },
      { text: 'Language', link: '/v0.1/language/syntax' },
      { text: 'Examples', link: '/v0.1/examples/' },
      { text: 'API Reference', link: '/v0.1/api/visitor-methods' },
      {
        text: `v${currentVersion}`,
        items: [
          {
            text: 'v0.1 (Current)',
            link: '/v0.1/guide/getting-started',
            activeMatch: '/v0.1/'
          }
        ]
      }
    ],

    sidebar: {
      '/v0.1/guide/': [
        {
          text: 'Introduction',
          items: [
            { text: 'Getting Started', link: '/v0.1/guide/getting-started' },
            { text: 'Why ReluxScript?', link: '/v0.1/guide/why-reluxscript' },
            { text: 'Core Concepts', link: '/v0.1/guide/concepts' }
          ]
        },
        {
          text: 'Installation',
          items: [
            { text: 'Installation', link: '/v0.1/guide/installation' },
            { text: 'Your First Plugin', link: '/v0.1/guide/first-plugin' }
          ]
        },
        {
          text: 'Compilation',
          items: [
            { text: 'Compiling to Babel', link: '/v0.1/guide/compiling-babel' },
            { text: 'Compiling to SWC', link: '/v0.1/guide/compiling-swc' }
          ]
        }
      ],
      '/v0.1/language/': [
        {
          text: 'Language Reference',
          items: [
            { text: 'Syntax Overview', link: '/v0.1/language/syntax' },
            { text: 'Types', link: '/v0.1/language/types' },
            { text: 'Expressions', link: '/v0.1/language/expressions' },
            { text: 'Pattern Matching', link: '/v0.1/language/pattern-matching' }
          ]
        },
        {
          text: 'AST Nodes',
          items: [
            { text: 'Node Types', link: '/v0.1/language/node-types' },
            { text: 'Visitor Pattern', link: '/v0.1/language/visitor-pattern' }
          ]
        }
      ],
      '/v0.1/examples/': [
        {
          text: 'Examples',
          items: [
            { text: 'Overview', link: '/v0.1/examples/' },
            { text: 'Remove Console', link: '/v0.1/examples/remove-console' },
            { text: 'Arrow Functions', link: '/v0.1/examples/arrow-functions' },
            { text: 'JSX Key Checker', link: '/v0.1/examples/jsx-keys' },
            { text: 'Hook Analyzer', link: '/v0.1/examples/hook-analyzer' }
          ]
        }
      ],
      '/v0.1/api/': [
        {
          text: 'API Reference',
          items: [
            { text: 'Visitor Methods', link: '/v0.1/api/visitor-methods' },
            { text: 'Node Constructors', link: '/v0.1/api/node-constructors' },
            { text: 'Context API', link: '/v0.1/api/context' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/reluxscript/reluxscript' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025-present ReluxScript • Light, Light, Write! ⚡'
    },

    search: {
      provider: 'local'
    }
  }
})
