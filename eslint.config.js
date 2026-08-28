import js from "@eslint/js";
import ts from "typescript-eslint";
import svelte from "eslint-plugin-svelte";
import svelteConfig from "./svelte.config.js";
import eslintConfigPrettier from "eslint-config-prettier";
import globals from "globals";

export default ts.config(
  js.configs.recommended,
  ...ts.configs.recommended,
  ...svelte.configs["flat/recommended"],
  eslintConfigPrettier,
  ...svelte.configs["flat/prettier"],
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
    },
    rules: {
      // Allow the conventional `_`-prefix for intentionally-unused args/vars
      // (e.g. a mock that ignores its inputs to satisfy an interface). Real
      // unused variables are still reported.
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          args: "all",
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    files: ["**/*.svelte", "**/*.svelte.ts", "**/*.svelte.js"],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
        extraFileExtensions: [".svelte"],
        parser: ts.parser,
        svelteConfig,
      },
    },
  },
  {
    ignores: [
      "src-tauri/**",
      "dist/**",
      "build/**",
      "node_modules/**",
      "target/**",
      ".claude/**",
    ],
  },
);
