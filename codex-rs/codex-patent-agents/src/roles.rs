use crate::knowledge_context::AutoKnowledgeConfig;
use crate::knowledge_context::KnowledgeContext;
use codex_patent_core::PatentError;
use codex_patent_core::ToolDomain;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

/// 最大 include 嵌套深度（含 root 层）。
/// 值 = 4 表示允许 root + 3 层递归 include。
const MAX_INCLUDE_DEPTH: usize = 4;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct MethodologyStep {
    pub step_number: usize,
    pub step_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentRoleConfig {
    #[serde(default)]
    pub role_id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub identity: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub developer_instructions: Option<String>,
    #[serde(default)]
    pub methodology: Vec<MethodologyStep>,
    #[serde(default)]
    pub output_format: String,
    #[serde(default)]
    pub primary_tools: Vec<String>,
    #[serde(default)]
    pub secondary_tools: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub auto_knowledge: Option<AutoKnowledgeConfig>,
    #[serde(default)]
    pub includes: Vec<String>,
}

/// 解析文本中的 `{{include:_shared/name}}` 内联标记，替换为共享模块内容。
///
/// 从 skills_shared_dir（通常是 `codex-rs/codex-patent-skills/assets/_shared/`）
/// 读取对应的 `.toml` 文件中的 `instructions` 字段。
///
/// 支持递归 include（最深 3 层），自动防循环引用。
pub fn resolve_text_includes(text: &str, skills_shared_dir: &Path) -> String {
    let mut visited = std::collections::HashSet::new();
    resolve_text_includes_inner(text, skills_shared_dir, 0, &mut visited)
}

fn resolve_text_includes_inner(
    text: &str,
    shared_dir: &Path,
    depth: usize,
    visited: &mut std::collections::HashSet<String>,
) -> String {
    if depth >= MAX_INCLUDE_DEPTH {
        return format!("<!-- include 超出最大深度({MAX_INCLUDE_DEPTH})，原始内容如下 -->\n{text}");
    }

    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(start) = rest.find("{{include:") {
        result.push_str(&rest[..start]);
        let after_marker = &rest[start + 10..];

        if let Some(end) = after_marker.find("}}") {
            let ref_path = after_marker[..end].trim();
            let canonical = ref_path.strip_prefix("_shared/").unwrap_or(ref_path);

            if visited.contains(canonical) {
                result.push_str(&format!("<!-- 循环引用跳过: {ref_path} -->"));
            } else {
                let file_name = format!("{canonical}.toml");
                let file_path = shared_dir.join(&file_name);

                if file_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&file_path) {
                        if let Ok(def) = toml::from_str::<serde_json::Value>(&content) {
                            if let Some(instructions) =
                                def.get("instructions").and_then(|v| v.as_str())
                            {
                                visited.insert(canonical.to_string());
                                let nested = resolve_text_includes_inner(
                                    instructions,
                                    shared_dir,
                                    depth + 1,
                                    visited,
                                );
                                result.push_str(&format!(
                                    "<!-- include: {ref_path} -->\n{nested}\n<!-- /include: {ref_path} -->"
                                ));
                            } else {
                                result.push_str(&format!("{{include:{ref_path}}}"));
                            }
                        } else {
                            result.push_str(&format!("{{include:{ref_path}}}"));
                        }
                    } else {
                        result.push_str(&format!("{{include:{ref_path}}}"));
                    }
                } else {
                    result.push_str(&format!("{{include:{ref_path}}}"));
                }
            }

            rest = &after_marker[end + 2..];
        } else {
            result.push_str(&rest[start..]);
            rest = "";
            break;
        }
    }

    result.push_str(rest);
    result
}

/// 查找包含共享技能资产的目录。
/// 优先从 `CODEX_PATENT_SKILLS_ASSETS` 环境变量，其次从 crate 路径推断。
pub fn find_skills_shared_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("CODEX_PATENT_SKILLS_ASSETS") {
        let path = PathBuf::from(dir).join("_shared");
        if path.is_dir() {
            return Some(path);
        }
    }

    let candidates = [
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .map(|p| p.join("codex-patent-skills/assets/_shared")),
        Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../codex-patent-skills/assets/_shared"),
        ),
        Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../codex-patent-skills/assets/_shared"),
        ),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|candidate| candidate.is_dir())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PatentAgentRole {
    Retriever,
    Analyzer,
    Writer,
    NoveltyChecker,
    CreativityChecker,
    InfringementChecker,
    InvalidityChecker,
    Reviewer,
    QualityChecker,
}

