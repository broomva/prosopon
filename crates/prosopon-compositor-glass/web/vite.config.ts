import { defineConfig } from "vite";
import preact from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [preact()],
  build: {
    outDir: "dist",
    // Keep `dist/.gitkeep` alive for include_dir! on fresh clones; library
    // builds write a fixed set of files so stale accumulation isn't a risk.
    emptyOutDir: false,
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
