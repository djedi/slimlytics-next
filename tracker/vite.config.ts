import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    lib: { entry: 'src/index.ts', name: 'Slimlytics', formats: ['es', 'iife'], fileName: (format) => format === 'iife' ? 'slimlytics.js' : 'slimlytics.es.js' },
    minify: 'esbuild'
  },
  test: { environment: 'jsdom', restoreMocks: true }
});
