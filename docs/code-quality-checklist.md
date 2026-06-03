# BCIP 代码质量检查清单

本文档提供 BCIP 项目代码质量检查的详细指南。所有代码变更必须通过这些检查才能提交。

---

## 快速检查命令

```bash
# 1. 编译检查
cargo check -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 2. Clippy 检查
cargo clippy -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 3. 格式化
cargo fmt

# 4. 测试
cargo nextest run -p codex-patent-domain -p codex-patent-agents -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills
```

---

## 检查项详情

### 1. 编译检查 (cargo check)

**零容忍错误**：
- 类型不匹配
- 语法错误
- 缺失 trait 实现

**常见问题**：

| 问题 | 示例 | 解决方法 |
|------|------|----------|
| 类型不匹配 | `expected String, found &str` | 使用 `.to_string()` 或修改函数签名 |
| 未导入 trait | `use of undeclared trait` | 添加 `#[derive(...)]` 或手动实现 |
| 生命周期错误 | `borrowed value does not live long enough` | 调整生命周期参数或使用 `clone()` |

### 2. Clippy 检查 (cargo clippy)

**零容忍警告**：

#### 2.1 redundant_closure

```rust
// ❌ 错误
let names: Vec<_> = items.iter().map(|item| item.name()).collect();

// ✅ 正确
let names: Vec<_> = items.iter().map(|item| item.name()).collect();
```

#### 2.2 collapsible_if

```rust
// ❌ 错误
if x > 0 {
    if y > 0 {
        do_something();
    }
}

// ✅ 正确
if x > 0 && y > 0 {
    do_something();
}
```

#### 2.3 uninlined_format_args

```rust
// ❌ 错误
println!("Hello, {}!", name);

// ✅ 正确
println!("Hello, {name}!");
```

#### 2.4 dead_code

```rust
// ❌ 错误
struct MyStruct {
    unused_field: i32,
    term_extraction: ThresholdConfig,  // 未使用但保留用于配置
}

// ✅ 正确
#[allow(dead_code)]
struct MyStruct {
    unused_field: i32,
    term_extraction: ThresholdConfig,
}
```

### 3. 格式化检查 (cargo fmt)

**自动修复**：
```bash
cargo fmt
```

**验证**：
```bash
cargo fmt --check
```

**常见问题**：
- 缩进不一致
- 行宽超过 100 字符
- 空行过多

### 4. 测试检查 (cargo nextest run)

**测试标准**：
- 使用 `pretty_assertions::assert_eq`
- 优先比较整个对象
- 使用 `nextest` 加速测试

**示例**：

```rust
use pretty_assertions::assert_eq;

// ✅ 推荐：整体比较
assert_eq!(expected_report, actual_report);

// ❌ 避免：逐字段断言
assert_eq!(expected_report.title, actual_report.title);
assert_eq!(expected_report.score, actual_report.score);
```

**运行测试**：
```bash
# 单个 crate
cargo nextest run -p codex-patent-domain

# 所有专利 crate
cargo nextest run -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 特定测试
cargo nextest run --test test_name
```

---

## 模块大小管理

### 检查模块大小

```bash
# 使用 cloc 统计代码行数
cloc --by-file codex-rs/codex-patent-domain/src/

# 或使用 wc
wc -l codex-rs/codex-patent-domain/src/*.rs
```

### 模块大小标准

| 大小 | 状态 | 操作 |
|------|------|------|
| < 400 LoC | ✅ 健康 | 保持 |
| 400-500 LoC | ⚠️ 警告 | 考虑拆分 |
| > 500 LoC | ❌ 超标 | 拆分到新模块 |
| > 800 LoC | 🔴 危险 | 强制拆分 |

### 拆分策略

**推荐**：
- 添加新模块而不是扩展现有模块
- 将相关功能和测试一起移动
- 保持模块职责单一

