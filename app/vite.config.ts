/// <reference types="vitest/config" />
import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

export default defineConfig({
  plugins: [react(), tailwindcss()],

  test: {
    environment: "jsdom",
    setupFiles: "./src/tests/setup.ts",
    include: ["src/**/*.test.ts?(x)"],
    env: { TZ: "UTC" },
    coverage: {
      provider: "v8" as const,
      reporter: ["text-summary", "lcov"],
      reportsDirectory: "coverage",
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/tests/**", "src/main.tsx", "src/vite-env.d.ts"],
    },
  },

  clearScreen: false,
  server: { port: 1420, strictPort: true },
});
