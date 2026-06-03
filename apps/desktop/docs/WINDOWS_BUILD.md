# Windows 桌面端打包

当前 `tauri.conf.json` 的 `bundle.targets` 默认为 **macOS**（`dmg` / `app`）。在 Windows 上打包需调整 targets 并准备环境。

## 环境

- [Tauri 前置依赖（Windows）](https://v2.tauri.app/start/prerequisites/)
- Node.js 22+、Rust stable
- PATH 中的 `bcip`（或打包前运行 `npm run prepare-sidecar` 生成 Windows triple 的 sidecar）

## 配置

在 **Windows 构建机** 上编辑 `src-tauri/tauri.conf.json`：

```json
"bundle": {
  "targets": ["nsis"],
  "externalBin": ["binaries/bcip"],
  ...
}
```

`prepare-sidecar.sh` 在 Git Bash / WSL 下会复制 `bcip-x86_64-pc-windows-msvc`（需本机已安装对应 triple 的 bcip）。

## 构建

```bash
cd apps/desktop
npm install
npm run prepare-sidecar
npm run build
npm run tauri build
```

产物通常在 `src-tauri/target/release/bundle/nsis/`。

## 签名与分发

- 测试版可使用未签名安装包；企业分发需 Authenticode 签名。
- 与 macOS 类似，首启可能需 SmartScreen 例外说明（见 `TESTING.md` 思路，可另写 Windows 段落）。

## CI（可选）

可在 `windows-latest` runner 上增加仅路径触发的 workflow，步骤与 `desktop-typescript` 类似，最后执行 `tauri build`。Mac 专用 target 勿在 Linux/macOS CI 中启用 `nsis`。
