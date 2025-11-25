import eslint from '@eslint/js';
import tseslint from '@typescript-eslint/eslint-plugin';
import tseslintParser from '@typescript-eslint/parser';
import prettier from 'eslint-config-prettier';
import globals from 'globals';

export default [
  {
    files: ['**/*.{js,ts}'],
    ignores: [
      'node_modules/',
      'dist/',
      'dist-ssr/',
      '.svelte-kit/',
      'src-tauri/',
      '.vscode/',
      '.idea/',
      '.DS_Store',
      '.env',
      '.env.local',
      '.env.*.local',
      'coverage/',
      '*.lcov',
      '.nyc_output',
      '*.tmp',
      '*.temp',
      '.cache/',
      'Thumbs.db',
      '*.rs',
      '*.rs.bk',
      '*.config.js',
      '*.config.cjs',
      '*.config.mjs',
      '*.config.ts',
      '*.config.json',
      '*.md',
      '*.mdx',
      '*.rst',
      'logs',
      '*.log',
      'npm-debug.log*',
      'yarn-debug.log*',
      'yarn-error.log*',
      'pnpm-debug.log*',
      'lerna-debug.log*',
      'bun-debug.log*',
      'bun-error.log*'
    ],
    languageOptions: {
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: {
        ...globals.browser,
        ...globals.node,
        ...globals.es2020
      },
      parser: tseslintParser,
      parserOptions: {
        project: './tsconfig.json'
      }
    },
    plugins: {
      '@typescript-eslint': tseslint
    },
    rules: {
      ...eslint.configs.recommended.rules,
      ...prettier.rules,
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': [
        'error',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_'
        }
      ],
      '@typescript-eslint/explicit-function-return-type': 'off',
      '@typescript-eslint/explicit-module-boundary-types': 'off',
      '@typescript-eslint/no-non-null-assertion': 'off'
    }
  }
];