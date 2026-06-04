//! SSE stream parser for Chat Completions API responses.
//!
//! Converts Chat Completions SSE chunks into the internal [`ResponseEvent`] stream,
//! matching the same interface as the Responses API stream.

use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::endpoint::chat_conversions::ChatCompletionsChunk;
use crate::error::ApiError;
use crate::telemetry::SseTelemetry;
use codex_client::ByteStream;
use codex_client::StreamResponse;
use codex_protocol::models::ReasoningItemContent;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::TokenUsage;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tokio::time::timeout;
use tracing::debug;
use tracing::trace;

/// Spawns a background task that reads Chat Completions SSE events and converts
/// them to [`ResponseEvent`]s on the returned [`ResponseStream`].
pub fn spawn_chat_completions_stream(
    stream_response: StreamResponse,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
    turn_state: Option<Arc<OnceLock<String>>>,
) -> ResponseStream {
    if let Some(turn_state) = turn_state.as_ref()
        && let Some(header_value) = stream_response
            .headers
            .get("x-codex-turn-state")
            .and_then(|v| v.to_str().ok())
    {
        let _ = turn_state.set(header_value.to_string());
    }
    let upstream_request_id = stream_response
        .headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);
    tokio::spawn(async move {
        process_chat_completions_sse(stream_response.bytes, tx_event, idle_timeout, telemetry)
            .await;
    });

    ResponseStream {
        rx_event,
        upstream_request_id,
    }
}

