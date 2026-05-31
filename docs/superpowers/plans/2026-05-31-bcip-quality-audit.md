# BCIP 全量质量审查 — 实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 对 BCIP 项目进行全量质量审查，修复所有编译错误、测试失败、断链问题，补全 4 条核心专利端到端测试链路，使项目完全可用。

**Architecture:** 4 阶段串行执行。阶段 1-3 为诊断（收集问题清单），阶段 4 为修复（按 P0→P1→P2→P3 顺序分批修复）。所有诊断结果输出到 `reports/` 目录。

**Tech Stack:** Rust 1.93.0 (cargo, nextest, insta), pnpm (TypeScript, ESLint, Playwright), uv (Python, pytest), just (task runner)

**Spec:** `docs/superpowers/specs/2026-05-31-bcip-quality-audit-design.md`

---

## 文件结构

```
reports/
├── build_errors_rust.txt          # Rust 编译错误输出
├── build_errors_desktop.txt       # 桌面应用构建错误
├── build_errors_sdk_python.txt    # Python SDK 构建错误
├── build_errors_sdk_ts.txt        # TS SDK 构建错误
├── shear_output.txt               # cargo-shear 输出
├── deny_output.txt                # cargo-deny 输出
├── test_failures_rust.txt         # Rust 测试失败清单
├── test_failures_sdk_python.txt   # Python 测试失败
├── test_failures_sdk_ts.txt       # TS 测试失败
├── test_failures_desktop.txt      # Playwright E2E 失败
├── dead_link_report.json          # 断链清单
└── QUALITY_AUDIT_REPORT.md        # 最终审计报告
```

---

### Task 1: Rust 全量编译诊断

- [ ] **Step 1: 运行 cargo check（比 build 快，优先用这个）**

```bash
cargo check --workspace --all-features 2>&1 | tee reports/build_errors_rust_check.txt
```

- [ ] **Step 2: 统计错误数量并分类**

用 ripgrep 提取错误行：
```bash
rg "^error" reports/build_errors_rust_check.txt | wc -l
rg "^error\[E" reports/build_errors_rust_check.txt || true
```

- [ ] **Step 3: 若 cargo check 通过，运行 cargo build**

```bash
cargo build --workspace --all-features 2>&1 | tee reports/build_errors_rust.txt
```

- [ ] **Step 4: 检查 clippy 警告数**

```bash
cargo clippy --workspace --all-features 2>&1 | rg "^warning" | wc -l
cargo clippy --workspace --all-features 2>&1 | tee reports/clippy_warnings.txt
```

- [ ] **Step 5: 分类输出**

对 Rust 错误按 crate 分组统计：
```bash
rg "^error" reports/build_errors_rust_check.txt | rg -o "-->\s+\S+" | sort | uniq -c | sort -rn | head -30
```

- [ ] **Step 6: Commit 诊断结果**

```bash
git add reports/build_errors_rust_check.txt reports/build_errors_rust.txt reports/clippy_warnings.txt 2>/dev/null; git commit -m "chore(audit): 阶段1 Rust 编译诊断结果"
```

---

### Task 2: 专利层专项编译诊断

**Files:**
- Create: `reports/build_errors_patent.txt`

- [ ] **Step 1: 逐个构建专利核心 crate**

```bash
for crate in codex-patent-core codex-patent-domain codex-patent-tools codex-patent-agents codex-patent-skills codex-patent-knowledge codex-patent-constitutional codex-patent-text codex-patent-scheduler codex-patent-assets; do
  echo "=== $crate ===" >> reports/build_errors_patent.txt
  cargo build -p "$crate" 2>&1 | rg "^error" >> reports/build_errors_patent.txt || true
  echo "" >> reports/build_errors_patent.txt
done
cat reports/build_errors_patent.txt
```

- [ ] **Step 2: 逐个 cargo check 专利 crate（更快得到类型错误）**

```bash
for crate in codex-patent-core codex-patent-domain codex-patent-tools codex-patent-agents codex-patent-skills codex-patent-knowledge codex-patent-constitutional codex-patent-text codex-patent-scheduler; do
  echo "=== $crate ==="
  cargo check -p "$crate" 2>&1 | rg "^error" || echo "  PASS"
done
```

- [ ] **Step 3: 检查 TUI crate**

```bash
cargo check -p codex-tui 2>&1 | rg "^error" || echo "codex-tui: PASS"
```

- [ ] **Step 4: Commit**

```bash
git add reports/build_errors_patent.txt; git commit -m "chore(audit): 阶段1 专利层编译诊断结果"
```

---

### Task 3: 桌面应用编译诊断

- [ ] **Step 1: 检查桌面应用是否独立可构建**

```bash
ls apps/desktop/package.json apps/desktop/src-tauri/Cargo.toml
```

- [ ] **Step 2: 安装依赖并构建前端**

```bash
cd apps/desktop && pnpm install 2>&1 | tail -5
pnpm run build 2>&1 | tee ../../reports/build_errors_desktop.txt
```

- [ ] **Step 3: 检查 Rust 后端（Tauri）**

