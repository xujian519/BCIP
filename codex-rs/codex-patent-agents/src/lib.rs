//! BCIP 专利智能体系统 - Agent 层
//!
//! 提供专利领域的专业 Agent 运行时、角色定义、知识上下文注入、
//! 反射引擎、学习闭环和多 LLM provider 路由。
//!
//! ## 核心能力
//!
//! - **Agent Runtime**: 独立的 agent 执行环境，支持线程调度
//! - **角色系统**: 9 个预定义专利专业角色（检索/分析/撰写/新颖性/创造性/侵权/无效/审查/质量）
//! - **知识上下文**: 根据角色和任务自动注入知识图谱和法律知识
//! - **反射引擎**: Agent 输出自动质量审查
//! - **学习闭环**: 反馈记录、统计分析和模型推荐
//! - **Provider 路由**: 支持 DeepSeek/Qwen/Moonshot/GLM/OpenAI/Anthropic

pub mod agent_manifest;
pub mod agent_runtime;
pub mod bcip_roles;
pub mod knowledge_context;
pub mod learning;
pub mod provider_router;
pub mod reflection;
pub mod roles;
pub mod scenario;

pub use agent_manifest::AgentManifest;
pub use agent_manifest::agent_store_dir;
pub use agent_manifest::iso8601_now;
pub use agent_manifest::list_agent_manifests;
pub use agent_manifest::load_manifest;
pub use agent_manifest::make_agent_id;
pub use agent_manifest::persist_manifest;
pub use agent_runtime::AgentSpawnInput;
pub use agent_runtime::PatentAgentRuntime;
pub use bcip_roles::config_file_contents;
pub use bcip_roles::patent_agent_role_configs;
pub use codex_patent_core::ApiKeyError;
pub use knowledge_context::AutoKnowledgeConfig;
pub use knowledge_context::KnowledgeContext;
pub use knowledge_context::RoleKeywords;
pub use learning::FeedbackData;
pub use learning::LearningStats;
pub use learning::LearningStore;
pub use learning::record_agent_feedback;
pub use provider_router::AgentProvider;
pub use provider_router::detect_provider;
pub use provider_router::mask_api_key;
pub use provider_router::resolve_api_key;
pub use provider_router::resolve_api_key_from_config;
pub use provider_router::resolve_provider_api_key;
pub use reflection::QualityIssue;
pub use reflection::ReflectionEngine;
pub use reflection::ReflectionResult;
pub use reflection::reflect_agent_result;
pub use roles::AgentRegistry;
pub use roles::AgentRoleConfig;
pub use roles::PatentAgentRole;
pub use scenario::ScenarioRegistry;
pub use scenario::ScenarioRule;
