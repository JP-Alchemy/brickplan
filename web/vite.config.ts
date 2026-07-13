import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';
import topLevelAwait from 'vite-plugin-top-level-await';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
  optimizeDeps: {
    // The wasm-pack output must not be pre-bundled: esbuild would inline
    // the .wasm asset URL incorrectly.
    exclude: ['planner'],
  },
  server: {
    fs: {
      // The planner package is symlinked from ../planner/pkg, outside the
      // web root, so the dev server needs the repo root allowed.
      allow: ['..'],
    },
  },
});