```bash
cd apps/desktop/src-tauri && cargo check 2>&1 | rg "^error" | tee ../../../reports/build_errors_desktop_rust.txt || echo "Tauri Rust: PASS"
```

- [ ] **Step 4: 回工作目录，Commit**

```bash
cd /Users/xujian/projects/BCIP
git add reports/build_errors_desktop.txt reports/build_errors_desktop_rust.txt 2>/dev/null
git commit -m "chore(audit): 阶段1 桌面应用编译诊断结果"
```

---

### Task 4: SDK 编译诊断

- [ ] **Step 1: Python SDK 构建**

```bash
ls sdk/python/pyproject.toml
cd sdk/python && uv build 2>&1 | tee ../../reports/build_errors_sdk_python.txt
```

- [ ] **Step 2: TypeScript SDK 构建**

```bash
cd /Users/xujian/projects/BCIP/sdk/typescript
pnpm install 2>&1 | tail -3
pnpm build 2>&1 | tee ../../reports/build_errors_sdk_ts.txt
```

- [ ] **Step 3: Commit**

```bash
cd /Users/xujian/projects/BCIP
git add reports/build_errors_sdk_python.txt reports/build_errors_sdk_ts.txt
git commit -m "chore(audit): 阶段1 SDK 编译诊断结果"
```

---

### Task 5: 依赖完整性检查

- [ ] **Step 1: 运行 cargo-shear（检测未使用依赖）**

```bash
cargo shear 2>&1 | tee reports/shear_output.txt
```

若 cargo-shear 未安装：
```bash
cargo install cargo-shear
cargo shear 2>&1 | tee reports/shear_output.txt
```

- [ ] **Step 2: 运行 cargo-deny（安全/许可审计）**

```bash
cargo deny check 2>&1 | tee reports/deny_output.txt
```

若 cargo-deny 未安装：
```bash
cargo install cargo-deny
cargo deny check 2>&1 | tee reports/deny_output.txt
```

- [ ] **Step 3: Commit**

```bash
git add reports/shear_output.txt reports/deny_output.txt
git commit -m "chore(audit): 阶段1 依赖完整性检查结果"
```

---

### Task 6: Rust 全量测试诊断

**Files:**
- Create: `reports/test_failures_rust.txt`

- [ ] **Step 1: 运行全量 Rust 测试（使用 nextest）**

```bash
just test 2>&1 | tee reports/test_failures_rust_full.txt
```

此步骤可能耗时 20-40 分钟。若超时，逐个 crate 运行。

- [ ] **Step 2: 提取失败测试清单**

```bash
rg "^  (FAIL|TIMEOUT|ABORT)" reports/test_failures_rust_full.txt | sort > reports/test_failures_rust.txt
```

- [ ] **Step 3: 按 crate 分组统计**

```bash
rg "^  (FAIL|TIMEOUT)" reports/test_failures_rust_full.txt | rg -o "crate::\w+" | sort | uniq -c | sort -rn
```

- [ ] **Step 4: 区分阻断/非阻断失败**

阻断失败 = 测试 panic/hang 导致后续测试无法执行：
```bash
rg "ABORT|SIGSEGV|panic" reports/test_failures_rust_full.txt || echo "无阻断失败"
```

- [ ] **Step 5: 识别 sandbox skip（预期行为）**

```bash
rg "CODEX_SANDBOX" reports/test_failures_rust_full.txt -B2 -A2 | head -30
```

这些 skip 不算失败，标注为预期行为。

- [ ] **Step 6: Commit**

```bash
git add reports/test_failures_rust_full.txt reports/test_failures_rust.txt
git commit -m "chore(audit): 阶段2 Rust 全量测试诊断结果"
```

---

### Task 7: 专利核心 crate 专项测试诊断

- [ ] **Step 1: 逐个运行专利核心 crate 测试**

```bash
for crate in codex-patent-core codex-patent-domain codex-patent-tools codex-patent-agents codex-patent-skills codex-patent-knowledge codex-patent-constitutional codex-patent-text codex-patent-scheduler; do
  echo "===== $crate ====="
  just test -p "$crate" 2>&1 | tail -20
  echo ""
done
```

- [ ] **Step 2: 记录失败测试名称和数量**

```bash
for crate in codex-patent-core codex-patent-domain codex-patent-tools codex-patent-agents codex-patent-skills codex-patent-knowledge codex-patent-constitutional codex-patent-text codex-patent-scheduler; do
  echo "=== $crate ==="
  just test -p "$crate" 2>&1 | rg "FAIL|failures:" || echo "  PASS"
done | tee reports/test_failures_patent.txt
```

- [ ] **Step 3: Commit**

```bash
git add reports/test_failures_patent.txt
git commit -m "chore(audit): 阶段2 专利核心测试诊断结果"
```

---

### Task 8: TUI 快照测试诊断

- [ ] **Step 1: 运行 TUI 测试**

```bash
just test -p codex-tui 2>&1 | tee reports/test_failures_tui.txt
```

- [ ] **Step 2: 检查 insta snapshot 变更**

```bash
cargo insta pending-snapshots -p codex-tui 2>&1 | tee reports/insta_pending.txt
```

- [ ] **Step 3: 若存在 snapshot diff，检查变更内容**

