use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SkillDefinition {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub includes: Vec<String>,
    pub required_tools: Vec<String>,
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
                if path.extension().map_or(false, |e| e == "toml") {
                    let content = std::fs::read_to_string(&path).map_err(|e| format!("{e}"))?;
                    let def: SkillDefinition = toml::from_str(&content).map_err(|e| format!("{e}"))?;
                    shared.insert(def.skill_id.clone(), def);
                }
            }
        }
        
        for entry in std::fs::read_dir(dir).map_err(|e| format!("{e}"))? {
            let entry = entry.map_err(|e| format!("{e}"))?;
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |e| e == "toml") {
                let content = std::fs::read_to_string(&path).map_err(|e| format!("{e}"))?;
                let def: SkillDefinition = toml::from_str(&content).map_err(|e| format!("{e}"))?;
                skills.insert(def.skill_id.clone(), def);
            }
        }
        Ok(Self { skills, shared })
    }

    pub fn resolve(&self, skill_id: &str) -> Result<String, String> {
        let skill = self.skills.get(skill_id).ok_or_else(|| format!("skill not found: {skill_id}"))?;
        let mut resolved = skill.instructions.clone();
        for include in &skill.includes {
            if let Some(shared_def) = self.shared.get(include) {
                resolved.push_str("\n\n");
                resolved.push_str(&shared_def.instructions);
            }
        }
        Ok(resolved)
    }

    pub fn list(&self) -> Vec<&str> {
        self.skills.keys().map(|s| s.as_str()).collect()
    }

    pub fn get(&self, skill_id: &str) -> Option<&SkillDefinition> {
        self.skills.get(skill_id)
    }
}