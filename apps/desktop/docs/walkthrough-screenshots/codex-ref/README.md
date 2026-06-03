# Codex 参考截图（可选）

将 **Codex 官方桌面版** 在相同场景下的截图放入此目录，文件名与 BCIP 走查一致：

| 文件 | 场景 |
|------|------|
| `C01.png` | 线程列表单行/列表区 |
| `C02.png` | 用户消息气泡 |
| `C03.png` | 助手消息块 |
| `C04.png` | 工具调用卡片（展开） |
| `C05.png` | 命令审批弹窗 |
| `C06.png` | MCP 服务器设置页 |
| `C07.png` | 设置左侧导航 |
| `C08.png` | 模型与推理设置 |
| `C09.png` | Agent 顶栏用量 |
| `C10.png` | Composer + slash palette |
| `C11.png` | Reasoning 折叠块 |
| `C12.png` | 断线 Footer |

**建议参数**

- 分辨率：**1440 × 900**（与 Playwright 走查一致）
- 主题：**深色**（与 mock 走查默认一致；若验收浅色，请在签收单注明）
- 格式：PNG，无损

参考图齐全后，`npm run walkthrough:report` 生成的 `review.html` 会显示 BCIP | Codex 并排对比。

> 参考图可能含敏感信息，默认 **不提交 git**；团队内通过共享盘或加密渠道传递。若需纳入仓库，请脱敏后单独 PR。