```bash
cargo insta show -p codex-tui 2>&1 | head -50
```

- [ ] **Step 4: Commit**

```bash
git add reports/test_failures_tui.txt reports/insta_pending.txt 2>/dev/null
git commit -m "chore(audit): 阶段2 TUI 快照测试诊断结果"
```

---

### Task 9: SDK 和桌面测试诊断

- [ ] **Step 1: Python SDK 测试**

```bash
cd /Users/xujian/projects/BCIP/sdk/python
uv run pytest -v 2>&1 | tee ../../reports/test_failures_sdk_python.txt
```

- [ ] **Step 2: TypeScript SDK 测试**

```bash
cd /Users/xujian/projects/BCIP/sdk/typescript
pnpm test 2>&1 | tee ../../reports/test_failures_sdk_ts.txt
```

- [ ] **Step 3: 桌面 E2E 测试（Playwright）**

```bash
cd /Users/xujian/projects/BCIP/apps/desktop
npx playwright test --reporter=list 2>&1 | tee ../../reports/test_failures_desktop.txt
```

- [ ] **Step 4: Commit**

```bash
cd /Users/xujian/projects/BCIP
git add reports/test_failures_sdk_python.txt reports/test_failures_sdk_ts.txt reports/test_failures_desktop.txt 2>/dev/null
git commit -m "chore(audit): 阶段2 SDK 和桌面测试诊断结果"
```

---

### Task 10: Rust 模块引用完整性扫描

**Files:**
- Create: `reports/module_issues.txt`

- [ ] **Step 1: 检查未声明的模块引用（编译时 import 路径错误）**

```bash
cargo check --workspace 2>&1 | rg "unresolved import|cannot find module|maybe a missing crate" | sort > reports/module_issues.txt
```

- [ ] **Step 2: 检查 unused import**

```bash
cargo check --workspace 2>&1 | rg "unused import" | sort >> reports/module_issues.txt
```

- [ ] **Step 3: 扫描 codex-rs 下所有 lib.rs 和 mod.rs 的模块声明与文件对应关系**

```bash
find codex-rs -name "lib.rs" -o -name "mod.rs" | while read f; do
  echo "=== $f ==="
  rg "^pub mod |^mod " "$f" | sed 's/;.*//' | awk '{print $NF}'
done | tee reports/mod_declarations.txt
```

- [ ] **Step 4: 验证每个 mod 声明都有对应文件**

```bash
python3 -c "
import os, re, sys
issues = []
with open('reports/mod_declarations.txt') as f:
    current_file = None
    for line in f:
        line = line.strip()
        if line.startswith('=== '):
            current_file = line[4:-5]
        elif line and current_file:
            parent_dir = os.path.dirname(current_file)
            mod_name = line
            expected_file = os.path.join(parent_dir, f'{mod_name}.rs')
            expected_dir_file = os.path.join(parent_dir, mod_name, 'mod.rs')
            if not os.path.exists(expected_file) and not os.path.exists(expected_dir_file):
                print(f'MISSING: {current_file} declares mod {mod_name} but no {mod_name}.rs or {mod_name}/mod.rs found')
" | tee reports/module_file_issues.txt
```

- [ ] **Step 5: Commit**

```bash
git add reports/module_issues.txt reports/mod_declarations.txt reports/module_file_issues.txt 2>/dev/null
git commit -m "chore(audit): 阶段3 模块引用完整性扫描"
```

---

### Task 11: Cargo 依赖和配置引用完整性扫描

- [ ] **Step 1: 检查 Cargo.toml 中引用了但未在代码中使用的依赖**

```bash
cargo shear 2>&1 | tee reports/shear_detailed.txt
```

- [ ] **Step 2: 检查 Cargo.toml 中未声明但在代码中使用的路径依赖**

```bash
grep -rn "codex-" codex-rs/**/Cargo.toml | rg "path\s*=" | head -30
```

- [ ] **Step 3: 检查 ConfigToml 字段完整性 — 查找所有定义字段及其使用点**

```bash
cd codex-rs/core/src
rg "pub\s+\w+:\s*" config.rs 2>/dev/null | head -50
cd /Users/xujian/projects/BCIP
rg "config\.\w+" codex-rs/ --include "*.rs" -l | head -20
```

- [ ] **Step 4: Commit**

```bash
git add reports/shear_detailed.txt 2>/dev/null
git commit -m "chore(audit): 阶段3 Cargo 依赖和配置引用扫描"
```

---

### Task 12: Skill/Agent 定义与注册一致性扫描

- [ ] **Step 1: 扫描所有 Skill TOML 文件**

```bash
find codex-rs -name "*.toml" -path "*/codex-patent-skills/*" | sort
```

- [ ] **Step 2: 查找 Skill 加载/注册代码**

```bash
rg "register.*skill\|skill.*register\|load.*skill" codex-rs/ --include "*.rs" -l
```

- [ ] **Step 3: 对每个 Skill TOML，检查是否存在加载代码引用**

