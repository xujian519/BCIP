# BCIP 全量质量审查 — 设计方案

> 日期：2026-05-31
> 目标：项目完全可用，无断链，端到端测试全部通过
> 方案：方案 A — 全量扫描-批量修复

## 一、整体架构

质量审查分为 4 个阶段，输出统一的问题清单和修复方案：

```
阶段 1：全量编译诊断
├── Rust workspace (codex-rs/)    → cargo build --workspace
├── 桌面应用 (apps/desktop/)       → pnpm build
├── Python SDK                     → uv build
├── TypeScript SDK                 → pnpm build
├── 依赖完整性检查                  → cargo-shear, cargo-deny
└── 输出：编译错误/警告清单

阶段 2：全量测试诊断
├── Rust tests (just test)         → 按 crate 分组结果
├── Python SDK tests (pytest)      → 失败 + 覆盖率
├── TypeScript SDK tests (jest)    → 失败分类
├── 桌面 E2E (Playwright)          → 失败 / flaky
└── 输出：测试失败清单（阻断/非阻断）

阶段 3：断链与完整性扫描
├── Rust 模块引用完整性
├── Cargo 依赖完整性
├── 配置引用完整性
├── Skill/Agent 定义与注册一致性
├── 文档内链有效性
├── CI 工作流完整性
├── 桌面应用路由完整性
├── SDK 导出完整性
└── 输出：断链清单 (DEAD_LINK_REPORT.json)

阶段 4：分批修复 + 端到端补全
├── P0 阻断修复：编译失败、测试崩溃、模块缺失
├── P1 功能修复：测试断言错误、逻辑缺陷
├── P2 测试补全：4 条核心 E2E 链路
├── P3 文档/CI：断链修复、配置对齐
└── 验证：全部构建 + 测试通过
```

最终产出：`QUALITY_AUDIT_REPORT.md`

## 二、阶段 1：全量编译诊断

### 诊断矩阵

| 构建目标 | 命令 | 覆盖内容 | 预期输出 |
|---------|------|---------|---------|
| Rust 全量 | `cargo build --workspace --all-features 2>&1` | 80+ crate 编译 | 错误分类 |
| Rust 检查 | `cargo check --workspace --all-features` | 类型检查 | 类型错误清单 |
| Rust 专利层 | `cargo build -p codex-patent-core -p codex-patent-domain -p codex-patent-tools -p codex-patent-agents -p codex-patent-skills -p codex-patent-knowledge -p codex-patent-constitutional -p codex-patent-text -p codex-patent-scheduler` | 专利核心链路 | 专利层专项错误 |
| 桌面前端 | `pnpm run build` (apps/desktop/) | React + Vite + Tauri | TS 类型错误、打包错误 |
| 桌面后端 | `cargo build -p desktop-app` | Tauri Rust 后端 | 绑定错误 |
| Python SDK | `uv run build` (sdk/python/) | 包构建 | 模块导入错误 |
| TS SDK | `pnpm build` (sdk/typescript/) | 包构建 | TS 编译错误 |

### 错误分级

- **P0 阻断**：编译失败，crate 不可用
- **P1 警告**：编译器 warning（clippy deny 级别），应清理
- **P2 关注**：deprecation notice、unused dependency 提示

### 依赖完整性

并行运行：
- `cargo-shear`：检测未使用依赖
- `cargo-deny`：安全/许可审计

## 三、阶段 2：全量测试诊断

### 测试执行矩阵

| 测试套件 | 命令 | 关键检查点 |
|---------|------|-----------|
| Rust 全量 | `just test` (cargo nextest) | 80+ crate，nextest 配置重试 |
| Rust 专利核心 | `just test -p codex-patent-core` 等 9 个 crate | 领域类型、工具注册、Agent 加载 |
| Rust 专利 E2E | `just test -p codex-patent-domain` | 8 条已有链路测试 |
| Rust TUI | `just test -p codex-tui` | insta 快照测试 |
| Python SDK | `uv run pytest` | 单元测试 |
| TypeScript SDK | `pnpm test` | jest 测试 |
| 桌面 E2E | `npx playwright test` | 浏览器自动化 |

### 失败分类

按根因归类：
- **编译类**：crate 未编译导致测试不可执行
- **环境类**：依赖 `CODEX_SANDBOX_NETWORK_DISABLED=1` 或 `CODEX_SANDBOX=seatbelt` 导致 skip — 预期行为，不算失败
- **断言类**：逻辑/数据变更导致断言不匹配
- **超时类**：CI 超时配置过短
- **快照类**：insta snapshot 过期，需 `cargo insta accept`

### 特殊处理

- Sandbox 环境变量引起的 skip 是预期行为
- TUI snapshot 变更需人工审查后 accept
- Bazel 和 Cargo 测试可能结果不同，优先修 Cargo 侧

