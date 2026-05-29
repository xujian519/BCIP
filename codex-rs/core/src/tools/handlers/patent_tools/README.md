# 专利工具集成说明

## 概述

本目录包含专利工具与 BCIP Tool 系统的桥接适配器，将 `codex-patent-tools` 中定义的专利工具注册到 BCIP 的工具注册表中。

## 功能特性

### 当前集成的工具

- **PatentSearch**: 使用本地专利数据库搜索专利（7500万+ 中国专利），毫秒级响应
- **GooglePatentsFetch**: 从 Google Patents 获取详细专利文档
- **SearchQueryBuilder**: 使用自然语言构建优化的专利检索查询
- **IterativeSearch**: 执行迭代式专利搜索，支持反馈驱动的查询优化
- **PatentDownload**: 从 Google Patents 下载专利 PDF 文档

### 架构设计

```
codex-patent-tools (独立 crate)
    ↓ (函数指针 + JSON Schema)
codex-patent-integration (核心内部模块)
    ↓ (实现 ToolExecutor + CoreToolRuntime)
BCIP Tool Registry (系统注册表)
    ↓ (模型可见工具列表)
AI 模型调用
```

## 使用方法

### 1. 启用专利工具功能

在编译时启用 `patent-tools` feature：

```bash
# 在项目根目录
cargo build --features patent-tools
```

或在 `Cargo.toml` 中启用：

```toml
[dependencies]
codex-core = { path = "core", features = ["patent-tools"] }
```

### 2. 验证工具注册

```rust
// 检查已注册的专利工具
let adapters = PatentToolAdapter::create_all_adapters();
for adapter in &adapters {
    println!("Registered: {}", adapter.tool_name().name);
}
```

### 3. 在代码中使用

专利工具会自动注册到 BCIP 工具系统中，模型可以直接调用：

```markdown
# 示例：搜索专利

用户：搜索关于"人工智能在医疗诊断"的中国专利

模型：
<function_calls>
<invoke name="PatentSearch">
<parameter name="query">人工智能 医疗诊断</parameter>
<parameter name="limit">10</parameter>
</invoke>
</function_calls>
```

## 工具规范

### PatentSearch

**描述**: 使用本地专利数据库搜索专利（7500万+ 中国专利），毫秒级响应

**参数**:
- `query` (string, required): 检索查询（关键词、申请人名称或分类号）
- `limit` (integer, optional): 返回的最大结果数

**示例**:
```json
{
  "query": "人工智能 医疗诊断",
  "limit": 10
}
```

### GooglePatentsFetch

**描述**: 从 Google Patents 获取详细专利文档

**参数**:
- `patent_number` (string, required): 专利号（如 CN101234567A, US1234567）
- `jurisdiction` (string, optional): 专利管辖地（CN, US, EP, WO）

**示例**:
```json
{
  "patent_number": "CN101234567A",
  "jurisdiction": "CN"
}
```

### SearchQueryBuilder

**描述**: 使用自然语言构建优化的专利检索查询

**参数**:
- `description` (string, required): 检索意图的自然语言描述
- `field` (string, optional): 特定搜索字段（标题、摘要、权利要求、申请人）

**示例**:
```json
{
  "description": "搜索关于深度学习在图像识别领域的专利",
  "field": "title"
}
```

### IterativeSearch

**描述**: 执行迭代式专利搜索，支持反馈驱动的查询优化

**参数**:
- `query` (string, required): 初始检索查询
- `max_iterations` (integer, optional): 最大迭代次数

**示例**:
```json
{
  "query": "量子计算",
  "max_iterations": 5
}
```

### PatentDownload

**描述**: 从 Google Patents 下载专利 PDF 文档

**参数**:
- `patent_number` (string, required): 要下载的专利号
- `format` (string, optional): 下载格式（pdf, txt）

**示例**:
```json
{
  "patent_number": "CN101234567A",
  "format": "pdf"
}
```

## 实现细节

### PatentToolAdapter

`PatentToolAdapter` 是核心桥接组件，它：

1. **包装专利工具函数指针**: 将 `codex-patent-tools` 中的函数指针包装为 BCIP 工具处理器
2. **生成工具规范**: 为每个工具生成符合 OpenAI 函数调用规范的 JSON Schema
3. **实现核心 trait**: 实现 `ToolExecutor<ToolInvocation>` 和 `CoreToolRuntime`
4. **处理函数调用**: 解析模型调用参数、执行工具处理函数、返回格式化结果

### 注册流程

1. `add_patent_tools()` 在 `spec_plan.rs` 中被调用
2. 调用 `PatentToolAdapter::create_all_adapters()` 创建所有适配器
3. 通过 `planned_tools.add_arc()` 将适配器注册到工具计划
4. 工具最终通过 `ToolRegistry` 注册到系统中

### 依赖隔离

通过 feature flag (`patent-tools`) 隔离专利工具依赖，避免在不使用专利工具时引入额外依赖：

```toml
[features]
patent-tools = ["codex-patent-tools"]
```

## 扩展指南

### 添加新的专利工具

1. 在 `codex-patent-tools/src/` 中实现工具逻辑
2. 在对应的 `register_*_tools()` 函数中注册工具
3. 在 `PatentToolAdapter::create_spec_for_tool()` 中添加工具规范
4. 重新编译以生成新的适配器

### 修改工具规范

工具规范定义在 `create_*_schema()` 函数中：

```rust
fn create_patent_search_schema() -> JsonSchema {
    JsonSchema {
        r#type: Some(JsonSchemaType::Object),
        description: Some("Patent search parameters".to_string()),
        properties: Some(vec![
            ("query".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("...".to_string()),
                ..Default::default()
            }),
            // ... 其他参数
        ].into_iter().collect()),
        required: Some(vec!["query".to_string()]),
        additional_properties: Some(AdditionalProperties::Bool(false)),
        ..Default::default()
    }
}
```

## 故障排查

### 编译错误

**错误**: `cyclic package dependency`
**原因**: 创建了独立的 `codex-patent-integration` crate 导致循环依赖
**解决**: 使用 core 内部模块而非独立 crate

### 工具未注册

**问题**: 启用 feature 后工具仍然不可用
**检查**:
1. 确认 feature 启用：`cargo build --features patent-tools`
2. 检查 `add_patent_tools()` 是否在 `spec_plan.rs` 中被调用
3. 验证工具列表：`PatentToolAdapter::create_all_adapters()`

### 函数调用失败

**问题**: 模型调用工具时返回错误
**检查**:
1. 参数解析：检查 JSON 参数是否符合工具规范
2. 处理函数：验证 `codex-patent-tools` 中的处理函数是否正确
3. 结果格式化：确认返回的 JSON 结构符合模型期望

## 性能考虑

- **本地搜索**: PatentSearch 使用本地数据库，毫秒级响应
- **网络请求**: GooglePatentsFetch 和 PatentDownload 需要网络访问
- **异步执行**: 所有工具调用都是异步的，支持并行调用
- **资源隔离**: 专利工具通过 feature flag 隔离，不影响默认构建

## 参考文档

- [BCIP 工具系统文档](../../tools/src/lib.rs)
- [专利工具实现](../../codex-patent-tools/)
- [工具注册规范](../spec_plan.rs)
- [核心 trait 定义](../registry.rs)

## 许可证

Apache-2.0