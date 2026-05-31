use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::Path;

/// 最大 include 嵌套深度（含 root 层）。
/// 值 = 4 表示允许 root + 3 层递归 include。
const MAX_INCLUDE_DEPTH: usize = 4;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillDefinition {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub includes: Vec<String>,
    pub required_tools: Vec<String>,
    #[serde(default)]
    pub activates_agents: Vec<String>,
    #[serde(default)]
    pub related_concepts: Vec<String>,
    #[serde(default)]
    pub phase: String,
}

pub struct SkillLoader {
    skills: HashMap<String, SkillDefinition>,
    shared: HashMap<String, SkillDefinition>,
}

impl SkillLoader {
    pub fn load(skills_dir: &str) -> Result<Self, String> {
        let dir = Path::new(skills_dir);
        let mut skills = HashMap::new();
        let mut shared = HashMap::new();
        let shared_dir = dir.join("_shared");

        if shared_dir.exists() {
            for entry in std::fs::read_dir(&shared_dir).map_err(|e| format!("{e}"))? {
                let entry = entry.map_err(|e| format!("{e}"))?;
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "toml") {
                    let content = std::fs::read_to_string(&path).map_err(|e| format!("{e}"))?;
                    let def: SkillDefinition =
                        toml::from_str(&content).map_err(|e| format!("{e}"))?;
                    shared.insert(def.skill_id.clone(), def);
                }
            }
        }

        for entry in std::fs::read_dir(dir).map_err(|e| format!("{e}"))? {
            let entry = entry.map_err(|e| format!("{e}"))?;
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "toml") {
                let content = std::fs::read_to_string(&path).map_err(|e| format!("{e}"))?;
                let def: SkillDefinition = toml::from_str(&content).map_err(|e| format!("{e}"))?;
                skills.insert(def.skill_id.clone(), def);
            }
        }
        Ok(Self { skills, shared })
    }

    /// 解析技能指令，支持：
    /// - 递归展开 `{{include:_shared/name}}` 内联标记，最深 3 层
    /// - 附加式 `includes` 列表（向后兼容）
    /// - 防循环引用
    pub fn resolve(&self, skill_id: &str) -> Result<String, String> {
        let skill = self
            .skills
            .get(skill_id)
            .ok_or_else(|| format!("skill not found: {skill_id}"))?;
        let mut visited = HashSet::new();
        self.resolve_with_depth(&skill.instructions, &skill.includes, 0, &mut visited)
    }

    pub fn resolve_instructions(
        &self,
        instructions: &str,
        includes: &[String],
    ) -> Result<String, String> {
        let mut visited = HashSet::new();
        self.resolve_with_depth(instructions, includes, 0, &mut visited)
    }

    fn resolve_with_depth(
        &self,
        instructions: &str,
        includes: &[String],
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> Result<String, String> {
        let mut result = self.resolve_inline_includes(instructions, depth, visited)?;

        for include in includes {
            let include_id = include.strip_prefix("_shared/").unwrap_or(include);
            if !visited.insert(include_id.to_string()) {
                continue;
            }
            if let Some(shared_def) = self.shared.get(include) {
                result.push_str("\n\n");
                let resolved = self.resolve_with_depth(
                    &shared_def.instructions,
                    &shared_def.includes,
                    depth + 1,
                    visited,
                )?;
                result.push_str(&resolved);
            }
        }
        Ok(result)
    }

    fn resolve_inline_includes(
        &self,
        text: &str,
        depth: usize,
        visited: &mut HashSet<String>,
    ) -> Result<String, String> {
        if depth >= MAX_INCLUDE_DEPTH {
            return Ok(format!(
                "<!-- include 超出最大深度({MAX_INCLUDE_DEPTH})，原始内容如下 -->\n{text}"
            ));
        }

        let mut result = String::with_capacity(text.len());
        let mut rest = text;

        while let Some(start) = rest.find("{{include:") {
            result.push_str(&rest[..start]);
            let after_marker = &rest[start + 10..];

            if let Some(end) = after_marker.find("}}") {
                let ref_path = after_marker[..end].trim();
                let include_id = ref_path.strip_prefix("_shared/").unwrap_or(ref_path);

                if visited.contains(include_id) {
                    result.push_str(&format!("<!-- 循环引用跳过: {ref_path} -->"));
                } else if let Some(shared_def) = self.shared.get(ref_path) {
                    visited.insert(include_id.to_string());
                    let nested =
                        self.resolve_inline_includes(&shared_def.instructions, depth + 1, visited)?;
                    result.push_str("<!-- include: ");
                    result.push_str(ref_path);
                    result.push_str(" -->\n");
                    result.push_str(&nested);
                    result.push_str("\n<!-- /include: ");
                    result.push_str(ref_path);
                    result.push_str(" -->");
                } else {
                    // 尝试作为 agent include 解析 — 从 shared 查不到时不报错，保留原标记
                    // 给外部调用方（如 agent 系统）自行处理的机会
                    result.push_str(&format!("{{include:{ref_path}}}"));
                }

                rest = &after_marker[end + 2..];
            } else {
                result.push_str(&rest[start..]);
                rest = "";
                break;
            }
        }

        result.push_str(rest);
        Ok(result)
    }

    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    pub fn get(&self, skill_id: &str) -> Option<&SkillDefinition> {
        self.skills.get(skill_id)
    }

    pub fn get_shared(&self, shared_id: &str) -> Option<&SkillDefinition> {
        self.shared.get(shared_id)
    }

    /// 获取技能对应的专利生命周期阶段，未设置时返回 "general"
    pub fn phase_for(&self, skill_id: &str) -> &str {
        self.skills
            .get(skill_id)
            .map(|s| s.phase.as_str())
            .filter(|p| !p.is_empty())
            .unwrap_or("general")
    }
}
