use super::*;
use codex_app_server_protocol::AgentCancelParams;
use codex_app_server_protocol::AgentCancelResponse;
use codex_app_server_protocol::AgentListResponse;
use codex_app_server_protocol::AgentSpawnParams;
use codex_app_server_protocol::AgentSpawnResponse;
use codex_app_server_protocol::AgentStatusParams;
use codex_app_server_protocol::AgentStatusResponse;

#[derive(Clone)]
pub(crate) struct AgentRuntimeRequestProcessor;

impl AgentRuntimeRequestProcessor {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn agent_spawn(
        &self,
        params: AgentSpawnParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let input = codex_patent_agents::AgentSpawnInput {
            description: params.description,
            prompt: params.prompt,
            subagent_type: params.subagent_type,
            name: params.name,
            model: params.model,
        };

        let manifest = codex_patent_agents::PatentAgentRuntime::spawn_agent(input)
            .map_err(|err| invalid_request(format!("agent spawn failed: {err}")))?;

        Ok(Some(
            AgentSpawnResponse {
                agent_id: manifest.agent_id,
                status: manifest.status,
            }
            .into(),
        ))
    }

    pub(crate) fn agent_status(
        &self,
        params: AgentStatusParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let manifest = codex_patent_agents::PatentAgentRuntime::get_agent_status(&params.agent_id)
            .map_err(|err| invalid_request(format!("agent status fetch failed: {err}")))?;

        Ok(Some(
            AgentStatusResponse {
                agent_id: manifest.agent_id,
                name: manifest.name,
                status: manifest.status,
                model: Some(manifest.model),
                output_file: Some(manifest.output_file.to_string_lossy().to_string()),
                error: manifest.error,
            }
            .into(),
        ))
    }

    pub(crate) fn agent_list(&self) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        let manifests = codex_patent_agents::PatentAgentRuntime::list_agents()
            .map_err(|err| invalid_request(format!("agent list failed: {err}")))?;

        let agents = manifests
            .into_iter()
            .map(|manifest| AgentStatusResponse {
                agent_id: manifest.agent_id,
                name: manifest.name,
                status: manifest.status,
                model: Some(manifest.model),
                output_file: Some(manifest.output_file.to_string_lossy().to_string()),
                error: manifest.error,
            })
            .collect();

        Ok(Some(AgentListResponse { agents }.into()))
    }

    pub(crate) fn agent_cancel(
        &self,
        params: AgentCancelParams,
    ) -> Result<Option<ClientResponsePayload>, JSONRPCErrorError> {
        codex_patent_agents::PatentAgentRuntime::cancel_agent(&params.agent_id)
            .map_err(|err| invalid_request(format!("agent cancel failed: {err}")))?;

        Ok(Some(
            AgentCancelResponse {
                cancelled: true,
                agent_id: params.agent_id,
            }
            .into(),
        ))
    }
}
