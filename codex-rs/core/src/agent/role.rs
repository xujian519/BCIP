//! Applies agent-role configuration layers on top of an existing session config.
//!
//! Roles are selected at spawn time and are loaded with the same config machinery as
//! `config.toml`. This module resolves built-in and user-defined role files, inserts the role as a
//! high-precedence layer, and preserves the caller's current provider and service tier unless the
//! role layer sets them. It does not decide when to spawn a sub-agent or which role to use; the
//! multi-agent tool handler owns that orchestration.

use crate::config::AgentRoleConfig;
use crate::config::Config;
use crate::config::ConfigOverrides;
use crate::config::agent_roles::parse_agent_role_file_contents;
use crate::config::deserialize_config_toml_with_base;
use anyhow::anyhow;
use codex_app_server_protocol::ConfigLayerSource;
use codex_config::ConfigLayerEntry;
use codex_config::ConfigLayerStack;
use codex_config::ConfigLayerStackOrdering;
use codex_config::config_toml::ConfigToml;
use codex_config::loader::resolve_relative_paths_in_config_toml;
use codex_exec_server::LOCAL_FS;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;
use toml::Value as TomlValue;

/// The role name used when a caller omits `agent_type`.
pub const DEFAULT_ROLE_NAME: &str = "default";
const AGENT_TYPE_UNAVAILABLE_ERROR: &str = "agent type is currently not available";

/// Applies a named role layer to `config` while preserving caller-owned provider settings.
///
/// The role layer is inserted at session-flag precedence so it can override persisted config, but
/// the caller's current `model_provider` and `service_tier` remain sticky runtime choices unless
/// the role explicitly sets the corresponding top-level config key. Rebuilding the config without
/// those overrides would make a spawned agent silently fall back to default settings.
pub(crate) async fn apply_role_to_config(
    config: &mut Config,
    role_name: Option<&str>,
) -> Result<(), String> {
    let role_name = role_name.unwrap_or(DEFAULT_ROLE_NAME);

    let role = resolve_role_config(config, role_name)
        .cloned()
        .ok_or_else(|| format!("unknown agent_type '{role_name}'"))?;

    apply_role_to_config_inner(config, role_name, &role)
        .await
        .map_err(|err| {
            tracing::warn!("failed to apply role to config: {err}");
            AGENT_TYPE_UNAVAILABLE_ERROR.to_string()
        })
}

async fn apply_role_to_config_inner(
    config: &mut Config,
    role_name: &str,
    role: &AgentRoleConfig,
) -> anyhow::Result<()> {
    let is_built_in = !config.agent_roles.contains_key(role_name);
    let Some(config_file) = role.config_file.as_ref() else {
        return Ok(());
    };
    let role_layer_toml = load_role_layer_toml(config, config_file, is_built_in, role_name).await?;
    if role_layer_toml
        .as_table()
        .is_some_and(toml::map::Map::is_empty)
    {
        return Ok(());
    }
    let preserve_current_provider = role_layer_toml.get("model_provider").is_none();
    let preserve_current_service_tier = role_layer_toml.get("service_tier").is_none();

    *config = reload::build_next_config(
        config,
        role_layer_toml,
        preserve_current_provider,
        preserve_current_service_tier,
    )
    .await?;
    config.active_agent_type = Some(role_name.to_string());
    Ok(())
}

async fn load_role_layer_toml(
    config: &Config,
    config_file: &Path,
    is_built_in: bool,
    role_name: &str,
) -> anyhow::Result<TomlValue> {
    let (role_config_toml, role_config_base) = if is_built_in {
        let role_config_contents = built_in::config_file_contents(config_file)
            .map(str::to_owned)
            .ok_or(anyhow!("No corresponding config content"))?;
        let mut role_config_toml: TomlValue = toml::from_str(&role_config_contents)?;
        inject_patent_role_context(&mut role_config_toml, role_name);
        let role_config_toml = strip_non_config_fields(role_config_toml);
        (role_config_toml, config.codex_home.as_path())
    } else {
        let role_config_contents = tokio::fs::read_to_string(config_file).await?;
        let role_config_base = config_file
            .parent()
            .ok_or(anyhow!("No corresponding config content"))?;
        let role_config_toml = parse_agent_role_file_contents(
            &role_config_contents,
            config_file,
            role_config_base,
            Some(role_name),
        )?
        .config;
        (role_config_toml, role_config_base)
    };

    deserialize_config_toml_with_base(role_config_toml.clone(), role_config_base)?;
    Ok(resolve_relative_paths_in_config_toml(
        role_config_toml,
        role_config_base,
    )?)
}