impl PatentAgentRole {
    pub fn role_id(&self) -> &'static str {
        match self {
            Self::Retriever => "retriever",
            Self::Analyzer => "analyzer",
            Self::Writer => "writer",
            Self::NoveltyChecker => "novelty_checker",
            Self::CreativityChecker => "creativity_checker",
            Self::InfringementChecker => "infringement_checker",
            Self::InvalidityChecker => "invalidity_checker",
            Self::Reviewer => "reviewer",
            Self::QualityChecker => "quality_checker",
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Retriever => "检索专家",
            Self::Analyzer => "分析专家",
            Self::Writer => "撰写专家",
            Self::NoveltyChecker => "新颖性评估专家",
            Self::CreativityChecker => "创造性评估专家",
            Self::InfringementChecker => "侵权分析专家",
            Self::InvalidityChecker => "无效分析专家",
            Self::Reviewer => "文件审查专家",
            Self::QualityChecker => "质量评估专家",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "retriever" => Some(Self::Retriever),
            "analyzer" => Some(Self::Analyzer),
            "writer" => Some(Self::Writer),
            "novelty_checker" => Some(Self::NoveltyChecker),
            "creativity_checker" => Some(Self::CreativityChecker),
            "infringement_checker" => Some(Self::InfringementChecker),
            "invalidity_checker" => Some(Self::InvalidityChecker),
            "reviewer" => Some(Self::Reviewer),
            "quality_checker" => Some(Self::QualityChecker),
            _ => None,
        }
    }

    /// 此角色的核心工具域（可见+可调用）。
    pub fn primary_domains(&self) -> &'static [ToolDomain] {
        match self {
            Self::Retriever => &[ToolDomain::Search, ToolDomain::WebSearch],
            Self::Analyzer => &[ToolDomain::Analysis, ToolDomain::Legal],
            Self::Writer => &[ToolDomain::Drafting, ToolDomain::Document],
            Self::NoveltyChecker => &[ToolDomain::Analysis, ToolDomain::Search],
            Self::CreativityChecker => &[ToolDomain::Analysis],
            Self::InfringementChecker => &[ToolDomain::Analysis, ToolDomain::Legal],
            Self::InvalidityChecker => {
                &[ToolDomain::Analysis, ToolDomain::Legal, ToolDomain::Search]
            }
            Self::Reviewer => &[ToolDomain::Review, ToolDomain::Quality],
            Self::QualityChecker => &[ToolDomain::Quality, ToolDomain::Review],
        }
    }

    /// 此角色的辅助工具域（可通过 ToolSearch 发现）。
    pub fn secondary_domains(&self) -> &'static [ToolDomain] {
        match self {
            Self::Retriever => &[ToolDomain::Legal, ToolDomain::Analysis],
            Self::Analyzer => &[ToolDomain::Search],
            Self::Writer => &[ToolDomain::Quality, ToolDomain::Search, ToolDomain::Legal],
            Self::NoveltyChecker => &[ToolDomain::Legal],
            Self::CreativityChecker => &[ToolDomain::Legal, ToolDomain::Search],
            Self::InfringementChecker => &[ToolDomain::Search],
            Self::InvalidityChecker => &[ToolDomain::Quality],
            Self::Reviewer => &[ToolDomain::Document, ToolDomain::Legal],
            Self::QualityChecker => &[ToolDomain::Document, ToolDomain::Drafting],
        }
    }

    /// 此角色的工具 Schema Token 预算上限。
    pub fn tool_token_budget(&self) -> usize {
        match self {
            Self::Retriever | Self::Analyzer | Self::InvalidityChecker => 8_000,
            Self::Writer | Self::Reviewer | Self::QualityChecker => 6_000,
            _ => 4_000,
        }
    }

    /// 此角色推荐的 LLM temperature。
    pub fn temperature(&self) -> f32 {
        match self {
            Self::Writer | Self::Reviewer | Self::QualityChecker => 0.3,
            Self::Retriever => 0.5,
            Self::Analyzer
            | Self::NoveltyChecker
            | Self::CreativityChecker
            | Self::InfringementChecker
            | Self::InvalidityChecker => 0.4,
        }
    }

    pub fn all() -> &'static [PatentAgentRole] {
        &[
            Self::Retriever,
            Self::Analyzer,
            Self::Writer,
            Self::NoveltyChecker,
            Self::CreativityChecker,
            Self::InfringementChecker,
            Self::InvalidityChecker,
            Self::Reviewer,
            Self::QualityChecker,
        ]
    }

    pub fn system_prompt(&self, config: &AgentRoleConfig) -> String {
        self.system_prompt_with_context(config, "", None, None)
    }

    /// 构建 agent 系统提示词。
    ///
    /// 优先使用 `developer_instructions`（bcip 新版格式，完整专业提示词）；
    /// 回退到旧版拼接模式（identity + methodology + output_format + constraints）。
    ///
    /// - `task_description`: 当前任务描述
    /// - `knowledge`: 知识上下文（可选）
    /// - `shared_dir`: 共享模块目录路径，用于解析 `{{include:_shared/name}}` 内联标记
    pub fn system_prompt_with_context(
        &self,
        config: &AgentRoleConfig,
        task_description: &str,
        knowledge: Option<&KnowledgeContext>,
        shared_dir: Option<&Path>,
    ) -> String {
        let mut prompt = format!("## 角色: {}\n\n", config.name);

        if let Some(ref dev_instr) = config.developer_instructions {
            let resolved = if let Some(dir) = shared_dir {
                resolve_text_includes(dev_instr, dir)
            } else {
                dev_instr.clone()
            };
            prompt.push_str(&resolved);
            prompt.push_str("\n\n");
        } else {
            let identity = if let Some(dir) = shared_dir {
                resolve_text_includes(&config.identity, dir)
            } else {
                config.identity.clone()
            };
            prompt.push_str(&identity);
            prompt.push_str("\n\n### 工作方法\n");
            for step in &config.methodology {
                prompt.push_str(&format!(
                    "{}. {}: {}\n",
                    step.step_number, step.step_name, step.description
                ));
            }
            prompt.push_str(&format!(
                "\n### 输出格式\n{}\n\n### 约束\n",
                config.output_format
            ));
            for c in &config.constraints {
                prompt.push_str(&format!("- {}\n", c));
            }
            prompt.push('\n');
        }

        if let Some(kc) = knowledge
            && kc.is_enabled()
        {
            let context = kc.resolve(self.role_id(), task_description);
            if !context.is_empty() {
                prompt.push_str("### 知识上下文\n\n");
                prompt.push_str(&context);
                prompt.push('\n');
            }
        }

        if let Some(dir) = shared_dir {
            for include in &config.includes {
                let include_path =
                    dir.join(format!("{}.toml", include.trim_start_matches("_shared/")));
                if let Ok(content) = std::fs::read_to_string(&include_path)
                    && let Ok(def) = toml::from_str::<serde_json::Value>(&content)
                    && let Some(instructions) = def.get("instructions").and_then(|v| v.as_str())
                {
                    prompt.push_str("\n\n");
                    prompt.push_str(&format!(
                        "<!-- include: {include} -->\n{instructions}\n<!-- /include: {include} -->"
                    ));
                }
            }
        }

        prompt
    }

    pub fn load_config(role_dir: &str) -> Result<HashMap<String, AgentRoleConfig>, PatentError> {
        let dir = Path::new(role_dir);
        let mut configs = HashMap::new();
        for role in Self::all() {
            let file_path = dir.join(format!("{}.toml", role.role_id()));
            if file_path.exists() {
                let content = std::fs::read_to_string(&file_path).map_err(|e| {
                    PatentError::Config(format!("read {}: {e}", file_path.display()))
                })?;
                let config: AgentRoleConfig = toml::from_str(&content).map_err(|e| {
                    PatentError::Config(format!("parse {}: {e}", file_path.display()))
                })?;
                configs.insert(role.role_id().to_string(), config);
            }
        }
        Ok(configs)
    }
}

