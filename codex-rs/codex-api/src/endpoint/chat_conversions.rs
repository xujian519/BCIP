//! Converts between the internal Responses API request format and the
//! OpenAI Chat Completions API format used by Chinese LLM providers
//! (DeepSeek, ZhiPu GLM, Kimi/Moonshot).

use codex_protocol::models::ContentItem;
use codex_protocol::models::ResponseItem;
use serde::Deserialize;
use serde_json::Value;

use crate::common::ResponsesApiRequest;

/// Converts a [`ResponsesApiRequest`] into a Chat Completions API request body.
pub fn convert_request(request: &ResponsesApiRequest) -> Value {
    let mut messages = Vec::new();

    // System instructions → system message
    if !request.instructions.is_empty() {
        messages.push(serde_json::json!({
            "role": "system",
            "content": request.instructions,
        }));
    }

    // Convert ResponseItem input to chat messages
    let mut pending_assistant: Option<Value> = None;
    for item in &request.input {
        match item {
            ResponseItem::Message { role, content, .. } => {
                // Flush any pending assistant message before adding a new one
                if let Some(msg) = pending_assistant.take() {
                    messages.push(msg);
                }
                let chat_content = convert_content_items(content);
                // Map Roles API `developer` to Chat Completions `system` for
                // Chinese LLM providers (DeepSeek, ZhiPu, Kimi) that don't
                // support the `developer` role.
                let chat_role = if role == "developer" {
                    "system"
                } else {
                    role.as_str()
                };
                messages.push(serde_json::json!({
                    "role": chat_role,
                    "content": chat_content,
                }));
            }
            ResponseItem::FunctionCall {
                name,
                arguments,
                call_id,
                ..
            } => {
                // Accumulate tool_calls into the current assistant message
                let tool_call = serde_json::json!({
                    "id": call_id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": arguments,
                    }
                });
                match &mut pending_assistant {
                    Some(msg) => {
                        msg["tool_calls"].as_array_mut().unwrap().push(tool_call);
                    }
                    None => {
                        pending_assistant = Some(serde_json::json!({
                            "role": "assistant",
                            "content": null,
                            "tool_calls": [tool_call],
                        }));
                    }
                }
            }
            ResponseItem::FunctionCallOutput { call_id, output } => {
                // Flush pending assistant first
                if let Some(msg) = pending_assistant.take() {
                    messages.push(msg);
                }
                let content = output.text_content().unwrap_or_default().to_string();
                messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": content,
                }));
            }
            ResponseItem::CustomToolCall {
                call_id,
                name,
                input,
                ..
            } => {
                let tool_call = serde_json::json!({
                    "id": call_id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": input,
                    }
                });
                match &mut pending_assistant {
                    Some(msg) => {
                        msg["tool_calls"].as_array_mut().unwrap().push(tool_call);
                    }
                    None => {
                        pending_assistant = Some(serde_json::json!({
                            "role": "assistant",
                            "content": null,
                            "tool_calls": [tool_call],
                        }));
                    }
                }
            }
            ResponseItem::CustomToolCallOutput {
                call_id, output, ..
            } => {
                if let Some(msg) = pending_assistant.take() {
                    messages.push(msg);
                }
                let content = output.text_content().unwrap_or_default().to_string();
                messages.push(serde_json::json!({
                    "role": "tool",
                    "tool_call_id": call_id,
                    "content": content,
                }));
            }
            // Skip items without Chat Completions equivalents
            ResponseItem::Reasoning { .. }
            | ResponseItem::LocalShellCall { .. }
            | ResponseItem::ToolSearchCall { .. }
            | ResponseItem::ToolSearchOutput { .. }
            | ResponseItem::WebSearchCall { .. }
            | ResponseItem::ImageGenerationCall { .. }
            | ResponseItem::Compaction { .. }
            | ResponseItem::ContextCompaction { .. }
            | ResponseItem::CompactionTrigger
            | ResponseItem::Other => {}
        }
    }

    // Flush remaining pending assistant message
    if let Some(msg) = pending_assistant.take() {
        messages.push(msg);
    }

    // Ensure every assistant message with tool_calls has corresponding tool responses.
    // Missing outputs can occur after context compaction or interrupted tool calls.
    ensure_tool_responses(&mut messages);

    // Convert tools
    let tools = convert_tools(&request.tools);

    let mut body = serde_json::json!({
        "model": request.model,
        "messages": messages,
        "stream": true,
    });

    if !tools.is_empty() {
        body["tools"] = Value::Array(tools);
        body["tool_choice"] = Value::String(request.tool_choice.clone());
    }

    body
}