const PATENT_ROLE_NON_CONFIG_KEYS: &[&str] = &[
    "description",
    "role_id",
    "name",
    "identity",
    "output_format",
    "methodology",
    "primary_tools",
    "secondary_tools",
    "constraints",
    "auto_knowledge",
    "includes",
    "related_concepts",
    "required_tools",
    "activates_agents",
    "phase",
    "skill_id",
    "instructions",
];

fn strip_non_config_fields(toml: TomlValue) -> TomlValue {
    let TomlValue::Table(mut table) = toml else {
        return toml;
    };
    table.retain(|key, _| !PATENT_ROLE_NON_CONFIG_KEYS.iter().any(|k| *k == key));
    TomlValue::Table(table)
}

/// For built-in patent roles: extract `auto_knowledge` config, resolve knowledge context,
/// and append it to `developer_instructions` before non-config fields get stripped.
#[cfg(feature = "patent-tools")]
fn inject_patent_role_context(toml: &mut TomlValue, role_name: &str) {
    use codex_patent_agents::roles::PatentAgentRole;

    if PatentAgentRole::from_str(role_name).is_none() {
        return;
    }

    let Some(table) = toml.as_table_mut() else {
        return;
    };

    let has_auto_knowledge = table
        .get("auto_knowledge")
        .and_then(|v| v.as_table())
        .is_some_and(|t| t.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false));

    if !has_auto_knowledge {
        return;
    }

    let ak_toml = match table.get("auto_knowledge").cloned() {
        Some(v) => v,
        None => {
            tracing::debug!("auto_knowledge config missing for role {role_name}");
            return;
        }
    };
    let ak_str = match toml::to_string(&ak_toml) {
        Ok(s) => s,
        Err(e) => {
            tracing::debug!("auto_knowledge serialization failed for role {role_name}: {e}");
            return;
        }
    };
    let ak_config: codex_patent_agents::knowledge_context::AutoKnowledgeConfig =
        match toml::from_str(&ak_str) {
            Ok(c) => c,
            Err(e) => {
                tracing::debug!("auto_knowledge deserialization failed for role {role_name}: {e}");
                return;
            }
        };

    let semantic_path = codex_patent_knowledge::paths::semantic_index_path();
    let knowledge_ctx = codex_patent_agents::knowledge_context::KnowledgeContext::new(
        &codex_patent_knowledge::paths::kg_db_path(),
        &codex_patent_knowledge::paths::law_db_path(),
        &codex_patent_knowledge::paths::card_index_path(),
        Some(&semantic_path),
        ak_config,
    );

    let knowledge_text = knowledge_ctx.resolve(role_name, "");
    if knowledge_text.is_empty() {
        tracing::debug!("knowledge context resolved empty for role {role_name}");
        return;
    }

    if let Some(TomlValue::String(instructions)) = table.get_mut("developer_instructions") {
        instructions.push_str("\n\n### 知识上下文\n\n");
        instructions.push_str(&knowledge_text);
    }
}

#[cfg(not(feature = "patent-tools"))]
fn inject_patent_role_context(_toml: &mut TomlValue, _role_name: &str) {}

pub(crate) fn resolve_role_config<'a>(
    config: &'a Config,
    role_name: &str,
) -> Option<&'a AgentRoleConfig> {
    config
        .agent_roles
        .get(role_name)
        .or_else(|| built_in::configs().get(role_name))
}

mod reload {
    use super::*;

    pub(super) async fn build_next_config(
        config: &Config,
        role_layer_toml: TomlValue,
        preserve_current_provider: bool,
        preserve_current_service_tier: bool,
    ) -> anyhow::Result<Config> {
        let config_layer_stack = build_config_layer_stack(config, &role_layer_toml)?;
        let merged_config = deserialize_effective_config(config, &config_layer_stack)?;

        let next_config = Config::load_config_with_layer_stack(
            LOCAL_FS.as_ref(),
            merged_config,
            reload_overrides(
                config,
                preserve_current_provider,
                preserve_current_service_tier,
            ),
            config.codex_home.clone(),
            config_layer_stack,
        )
        .await?;
        Ok(next_config)
    }

    fn build_config_layer_stack(
        config: &Config,
        role_layer_toml: &TomlValue,
    ) -> anyhow::Result<ConfigLayerStack> {
        let mut layers = existing_layers(config);
        insert_layer(&mut layers, role_layer(role_layer_toml.clone()));
        Ok(ConfigLayerStack::new(
            layers,
            config.config_layer_stack.requirements().clone(),
            config.config_layer_stack.requirements_toml().clone(),
        )?)
    }

    fn deserialize_effective_config(
        config: &Config,
        config_layer_stack: &ConfigLayerStack,
    ) -> anyhow::Result<ConfigToml> {
        Ok(deserialize_config_toml_with_base(
            config_layer_stack.effective_config(),
            &config.codex_home,
        )?)
    }

