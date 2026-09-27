import js from '@eslint/js';
import { defineConfig } from 'eslint/config';
import sonarjs from 'eslint-plugin-sonarjs';
import tseslint from 'typescript-eslint';

export default defineConfig(
  { ignores: ['dist', 'dist-base', 'src/generated', 'src/wasm'] },
  js.configs.recommended,
  ...tseslint.configs.strictTypeChecked,
  ...tseslint.configs.stylisticTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
      },
    },
    plugins: { sonarjs },
    rules: {
      'sonarjs/cognitive-complexity': ['error', 8],
      complexity: ['error', 8],
      'max-depth': ['error', 2],
      'max-lines-per-function': ['error', { max: 40, skipBlankLines: true }],
      'max-lines': ['error', { max: 200, skipBlankLines: true }],
      'max-params': ['error', 4],
      'no-console': ['error', { allow: ['error'] }],
      eqeqeq: 'error',
    },
  },
);
