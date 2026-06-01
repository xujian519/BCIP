# Web Search Integration Design

## Goal

为 BCIP TUI 和桌面端接入稳定的网页搜索能力，支持 Agent 自动调用和用户手动搜索。首选 AnySearch，支持用户配置 API key（留空则匿名访问），架构预留多引擎扩展。

## Requirements

1. Agent 执行专利任务时可自动调用搜索（查现有技术、法律条文等）
2. 用户可在 TUI/桌面端手动触发搜索
3. 用户可在 config.toml 中选择搜索引擎和配置 API key
4. 首先实现 AnySearch provider（匿名可用、有 API key 时更快）
5. 架构预留 Tavily、EXA 等引擎的扩展能力

## Architecture Overview

```
config.toml [web_search]
       │
       ▼
codex-web-search (新 crate)
  ├── SearchProvider trait
  ├── AnySearchProvider (HTTP → api.anysearch.com)
  └── (未来: TavilyProvider, ExaProvider)
       │
   ┌───┴────┐
   ▼        ▼
  TUI     Desktop
  Agent   User UI
```

## Configuration

在 `~/.bcip/config.toml` 新增 section：

```toml
[web_search]
provider = "anysearch"    # "anysearch" (默认), 未来 "tavily", "exa"
api_key = ""              # 留空匿名访问，有值走认证模式
max_results = 10          # 默认最大结果数 (1-100)
zone = "cn"               # "cn" 或 "intl"
```

配置读取复用 `codex-config` crate 的 `ConfigToml` 体系。与现有 `web_search_mode`（宿主搜索/模型端搜索）独立，不冲突。

## Crate Structure

```
codex-rs/web-search/
├── Cargo.toml
└── src/
    ├── lib.rs           # pub mod + re-exports
    ├── provider.rs      # SearchProvider trait
    ├── anysearch.rs     # AnySearchProvider 实现
    ├── types.rs         # SearchResult, ExtractResult, SearchQuery 等
    └── error.rs         # WebSearchError
```

## Core Types

### SearchProvider Trait

使用 RPITIT（不用 `#[async_trait]`），遵循 AGENTS.md 规范：

```rust
trait SearchProvider: Send + Sync {
    fn search(&self, query: SearchQuery) -> impl Future<Output = Result<Vec<SearchResult>, WebSearchError>> + Send;
    fn extract(&self, url: &str) -> impl Future<Output = Result<ExtractResult, WebSearchError>> + Send;
    fn batch_search(&self, queries: Vec<SearchQuery>) -> impl Future<Output = Result<Vec<Vec<SearchResult>>, WebSearchError>> + Send;
}
```

### SearchQuery

```rust
struct SearchQuery {
    query: String,
    domain: Option<String>,       // 垂直领域: "academic", "legal", "finance" 等
    sub_domain: Option<String>,   // 子领域: "academic.general"
    max_results: Option<u32>,     // 1-100, 默认 10
    freshness: Option<Freshness>, // Day/Week/Month/Year
    zone: Option<Zone>,           // Cn/Intl
}

enum Freshness { Day, Week, Month, Year }
enum Zone { Cn, Intl }
```

### SearchResult

```rust
struct SearchResult {
    title: String,
    url: String,
    content: String,
    score: f64,
}
```

### ExtractResult

```rust
struct ExtractResult {
    url: String,
    content: String,  // Markdown 格式
    length: usize,
}
```

### WebSearchError

```rust
enum WebSearchError {
    Http(reqwest::Error),
    Api { code: i32, message: String },
    RateLimited { retry_after: Option<u64> },
    InvalidConfig(String),
}
```

## AnySearch Provider Implementation

### API Endpoints

| 功能 | Method | URL |
|------|--------|-----|
| 搜索 | POST | `https://api.anysearch.com/v1/search` |
| 批量搜索 | POST | `https://api.anysearch.com/v1/batch_search` |
| 内容提取 | POST | 需确认（匿名返回 404，可能需要 key 或不同路径） |

### Auth Behavior

| 场景 | 行为 |
|------|------|
| 无 key | 匿名访问，HTTP 200，速度 2-8s |
| 有 key | `Authorization: Bearer <key>` header，速度更快，限额更高 |
| Key 耗尽 | API 返回错误，提示用户配置新 key |

### 实测数据（匿名，2026-06-01）

| 测试 | 结果 |
|------|------|
| 通用搜索 | HTTP 200，搜索时间 2-8s |
| 垂直搜索（学术） | HTTP 200，搜索时间 ~6s |
| extract | `/v1/extract` 返回 404，需确认正确端点 |
| 返回格式 | JSON，含 title/url/content/score/quality_score |

## Tool Registration

在 `codex-patent-tools` 注册三个工具函数：

1. **`web_search(query, domain?, max_results?, freshness?)`** — 单次搜索
2. **`web_extract(url)`** — URL 内容提取
3. **`web_batch_search(queries)`** — 批量并行搜索（2-5 个）

Agent 通过工具调用链：判断需要搜索 → 调用工具 → 工具从 session config 读取 provider/api_key → 调用 SearchProvider → 返回结构化结果。

## Integration Points

| 层级 | 位置 | 改动 |
|------|------|------|
| 配置 | `codex-config` config_toml.rs | 新增 `[web_search]` section 解析 |
| 核心 | 新建 `codex-web-search` crate | Provider trait + AnySearch 实现 |
| 工具 | `codex-patent-tools` | 注册 3 个搜索工具函数 |
| TUI | 复用现有 WebSearch HistoryCell | 可能调整结果显示格式 |
| 桌面 | `app-server-protocol` v2 | 新增搜索 RPC（如 `search/query`） |
| 桌面前端 | `apps/desktop/src` | 搜索触发 UI（slash command 或按钮） |

## Phasing

### Phase 1 — 核心能力（MVP）
- 新建 `codex-web-search` crate，实现 `SearchProvider` trait 和 `AnySearchProvider`
- `codex-config` 新增 `[web_search]` 配置解析
- `codex-patent-tools` 注册搜索工具函数
- TUI Agent 可通过工具调用搜索

### Phase 2 — 桌面端集成
- `app-server-protocol` v2 新增搜索 RPC（`search/query`、`search/extract`）
- 桌面前端添加 `/search` slash command 和搜索结果展示组件
- 用户可手动触发搜索

### Phase 3 — 扩展（按需）
- 新增 `TavilyProvider` / `ExaProvider`
- 桌面端设置页面可视化配置搜索引擎和 API key

## Open Issues

1. **Extract 端点**：匿名模式下 `/v1/extract` 返回 404，需确认正确路径。实施时用 API key 测试，或查阅 MCP server 文档确认端点
2. **匿名额度**：文档未明确匿名配额数字，实施时在代码中加日志监控，长期观察后补充文档