/// For every assistant message that contains `tool_calls`, ensure each `tool_call_id`
/// has a matching `tool` role message in the list. Missing responses are filled with
/// "aborted" to satisfy the Chat Completions API constraint.
fn ensure_tool_responses(messages: &mut Vec<Value>) {
    // Collect (index, call_ids) for each assistant message with tool_calls
    let mut assistant_tool_calls: Vec<(usize, Vec<String>)> = Vec::new();
    for (idx, msg) in messages.iter().enumerate() {
        if msg["role"] != "assistant" {
            continue;
        }
        if let Some(calls) = msg["tool_calls"].as_array() {
            let call_ids: Vec<String> = calls
                .iter()
                .filter_map(|c| c["id"].as_str().map(String::from))
                .collect();
            if !call_ids.is_empty() {
                assistant_tool_calls.push((idx, call_ids));
            }
        }
    }

    // Collect all existing tool_call_ids from tool messages
    let existing_tool_ids: std::collections::HashSet<String> = messages
        .iter()
        .filter(|m| m["role"] == "tool")
        .filter_map(|m| m["tool_call_id"].as_str().map(String::from))
        .collect();

    // Insert synthetic tool responses for missing call_ids
    let mut insertions: Vec<(usize, Value)> = Vec::new();
    for (assistant_idx, call_ids) in &assistant_tool_calls {
        for call_id in call_ids {
            if !existing_tool_ids.contains(call_id) {
                insertions.push((
                    *assistant_idx,
                    serde_json::json!({
                        "role": "tool",
                        "tool_call_id": call_id,
                        "content": "aborted",
                    }),
                ));
            }
        }
    }

    // Insert in reverse order to preserve indices
    for (after_idx, tool_msg) in insertions.into_iter().rev() {
        messages.insert(after_idx + 1, tool_msg);
    }
}

fn convert_content_items(items: &[ContentItem]) -> Value {
    if items.len() == 1 {
        // Single text item → plain string (most common case, better compat)
        match &items[0] {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                return Value::String(text.clone());
            }
            ContentItem::InputImage { .. } => {}
        }
    }

    let parts: Vec<Value> = items
        .iter()
        .map(|item| match item {
            ContentItem::InputText { text } | ContentItem::OutputText { text } => {
                serde_json::json!({ "type": "text", "text": text })
            }
            ContentItem::InputImage { image_url, .. } => {
                serde_json::json!({
                    "type": "image_url",
                    "image_url": { "url": image_url }
                })
            }
        })
        .collect();
    Value::Array(parts)
}

fn convert_tools(tools: &[Value]) -> Vec<Value> {
    tools
        .iter()
        .filter_map(|tool| {
            // Responses API function tool format:
            //   {"type": "function", "name": "...", "description": "...", "parameters": {...}}
            // Chat Completions expects:
            //   {"type": "function", "function": {"name": "...", "description": "...", "parameters": {...}}}
            if tool.get("type").and_then(Value::as_str) == Some("function") {
                // If already wrapped in "function" key, pass through
                if tool.get("function").is_some() {
                    return Some(tool.clone());
                }
                // Responses API flat format → wrap in "function" key
                let func: serde_json::Map<String, Value> = tool
                    .as_object()
                    .map(|obj| {
                        obj.iter()
                            .filter(|(k, _)| *k != "type")
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    })
                    .unwrap_or_default();
                if func.is_empty() {
                    return None;
                }
                return Some(serde_json::json!({
                    "type": "function",
                    "function": func,
                }));
            }
            None
        })
        .collect()
}