pub struct AgentRegistry {
    configs: HashMap<String, AgentRoleConfig>,
}

impl AgentRegistry {
    pub fn new(role_dir: &str) -> Result<Self, PatentError> {
        Ok(Self {
            configs: PatentAgentRole::load_config(role_dir)?,
        })
    }

    pub fn get(&self, role_id: &str) -> Option<&AgentRoleConfig> {
        self.configs.get(role_id)
    }

    pub fn list_roles(&self) -> Vec<&str> {
        self.configs.keys().map(|s| s.as_str()).collect()
    }

    pub fn system_prompt_for(&self, role_id: &str) -> Option<String> {
        let role = PatentAgentRole::from_str(role_id)?;
        let config = self.configs.get(role_id)?;
        Some(role.system_prompt(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_from_str_all_roles() {
        let role_ids = [
            "retriever",
            "analyzer",
            "writer",
            "novelty_checker",
            "creativity_checker",
            "infringement_checker",
            "invalidity_checker",
            "reviewer",
            "quality_checker",
        ];
        for &id in &role_ids {
            let role = PatentAgentRole::from_str(id)
                .unwrap_or_else(|| panic!("from_str should parse '{id}'"));
            assert_eq!(role.role_id(), id, "round-trip failed for '{id}'");
        }
        assert_eq!(role_ids.len(), PatentAgentRole::all().len());
    }

    #[test]
    fn test_from_str_nonexistent_returns_none() {
        assert!(PatentAgentRole::from_str("nonexistent").is_none());
        assert!(PatentAgentRole::from_str("").is_none());
        assert!(PatentAgentRole::from_str("WRITER").is_none());
    }

    #[test]
    fn test_resolve_text_includes_no_markers() {
        let text = "Hello world, no includes here.";
        let dir = tempfile::tempdir().unwrap();
        let result = resolve_text_includes(text, dir.path());
        assert_eq!(result, text);
    }

    #[test]
    fn test_resolve_text_includes_with_file() {
        let dir = tempfile::tempdir().unwrap();
        let shared_dir = dir.path().join("_shared");
        fs::create_dir(&shared_dir).unwrap();

        let toml_content = r#"instructions = "shared module content""#;
        fs::write(shared_dir.join("helper.toml"), toml_content).unwrap();

        let text = "before {{include:_shared/helper}} after";
        let result = resolve_text_includes(text, &shared_dir);
        assert!(result.contains("shared module content"));
        assert!(result.contains("<!-- include: _shared/helper -->"));
        assert!(result.contains("<!-- /include: _shared/helper -->"));
        assert!(result.contains("before"));
        assert!(result.contains("after"));
    }

    #[test]
    fn test_resolve_text_includes_cycle_detection() {
        let dir = tempfile::tempdir().unwrap();
        let shared_dir = dir.path().join("_shared");
        fs::create_dir(&shared_dir).unwrap();

        // a includes b, b includes a => cycle
        let a_content = r#"instructions = "A-content {{include:_shared/b}}""#;
        let b_content = r#"instructions = "B-content {{include:_shared/a}}""#;
        fs::write(shared_dir.join("a.toml"), a_content).unwrap();
        fs::write(shared_dir.join("b.toml"), b_content).unwrap();

        let text = "{{include:_shared/a}}";
        let result = resolve_text_includes(text, &shared_dir);
        assert!(result.contains("A-content"));
        assert!(result.contains("B-content"));
        assert!(result.contains("循环引用跳过"));
    }

    #[test]
    fn test_resolve_text_includes_max_depth() {
        // When called at MAX_INCLUDE_DEPTH, returns depth-exceeded comment
        let dir = tempfile::tempdir().unwrap();
        let shared_dir = dir.path().join("_shared");
        fs::create_dir(&shared_dir).unwrap();

        // Manually test the inner function at max depth
        let text = "some content here";
        let mut visited = std::collections::HashSet::new();
        let result =
            resolve_text_includes_inner(text, &shared_dir, MAX_INCLUDE_DEPTH, &mut visited);
        assert!(result.contains("include 超出最大深度"));
        assert!(result.contains(text));
    }
}
