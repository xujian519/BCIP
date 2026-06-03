# Playwright E2E

在 `VITE_DEV_MOCK=1` 下通过 Vite 启动应用，覆盖主壳层、mock 对话与全局快捷键。

| 文件 | 覆盖 |
|------|------|
| `shell.spec.ts` | 标题栏、演示横幅、阶段 Tab |
| `mock-chat.spec.ts` | 发送「你好」→ mock 回复 |
| `shortcuts.spec.ts` | 命令面板、设置（⌘⇧P / ⌘,） |

```bash
npm run test:e2e:install   # 首次：浏览器安装到 node_modules（PLAYWRIGHT_BROWSERS_PATH=0）
npm run test:e2e
```

仓库根目录：`npm run desktop:e2e`。
