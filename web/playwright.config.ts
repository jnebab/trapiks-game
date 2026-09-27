import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: 'e2e',
  testIgnore: process.env.TRAPIKS_PERF === '1' ? [] : ['perf/**'],
  webServer: [
    {
      command: 'pnpm exec vite preview --port 4173 --strictPort',
      port: 4173,
      reuseExistingServer: false,
    },
    {
      command:
        'TRAPIKS_BASE=/trapiks-game/ pnpm exec vite build --outDir dist-base && pnpm exec vite preview --outDir dist-base --base /trapiks-game/ --port 4174 --strictPort',
      port: 4174,
      reuseExistingServer: false,
      timeout: 120_000,
    },
  ],
  use: {
    baseURL: 'http://localhost:4173',
    launchOptions: { args: ['--use-angle=swiftshader'] },
  },
});