```bash
for skill_file in $(find codex-rs -name "*.toml" -path "*/codex-patent-skills/*"); do
  skill_name=$(basename "$skill_file" .toml | sed 's/-/_/g')
  found=$(rg -l "$skill_name" codex-rs/ --include "*.rs" 2>/dev/null | wc -l | tr -d ' ')
  if [ "$found" -eq "0" ]; then
    echo "ORPHAN: $skill_file has no Rust code reference"
  fi
done | tee reports/skill_registration_issues.txt
```

- [ ] **Step 4: 同样扫描 Agent TOML 文件**

```bash
find codex-rs -name "*.toml" -path "*/codex-patent-agents/*" | sort
for agent_file in $(find codex-rs -name "*.toml" -path "*/codex-patent-agents/*"); do
  agent_name=$(basename "$agent_file" .toml | sed 's/-/_/g')
  found=$(rg -l "$agent_name" codex-rs/ --include "*.rs" 2>/dev/null | wc -l | tr -d ' ')
  if [ "$found" -eq "0" ]; then
    echo "ORPHAN: $agent_file has no Rust code reference"
  fi
done | tee -a reports/skill_registration_issues.txt
```

- [ ] **Step 5: Commit**

```bash
git add reports/skill_registration_issues.txt 2>/dev/null
git commit -m "chore(audit): 阶段3 Skill/Agent 注册一致性扫描"
```

---

### Task 13: 文档内链有效性扫描

- [ ] **Step 1: 扫描 docs/ 目录下的所有 markdown 链接**

```bash
rg "\[.*\]\((.*)\)" docs/ --include "*.md" -n | rg -v "^.*http" | tee reports/doc_links.txt
```

- [ ] **Step 2: 以 Python 脚本验证每个文件内链指向的文件是否存在**

```bash
python3 -c "
import os, re

report = []
for root, dirs, files in os.walk('docs'):
    for f in files:
        if f.endswith('.md'):
            filepath = os.path.join(root, f)
            with open(filepath, 'r', encoding='utf-8') as fp:
                content = fp.read()
            links = re.findall(r'\[([^\]]*)\]\(([^\)]*)\)', content)
            for text, link in links:
                if link.startswith('http'):
                    continue
                if '#' in link:
                    link = link.split('#')[0]
                if not link:
                    continue
                target = os.path.normpath(os.path.join(os.path.dirname(filepath), link))
                if not os.path.exists(target) and not link.startswith('mailto:'):
                    report.append(f'{filepath}: LINK BROKEN [{text}]({link}) -> {target}')
for r in sorted(report):
    print(r)
" | tee reports/doc_broken_links.txt
```

- [ ] **Step 3: 同样扫描 AGENTS.md 和 CLAUDE.md 内链**

```bash
python3 -c "
import os, re

for f in ['AGENTS.md', 'CLAUDE.md']:
    if not os.path.exists(f):
        continue
    with open(f, 'r', encoding='utf-8') as fp:
        content = fp.read()
    links = re.findall(r'\[([^\]]*)\]\(([^\)]*)\)', content)
    for text, link in links:
        if link.startswith('http') or link.startswith('mailto:'):
            continue
        if '#' in link:
            link = link.split('#')[0]
        if not link:
            continue
        target = os.path.normpath(os.path.join(os.path.dirname(f), link))
        if not os.path.exists(target):
            print(f'{f}: LINK BROKEN [{text}]({link}) -> {target}')
" | tee reports/root_doc_broken_links.txt
```

- [ ] **Step 4: Commit**

```bash
git add reports/doc_links.txt reports/doc_broken_links.txt reports/root_doc_broken_links.txt 2>/dev/null
git commit -m "chore(audit): 阶段3 文档内链有效性扫描"
```

---

### Task 14: CI 工作流完整性扫描

- [ ] **Step 1: 列出所有 CI workflow 文件**

```bash
ls -la .github/workflows/
```

- [ ] **Step 2: 检查每个 workflow 的 job 依赖和 action 版本**

```bash
python3 -c "
import os, yaml

for f in sorted(os.listdir('.github/workflows/')):
    if not f.endswith('.yml') and not f.endswith('.yaml'):
        continue
    filepath = os.path.join('.github/workflows', f)
    with open(filepath, 'r') as fp:
        try:
            data = yaml.safe_load(fp)
        except:
            print(f'{f}: YAML parse error')
            continue
    if not data or not isinstance(data, dict):
        continue
    jobs = data.get('jobs', data.get(True, data.get('on', {})))
    if isinstance(jobs, dict):
        for job_name, job_def in jobs.items():
            needs = job_def.get('needs', []) if isinstance(job_def, dict) else []
            if needs:
                undefined = [n for n in (needs if isinstance(needs, list) else [needs]) if n not in jobs]
                if undefined:
                    print(f'{f}: job \"{job_name}\" needs undefined jobs: {undefined}')
            steps = job_def.get('steps', []) if isinstance(job_def, dict) else []
            for step in steps:
                uses = step.get('uses', '') if isinstance(step, dict) else ''
                if uses and uses.startswith('actions/') and '@' in uses:
                    pass  # standard action, OK
" 2>&1
```

- [ ] **Step 3: 检查 .bazelrc 中的 CI profile 引用**

```bash
rg "config:" .bazelrc
```

- [ ] **Step 4: Commit**

