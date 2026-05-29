use crate::function_tool::FunctionCallError;
use crate::tools::context::ToolInvocation;
use crate::tools::context::ToolOutput;
use crate::tools::context::ToolPayload;
use crate::tools::registry::CoreToolRuntime;
use crate::tools::registry::ToolExecutor;
use codex_protocol::openai_models::InputModality;
use codex_protocol::openai_models::OutputModality;
use codex_protocol::openai_models::ToolType;
use codex_tools::JsonSchema;
use codex_tools::JsonSchemaType;
use codex_tools::ResponsesApiTool;
use codex_tools::ToolName;
use codex_tools::ToolSpec;
use codex_tools::AdditionalProperties;
use std::sync::Arc;

#[cfg(feature = "patent-tools")]
type PatentHandler = fn(
    serde_json::Value,
) -> std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send>,
>;

#[cfg(feature = "patent-tools")]
pub struct PatentToolAdapter {
    name: ToolName,
    spec: ToolSpec,
    handler: PatentHandler,
}

#[cfg(feature = "patent-tools")]
impl PatentToolAdapter {
    pub fn new(name: ToolName, spec: ToolSpec, handler: PatentHandler) -> Self {
        Self { name, spec, handler }
    }

    pub fn create_all_adapters() -> Vec<Arc<dyn CoreToolRuntime>> {
        let search_tools = codex_patent_tools::register_search_tools();
        let mut adapters = Vec::new();

        for (tool_name, handler) in search_tools {
            let adapter = Self::create_adapter_for_tool(&tool_name, handler);
            adapters.push(Arc::new(adapter));
        }

        adapters
    }

    fn create_adapter_for_tool(tool_name: &str, handler: PatentHandler) -> Self {
        let tool_name = ToolName::plain(tool_name.to_string());
        let spec = Self::create_spec_for_tool(tool_name.name.as_str());
        Self::new(tool_name, spec, handler)
    }

    fn create_spec_for_tool(tool_name: &str) -> ToolSpec {
        let (description, parameters_schema) = match tool_name {
            "PatentSearch" => (
                "Search patents using the local patent database with 75+ million Chinese patents. Supports keyword, applicant, and IPC classification searches with millisecond response time.",
                create_patent_search_schema(),
            ),
            "GooglePatentsFetch" => (
                "Fetch detailed patent documents from Google Patents database using patent numbers or search queries.",
                create_google_patents_fetch_schema(),
            ),
            "SearchQueryBuilder" => (
                "Build optimized patent search queries using natural language descriptions. Supports Boolean operators, field-specific searches, and query refinement.",
                create_search_query_builder_schema(),
            ),
            "IterativeSearch" => (
                "Perform iterative patent searches with automatic query refinement based on results. Supports feedback-driven search optimization.",
                create_iterative_search_schema(),
            ),
            "PatentDownload" => (
                "Download patent documents as PDF files from Google Patents. Supports batch download for multiple patents.",
                create_patent_download_schema(),
            ),
            _ => (
                &format!("Patent tool: {tool_name}"),
                JsonSchema::default(),
            ),
        };

        ToolSpec::Function(ResponsesApiTool {
            name: tool_name.to_string(),
            description: description.to_string(),
            strict: true,
            parameters: parameters_schema,
            output_schema: None,
            defer_loading: None,
        })
    }

    async fn execute_handler(&self, arguments: serde_json::Value) -> Result<serde_json::Value, FunctionCallError> {
        (self.handler)(arguments)
            .await
            .map_err(|e| FunctionCallError::RespondToModel(format!("Patent tool error: {e}")))
    }
}

#[cfg(feature = "patent-tools")]
#[async_trait::async_trait]
impl ToolExecutor<ToolInvocation> for PatentToolAdapter {
    fn tool_name(&self) -> ToolName {
        self.name.clone()
    }

    fn spec(&self) -> ToolSpec {
        self.spec.clone()
    }

    fn exposure(&self) -> crate::tools::registry::ToolExposure {
        crate::tools::registry::ToolExposure::Direct
    }

