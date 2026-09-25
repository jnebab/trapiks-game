import { defineConfig } from 'vitest/config';

export default defineConfig({
  worker: { format: 'es' },
  build: { target: 'es2022' },
  server: { host: true },
  test: { environment: 'node', include: ['src/**/*.test.ts'] },
});