## 四、阶段 3：断链与完整性扫描

### 扫描维度

| 维度 | 工具/方法 | 检查内容 |
|------|----------|---------|
| Rust 模块引用 | `cargo check` 输出 + 文件扫描 | 缺失 .rs 文件、未声明模块、循环引用 |
| Cargo 依赖 | `cargo-shear` + `cargo tree` | 未使用依赖、版本冲突、feature 未启用 |
| Rust import/use | 编译 warning `unused_imports` + mod.rs/lib.rs 导出扫描 | 断掉的 re-export、未导出的公共 API |
| 配置引用 | `ConfigToml` 字段 → 使用点扫描 | 定义了但未读取的配置、默认值缺失 |
| Skill/Agent 文件 | `*.toml` 定义 → handler 注册扫描 | 定义但未注册的技能/Agent |
| 文档内链 | markdown 链接扫描 (docs/、AGENTS.md、CLAUDE.md) | 404 链接、锚点不存在 |
| CI 工作流 | `.github/workflows/*.yml` 审查 | 不存在的 job 依赖、缺失 secret、过期 action |
| 桌面应用路由 | React Router 路由 → 页面组件扫描 | 404 路由、缺失的组件引用 |
| SDK 导出 | `__init__.py` / `index.ts` 导出 → 实际模块扫描 | 断掉的导出 |

### 输出格式

`DEAD_LINK_REPORT.json`，每项包含：
- `type`：module / dependency / config / skill / doc / ci / route / export
- `location`：文件路径 + 行号
- `severity`：P0 / P1 / P2
- `suggestion`：修复建议

## 五、阶段 4：分批修复与端到端补全

### P0 阻断修复（第 1 批）

目标：所有 crate 编译通过，核心测试套件不崩溃。

| 问题类型 | 修复策略 |
|---------|---------|
| 编译错误 | 缺失类型补定义、feature gate 修正、import 路径修复 |
| 模块缺失 | 补充 `mod` 声明或文件、修复 `lib.rs` 导出 |
| 测试崩溃 | panic/fail 而非 assertion fail → 修环境依赖或参数校验 |
| 循环依赖 | 提取公共类型到上层 crate |
| 配置缺失 | 补默认值或补配置文件 |

每项修复后运行对应 crate 测试验证。

### P1 功能修复（第 2 批）

目标：所有测试断言通过，insta 快照更新。

- 断言不匹配 → 检查逻辑是否正确，修代码或修测试
- insta snapshot 过期 → 逐个审查 `.snap.new` 文件，`cargo insta accept`
- Flaky 测试 → 增加超时、添加 retry、或标记为 `#[ignore]` 后记录

### P2 端到端测试补全（第 3 批）

为专利核心链路编写新 E2E 测试，按 `core_test_support::responses` 模式编写：

| 链路 | 覆盖内容 | 参考模式 |
|------|---------|---------|
| 检索链路 | 关键词检索 → 语义检索 → 结果排序 → 详情获取 | `codex-patent-domain` 搜索测试 |
| 分析链路 | 专利文本输入 → 特征提取 → 新颖性/创造性判断 | `codex-patent-domain` 分析测试 |
| 撰写链路 | 技术交底书 → 说明书生成 → 权利要求撰写 | `PatentToolHandler` 工具注册 |
| 审查链路 | 审查意见输入 → 对比文件分析 → OA 答复生成 | Agent 角色 + skill 定义 |

### P3 文档/CI 修复（第 4 批）

- 断掉的 markdown 链接 → 修路径或删除
- CI workflow 问题 → 补缺失 job、升级过期 action
- `ConfigToml` 孤字段 → 删除或补使用点
- 桌面路由 404 → 补 page 组件或删除路由

### 最终验证

```bash
just test              # 全部 Rust 测试通过
cargo build --workspace # 全部编译通过
cargo-shear            # 无未使用依赖
uv run pytest          # Python SDK 测试通过
pnpm test              # TS SDK 测试通过
npx playwright test    # 桌面 E2E 通过
```

### 产出

更新 `QUALITY_AUDIT_REPORT.md`，包含：
- 每批修复的项目数和影响范围
- E2E 测试覆盖链路和结果
- 仍遗留的已知问题（如有）

## 六、成功标准

| 标准 | 指标 |
|------|------|
| 编译 | `cargo build --workspace --all-features` 零错误 |
| 测试 | `just test` 全部通过（skip 除外） |
| SDK | Python + TypeScript 编译 + 测试通过 |
| 桌面 | Tauri 构建 + Playwright E2E 通过 |
| 依赖 | `cargo-shear` 零未使用依赖 |
| 断链 | `DEAD_LINK_REPORT.json` 为空 |
| E2E 覆盖 | 4 条核心专利链路有端到端测试 |
