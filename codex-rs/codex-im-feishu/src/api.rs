use reqwest::Client;
use serde_json::Value;
use tracing::info;

#[derive(Debug)]
pub struct FeishuApi {
    client: Client,
    app_id: String,
    app_secret: String,
}

#[derive(Debug, serde::Deserialize)]
struct TenantTokenResponse {
    code: i32,
    msg: Option<String>,
    tenant_access_token: Option<String>,
}

impl FeishuApi {
    pub fn new(client: Client, app_id: String, app_secret: String) -> Self {
        Self {
            client,
            app_id,
            app_secret,
        }
    }

    pub async fn get_tenant_access_token(&self) -> Result<String, String> {
        let url = "https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal";
        let resp: TenantTokenResponse = self
            .client
            .post(url)
            .json(&serde_json::json!({
                "app_id": self.app_id,
                "app_secret": self.app_secret,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if resp.code != 0 {
            return Err(resp.msg.unwrap_or_else(|| "unknown".into()));
        }

        let token = resp.tenant_access_token.ok_or("no token returned")?;
        info!("飞书 tenant_access_token 已获取");
        Ok(token)
    }

    pub async fn list_messages(&self, token: &str) -> Result<Vec<super::FeishuMessage>, String> {
        let url = "https://open.feishu.cn/open-apis/im/v1/messages";
        let resp: Value = self
            .client
            .get(url)
            .bearer_auth(token)
            .query(&[("page_size", "20")])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if resp["code"].as_i64().unwrap_or(-1) != 0 {
            let msg = resp["msg"].as_str().unwrap_or("unknown");
            return Err(msg.to_string());
        }

        let items = resp["data"]["items"].as_array();

        let mut messages = Vec::new();
        if let Some(items) = items {
            for item in items {
                if let Some(msg) = Self::parse_message_item(item) {
                    messages.push(msg);
                }
            }
        }

        Ok(messages)
    }

    fn parse_message_item(item: &Value) -> Option<super::FeishuMessage> {
        let message_id = item["message_id"].as_str()?.to_string();
        let chat_id = item["chat_id"].as_str()?.to_string();
        let sender_id = item["sender"]["id"]["user_id"]
            .as_str()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let content_str = item["body"]["content"].as_str().unwrap_or("");
        let content = if content_str.is_empty() {
            None
        } else {
            serde_json::from_str::<Value>(content_str)
                .ok()
                .and_then(|v| v["text"].as_str().map(|s| s.to_string()))
        };

        Some(super::FeishuMessage {
            message_id,
            chat_id,
            sender_id,
            content,
        })
    }

    pub async fn send_message(
        &self,
        token: &str,
        chat_id: &str,
        text: &str,
    ) -> Result<super::SentMessage, String> {
        let content = serde_json::json!({
            "text": text
        });

        let url = "https://open.feishu.cn/open-apis/im/v1/messages";
        let body = serde_json::json!({
            "receive_id": chat_id,
            "msg_type": "text",
            "content": content.to_string(),
        });

        let resp: Value = self
            .client
            .post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if resp["code"].as_i64().unwrap_or(-1) != 0 {
            let msg = resp["msg"].as_str().unwrap_or("unknown");
            return Err(msg.to_string());
        }

        let message_id = resp["data"]["message_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();

        Ok(super::SentMessage { message_id })
    }
}