```bash
git add reports/ci_workflow_scan.txt 2>/dev/null
git commit -m "chore(audit): 阶段3 CI 工作流完整性扫描"
```

---

### Task 15: 生成断链汇总报告

- [ ] **Step 1: 聚合所有扫描结果到 JSON 报告**

```bash
python3 << 'PYEOF' > reports/dead_link_report.json
import json, os

report = []

def add(dtype, location, severity, suggestion):
    report.append({"type": dtype, "location": location, "severity": severity, "suggestion": suggestion})

# 模块引用问题
if os.path.exists('reports/module_issues.txt'):
    with open('reports/module_issues.txt') as f:
        for line in f:
            line = line.strip()
            if line:
                add("module", line, "P0", "修复 import 路径或模块声明")

if os.path.exists('reports/module_file_issues.txt'):
    with open('reports/module_file_issues.txt') as f:
        for line in f:
            line = line.strip()
            if line and line != 'MISSING':
                add("module", line, "P0", "创建缺失的模块文件或移除声明")

# 文档断链
if os.path.exists('reports/doc_broken_links.txt'):
    with open('reports/doc_broken_links.txt') as f:
        for line in f:
            line = line.strip()
            if line:
                add("doc", line, "P2", "修复或删除断链")

if os.path.exists('reports/root_doc_broken_links.txt'):
    with open('reports/root_doc_broken_links.txt') as f:
        for line in f:
            line = line.strip()
            if line:
                add("doc", line, "P2", "修复或删除断链")

# Skill/Agent 注册
if os.path.exists('reports/skill_registration_issues.txt'):
    with open('reports/skill_registration_issues.txt') as f:
        for line in f:
            line = line.strip()
            if line.startswith('ORPHAN'):
                add("skill", line, "P1", "注册或删除孤立的 TOML 定义")

# cargo-shear 未使用依赖
if os.path.exists('reports/shear_detailed.txt'):
    with open('reports/shear_detailed.txt') as f:
        for line in f:
            line = line.strip()
            if 'unused' in line.lower():
                add("dependency", line, "P2", "移除未使用的 Cargo 依赖")

print(json.dumps(report, indent=2, ensure_ascii=False))
PYEOF
```

- [ ] **Step 2: 统计各类问题数量**

```bash
python3 -c "
import json
with open('reports/dead_link_report.json') as f:
    data = json.load(f)
by_type = {}
by_severity = {}
for item in data:
    by_type[item['type']] = by_type.get(item['type'], 0) + 1
    by_severity[item['severity']] = by_severity.get(item['severity'], 0) + 1
print('按类型:', json.dumps(by_type, indent=2))
print('按严重度:', json.dumps(by_severity, indent=2))
print(f'总计: {len(data)} 个问题')
"
```

- [ ] **Step 3: Commit**

```bash
git add reports/dead_link_report.json
git commit -m "chore(audit): 阶段3 断链汇总报告"
```

---

### Task 16: P0 阻断修复 — 编译错误

> **前置条件:** Task 1-4 诊断结果已有

- [ ] **Step 1: 读取诊断报告，确认 P0 编译错误清单**

```bash
echo "=== Rust Compile Errors ==="
rg "^error\[" reports/build_errors_rust_check.txt | sort -u
echo ""
echo "=== Patent Crate Errors ==="
rg "^error\.*:" reports/build_errors_patent.txt | sort -u
```

- [ ] **Step 2: 优先修复核心 crate — codex-patent-core**

```bash
cargo check -p codex-patent-core 2>&1 | rg "^error\[" 
```

对于每个具体错误，按以下模式修复：
- `error[E0433]: failed to resolve: use of undeclared type` → 补 use 语句或类型定义
- `error[E0432]: unresolved import` → 修复 import 路径或 Cargo.toml 依赖
- `error[E0425]: cannot find value` → 补定义或修复引用
- `error[E0603]: module is private` → 添加 `pub` 修饰符或调整模块可见性

- [ ] **Step 3: 每个 crate 修复后立即验证**

```bash
cargo check -p <crate_name> 2>&1 | rg "^error" || echo "PASS"
```

- [ ] **Step 4: 全量编译验证**

```bash
cargo check --workspace --all-features 2>&1 | rg "^error" | wc -l
```

- [ ] **Step 5: Commit 每个修复（原子提交）**

```bash
git add <修复的文件> 
git commit -m "fix(audit): 修复 <crate> 编译错误 — <简要描述>"
```

---

### Task 17: P0 阻断修复 — 测试崩溃

> **前置条件:** Task 16 完成后编译通过

- [ ] **Step 1: 读取测试失败清单，筛选 PANIC/ABORT 类崩溃**

```bash
rg "PANIC|ABORT|SIGSEGV" reports/test_failures_rust_full.txt
```

- [ ] **Step 2: 逐个修复测试崩溃**

对于每个崩溃测试：
- 找到测试文件路径和函数名
- 修复 panic 的根因（空指针、unwrap 失败、缺文件等）
- 运行单个测试验证：`just test -p <crate> <test_name>`

- [ ] **Step 3: 排除 sandbox 环境导致的 skip（预期行为）**

