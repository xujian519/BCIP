# Contributing to BCIP

## 开发环境搭建

### 前置要求

- Rust 1.80+（推荐最新 stable）
- Node.js 18+（桌面端开发）
- just（任务运行器）: `cargo install just`
- Bazel（可选，用于部分构建任务）

### 安装步骤

```bash
git clone https://github.com/xujian519/BCIP.git
cd BCIP
just bootstrap    # 初始化开发环境
```

### 验证环境

```bash
just check        # 编译检查
just test         # 运行全量测试
```

## 代码规范

### Rust

- 运行 `just fmt` 自动格式化（提交前必须）
- 运行 `just fix -p <crate>` 修复 lint 问题
- 测试使用 `just test -p <crate>`
- 遵循 [Karpathy 编码原则](./CLAUDE.md)：

1. **编码前思考** — 不确定时询问而非猜测
2. **简洁优先** — 能用 50 行解决的不要写 200 行
3. **精准修改** — 不改相邻代码/注释/格式
4. **目标驱动** — 定义成功标准，循环直到验证通过

### 提交信息

使用 Conventional Commits 格式：

```
type(scope): 简短描述

详细说明（可选）
```

类型: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `perf`

范围: `codex-rs`, `desktop`, `protocol`, `patent-*`, `tui`

## 测试

```bash
just test -p codex-core                    # 单个 crate
just test -p codex-patent-tools            # 专利工具
just test --manifest-path codex-rs/Cargo.toml  # 指定 manifest
```

### 测试原则

- 每个 bug 修复必须附带回归测试
- 公共 API 变更必须更新相关测试
- 集成测试放在各 crate 的 `tests/` 目录

## Pull Request 流程

1. 从 `main` 创建特性分支
2. 确保所有测试通过
3. 确保 `just fmt` 无变更
4. 创建 PR，描述变更内容和动机
5. 等待 CI 和代码审查

## 项目结构

```
BCIP/
├── codex-rs/          # Rust workspace（核心代码）
│   ├── core/          # Agent 总线、会话管理、工具注册
│   ├── protocol/      # 通信协议定义
│   ├── exec/          # 执行引擎
│   ├── tui/           # 终端界面
│   ├── codex-patent-*/  # 专利领域模块（9 个 crate）
│   └── ...
├── apps/desktop/      # Tauri 桌面应用
├── docs/              # 文档
├── CLAUDE.md          # AI 协作指南
├── AGENTS.md          # Agent 系统文档
└── justfile           # 任务定义
```

## 联系方式

- Issues: https://github.com/xujian519/BCIP/issues
- Email: xujian519@gmail.com