**特别关注的高频文件**：
- `codex-rs/tui/src/app.rs`
- `codex-rs/tui/src/bottom_pane/chat_composer.rs`
- `codex-rs/tui/src/bottom_pane/footer.rs`
- `codex-rs/tui/src/chatwidget.rs`

---

## 依赖变更流程

### 修改依赖后必须执行

```bash
# 1. 更新 Bazel lockfile
cd /Users/xujian/projects/BCIP
just bazel-lock-update

# 2. 验证 lockfile 无漂移
just bazel-lock-check
```

### 依赖变更检查项

- [ ] 更新 `Cargo.toml`
- [ ] 运行 `just bazel-lock-update`
- [ ] 运行 `just bazel-lock-check`
- [ ] 包含 `MODULE.bazel.lock` 在提交中
- [ ] 运行完整测试套件

---

## Git Pre-commit Hook

### 自动化检查脚本

创建 `.git/hooks/pre-commit`：

```bash
#!/bin/bash
set -e

echo "🔍 运行代码质量检查..."

# 进入 codex-rs 目录
cd codex-rs

# 1. 编译检查
echo "📦 编译检查..."
cargo check -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 2. Clippy 检查
echo "🔍 Clippy 检查..."
cargo clippy -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

# 3. 格式化检查
echo "✨ 格式化检查..."
cargo fmt --check

# 4. 测试检查
echo "🧪 测试检查..."
cargo nextest run -p codex-patent-domain -p codex-patent-agents -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

echo "✅ 所有检查通过！"
```

### 启用 Hook

```bash
chmod +x .git/hooks/pre-commit
```

---

## 常见问题排查

### 问题 1：类型不匹配

**症状**：
```
error[E0308]: mismatched types
   --> src/file.rs:10:20
    |
10  |     blocks.push("缺少技术手段");
    |                    ^^^^^^^^^^^^^^^ expected struct `String`, found `&str`
```

**解决方法**：
```rust
blocks.push("缺少技术手段".to_string());
```

### 问题 2：Clippy 警告

**症状**：
```
warning: redundant closure
   --> src/agent_manifest.rs:85:22
    |
85  |         .map(|e| PatentError::Io(e))  // ✅ 实际上这个是需要的，因为类型转换
    |                      ^^^^^^^^^^^^^^^^ help: replace the closure with `PatentError::Io`
```

**解决方法**：
- 如果确实需要闭包（类型转换），可以在函数前添加注释说明
- 或者在函数签名中使用 `into()` 方法

### 问题 3：测试失败

**症状**：
```
thread 'quality_rules::tests::test_yaml_parses_successfully' panicked at ...
assertion `left == right` failed
  left: 0
 right: 0.7
```

**解决方法**：
- 检查数据类型是否匹配
- 添加类型转换：`as_usize()`
- 修改字段类型：`f64` 而不是 `usize`

---

## CI/CD 集成

### GitHub Actions 配置

创建 `.github/workflows/rust-quality.yml`：

```yaml
name: Rust Quality Check

on: [push, pull_request]

jobs:
  quality:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy

      - name: Install nextest
        run: cargo install --locked cargo-nextest

      - name: Check
        run: |
          cd codex-rs
          cargo check -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

      - name: Clippy
        run: |
          cd codex-rs
          cargo clippy -p codex-patent-domain -p codex-patent-agents -p codex-patent-tools --no-default-features -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills

      - name: Format
        run: |
          cd codex-rs
          cargo fmt --check

      - name: Test
        run: |
          cd codex-rs
          cargo nextest run -p codex-patent-domain -p codex-patent-agents -p codex-patent-workflow -p codex-patent-constitutional -p codex-patent-skills
```

---

## 参考资料

- [Rust Clippy Lints](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Rust Format Guide](https://rust-lang.github.io/rustfmt/)
- [BCIP 智能体系统架构](./patent-system-map.md)
- [AGENTS.md](../AGENTS.md)
- [OpenCode 配置](../.opencode/opencode.json)