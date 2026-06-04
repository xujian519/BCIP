/**
 * 在 Tauri 桌面端使用本地打包的 pdf.js worker，避免 CDN 被 CSP 拦截。
 *
 * 关键：worker 版本必须与 react-pdf 内部使用的 pdfjs-dist 版本一致。
 * react-pdf 将 pdfjs-dist 作为直接依赖（非 peerDependency），因此有自己的嵌套副本。
 * 我们通过 Vite alias 确保顶层 pdfjs-dist 解析到 react-pdf 的版本来避免版本冲突。
 */
import { pdfjs } from 'react-pdf';
import workerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url';

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;
