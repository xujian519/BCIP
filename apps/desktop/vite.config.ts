import path from "path"
import react from "@vitejs/plugin-react"
import { defineConfig } from "vitest/config"
import { inspectAttr } from 'plugin-inspect-react-code'

// https://vite.dev/config/
export default defineConfig(({ command }) => ({
  base: './',
  plugins: [
    ...(command === 'serve' ? [inspectAttr()] : []),
    react(),
  ],
  test: {
    environment: 'node',
    include: ['src/**/*.test.ts'],
  },
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
      // react-pdf 将 pdfjs-dist 作为直接依赖捆绑（当前 5.4.x），而项目顶层安装了
      // 不同版本（5.7.x）。将所有 pdfjs-dist 导入统一解析到 react-pdf 的嵌套版本，
      // 确保 worker 与主库版本一致，避免 PDF 预览运行时报错。
      "pdfjs-dist": path.resolve(__dirname, "node_modules/react-pdf/node_modules/pdfjs-dist"),
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
}));