识别并标注标记为 `CODEX_SANDBOX_NETWORK_DISABLED=1` 的 skip：
```bash
rg "CODEX_SANDBOX" codex-rs/ --include "*.rs" -l | head -10
```

这些 skip 是预期行为，记录到报告但不需要修复。

- [ ] **Step 4: 全量重跑确认崩溃清零**

```bash
just test 2>&1 | rg "ABORT|PANIC|SIGSEGV" | wc -l
# 期望输出: 0
```

- [ ] **Step 5: Commit**

```bash
git add <修复的文件>
git commit -m "fix(audit): 修复测试崩溃 — <crate>/<test_name>"
```

---

### Task 18: P1 功能修复 — 测试断言修复

> **前置条件:** P0 修复完成，无编译错误和测试崩溃

- [ ] **Step 1: 列出所有断言失败（非崩溃）**

```bash
rg "FAIL\s" reports/test_failures_rust_full.txt | rg -v "SKIP|ABORT|PANIC" | head -40
```

- [ ] **Step 2: 逐个分析并修复**

对每个 FAIL：
1. 查看测试代码：找到 `#[test]` 函数
2. 判断是测试过时还是代码逻辑错误：
   - 若测试过时（需求变更但测试未更新）→ 更新测试断言
   - 若代码逻辑错误 → 修复代码
3. 运行单个测试验证

- [ ] **Step 3: 修复类型不匹配的断言（常见模式）**

```rust
// 常见修复模式 1：字段新增 → 补字段
assert_eq!(actual, Expected { new_field: default_value, ..original });

// 常见修复模式 2：枚举变体新增 → 补分支
match result {
    ExistingVariant => { /* ... */ }
    NewVariant => { /* 补处理逻辑 */ }
}

// 常见修复模式 3：返回值类型变更 → 调整断言
let result: NewType = function();
assert_eq!(result, expected_new_type);
```

- [ ] **Step 4: 逐个 crate 验证修复**

```bash
just test -p <crate> 2>&1 | tail -5
```

- [ ] **Step 5: Commit**

```bash
git add <修复的文件>
git commit -m "fix(audit): 修复测试断言 — <crate>: <简要描述>"
```

---

### Task 19: P1 功能修复 — TUI 快照更新

> **前置条件:** codex-tui 测试中的 snapshot test 失败

- [ ] **Step 1: 查看 pending snapshots**

```bash
cargo insta pending-snapshots -p codex-tui
```

- [ ] **Step 2: 逐个审查 snapshot diff**

```bash
cargo insta show -p codex-tui
```

按以下规则判断：
- 若 diff 是预期内的 UI 变更（如文案调整、颜色修改）→ accept
- 若 diff 显示了意外的 UI 退化（如布局错乱、文字丢失）→ 修复代码而非 accept

- [ ] **Step 3: 接受预期内的 snapshot 变更**

```bash
cargo insta accept -p codex-tui
```

- [ ] **Step 4: 重新运行 TUI 测试确认通过**

```bash
just test -p codex-tui 2>&1 | tail -5
# 期望输出: 0 failures
```

- [ ] **Step 5: Commit**

```bash
git add codex-rs/tui/src/**/snapshots/
git commit -m "test(audit): 更新 TUI insta snapshots"
```

---

### Task 20: P2 端到端测试补全 — 专利检索链路

**Files:**
- Create: `codex-rs/codex-patent-domain/tests/e2e_search.rs`

> 按 `core_test_support::responses` 模式编写

- [ ] **Step 1: 检查现有测试模式**

```bash
find codex-rs/codex-patent-domain/tests -name "e2e*.rs" -o -name "*test*.rs" | head -10
```

阅读一个已有的 E2E 测试文件作为模板。

- [ ] **Step 2: 编写检索链路 E2E 测试**

```rust
use core_test_support::responses;
use codex_core::Op;

#[tokio::test]
async fn e2e_patent_search_keyword() {
    let server = responses::start_server().await;
    
    // Mock 关键词检索响应
    let mock = responses::mount_sse_once(&server, responses::sse(vec![
        responses::ev_response_created("resp-search-1"),
        responses::ev_function_call(
            "call-search-1",
            "patent_keyword_search",
            r#"{"keywords": "深度学习 图像识别", "limit": 10}"#,
        ),
        responses::ev_function_call_output(
            "call-search-1",
            r#"{"results": [{"patent_id": "CN202310000001", "title": "基于深度学习的图像识别方法", "applicant": "华为"}]}"#,
        ),
        responses::ev_completed("resp-search-1"),
    ])).await;

    let result = mock.single_request();
    let body = result.body_json();
    assert!(body["input"].as_str().unwrap().contains("深度学习"));
}
```

- [ ] **Step 3: 运行测试确认通过**

```bash
just test -p codex-patent-domain -- e2e_patent_search_keyword
```

- [ ] **Step 4: 补充语义检索变体测试**

在同一个测试文件中添加 `e2e_patent_search_semantic` 测试。

- [ ] **Step 5: Commit**

```bash
git add codex-rs/codex-patent-domain/tests/e2e_search.rs
git commit -m "test(audit): 补全专利检索链路 E2E 测试"
```

