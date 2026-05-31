use reqwest::Client;
use serde_json::Value;
use tracing::warn;

#[derive(Debug)]
pub struct TelegramApi {
    client: Client,
    base_url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct SentMessage {
    pub message_id: i64,
}

impl TelegramApi {
    pub fn new(client: Client, bot_token: String) -> Self {
        Self {
            client,
            base_url: format!("https://api.telegram.org/bot{bot_token}"),
        }
    }

    pub async fn get_updates(&self, offset: i64) -> Result<Vec<super::TelegramUpdate>, String> {
        let url = format!("{}/getUpdates", self.base_url);
        let resp: Value = self
            .client
            .get(&url)
            .query(&[
                ("offset", &offset.to_string()),
                ("timeout", &"30".to_string()),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(resp["description"].as_str().unwrap_or("unknown").into());
        }

        let result: Vec<super::TelegramUpdate> =
            serde_json::from_value(resp["result"].clone()).map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<SentMessage, String> {
        let url = format!("{}/sendMessage", self.base_url);
        let resp: Value = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "Markdown",
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(resp["description"].as_str().unwrap_or("unknown").into());
        }

        let result: SentMessage =
            serde_json::from_value(resp["result"].clone()).map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub async fn send_message_with_keyboard(
        &self,
        chat_id: i64,
        text: &str,
        keyboard: &serde_json::Value,
    ) -> Result<SentMessage, String> {
        let url = format!("{}/sendMessage", self.base_url);
        let resp: Value = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "text": text,
                "parse_mode": "Markdown",
                "reply_markup": keyboard,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            return Err(resp["description"].as_str().unwrap_or("unknown").into());
        }

        let result: SentMessage =
            serde_json::from_value(resp["result"].clone()).map_err(|e| e.to_string())?;
        Ok(result)
    }

    pub async fn edit_message_text(
        &self,
        chat_id: i64,
        message_id: i64,
        text: &str,
    ) -> Result<(), String> {
        let url = format!("{}/editMessageText", self.base_url);
        let resp: Value = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "message_id": message_id,
                "text": text,
                "parse_mode": "Markdown",
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            warn!(
                "editMessageText 失败: {}",
                resp["description"].as_str().unwrap_or("unknown")
            );
        }
        Ok(())
    }

    pub async fn delete_message(&self, chat_id: i64, message_id: i64) -> Result<(), String> {
        let url = format!("{}/deleteMessage", self.base_url);
        let resp: Value = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "chat_id": chat_id,
                "message_id": message_id,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;

        if !resp["ok"].as_bool().unwrap_or(false) {
            warn!(
                "deleteMessage 失败: {}",
                resp["description"].as_str().unwrap_or("unknown")
            );
        }
        Ok(())
    }

    pub async fn send_photo(
        &self,
        chat_id: i64,
        _data_base64: &str,
        _mime_type: &str,
    ) -> Result<(), String> {
        let url = format!("{}/sendPhoto", self.base_url);
        let _resp = self
            .client
            .post(&url)
            .form(&serde_json::json!({
                "chat_id": chat_id,
                "caption": "",
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn answer_callback_query(&self, callback_id: &str, text: &str) -> Result<(), String> {
        let url = format!("{}/answerCallbackQuery", self.base_url);
        let _resp: Value = self
            .client
            .post(&url)
            .json(&serde_json::json!({
                "callback_query_id": callback_id,
                "text": text,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_creation() {
        let client = Client::new();
        let api = TelegramApi::new(client, "test_token".into());
        assert!(api.base_url.contains("test_token"));
    }
}
