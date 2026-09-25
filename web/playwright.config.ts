import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'e2e',
  webServer: {
    command: 'pnpm exec vite preview --port 4173 --strictPort',
    port: 4173,
    reuseExistingServer: false,
  },
  use: {
    baseURL: 'http://localhost:4173',
    launchOptions: { args: ['--use-angle=swiftshader'] },
  },
});