---

### Task 21: P2 端到端测试补全 — 专利分析链路

**Files:**
- Create: `codex-rs/codex-patent-domain/tests/e2e_analysis.rs`

- [ ] **Step 1: 编写分析链路 E2E 测试**

```rust
use core_test_support::responses;
use codex_core::Op;

#[tokio::test]
async fn e2e_patent_novelty_analysis() {
    let server = responses::start_server().await;
    
    let mock = responses::mount_sse_once(&server, responses::sse(vec![
        responses::ev_response_created("resp-analysis-1"),
        responses::ev_function_call(
            "call-analyze-1",
            "patent_novelty_check",
            r#"{"patent_text": "一种基于Transformer的图像分割方法...", "prior_art": "现有技术对比文本"}"#,
        ),
        responses::ev_function_call_output(
            "call-analyze-1",
            r#"{"novel": true, "confidence": 0.85, "analysis": "具有新颖性，与现有技术存在显著区别"}"#,
        ),
        responses::ev_completed("resp-analysis-1"),
    ])).await;

    let result = mock.single_request();
    let body = result.body_json();
    assert!(body["input"].is_string());
}
```

- [ ] **Step 2: 补充创造性分析变体**

添加 `e2e_patent_creativity_analysis` 测试。

- [ ] **Step 3: 运行测试确认**

```bash
just test -p codex-patent-domain -- e2e_patent
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/codex-patent-domain/tests/e2e_analysis.rs
git commit -m "test(audit): 补全专利分析链路 E2E 测试"
```

---

### Task 22: P2 端到端测试补全 — 专利撰写链路

**Files:**
- Create: `codex-rs/codex-patent-domain/tests/e2e_drafting.rs`

- [ ] **Step 1: 编写撰写链路 E2E 测试**

```rust
use core_test_support::responses;

#[tokio::test]
async fn e2e_patent_drafting_claims() {
    let server = responses::start_server().await;
    
    let mock = responses::mount_sse_once(&server, responses::sse(vec![
        responses::ev_response_created("resp-draft-1"),
        responses::ev_function_call(
            "call-draft-1",
            "patent_claim_drafting",
            r#"{"disclosure": "一种基于注意力机制的目标检测方法...", "patent_type": "invention"}"#,
        ),
        responses::ev_function_call_output(
            "call-draft-1",
            r#"{"claims": [{"num": 1, "type": "independent", "text": "一种目标检测方法，其特征在于..."}]}"#,
        ),
        responses::ev_completed("resp-draft-1"),
    ])).await;

    let result = mock.single_request();
    assert!(result.body_json()["input"].is_string());
}
```

- [ ] **Step 2: 补充说明书生成变体**

添加 `e2e_patent_specification_drafting` 测试。

- [ ] **Step 3: 运行测试确认**

```bash
just test -p codex-patent-domain -- e2e_patent_drafting
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/codex-patent-domain/tests/e2e_drafting.rs
git commit -m "test(audit): 补全专利撰写链路 E2E 测试"
```

---

### Task 23: P2 端到端测试补全 — 专利审查链路

**Files:**
- Create: `codex-rs/codex-patent-domain/tests/e2e_examination.rs`

- [ ] **Step 1: 编写审查链路 E2E 测试**

```rust
use core_test_support::responses;

#[tokio::test]
async fn e2e_patent_oa_response() {
    let server = responses::start_server().await;
    
    let mock = responses::mount_sse_once(&server, responses::sse(vec![
        responses::ev_response_created("resp-oa-1"),
        responses::ev_function_call(
            "call-oa-1",
            "patent_oa_response",
            r#"{"office_action": "审查意见通知书...权利要求1不具备创造性...", "patent_id": "CN202310000001"}"#,
        ),
        responses::ev_function_call_output(
            "call-oa-1",
            r#"{"response": "针对审查意见的答复如下...修改后的权利要求1...", "amended_claims": [...]}"#,
        ),
        responses::ev_completed("resp-oa-1"),
    ])).await;

    let result = mock.single_request();
    assert!(result.body_json()["input"].is_string());
}
```

- [ ] **Step 2: 补充审查员模拟变体**

添加 `e2e_patent_examiner_simulation` 测试。

- [ ] **Step 3: 运行测试确认**

```bash
just test -p codex-patent-domain -- e2e_patent_oa
```

- [ ] **Step 4: Commit**

```bash
git add codex-rs/codex-patent-domain/tests/e2e_examination.rs
git commit -m "test(audit): 补全专利审查链路 E2E 测试"
```

---

### Task 24: P3 文档断链修复

> **前置条件:** Task 13 的 `reports/doc_broken_links.txt` 已有结果

- [ ] **Step 1: 读取断链清单**

```bash
cat reports/doc_broken_links.txt
cat reports/root_doc_broken_links.txt
```

- [ ] **Step 2: 逐条修复**

对每条断链：
- 若目标文件存在但路径错误 → 修正路径
- 若目标文件不存在 → 删除该链接或改为纯文本
- 若锚点不存在 → 修正或移除锚点

- [ ] **Step 3: 重新运行文档扫描确认清零**