/// A single chunk from a Chat Completions SSE stream.
#[derive(Debug, Deserialize)]
pub struct ChatCompletionsChunk {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub choices: Vec<ChunkChoice>,
    #[serde(default)]
    pub usage: Option<ChunkUsage>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkChoice {
    #[serde(default)]
    pub delta: ChunkDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
pub struct ChunkDelta {
    pub role: Option<String>,
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ChunkToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkToolCall {
    pub index: usize,
    pub id: Option<String>,
    #[serde(default)]
    pub function: Option<ChunkFunction>,
}

#[derive(Debug, Default, Deserialize)]
pub struct ChunkFunction {
    pub name: Option<String>,
    pub arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChunkUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::models::ResponseItem;

    fn make_request(input: Vec<ResponseItem>) -> ResponsesApiRequest {
        ResponsesApiRequest {
            model: "test-model".to_string(),
            instructions: String::new(),
            input,
            tools: vec![],
            tool_choice: "auto".to_string(),
            parallel_tool_calls: false,
            reasoning: None,
            store: false,
            stream: true,
            include: vec![],
            service_tier: None,
            prompt_cache_key: None,
            text: None,
            client_metadata: None,
        }
    }

    #[test]
    fn converts_simple_user_message() {
        let req = make_request(vec![ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: "Hello".to_string(),
            }],
            phase: None,
        }]);
        let body = convert_request(&req);
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[0]["content"], "Hello");
    }

    #[test]
    fn converts_instructions_to_system() {
        let mut req = make_request(vec![]);
        req.instructions = "You are helpful.".to_string();
        let body = convert_request(&req);
        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are helpful.");
    }

    #[test]
    fn converts_function_call_and_output() {
        let req = make_request(vec![
            ResponseItem::FunctionCall {
                id: None,
                name: "get_weather".to_string(),
                namespace: None,
                arguments: r#"{"city":"Shanghai"}"#.to_string(),
                call_id: "call_123".to_string(),
            },
            ResponseItem::FunctionCallOutput {
                call_id: "call_123".to_string(),
                output: codex_protocol::models::FunctionCallOutputPayload::from_text(
                    "Sunny, 25°C".to_string(),
                ),
            },
        ]);
        let body = convert_request(&req);
        let messages = body["messages"].as_array().unwrap();
        // Should have assistant (with tool_calls) + tool result
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0]["role"], "assistant");
        assert!(messages[0]["tool_calls"].is_array());
        assert_eq!(messages[1]["role"], "tool");
        assert_eq!(messages[1]["tool_call_id"], "call_123");
    }

    #[test]
    fn parses_chat_completions_chunk() {
        let data =
            r#"{"id":"chatcmpl-1","choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunk: ChatCompletionsChunk = serde_json::from_str(data).unwrap();
        assert_eq!(chunk.id, "chatcmpl-1");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].delta.content.as_deref(), Some("Hello"));
    }

    #[test]
    fn parses_tool_call_chunk() {
        let data = r#"{"id":"chatcmpl-2","choices":[{"delta":{"tool_calls":[{"index":0,"id":"call_1","function":{"name":"run","arguments":"{\""}}]}}]}"#;
        let chunk: ChatCompletionsChunk = serde_json::from_str(data).unwrap();
        let tc = &chunk.choices[0].delta.tool_calls.as_ref().unwrap()[0];
        assert_eq!(tc.id.as_deref(), Some("call_1"));
        assert_eq!(tc.function.as_ref().unwrap().name.as_deref(), Some("run"));
    }
}
