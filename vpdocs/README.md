# Minimact Documentation (VitePress)

This is the VitePress-powered documentation site for Minimact.

## Development

```bash
npm run dev
```

Visit http://localhost:5173

## Build

```bash
npm run build
npm run preview
```

## Structure

```
docs-mvp/
├── .vitepress/
│   └── config.ts          # VitePress configuration
├── guide/
│   ├── getting-started.md # Installation & setup
│   ├── concepts.md        # Core concepts
│   └── predictive-rendering.md
├── api/
│   └── hooks.md           # API reference
├── examples.md            # Code examples
└── index.md               # Homepage
```

## Customization

Edit `.vitepress/config.ts` to customize:
- Navigation
- Sidebar
- Theme colors
- Social links
- Search