    fn supports_parallel_tool_calls(&self) -> bool {
        true
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn ToolOutput>, FunctionCallError> {
        let ToolPayload::Function { arguments } = invocation.payload else {
            return Err(FunctionCallError::RespondToModel(
                format!("Invalid payload for {}: expected Function", self.name.name),
            ));
        };

        let arguments_value: serde_json::Value = serde_json::from_str(&arguments)
            .map_err(|e| FunctionCallError::RespondToModel(format!("Invalid arguments JSON: {e}")))?;

        let result = self.execute_handler(arguments_value).await?;

        Ok(Box::new(codex_tools::JsonToolOutput::new(result)))
    }
}

#[cfg(feature = "patent-tools")]
impl CoreToolRuntime for PatentToolAdapter {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}

fn create_patent_search_schema() -> JsonSchema {
    JsonSchema {
        r#type: Some(JsonSchemaType::Object),
        description: Some("Patent search parameters".to_string()),
        properties: Some(vec![
            ("query".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Search query for patents (keywords, applicant name, or classification)".to_string()),
                ..Default::default()
            }),
            ("limit".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::Integer),
                description: Some("Maximum number of results to return".to_string()),
                ..Default::default()
            }),
        ].into_iter().collect()),
        required: Some(vec!["query".to_string()]),
        additional_properties: Some(AdditionalProperties::Bool(false)),
        ..Default::default()
    }
}

fn create_google_patents_fetch_schema() -> JsonSchema {
    JsonSchema {
        r#type: Some(JsonSchemaType::Object),
        description: Some("Google Patents fetch parameters".to_string()),
        properties: Some(vec![
            ("patent_number".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Patent number to fetch (e.g., CN101234567A, US1234567)".to_string()),
                ..Default::default()
            }),
            ("jurisdiction".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Patent jurisdiction (CN, US, EP, WO)".to_string()),
                ..Default::default()
            }),
        ].into_iter().collect()),
        required: Some(vec!["patent_number".to_string()]),
        additional_properties: Some(AdditionalProperties::Bool(false)),
        ..Default::default()
    }
}

fn create_search_query_builder_schema() -> JsonSchema {
    JsonSchema {
        r#type: Some(JsonSchemaType::Object),
        description: Some("Search query builder parameters".to_string()),
        properties: Some(vec![
            ("description".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Natural language description of the search intent".to_string()),
                ..Default::default()
            }),
            ("field".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Specific field to search (title, abstract, claims, applicant)".to_string()),
                ..Default::default()
            }),
        ].into_iter().collect()),
        required: Some(vec!["description".to_string()]),
        additional_properties: Some(AdditionalProperties::Bool(false)),
        ..Default::default()
    }
}

fn create_iterative_search_schema() -> JsonSchema {
    JsonSchema {
        r#type: Some(JsonSchemaType::Object),
        description: Some("Iterative search parameters".to_string()),
        properties: Some(vec![
            ("query".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Initial search query".to_string()),
                ..Default::default()
            }),
            ("max_iterations".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::Integer),
                description: Some("Maximum number of iterations".to_string()),
                ..Default::default()
            }),
        ].into_iter().collect()),
        required: Some(vec!["query".to_string()]),
        additional_properties: Some(AdditionalProperties::Bool(false)),
        ..Default::default()
    }
}

fn create_patent_download_schema() -> JsonSchema {
    JsonSchema {
        r#type: Some(JsonSchemaType::Object),
        description: Some("Patent download parameters".to_string()),
        properties: Some(vec![
            ("patent_number".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Patent number to download".to_string()),
                ..Default::default()
            }),
            ("format".to_string(), JsonSchema {
                r#type: Some(JsonSchemaType::String),
                description: Some("Download format (pdf, txt)".to_string()),
                ..Default::default()
            }),
        ].into_iter().collect()),
        required: Some(vec!["patent_number".to_string()]),
        additional_properties: Some(AdditionalProperties::Bool(false)),
        ..Default::default()
    }
}

#[cfg(test)]
#[cfg(feature = "patent-tools")]
mod tests {
    use super::*;

    #[test]
    fn test_create_adapters() {
        let adapters = PatentToolAdapter::create_all_adapters();
        assert!(!adapters.is_empty());

        let adapter_names: Vec<_> = adapters
            .iter()
            .map(|a| a.tool_name().name.clone())
            .collect();

        assert!(adapter_names.contains(&"PatentSearch".to_string()));
        assert!(adapter_names.contains(&"GooglePatentsFetch".to_string()));
    }
}