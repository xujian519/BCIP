//! 场景规则系统
//!
//! 定义专利任务场景的编排规则，支持拓扑排序和并行分组执行。
//! 每个场景包含元数据、提示词模板、法律依据和处理步骤。

use codex_patent_core::PatentError;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

/// 场景规则定义
///
/// 描述一个专利任务的完整处理场景，包括元数据、提示词模板、
/// 法律依据和处理步骤。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioRule {
    pub scenario: ScenarioMeta,
    pub prompts: ScenarioPrompts,
    #[serde(default)]
    pub legal_basis: LegalBasis,
    #[serde(default)]
    pub processing: ScenarioProcessing,
}

/// 场景元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioMeta {
    pub rule_id: String,
    pub domain: String,
    pub task_type: String,
    pub phase: String,
    #[serde(default = "default_agent_level")]
    pub agent_level: String,
}

fn default_agent_level() -> String {
    "L2".into()
}

/// 场景提示词模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioPrompts {
    pub system_template: String,
    pub user_template: String,
}

/// 法律依据定义
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LegalBasis {
    #[serde(default)]
    pub laws: Vec<String>,
    #[serde(default)]
    pub reference_cases: Vec<String>,
}

/// 场景处理步骤定义
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScenarioProcessing {
    #[serde(default)]
    pub steps: Vec<ScenarioStep>,
}

/// 单个场景处理步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStep {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub hitl: bool,
}

impl ScenarioStep {
    pub fn id(&self) -> &str {
        &self.name
    }
}

impl ScenarioProcessing {
    /// 拓扑排序：按依赖关系排列步骤
    pub fn topological_order(&self) -> Vec<&ScenarioStep> {
        let steps = &self.steps;
        let mut visited = std::collections::HashSet::new();
        let mut result = Vec::with_capacity(steps.len());
        fn visit<'a>(
            step: &'a ScenarioStep,
            steps: &'a [ScenarioStep],
            visited: &mut std::collections::HashSet<String>,
            result: &mut Vec<&'a ScenarioStep>,
        ) {
            if !visited.insert(step.name.clone()) {
                return;
            }
            for dep in &step.depends_on {
                if let Some(dep_step) = steps.iter().find(|s| s.name == *dep) {
                    visit(dep_step, steps, visited, result);
                }
            }
            result.push(step);
        }
        for step in steps {
            visit(step, steps, &mut visited, &mut result);
        }
        result
    }

    /// 分组：返回可并行执行的步骤组列表
    pub fn parallel_groups(&self) -> Vec<Vec<&ScenarioStep>> {
        let ordered = self.topological_order();
        let mut groups: Vec<Vec<&ScenarioStep>> = Vec::new();
        let mut completed: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut remaining: Vec<&ScenarioStep> = ordered;
        while !remaining.is_empty() {
            let ready: Vec<&ScenarioStep> = remaining
                .iter()
                .filter(|s| {
                    s.depends_on
                        .iter()
                        .all(|dep| completed.contains(dep.as_str()))
                })
                .copied()
                .collect();
            if ready.is_empty() {
                groups.push(remaining.into_iter().collect());
                break;
            }
            for s in &ready {
                completed.insert(s.name.clone());
            }
            remaining.retain(|s| !completed.contains(s.name.as_str()));
            groups.push(ready);
        }
        groups
    }
}

/// 场景规则注册表
///
/// 管理所有专利任务场景的注册、查找和提示词解析。
#[derive(Debug, Default)]
pub struct ScenarioRegistry {
    rules: HashMap<String, ScenarioRule>,
}

impl ScenarioRegistry {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    /// 注册一个场景规则
    pub fn register(&mut self, rule: ScenarioRule) {
        self.rules.insert(rule.scenario.task_type.clone(), rule);
    }

    /// 批量注册（从 TOML 文件内容）
    pub fn register_from_toml(&mut self, toml_content: &str) -> Result<(), PatentError> {
        let rule: ScenarioRule = toml::from_str(toml_content)
            .map_err(|e| PatentError::Config(format!("TOML 解析失败: {e}")))?;
        self.register(rule);
        Ok(())
    }

    /// 按 task_type 查找场景规则
    pub fn find(&self, task_type: &str) -> Option<&ScenarioRule> {
        self.rules.get(task_type)
    }

