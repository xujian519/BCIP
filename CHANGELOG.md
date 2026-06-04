# Changelog

All notable changes to BCIP will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Releases are also available on the [GitHub releases page](https://github.com/xujian519/BCIP/releases).

## [Unreleased]

### Fixed

- **Agent Bus 竞态条件**: `send()` 方法现使用 `write().await` 替代 `try_write()`，确保消息历史不丢失
- **死信队列静默丢弃**: `record_dead_letter()` 改为 async，高并发下不再丢失失败消息
- **Registry CAS 活锁**: `try_increment_spawned` 添加 `spin_loop_hint()` 和 64 次重试上限
- **Graph Executor 并行度上界**: `max_parallel` 添加上限 32，防止极端值导致资源耗尽
- **Embedding Client 时间计算**: 提取 `epoch_secs()` 函数，统一时间获取逻辑
- **Google Patents 时间计算**: 同上修复 `unwrap_or_default()` 精度丢失

### Changed

- **HTTP 连接池**: `google_patents.rs` 和 `download_patent` 共享 `OnceLock<reqwest::Client>`，避免每次请求新建 client
- **LRU 缓存淘汰策略**: `EmbeddingClient` 缓存满时改为批量淘汰 100 条旧条目，不再全量清空
- **向量搜索 Top-K 优化**: `VectorIndex::search()` 改用 min-heap 实现 O(n·log k) 选择，预计算向量范数
- **Google Patents regex 缓存**: 所有 HTML 解析 regex 使用 `OnceLock` 编译一次
- **CJK 分词器优化**: 使用 `std::mem::take` 避免中间 clone，用 `&str` slice 替代 `ch.to_string()`
- **法律数据库 prepared statement 缓存**: `law_db.rs` 使用 `prepare_cached` 替代 `prepare`

### Added

- **CHANGELOG.md**: 从占位符重写为完整变更日志
- **docs/config.md**: 填充完整配置参考文档
- **CONTRIBUTING.md**: 新建贡献指南
- **docs/testing.md**: 新建测试策略文档
- **docs/desktop-user-guide.md**: 新建桌面端用户指南

## [0.1.0] - 2026-06-01

### Added

- 专利全生命周期智能体系统（9 个 Agent 角色）
- 50+ 专利工具函数（搜索、分析、撰写、OA 答复、质量检查）
- 知识图谱 + 法律数据库 + 知识卡片统一搜索
- CJK 分词器 + 多度量相似度引擎
- 35 条专利法约束规则引擎
- DAG 工作流执行引擎
- TUI 多 Agent 并发界面
- 协议与通信模块（心跳/序列化/传输抽象）
- 桌面端应用（Tauri）
