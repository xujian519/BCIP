pub mod advanced_analysis;
pub mod analysis_tools;
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

/// 注册全部专利工具
pub fn register_all_tools() -> HashMap<String, ToolHandler> {
    let mut tools = search_tools::register_search_tools();
    tools.extend(web_search_tools::register_web_search_tools());
    tools.extend(drafting_tools::register_drafting_tools());
    tools.extend(oa_tools::register_oa_tools());
    tools.extend(quality_tools::register_quality_tools());
    tools.extend(analysis_tools::register_analysis_tools());
    tools.extend(document_tools::register_document_tools());
    tools.extend(legal_tools::register_legal_tools());
    tools.extend(management_tools::register_management_tools());
    tools.extend(review_tools::register_review_tools());
    tools.extend(evaluation_tools::register_evaluation_tools());
    tools.extend(council_tools::register_council_tools());
    tools.extend(simulator_tools::register_simulator_tools());
    tools
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
