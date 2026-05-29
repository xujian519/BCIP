use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

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
    pub role_id: String,
    pub name: String,
    pub identity: String,
    pub methodology: Vec<MethodologyStep>,
    pub output_format: String,
    pub primary_tools: Vec<String>,
    pub secondary_tools: Vec<String>,
    pub constraints: Vec<String>,
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
        let mut prompt = format!(
            "## 角色: {}\n\n{}\n\n### 工作方法\n",
            config.name, config.identity
        );
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
        prompt
    }

    pub fn load_config(role_dir: &str) -> Result<HashMap<String, AgentRoleConfig>, String> {
        let dir = Path::new(role_dir);
        let mut configs = HashMap::new();
        for role in Self::all() {
            let file_path = dir.join(format!("{}.toml", role.role_id()));
            if file_path.exists() {
                let content = std::fs::read_to_string(&file_path)
                    .map_err(|e| format!("read {}: {e}", file_path.display()))?;
                let config: AgentRoleConfig = toml::from_str(&content)
                    .map_err(|e| format!("parse {}: {e}", file_path.display()))?;
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
    pub fn new(role_dir: &str) -> Result<Self, String> {
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
