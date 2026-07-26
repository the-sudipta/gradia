import { defineConfig } from "vite";

export default defineConfig({
  root: "src",
  base: "./",
  clearScreen: false,
  server: {
    strictPort: true,
    port: 1420
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
    target: "es2021"
  }
});
