# 设计走查截图（C01–C12）

| 文件 | 说明 |
|------|------|
| `C01.png` … `C12.png` | BCIP 当前 UI，由 `npm run walkthrough:capture` 生成 |
| `codex-ref/C01.png` … | 可选 Codex 参考（见 [codex-ref/README.md](./codex-ref/README.md)） |
| `review.html` | 并排评审页，由 `npm run walkthrough:report` 生成（可 gitignore） |
| `DESIGN_SIGNOFF.md` | 设计签收单（评审后手工提交或从 review.html 导出） |

**快速开始**

```bash
cd apps/desktop
npm run walkthrough:acceptance
open docs/walkthrough-screenshots/review.html
```

拍摄与验收说明：

- 组件对照表：[CODEX_PIXEL_WALKTHROUGH.md](../CODEX_PIXEL_WALKTHROUGH.md)
- 验收流程：[DESIGN_ACCEPTANCE.md](../DESIGN_ACCEPTANCE.md)