    fn existing_layers(config: &Config) -> Vec<ConfigLayerEntry> {
        config
            .config_layer_stack
            .get_layers(
                ConfigLayerStackOrdering::LowestPrecedenceFirst,
                /*include_disabled*/ true,
            )
            .into_iter()
            .cloned()
            .collect()
    }

    fn insert_layer(layers: &mut Vec<ConfigLayerEntry>, layer: ConfigLayerEntry) {
        let insertion_index =
            layers.partition_point(|existing_layer| existing_layer.name <= layer.name);
        layers.insert(insertion_index, layer);
    }

    fn role_layer(role_layer_toml: TomlValue) -> ConfigLayerEntry {
        ConfigLayerEntry::new(ConfigLayerSource::SessionFlags, role_layer_toml)
    }

    fn reload_overrides(
        config: &Config,
        preserve_current_provider: bool,
        preserve_current_service_tier: bool,
    ) -> ConfigOverrides {
        ConfigOverrides {
            cwd: Some(config.cwd.to_path_buf()),
            model_provider: preserve_current_provider.then(|| config.model_provider_id.clone()),
            service_tier: preserve_current_service_tier.then(|| config.service_tier.clone()),
            codex_linux_sandbox_exe: config.codex_linux_sandbox_exe.clone(),
            main_execve_wrapper_exe: config.main_execve_wrapper_exe.clone(),
            ..Default::default()
        }
    }
}

pub(crate) mod spawn_tool_spec {
    use super::*;

    /// Builds the spawn-agent tool description text from built-in and configured roles.
    pub(crate) fn build(user_defined_agent_roles: &BTreeMap<String, AgentRoleConfig>) -> String {
        let built_in_roles = built_in::configs();
        build_from_configs(built_in_roles, user_defined_agent_roles)
    }

    // This function is not inlined for testing purpose.
    fn build_from_configs(
        built_in_roles: &BTreeMap<String, AgentRoleConfig>,
        user_defined_roles: &BTreeMap<String, AgentRoleConfig>,
    ) -> String {
        let mut seen = BTreeSet::new();
        let mut formatted_roles = Vec::new();
        for (name, declaration) in user_defined_roles {
            if seen.insert(name.as_str()) {
                formatted_roles.push(format_role(name, declaration));
            }
        }
        for (name, declaration) in built_in_roles {
            if seen.insert(name.as_str()) {
                formatted_roles.push(format_role(name, declaration));
            }
        }

        format!(
            "Optional type name for the new agent. If omitted, `{DEFAULT_ROLE_NAME}` is used.\nAvailable roles:\n{}",
            formatted_roles.join("\n"),
        )
    }

    fn format_role(name: &str, declaration: &AgentRoleConfig) -> String {
        if let Some(description) = &declaration.description {
            let locked_settings_note = declaration
                .config_file
                .as_ref()
                .and_then(|config_file| {
                    built_in::config_file_contents(config_file)
                        .map(str::to_owned)
                        .or_else(|| std::fs::read_to_string(config_file).ok())
                })
                .and_then(|contents| toml::from_str::<TomlValue>(&contents).ok())
                .map(|role_toml| {
                    let model = role_toml
                        .get("model")
                        .and_then(TomlValue::as_str);
                    let reasoning_effort = role_toml
                        .get("model_reasoning_effort")
                        .and_then(TomlValue::as_str);
                    let service_tier = role_toml
                        .get("service_tier")
                        .and_then(TomlValue::as_str);

                    let model_and_reasoning_note = match (model, reasoning_effort) {
                        (Some(model), Some(reasoning_effort)) => format!(
                            "\n- This role's model is set to `{model}` and its reasoning effort is set to `{reasoning_effort}`. These settings cannot be changed."
                        ),
                        (Some(model), None) => {
                            format!(
                                "\n- This role's model is set to `{model}` and cannot be changed."
                            )
                        }
                        (None, Some(reasoning_effort)) => {
                            format!(
                                "\n- This role's reasoning effort is set to `{reasoning_effort}` and cannot be changed."
                            )
                        }
                        (None, None) => String::new(),
                    };
                    let service_tier_note = service_tier
                        .map(|service_tier| {
                            format!(
                                "\n- This role's service tier is set to `{service_tier}`. If it is supported by the resolved model, it takes precedence over a valid spawn request service tier."
                            )
                        })
                        .unwrap_or_default();
                    format!("{model_and_reasoning_note}{service_tier_note}")
                })
                .unwrap_or_default();
            format!("{name}: {{\n{description}{locked_settings_note}\n}}")
        } else {
            format!("{name}: no description")
        }
    }
}

mod built_in {
    use super::*;

