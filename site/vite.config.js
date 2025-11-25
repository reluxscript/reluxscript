import { defineConfig } from 'vite'
import { resolve } from 'path'

export default defineConfig({
  assetsInclude: ['**/*.wasm'],
  optimizeDeps: {
    exclude: ['reluxscript']
  },
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        playground: resolve(__dirname, 'playground.html')
      }
    }
  },
  server: {
    fs: {
      // Allow serving files from the pkg directory
      allow: ['..']
    }
  }
})
