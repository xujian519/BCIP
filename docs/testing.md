# Testing Strategy

## 测试分层

| 层级 | 范围 | 运行命令 | 覆盖目标 |
|------|------|----------|----------|
| 单元测试 | 单个函数/模块 | `cargo test -p <crate>` | 核心逻辑路径 |
| 集成测试 | 跨模块交互 | `cargo test --test <name>` | 模块边界 |
| 并发测试 | 多线程/异步竞争 | 包含 `concurrency` 的测试 | 线程安全 |

## 运行测试

```bash
# 单个 crate
just test -p codex-core

# 全 workspace（从 codex-rs/ 目录运行）
just test

# 指定 manifest
cargo test --manifest-path codex-rs/Cargo.toml -p <crate>

# 运行特定测试
cargo test --manifest-path codex-rs/Cargo.toml -p codex-core -- "bus::tests"

# 格式化 + 测试
just fmt && just test
```

## 测试规范

### 命名约定

- 测试文件: `<module>_tests.rs` 或 `<module>_concurrency_tests.rs`
- 测试函数: `test_<behavior>_<condition>_<expected>`
- 并发测试: `concurrent_<scenario>`

### 必须覆盖

1. **Bug 修复**: 每个修复附带回归测试
2. **公共 API**: 所有 `pub` 函数的边界情况
3. **错误路径**: `Result::Err` 分支
4. **并发安全**: 共享状态的竞争条件

### 不强制要求

- 私有函数测试（通过公共 API 间接覆盖）
- 集成测试依赖外部服务（使用 `#[ignore]` 标记）

## 测试覆盖率

查看各 crate 测试数量:

```bash
cargo test --manifest-path codex-rs/Cargo.toml -- --list 2>&1 | grep "test$" | wc -l
```

## CI 流程

1. `just fmt` — 格式检查
2. `cargo check` — 编译检查
3. `just test` — 全量测试
4. `just fix` — lint 修复（如有）