/// Accumulated state for tool calls being streamed incrementally.
#[derive(Default)]
struct ToolCallAccumulator {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

async fn process_chat_completions_sse(
    stream: ByteStream,
    tx_event: mpsc::Sender<Result<ResponseEvent, ApiError>>,
    idle_timeout: Duration,
    telemetry: Option<Arc<dyn SseTelemetry>>,
) {
    let mut stream = stream.eventsource();
    let mut tool_calls: Vec<ToolCallAccumulator> = Vec::new();
    let mut response_id = String::from("chat_unknown");
    let mut text_item_added = false;
    let mut reasoning_buffer = String::new();
    let mut reasoning_item_added = false;

    loop {
        let start = Instant::now();
        let response = timeout(idle_timeout, stream.next()).await;
        if let Some(t) = telemetry.as_ref() {
            t.on_sse_poll(&response, start.elapsed());
        }
        let sse = match response {
            Ok(Some(Ok(sse))) => sse,
            Ok(Some(Err(e))) => {
                debug!("Chat SSE Error: {e:#}");
                let _ = tx_event.send(Err(ApiError::Stream(e.to_string()))).await;
                return;
            }
            Ok(None) => {
                // Stream ended without a finish_reason → treat as completed
                emit_reasoning_item_done(
                    &mut reasoning_buffer,
                    &mut reasoning_item_added,
                    &tx_event,
                )
                .await;
                emit_pending_tool_calls(&mut tool_calls, &tx_event).await;
                emit_text_item_done(&mut text_item_added, &tx_event).await;
                let _ = tx_event
                    .send(Ok(ResponseEvent::Completed {
                        response_id,
                        token_usage: None,
                        end_turn: Some(true),
                    }))
                    .await;
                return;
            }
            Err(_) => {
                let _ = tx_event
                    .send(Err(ApiError::Stream(
                        "idle timeout waiting for Chat SSE".into(),
                    )))
                    .await;
                return;
            }
        };

        trace!("Chat SSE event: {}", &sse.data);

        // Skip keep-alive comments or empty data
        if sse.data.trim().is_empty() || sse.data == "[DONE]" {
            if sse.data == "[DONE]" {
                // Emit any pending tool calls before completing
                emit_reasoning_item_done(
                    &mut reasoning_buffer,
                    &mut reasoning_item_added,
                    &tx_event,
                )
                .await;
                emit_pending_tool_calls(&mut tool_calls, &tx_event).await;
                emit_text_item_done(&mut text_item_added, &tx_event).await;
                let _ = tx_event
                    .send(Ok(ResponseEvent::Completed {
                        response_id,
                        token_usage: None,
                        end_turn: Some(true),
                    }))
                    .await;
                return;
            }
            continue;
        }

        let chunk: ChatCompletionsChunk = match serde_json::from_str(&sse.data) {
            Ok(chunk) => chunk,
            Err(e) => {
                debug!("Failed to parse Chat SSE chunk: {e}, data: {}", &sse.data);
                continue;
            }
        };

        if !chunk.id.is_empty() {
            response_id = chunk.id;
        }

        for choice in chunk.choices {
            if let Some(reasoning) = &choice.delta.reasoning_content
                && !reasoning.is_empty()
            {
                if !reasoning_item_added {
                    reasoning_item_added = true;
                    if tx_event
                        .send(Ok(ResponseEvent::OutputItemAdded(
                            ResponseItem::Reasoning {
                                id: format!("chat-reasoning-{response_id}"),
                                summary: vec![],
                                content: None,
                                encrypted_content: None,
                            },
                        )))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                reasoning_buffer.push_str(reasoning);
                if tx_event
                    .send(Ok(ResponseEvent::ReasoningContentDelta {
                        delta: reasoning.clone(),
                        content_index: 0,
                    }))
                    .await
                    .is_err()
                {
                    return;
                }
            }

            // Text delta
            if let Some(content) = &choice.delta.content {
                if !text_item_added {
                    text_item_added = true;
                    if tx_event
                        .send(Ok(ResponseEvent::OutputItemAdded(ResponseItem::Message {
                            id: None,
                            role: "assistant".into(),
                            content: vec![],
                            phase: None,
                        })))
                        .await
                        .is_err()
                    {
                        return;
                    }
                }
                if tx_event
                    .send(Ok(ResponseEvent::OutputTextDelta(content.clone())))
                    .await
                    .is_err()
                {
                    return;
                }
            }

            // Tool call deltas
            if let Some(tc_deltas) = &choice.delta.tool_calls {
                for tc_delta in tc_deltas {
                    let idx = tc_delta.index;
                    // Ensure accumulator exists
                    if idx >= tool_calls.len() {
                        tool_calls.resize_with(idx + 1, ToolCallAccumulator::default);
                    }
                    let acc = &mut tool_calls[idx];
                    if let Some(id) = &tc_delta.id {
                        acc.id = Some(id.clone());
                    }
                    if let Some(func) = &tc_delta.function {
                        if let Some(name) = &func.name {
                            acc.name = Some(name.clone());
                        }
                        if let Some(args) = &func.arguments {
                            acc.arguments.push_str(args);
                        }
                    }
                }
            }

            // Finish
            if let Some(reason) = &choice.finish_reason {
                emit_reasoning_item_done(
                    &mut reasoning_buffer,
                    &mut reasoning_item_added,
                    &tx_event,
                )
                .await;
                // Emit pending tool calls
                emit_pending_tool_calls(&mut tool_calls, &tx_event).await;
                emit_text_item_done(&mut text_item_added, &tx_event).await;

                let end_turn = match reason.as_str() {
                    "tool_calls" | "function_call" => Some(false),
                    _ => Some(true),
                };

                let token_usage = chunk.usage.as_ref().map(|u| TokenUsage {
                    input_tokens: u.prompt_tokens,
                    cached_input_tokens: 0,
                    output_tokens: u.completion_tokens,
                    reasoning_output_tokens: 0,
                    total_tokens: u.total_tokens,
                });

                let _ = tx_event
                    .send(Ok(ResponseEvent::Completed {
                        response_id: response_id.clone(),
                        token_usage,
                        end_turn,
                    }))
                    .await;
                return;
            }
        }
    }
}

async fn emit_reasoning_item_done(
    reasoning_buffer: &mut String,
    reasoning_item_added: &mut bool,
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    if !*reasoning_item_added {
        return;
    }
    *reasoning_item_added = false;
    let text = std::mem::take(reasoning_buffer);
    let content = if text.is_empty() {
        None
    } else {
        Some(vec![ReasoningItemContent::Text { text }])
    };
    let item = ResponseItem::Reasoning {
        id: "chat-reasoning".to_string(),
        summary: vec![],
        content,
        encrypted_content: None,
    };
    let _ = tx_event
        .send(Ok(ResponseEvent::OutputItemDone(item)))
        .await;
}

async fn emit_pending_tool_calls(
    tool_calls: &mut Vec<ToolCallAccumulator>,
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    for acc in tool_calls.drain(..) {
        let call_id = acc.id.unwrap_or_default();
        let name = acc.name.unwrap_or_default();
        let arguments = acc.arguments;

        // Emit as FunctionCall item
        let item = ResponseItem::FunctionCall {
            id: None,
            name,
            namespace: None,
            arguments,
            call_id,
        };
        if tx_event
            .send(Ok(ResponseEvent::OutputItemDone(item)))
            .await
            .is_err()
        {
            return;
        }
    }
}

async fn emit_text_item_done(
    text_item_added: &mut bool,
    tx_event: &mpsc::Sender<Result<ResponseEvent, ApiError>>,
) {
    if !*text_item_added {
        return;
    }
    *text_item_added = false;
    let _ = tx_event
        .send(Ok(ResponseEvent::OutputItemDone(ResponseItem::Message {
            id: None,
            role: "assistant".into(),
            content: vec![],
            phase: None,
        })))
        .await;
}
