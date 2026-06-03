# bcip sidecar

由 `npm run prepare-sidecar` 生成 `bcip-<host-triple>`（已 gitignore）。

- 若 PATH 有 `bcip`：复制真实二进制
- 否则：生成包装脚本，运行时 `exec bcip`

`tauri.conf.json` 的 `externalBin` 依赖此文件；首次 `tauri dev` / `cargo check` 前请执行：

```bash
npm run prepare-sidecar
```

打包：`npm run tauri:build:bundle`
