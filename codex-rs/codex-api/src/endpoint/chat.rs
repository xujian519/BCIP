//! Chat Completions API client for Chinese LLM providers.
//!
//! Mirrors [`ResponsesClient`] but uses the `/chat/completions` endpoint and
//! converts request/response formats via [`chat_conversions`].

use crate::auth::SharedAuthProvider;
use crate::common::ResponseStream;
use crate::common::ResponsesApiRequest;
use crate::endpoint::chat_conversions;
use crate::endpoint::session::EndpointSession;
use crate::error::ApiError;
use crate::sse::chat_completions::spawn_chat_completions_stream;
use crate::telemetry::SseTelemetry;
use codex_client::HttpTransport;
use codex_client::RequestCompression;
use codex_client::RequestTelemetry;
use http::HeaderMap;
use http::Method;
use serde_json::Value;
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::instrument;

pub struct ChatClient<T: HttpTransport> {
    session: EndpointSession<T>,
    sse_telemetry: Option<Arc<dyn SseTelemetry>>,
}

#[derive(Default)]
pub struct ChatOptions {
    pub session_id: Option<String>,
    pub thread_id: Option<String>,
    pub extra_headers: HeaderMap,
    pub turn_state: Option<Arc<OnceLock<String>>>,
}

impl<T: HttpTransport> ChatClient<T> {
    pub fn new(
        transport: T,
        provider: crate::provider::Provider,
        auth: SharedAuthProvider,
    ) -> Self {
        Self {
            session: EndpointSession::new(transport, provider, auth),
            sse_telemetry: None,
        }
    }

    pub fn with_telemetry(
        self,
        request: Option<Arc<dyn RequestTelemetry>>,
        sse: Option<Arc<dyn SseTelemetry>>,
    ) -> Self {
        Self {
            session: self.session.with_request_telemetry(request),
            sse_telemetry: sse,
        }
    }

    #[instrument(
        name = "chat.stream_request",
        level = "info",
        skip_all,
        fields(
            transport = "chat_http",
            http.method = "POST",
            api.path = "chat/completions"
        )
    )]
    pub async fn stream_request(
        &self,
        request: ResponsesApiRequest,
        options: ChatOptions,
    ) -> Result<ResponseStream, ApiError> {
        let body = chat_conversions::convert_request(&request);
        self.stream(body, options).await
    }

    fn path() -> &'static str {
        "chat/completions"
    }

    #[instrument(
        name = "chat.stream",
        level = "info",
        skip_all,
        fields(
            transport = "chat_http",
            http.method = "POST",
            api.path = "chat/completions"
        )
    )]
    pub async fn stream(
        &self,
        body: Value,
        options: ChatOptions,
    ) -> Result<ResponseStream, ApiError> {
        let ChatOptions {
            extra_headers,
            turn_state,
            ..
        } = options;

        let stream_response = self
            .session
            .stream_with(
                Method::POST,
                Self::path(),
                extra_headers,
                Some(body),
                |req| {
                    req.compression = RequestCompression::None;
                },
            )
            .await?;

        let idle_timeout = self.session.provider().stream_idle_timeout;
        Ok(spawn_chat_completions_stream(
            stream_response,
            idle_timeout,
            self.sse_telemetry.clone(),
            turn_state,
        ))
    }
}
