# BCIP 桌面端（云熙）

Tauri v2 + React，经 **app-server JSON-RPC** 与 CLI/TUI 共用 **`~/.bcip`**（`BCIP_HOME`），与官方 Codex 桌面 `~/.codex` 隔离。

**用户操作说明**：[桌面端操作指南](../../DESKTOP_USER_GUIDE.md)（仓库根目录）。

## 开发

脚本在 **`apps/desktop/package.json`**。可在仓库根目录或本目录执行：

**方式 A — 仓库根目录 `BCIP/`（推荐）**

```bash
cd /path/to/BCIP
npm install --prefix apps/desktop   # 首次安装前端依赖
npm run desktop:ci                  # lint + 构建 + generate-ts（根目录转发）
npm run prepare-sidecar             # 根 package.json 已转发到 apps/desktop
npm run tauri:dev
```

**方式 B — 进入桌面子目录**

```bash
cd apps/desktop
npm install
npm run prepare-sidecar
npm run tauri:dev
```

```bash
# 确认当前目录
pwd   # 根目录应为 .../BCIP，或 .../BCIP/apps/desktop
```

仅前端（浏览器 mock）：

```bash
npm run dev
```

本地 CI 等价（lint + 构建 + generate-ts + Playwright E2E）：

```bash
npm run test:e2e:install   # 首次
npm run ci
```

Playwright E2E（自动启动 `npm run dev` / `VITE_DEV_MOCK=1`）：

```bash
npx playwright install chromium   # 首次
npm run test:e2e
# 仓库根目录：npm run desktop:e2e
```

冒烟（lint + 构建 + cargo check + **stdio JSON-RPC**：`initialize` → `thread/start`，可选 `turn/start`）：

```bash
npm run smoke
# 仅 RPC（需 PATH 中有 bcip）：
python3 scripts/smoke-app-server-rpc.py
# 完整对话回合（需模型/API）：
BCIP_SMOKE_SKIP_TURN=0 npm run smoke
```

浏览器 mock：`npm run dev`（默认 `VITE_DEV_MOCK=1`）。专利 mock 中心区仅在 **未连接 app-server** 或 **VITE_DEV_MOCK=1** 时显示。

```bash
npm run check:generate-ts   # 与 bcip app-server generate-ts 输出 diff（CI 同脚本）
```

像素走查 C01–C12：见 [docs/CODEX_PIXEL_WALKTHROUGH.md](docs/CODEX_PIXEL_WALKTHROUGH.md)；自动生成截图：`npm run walkthrough:capture`。

Windows 打包：见 [docs/WINDOWS_BUILD.md](docs/WINDOWS_BUILD.md)。

打包内置 bcip（需 PATH 中已有 `bcip`）：

```bash
npm run prepare-sidecar   # 生成 src-tauri/binaries/bcip-<triple>
npm run tauri:build:bundle
```

Agent 修改工作区文件后，侧栏文件树与当前预览会自动刷新（`item/fileChange/patchUpdated`）。

## 根目录转发

在 `BCIP/` 根目录可用：`npm run desktop:ci`、`npm run desktop:smoke`、`npm run tauri:dev`、`npm run prepare-sidecar`（见根 `package.json`）。

## 依赖

- 系统 PATH 中的 `bcip`，或打包 sidecar（见 `src-tauri/binaries/README.md`）
- Rust / Node 与 [Tauri 前置](https://v2.tauri.app/start/prerequisites/)

## 交付自检（Mac）

1. `npm run desktop:ci`（或在本目录 `npm run ci`）
2. `npm run prepare-sidecar`（首次）
3. `npm run tauri:dev`
4. Boot 成功 → 状态栏 **已连接** → 右侧发「你好」有流式回复
5. 中心区（未打开文件时）同步显示 Agent 输出
6. `npm run desktop:smoke` 通过（含 E2E；跳过 E2E：`BCIP_SMOKE_SKIP_E2E=1`）

## 文档

- [docs/CODEX_PIXEL_WALKTHROUGH.md](docs/CODEX_PIXEL_WALKTHROUGH.md) — C01–C12 走查
- [docs/TESTING.md](docs/TESTING.md) — 内测说明
- `docs/plans/2026-05-30-desktop-implementation-plan.md` — 分阶段落地计划（仓库 `docs/plans/`）
- `docs/plans/2026-05-30-desktop-codex-parity-strategy.md` — 架构决策
