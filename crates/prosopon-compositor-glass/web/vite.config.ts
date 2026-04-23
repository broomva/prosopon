import { defineConfig } from "vite";
import preact from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [preact()],
  build: {
    outDir: "dist",
    emptyOutDir: true,
    lib: {
      entry: "src/index.tsx",
      name: "ProsoponGlass",
      fileName: "index",
      formats: ["es"],
    },
    rollupOptions: {
      external: [],
      output: {
        inlineDynamicImports: true,
        assetFileNames: "assets/[name][extname]",
      },
    },
    sourcemap: true,
  },
  test: {
    environment: "happy-dom",
    setupFiles: ["./tests/setup.ts"],
    globals: false,
  },
});