    /// 列出所有已注册的场景规则
    pub fn list(&self) -> Vec<&ScenarioRule> {
        self.rules.values().collect()
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// 替换提示词中的变量
    pub fn resolve_prompt(
        rule: &ScenarioRule,
        variables: &HashMap<String, String>,
    ) -> (String, String) {
        let mut system = rule.prompts.system_template.clone();
        let mut user = rule.prompts.user_template.clone();
        for (key, value) in variables {
            let placeholder = format!("{{{}}}", key);
            system = system.replace(&placeholder, value);
            user = user.replace(&placeholder, value);
        }
        (system, user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_example_rule() {
        let toml = r#"
[scenario]
rule_id = "test-rule"
domain = "patent"
task_type = "novelty_analysis"
phase = "analysis"

[prompts]
system_template = "You are a patent examiner."
user_template = "Analyze {patent_text} against {prior_art_text}."
"#;
        let mut reg = ScenarioRegistry::new();
        reg.register_from_toml(toml).unwrap();
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn resolve_variables() {
        let toml = r#"
[scenario]
rule_id = "test-rule"
domain = "patent"
task_type = "test"
phase = "test"

[prompts]
system_template = "System: {name}"
user_template = "User: {query}"
"#;
        let mut reg = ScenarioRegistry::new();
        reg.register_from_toml(toml).unwrap();
        let rule = reg.find("test").unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".into(), "XiaoNuo".into());
        vars.insert("query".into(), "Hello".into());
        let (sys, user) = ScenarioRegistry::resolve_prompt(rule, &vars);
        assert_eq!(sys, "System: XiaoNuo");
        assert_eq!(user, "User: Hello");
    }

    #[test]
    fn empty_registry_has_no_rules() {
        let reg = ScenarioRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn find_nonexistent_returns_none() {
        let reg = ScenarioRegistry::new();
        assert!(reg.find("nonexistent").is_none());
    }

    #[test]
    fn topological_order_no_deps() {
        let processing = ScenarioProcessing {
            steps: vec![
                ScenarioStep {
                    name: "a".into(),
                    description: "step a".into(),
                    depends_on: vec![],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
                ScenarioStep {
                    name: "b".into(),
                    description: "step b".into(),
                    depends_on: vec![],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
            ],
        };
        let order = processing.topological_order();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn topological_order_with_deps() {
        let processing = ScenarioProcessing {
            steps: vec![
                ScenarioStep {
                    name: "b".into(),
                    description: "step b".into(),
                    depends_on: vec!["a".into()],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
                ScenarioStep {
                    name: "a".into(),
                    description: "step a".into(),
                    depends_on: vec![],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
            ],
        };
        let order = processing.topological_order();
        assert_eq!(order.len(), 2);
        assert_eq!(order[0].name, "a");
        assert_eq!(order[1].name, "b");
    }

    #[test]
    fn parallel_groups_independent() {
        let processing = ScenarioProcessing {
            steps: vec![
                ScenarioStep {
                    name: "a".into(),
                    description: "a".into(),
                    depends_on: vec![],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
                ScenarioStep {
                    name: "b".into(),
                    description: "b".into(),
                    depends_on: vec![],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
            ],
        };
        let groups = processing.parallel_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].len(), 2);
    }

    #[test]
    fn parallel_groups_sequential() {
        let processing = ScenarioProcessing {
            steps: vec![
                ScenarioStep {
                    name: "b".into(),
                    description: "b".into(),
                    depends_on: vec!["a".into()],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
                ScenarioStep {
                    name: "a".into(),
                    description: "a".into(),
                    depends_on: vec![],
                    agent: None,
                    tool: None,
                    hitl: false,
                },
            ],
        };
        let groups = processing.parallel_groups();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].len(), 1);
        assert_eq!(groups[0][0].name, "a");
        assert_eq!(groups[1][0].name, "b");
    }

    #[test]
    fn register_from_toml_invalid() {
        let mut reg = ScenarioRegistry::new();
        let result = reg.register_from_toml("not valid toml [[[");
        assert!(result.is_err());
    }

    #[test]
    fn resolve_prompt_no_variables() {
        let toml = r#"
[scenario]
rule_id = "test"
domain = "patent"
task_type = "t1"
phase = "test"

[prompts]
system_template = "No vars here"
user_template = "Also no vars"
"#;
        let mut reg = ScenarioRegistry::new();
        reg.register_from_toml(toml).unwrap();
        let rule = reg.find("t1").unwrap();
        let (sys, user) = ScenarioRegistry::resolve_prompt(rule, &HashMap::new());
        assert_eq!(sys, "No vars here");
        assert_eq!(user, "Also no vars");
    }

    #[test]
    fn resolve_prompt_missing_variable_keeps_placeholder() {
        let toml = r#"
[scenario]
rule_id = "test"
domain = "patent"
task_type = "t2"
phase = "test"

[prompts]
system_template = "Hello {name}"
user_template = "Query: {query}"
"#;
        let mut reg = ScenarioRegistry::new();
        reg.register_from_toml(toml).unwrap();
        let rule = reg.find("t2").unwrap();
        let mut vars = HashMap::new();
        vars.insert("name".into(), "World".into());
        let (sys, user) = ScenarioRegistry::resolve_prompt(rule, &vars);
        assert_eq!(sys, "Hello World");
        assert_eq!(user, "Query: {query}");
    }

    #[test]
    fn list_returns_all_registered() {
        let mut reg = ScenarioRegistry::new();
        let toml1 = r#"
[scenario]
rule_id = "r1"
domain = "patent"
task_type = "type_a"
phase = "test"

[prompts]
system_template = "sys"
user_template = "usr"
"#;
        let toml2 = r#"
[scenario]
rule_id = "r2"
domain = "patent"
task_type = "type_b"
phase = "test"

[prompts]
system_template = "sys"
user_template = "usr"
"#;
        reg.register_from_toml(toml1).unwrap();
        reg.register_from_toml(toml2).unwrap();
        assert_eq!(reg.list().len(), 2);
    }

    #[test]
    fn scenario_step_id_returns_name() {
        let step = ScenarioStep {
            name: "my_step".into(),
            description: "desc".into(),
            depends_on: vec![],
            agent: None,
            tool: None,
            hitl: false,
        };
        assert_eq!(step.id(), "my_step");
    }

    #[test]
    fn legal_basis_default() {
        let lb = LegalBasis::default();
        assert!(lb.laws.is_empty());
        assert!(lb.reference_cases.is_empty());
    }
}
