import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import pluginVue from "eslint-plugin-vue";
import { defineConfig } from "eslint/config";

export default defineConfig([
  {
    // 注意：这里只应该有 ignores 属性，没有其他属性
    ignores: [
      "**/dist/**", // 忽略 dist 目录下的所有文件
      "**/node_modules/**", // 忽略 node_modules 目录
      // 可以在这里添加其他需要忽略的模式
      // "**/build/**", // 例如：忽略 build 目录
      // "**/.nuxt/**", // 例如：忽略 .nuxt 目录
    ]
  },
  { files: ["**/*.{js,mjs,cjs,ts,mts,cts,vue}"], plugins: { js }, extends: ["js/recommended"], languageOptions: { globals: {...globals.browser, ...globals.node} } },
  tseslint.configs.recommended,
  pluginVue.configs["flat/essential"],
  { files: ["**/*.vue"], languageOptions: { parserOptions: { parser: tseslint.parser } } },
  { rules: { "@typescript-eslint/no-explicit-any": "off" } },
  { languageOptions: { globals: { __APP_VERSION__: "readonly" } } },
]);
