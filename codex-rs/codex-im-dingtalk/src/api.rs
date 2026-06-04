use reqwest::Client;
use tracing::info;

use crate::SentMessage;

/// DingTalk OpenAPI client.
#[derive(Debug)]
pub(crate) struct DingtalkApi {
    client: Client,
    app_key: String,
    app_secret: String,
}

impl DingtalkApi {
    pub(crate) fn new(client: Client, app_key: String, app_secret: String) -> Self {
        Self {
            client,
            app_key,
            app_secret,
        }
    }

    /// Get an access token using appKey/appSecret.
    pub(crate) async fn get_access_token(&self) -> Result<String, String> {
        let url = format!(
            "https://oapi.dingtalk.com/gettoken?appkey={}&appsecret={}",
            self.app_key, self.app_secret
        );

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?;

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("解析失败: {e}"))?;

        let errcode = json["errcode"].as_i64().unwrap_or(-1);
        if errcode != 0 {
            let errmsg = json["errmsg"].as_str().unwrap_or("未知错误");
            return Err(format!("钉钉 API 错误 [{errcode}]: {errmsg}"));
        }

        json["access_token"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "响应中缺少 access_token".to_string())
    }

    /// Send a text message to a conversation.
    pub(crate) async fn send_message(
        &self,
        token: &str,
        conversation_id: &str,
        text: &str,
    ) -> Result<SentMessage, String> {
        info!(conversation_id, text_len = text.len(), "发送钉钉消息");

        let url = "https://oapi.dingtalk.com/topapi/message/corpconversation/asyncsend_v2";

        let body = serde_json::json!({
            "agent_id": conversation_id,
            "userid_list": conversation_id,
            "msg": {
                "msgtype": "text",
                "text": {
                    "content": text
                }
            }
        });

        let resp = self
            .client
            .post(url)
            .query(&[("access_token", token)])
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("请求失败: {e}"))?;

        let json: serde_json::Value = resp.json().await.map_err(|e| format!("解析失败: {e}"))?;

        let errcode = json["errcode"].as_i64().unwrap_or(-1);
        if errcode != 0 {
            let errmsg = json["errmsg"].as_str().unwrap_or("未知错误");
            return Err(format!("发送失败 [{errcode}]: {errmsg}"));
        }

        let task_id = json["task_id"].as_i64().unwrap_or(0).to_string();

        Ok(SentMessage {
            message_id: task_id,
        })
    }
}
