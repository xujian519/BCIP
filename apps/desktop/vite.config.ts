import path from "path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vite"
import { inspectAttr } from 'plugin-inspect-react-code'

// https://vite.dev/config/
export default defineConfig({
  base: './',
  plugins: [inspectAttr(), react()],
  server: {
    port: 5173,
    strictPort: true,
    host: '127.0.0.1',
    hmr: {
      host: '127.0.0.1',
      port: 5173,
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "@eigenpal/docx-editor-react/dist/styles.css":
        path.resolve(__dirname, "node_modules/@eigenpal/docx-editor-react/dist/styles.css"),
    },
  },
  ssr: {
    noExternal: ['@eigenpal/docx-editor-react'],
  },
  optimizeDeps: {
    include: ['@eigenpal/docx-editor-react'],
  },
});