```bash
python3 -c "
import os, re
found = 0
for root, dirs, files in os.walk('docs'):
    for f in files:
        if f.endswith('.md'):
            filepath = os.path.join(root, f)
            with open(filepath, 'r') as fp:
                content = fp.read()
            links = re.findall(r'\[([^\]]*)\]\(([^\)]*)\)', content)
            for text, link in links:
                if link.startswith('http') or link.startswith('mailto:') or not link:
                    continue
                clean_link = link.split('#')[0]
                if not clean_link:
                    continue
                target = os.path.normpath(os.path.join(os.path.dirname(filepath), clean_link))
                if not os.path.exists(target):
                    print(f'BROKEN: {filepath} -> [{text}]({link})')
                    found += 1
print(f'Total broken links: {found}')
"
# 期望输出: Total broken links: 0
```

- [ ] **Step 4: Commit**

```bash
git add docs/
git commit -m "fix(audit): 修复文档断链"
```

---

### Task 25: 最终验证和审计报告

**Files:**
- Create: `reports/QUALITY_AUDIT_REPORT.md`

- [ ] **Step 1: 全量编译验证**

```bash
cargo build --workspace --all-features 2>&1 | rg "^error" | wc -l
# 期望: 0
```

- [ ] **Step 2: 全量测试验证**

```bash
just test 2>&1 | tail -20
# 期望: 0 failures (排除 sandbox skip)
```

- [ ] **Step 3: SDK 测试验证**

```bash
cd sdk/python && uv run pytest -q 2>&1 | tail -3
cd /Users/xujian/projects/BCIP/sdk/typescript && pnpm test 2>&1 | tail -5
```

- [ ] **Step 4: 桌面测试验证**

```bash
cd /Users/xujian/projects/BCIP/apps/desktop
npx playwright test --reporter=dot 2>&1 | tail -5
```

- [ ] **Step 5: 生成最终审计报告**

```bash
cat > reports/QUALITY_AUDIT_REPORT.md << 'EOF'
# BCIP 质量审计报告

**日期:** 2026-05-31
**目标:** 项目完全可用，无断链，端到端测试全部通过

## 编译状态

| 模块 | Rust | 桌面 | Python SDK | TS SDK |
|------|------|------|-----------|--------|
| 编译错误 | (待填充) | (待填充) | (待填充) | (待填充) |

## 测试状态

| 测试套件 | 总数 | 通过 | 失败 | 跳过 |
|---------|------|------|------|------|
| Rust | (待填充) | (待填充) | (待填充) | (待填充) |
| Python SDK | (待填充) | (待填充) | (待填充) | (待填充) |
| TS SDK | (待填充) | (待填充) | (待填充) | (待填充) |
| 桌面 E2E | (待填充) | (待填充) | (待填充) | (待填充) |

## 断链状态

| 类型 | 数量 | 已修复 |
|------|------|--------|
| 模块引用 | (待填充) | (待填充) |
| 依赖 | (待填充) | (待填充) |
| 文档 | (待填充) | (待填充) |
| CI | (待填充) | (待填充) |

## 端到端测试覆盖

| 链路 | 状态 | 测试文件 |
|------|------|---------|
| 专利检索 | (待填充) | tests/e2e_search.rs |
| 专利分析 | (待填充) | tests/e2e_analysis.rs |
| 专利撰写 | (待填充) | tests/e2e_drafting.rs |
| 专利审查 | (待填充) | tests/e2e_examination.rs |

## 遗留问题

(待填充)
EOF
```

- [ ] **Step 6: 填充报告数据并 Commit**

```bash
# 填充编译状态
echo -n "Rust:" $(cargo build --workspace --all-features 2>&1 | rg "^error" | wc -l | tr -d ' ')

# 填充测试状态  
RUST_TOTAL=$(rg "^  (PASS|FAIL|SKIP)" reports/test_failures_rust_full.txt 2>/dev/null | wc -l | tr -d ' ')
echo "Rust total: $RUST_TOTAL"
```

- [ ] **Step 7: Final commit**

```bash
git add reports/QUALITY_AUDIT_REPORT.md
git commit -m "docs(audit): 最终质量审计报告"
```

---

## 附录：常见修复模式速查

### Rust 编译错误

| 错误代码 | 常见原因 | 修复方式 |
|---------|---------|---------|
| E0432 | import 路径错误 | 修正 use 语句或 Cargo.toml 依赖 |
| E0433 | 未声明类型/模块 | 补 use 语句或模块声明 |
| E0425 | 未找到变量/函数 | 补定义或修正名称 |
| E0603 | 模块/类型为私有 | 添加 pub 修饰符 |
| E0277 | trait bound 不满足 | 实现缺失的 trait 或修改泛型约束 |
| E0599 | 方法不存在 | 补方法实现或 use 对应 trait |
| E0308 | 类型不匹配 | 修改类型或添加转换 |

### Cargo 依赖问题

| 问题 | 修复方式 |
|------|---------|
| unused dependency | 从 Cargo.toml 移除 |
| 缺失的 path 依赖 | 在 workspace Cargo.toml 中声明成员 |
| feature 缺失 | 添加 feature 或启用 `default-features = false` |