    /// Returns the cached built-in role declarations defined in this module.
    pub(super) fn configs() -> &'static BTreeMap<String, AgentRoleConfig> {
        static CONFIG: LazyLock<BTreeMap<String, AgentRoleConfig>> = LazyLock::new(|| {
            #[allow(unused_mut)]
            let mut roles = BTreeMap::from([
                (
                    DEFAULT_ROLE_NAME.to_string(),
                    AgentRoleConfig {
                        description: Some("Default agent.".to_string()),
                        config_file: None,
                        nickname_candidates: None,
                    }
                ),
                (
                    "explorer".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use `explorer` for specific codebase questions.
Explorers are fast and authoritative.
They must be used to ask specific, well-scoped questions on the codebase.
Rules:
- In order to avoid redundant work, you should avoid exploring the same problem that explorers have already covered. Typically, you should trust the explorer results without additional verification. You are still allowed to inspect the code yourself to gain the needed context!
- You are encouraged to spawn up multiple explorers in parallel when you have multiple distinct questions to ask about the codebase that can be answered independently. This allows you to get more information faster without waiting for one question to finish before asking the next. While waiting for the explorer results, you can continue working on other local tasks that do not depend on those results. This parallelism is a key advantage of delegation, so use it whenever you have multiple questions to ask.
- Reuse existing explorers for related questions."#.to_string()),
                        config_file: Some("explorer.toml".to_string().parse().unwrap_or_default()),
                        nickname_candidates: None,
                    }
                ),
                (
                    "worker".to_string(),
                    AgentRoleConfig {
                        description: Some(r#"Use for execution and production work.
Typical tasks:
- Implement part of a feature
- Fix tests or bugs
- Split large refactors into independent chunks
Rules:
- Explicitly assign **ownership** of the task (files / responsibility). When the subtask involves code changes, you should clearly specify which files or modules the worker is responsible for. This helps avoid merge conflicts and ensures accountability. For example, you can say "Worker 1 is responsible for updating the authentication module, while Worker 2 will handle the database layer." By defining clear ownership, you can delegate more effectively and reduce coordination overhead.
- Always tell workers they are **not alone in the codebase**, and they should not revert the edits made by others, and they should adjust their implementation to accommodate the changes made by others. This is important because there may be multiple workers making changes in parallel, and they need to be aware of each other's work to avoid conflicts and ensure a cohesive final product."#.to_string()),
                        config_file: None,
                        nickname_candidates: None,
                    }
                ),
            ]);

            #[cfg(feature = "patent-tools")]
            for (name, config_file) in patent_roles() {
                roles.insert(name, config_file);
            }

            roles
        });
        &CONFIG
    }

    /// Resolves a built-in role `config_file` path to embedded content.
    pub(super) fn config_file_contents(path: &Path) -> Option<&'static str> {
        const EXPLORER: &str = include_str!("builtins/explorer.toml");
        const AWAITER: &str = include_str!("builtins/awaiter.toml");
        match path.to_str()? {
            "explorer.toml" => Some(EXPLORER),
            "awaiter.toml" => Some(AWAITER),
            #[allow(unused_variables)]
            other => {
                #[cfg(feature = "patent-tools")]
                if let Some(content) = patent_config_file_contents(other) {
                    return Some(content);
                }
                None
            }
        }
    }

    #[cfg(feature = "patent-tools")]
    fn patent_roles() -> Vec<(String, AgentRoleConfig)> {
        let definitions: &[(&str, &str)] = &[
            (
                "retriever",
                "专利检索专家 — 多源专利检索、检索式构建、对比文件筛选",
            ),
            (
                "analyzer",
                "专利技术分析专家 — 权利要求解析、技术特征提取、四层对比分析",
            ),
            ("writer", "专利文件撰写专家 — 说明书/权利要求书/摘要撰写"),
            (
                "novelty_checker",
                "新颖性评估专家 — 三步法新颖性判断、逐特征对比",
            ),
            (
                "creativity_checker",
                "创造性评估专家 — 问题-解决方案法创造性分析",
            ),
            (
                "infringement_checker",
                "侵权分析专家 — 全面覆盖+等同原则侵权分析",
            ),
            ("invalidity_checker", "无效分析专家 — 无效理由和证据分析"),
            ("reviewer", "文件审查专家 — 格式规范和内容质量审查"),
            ("quality_checker", "质量评估专家 — 多维度专利质量评估"),
        ];
        definitions
            .iter()
            .map(|(name, desc)| {
                let config_file = format!("patent/{name}.toml");
                (
                    name.to_string(),
                    AgentRoleConfig {
                        description: Some(desc.to_string()),
                        config_file: Some(config_file.parse().unwrap_or_default()),
                        nickname_candidates: None,
                    },
                )
            })
            .collect()
    }

    #[cfg(feature = "patent-tools")]
    fn patent_config_file_contents(path: &str) -> Option<&'static str> {
        codex_patent_agents::bcip_roles::config_file_contents(Path::new(path))
    }
}

#[cfg(test)]
#[path = "role_tests.rs"]
mod tests;
