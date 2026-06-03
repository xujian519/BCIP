//! Patent Agent Bridge — spawns patent-specialized sub-agents via BCIP Core AgentControl.
//!
//! This tool bridges the patent agent runtime to the BCIP multi-agent system.
//! It reuses the spawn_agent v2 pattern but constrains `agent_type` to the
//! `PatentAgentRole` enum and automatically maps role → tool domain exposure.

use crate::agent::control::SpawnAgentOptions;
use crate::agent::next_thread_spawn_depth;
use crate::agent::role::apply_role_to_config;
use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::context::boxed_tool_output;
use crate::tools::handlers::multi_agents_common::apply_requested_spawn_agent_model_overrides;
use crate::tools::handlers::multi_agents_common::apply_spawn_agent_overrides;
use crate::tools::handlers::multi_agents_common::apply_spawn_agent_runtime_overrides;
use crate::tools::handlers::multi_agents_common::apply_spawn_agent_service_tier;
use crate::tools::handlers::multi_agents_common::build_agent_spawn_config;
use crate::tools::handlers::multi_agents_common::collab_spawn_error;
use crate::tools::handlers::multi_agents_common::function_arguments;
use crate::tools::handlers::multi_agents_common::parse_collab_input;
use crate::tools::handlers::multi_agents_common::thread_spawn_source;
use crate::tools::handlers::multi_agents_common::tool_output_code_mode_result;
use crate::tools::handlers::multi_agents_common::tool_output_json_text;
use crate::tools::handlers::multi_agents_common::tool_output_response_item;
use crate::tools::handlers::parse_arguments;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_patent_agents::roles::PatentAgentRole;
use codex_protocol::models::ResponseInputItem;
use codex_tools::JsonSchema;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;

/// Patent agent bridge handler.
pub(crate) struct Handler;

