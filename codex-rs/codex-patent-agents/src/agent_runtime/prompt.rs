//! 系统 prompt 构建

use crate::roles::PatentAgentRole;

const MAX_KNOWLEDGE_PREFIX_LEN: usize = 8000;

pub(crate) fn build_system_prompt(
    subagent_type: &str,
    model: &str,
    knowledge_prefix: &str,
) -> String {
    let role = PatentAgentRole::from_str(subagent_type);
    let role_name = role.map(|r| r.name()).unwrap_or("通用助手");

    let mut prompt = format!(
        "你是 BCIP 专利智能体系统的 {role_name}。\
         请基于用户提供的任务要求，给出专业、准确、完整的分析和建议。\n\n\
         ## 行为准则\n\
         - 基于事实和法律条文进行分析，不做无根据的推测\n\
         - 输出结构清晰，使用 Markdown 格式\n\
         - 如遇不确定内容，明确标注并给出建议\n"
    );

    if !knowledge_prefix.is_empty() {
        prompt.push_str("\n## 知识上下文\n");
        if knowledge_prefix.len() > MAX_KNOWLEDGE_PREFIX_LEN {
            tracing::warn!(
                original_len = knowledge_prefix.len(),
                truncated_to = MAX_KNOWLEDGE_PREFIX_LEN,
                "知识上下文过长，已截断"
            );
            let truncated: String = knowledge_prefix
                .chars()
                .take(MAX_KNOWLEDGE_PREFIX_LEN)
                .collect();
            prompt.push_str(&truncated);
        } else {
            prompt.push_str(knowledge_prefix);
        }
        prompt.push('\n');
    }

    if let Some(r) = role {
        let domains: Vec<&str> = match r {
            PatentAgentRole::Retriever => vec!["专利检索", "Web搜索"],
            PatentAgentRole::Analyzer => vec!["权利要求分析", "法律分析"],
            PatentAgentRole::Writer => vec!["专利撰写", "文档处理"],
            PatentAgentRole::NoveltyChecker => vec!["新颖性分析", "专利检索"],
            PatentAgentRole::CreativityChecker => vec!["创造性分析"],
            PatentAgentRole::InfringementChecker => vec!["侵权分析", "法律分析"],
            PatentAgentRole::InvalidityChecker => vec!["无效分析", "法律分析", "专利检索"],
            PatentAgentRole::Reviewer => vec!["文件审查", "质量检查"],
            PatentAgentRole::QualityChecker => vec!["质量评估", "文件审查"],
        };
        prompt.push_str(&format!("\n## 专业领域\n{}\n", domains.join("、")));
    }

    prompt.push_str(&format!("\n## 当前模型\n{model}\n"));
    prompt
}
