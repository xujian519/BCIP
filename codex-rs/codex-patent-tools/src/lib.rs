//! # codex-patent-tools
//!
//! 专利智能体工具集。包含检索、撰写、分析、质检、评估、法律咨询等
//! 50+ 工具函数，按域（[`ToolDomain`]）分组成 13 个模块。
//!
//! 核心入口：
//! - [`register_all_tools`] / [`register_all_tools_with_domains`] — 全局注册
//! - [`filter_tools_by_role`] — 按角色域过滤工具
//!
//! ## 工具域
//!
//! | 模块 | 域 | 功能 |
//! |------|-----|------|
//! | `search_tools` | Search | 专利检索 |
//! | `drafting_tools` | Drafting | 权利要求/说明书撰写 |
//! | `analysis_tools` | Analysis | 新颖性/创造性/实用性分析 |
//! | `quality_tools` | Quality | 质量检查 |
//! | `oa_tools` | Oa | 审查意见答复 |
//! | `legal_tools` | Legal | 法律咨询 |
//! | `document_tools` | Document | 文档处理 |
//! | `management_tools` | Management | 案件管理 |
//! | `review_tools` | Review | 评审 |
//! | `evaluation_tools` | Evaluation | 评估 |
//! | `council_tools` | Council | 无效/复审 |
//! | `simulator_tools` | Simulator | 审查员模拟 |
//! | `web_search_tools` | WebSearch | 互联网搜索 |

pub mod advanced_analysis;
pub mod analysis_tools;
pub mod common;
pub mod council_tools;
pub mod document_tools;
pub mod drafting_tools;
pub mod evaluation_tools;
pub mod google_patents;
pub mod legal_tools;
pub mod management_tools;
pub mod oa_tools;
pub mod patent_document;
pub mod patent_search;
pub mod quality_tools;
pub mod review_tools;
pub mod search_tools;
pub mod simulator_tools;
pub mod web_search_tools;

pub use document_tools::register_document_tools;
pub use search_tools::register_search_tools;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub use codex_patent_core::ToolDomain;

pub type ToolHandler =
    fn(
        serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send>>;

/// 带域分类的工具元数据。
pub struct ToolMeta {
    pub domain: ToolDomain,
    pub handler: ToolHandler,
}

/// 按角色域过滤工具集，减少每个 Agent 可见的工具噪音。
///
/// 仅保留主域（primary_domains）和辅助域（secondary_domains）中的工具，
/// 排除与当前角色无关的工具域。
pub fn filter_tools_by_role(
    tools: &HashMap<String, ToolMeta>,
    role: &codex_patent_agents::roles::PatentAgentRole,
) -> HashMap<String, ToolMeta> {
    let visible_domains: Vec<ToolDomain> = role.all_domains();
    tools
        .iter()
        .filter(|(_, meta)| visible_domains.contains(&meta.domain))
        .map(|(k, v)| {
            (
                k.clone(),
                ToolMeta {
                    domain: v.domain,
                    handler: v.handler,
                },
            )
        })
        .collect()
}

/// 注册全部专利工具，委托给 [`register_all_tools_with_domains`] 并剥离域元数据。
pub fn register_all_tools() -> HashMap<String, ToolHandler> {
    register_all_tools_with_domains()
        .into_iter()
        .map(|(name, meta)| (name, meta.handler))
        .collect()
}

/// 注册全部专利工具并附带域分类元数据。
pub fn register_all_tools_with_domains() -> HashMap<String, ToolMeta> {
    let mut tools = HashMap::new();
    insert_with_domain(
        &mut tools,
        ToolDomain::Search,
        search_tools::register_search_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::WebSearch,
        web_search_tools::register_web_search_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Drafting,
        drafting_tools::register_drafting_tools(),
    );
    insert_with_domain(&mut tools, ToolDomain::Oa, oa_tools::register_oa_tools());
    insert_with_domain(
        &mut tools,
        ToolDomain::Quality,
        quality_tools::register_quality_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Analysis,
        analysis_tools::register_analysis_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Document,
        document_tools::register_document_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Legal,
        legal_tools::register_legal_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Management,
        management_tools::register_management_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Review,
        review_tools::register_review_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Evaluation,
        evaluation_tools::register_evaluation_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Council,
        council_tools::register_council_tools(),
    );
    insert_with_domain(
        &mut tools,
        ToolDomain::Simulator,
        simulator_tools::register_simulator_tools(),
    );
    tools
}

fn insert_with_domain(
    target: &mut HashMap<String, ToolMeta>,
    domain: ToolDomain,
    handlers: HashMap<String, ToolHandler>,
) {
    for (name, handler) in handlers {
        target.insert(name, ToolMeta { domain, handler });
    }
}