#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("patent_spawn_agent")
    }

    fn spec(&self) -> ToolSpec {
        create_patent_spawn_agent_spec()
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        handle_patent_spawn_agent(invocation)
            .await
            .map(boxed_tool_output)
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

async fn handle_patent_spawn_agent(
    invocation: ToolInvocation,
) -> Result<PatentSpawnResult, FunctionCallError> {
    let ToolInvocation {
        session,
        turn,
        payload,
        ..
    } = invocation;
    let arguments = function_arguments(payload)?;
    let args: PatentSpawnArgs = parse_arguments(&arguments)?;

    // Validate the role.
    let role = PatentAgentRole::from_str(&args.role).ok_or_else(|| {
        let valid_roles: Vec<&str> = PatentAgentRole::all().iter().map(|r| r.role_id()).collect();
        FunctionCallError::RespondToModel(format!(
            "Unknown patent agent role `{}`. Valid roles: {}",
            args.role,
            valid_roles.join(", ")
        ))
    })?;

    let role_id = role.role_id();
    let task_name = format!("{role_id}-{}", args.task_name);

    // Build initial operation.
    let initial_operation = parse_collab_input(Some(args.message), /*items*/ None)?;

    // Compute depth.
    let session_source = turn.session_source.clone();
    let child_depth = next_thread_spawn_depth(&session_source);

    // Build config from parent turn.
    let mut config =
        build_agent_spawn_config(&session.get_base_instructions().await, turn.as_ref())?;

    // Apply model/reasoning overrides if provided.
    apply_requested_spawn_agent_model_overrides(
        &session,
        turn.as_ref(),
        &mut config,
        args.model.as_deref(),
        None,
    )
    .await?;

    // Apply service tier.
    apply_spawn_agent_service_tier(
        &session,
        &mut config,
        turn.config.service_tier.as_deref(),
        args.service_tier.as_deref(),
    )
    .await?;

    // Apply the patent role as the agent role.
    apply_role_to_config(&mut config, Some(role_id))
        .await
        .map_err(FunctionCallError::RespondToModel)?;

    // Apply runtime overrides.
    apply_spawn_agent_runtime_overrides(&mut config, turn.as_ref())?;
    apply_spawn_agent_overrides(&mut config, child_depth);

    // Set the role on the spawn source.
    let spawn_source = thread_spawn_source(
        session.conversation_id,
        &turn.session_source,
        child_depth,
        Some(role_id),
        Some(task_name.clone()),
    )?;

    // Spawn via AgentControl.
    let spawned = Box::pin(session.services.agent_control.spawn_agent_with_metadata(
        config,
        initial_operation,
        Some(spawn_source),
        SpawnAgentOptions {
            fork_parent_spawn_call_id: None,
            fork_mode: None,
            environments: Some(turn.environments.to_selections()),
        },
    ))
    .await
    .map_err(collab_spawn_error)?;

    // Extract metadata for the response.
    let agent_snapshot = session
        .services
        .agent_control
        .get_agent_config_snapshot(spawned.thread_id)
        .await;

    let new_agent_path = agent_snapshot
        .as_ref()
        .and_then(|s| s.session_source.get_agent_path().map(String::from))
        .or_else(|| spawned.metadata.agent_path.map(String::from))
        .ok_or_else(|| {
            FunctionCallError::RespondToModel(
                "spawned patent agent is missing a canonical task name".to_string(),
            )
        })?;

    Ok(PatentSpawnResult {
        task_name: new_agent_path,
        role: role_id.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Args / Result types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatentSpawnArgs {
    /// The patent agent role to spawn (e.g. "retriever", "analyzer").
    role: String,
    /// The initial task message for the agent.
    message: String,
    /// Short name appended to the canonical task path.
    task_name: String,
    /// Optional model override.
    model: Option<String>,
    /// Optional service tier override.
    service_tier: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct PatentSpawnResult {
    task_name: String,
    role: String,
}

impl ToolOutput for PatentSpawnResult {
    fn log_preview(&self) -> String {
        tool_output_json_text(self, "patent_spawn_agent")
    }

    fn success_for_logging(&self) -> bool {
        true
    }

    fn to_response_item(&self, call_id: &str, payload: &ToolPayload) -> ResponseInputItem {
        tool_output_response_item(call_id, payload, self, Some(true), "patent_spawn_agent")
    }

    fn code_mode_result(&self, _payload: &ToolPayload) -> JsonValue {
        tool_output_code_mode_result(self, "patent_spawn_agent")
    }
}

// ---------------------------------------------------------------------------
// Tool spec
// ---------------------------------------------------------------------------

fn create_patent_spawn_agent_spec() -> ToolSpec {
    let role_enum: Vec<JsonValue> = PatentAgentRole::all()
        .iter()
        .map(|r| JsonValue::String(r.role_id().to_string()))
        .collect();

    let mut properties = BTreeMap::new();
    properties.insert(
        "role".to_string(),
        JsonSchema::string_enum(
            role_enum,
            Some(
                "Patent agent role to spawn. Each role has specialized tools and prompts."
                    .to_string(),
            ),
        ),
    );
    properties.insert(
        "message".to_string(),
        JsonSchema::string(Some(
            "Initial task description for the patent agent.".to_string(),
        )),
    );
    properties.insert(
        "task_name".to_string(),
        JsonSchema::string(Some(
            "Short task identifier (lowercase letters, digits, underscores). Appended to role prefix.".to_string(),
        )),
    );
    properties.insert(
        "model".to_string(),
        JsonSchema::string(Some(
            "Optional model override. Inherits parent model by default.".to_string(),
        )),
    );
    properties.insert(
        "service_tier".to_string(),
        JsonSchema::string(Some("Optional service tier override.".to_string())),
    );

    ToolSpec::Function(ResponsesApiTool {
        name: "patent_spawn_agent".to_string(),
        description: concat!(
            "Spawn a specialized patent sub-agent with a specific role. ",
            "Each role (retriever, analyzer, writer, novelty_checker, creativity_checker, ",
            "infringement_checker, invalidity_checker, reviewer, quality_checker) ",
            "has tailored tools and domain expertise. ",
            "The agent inherits your current model unless overridden."
        )
        .to_string(),
        strict: false,
        defer_loading: None,
        parameters: JsonSchema::object(
            properties,
            Some(vec![
                "role".to_string(),
                "message".to_string(),
                "task_name".to_string(),
            ]),
            Some(false.into()),
        ),
        output_schema: Some(serde_json::json!({
            "type": "object",
            "properties": {
                "task_name": {
                    "type": "string",
                    "description": "Canonical task name for the spawned patent agent."
                },
                "role": {
                    "type": "string",
                    "description": "The patent agent role that was spawned."
                }
            },
            "required": ["task_name", "role"],
            "additionalProperties": false
        })),
    })
}
