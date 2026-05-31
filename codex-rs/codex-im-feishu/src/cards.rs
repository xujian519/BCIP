#[allow(dead_code)]
pub struct CardBuilder {
    header: Option<CardHeader>,
    elements: Vec<CardElement>,
}

#[allow(dead_code)]
pub struct CardHeader {
    pub title: String,
    pub template: Option<String>,
}

#[allow(dead_code)]
pub enum CardElement {
    Markdown(String),
    Divider,
    Field { label: String, value: String },
    Action(Vec<CardAction>),
}

#[allow(dead_code)]
pub struct CardAction {
    pub text: String,
    pub value: String,
    pub action_type: ActionType,
}

#[allow(dead_code)]
pub enum ActionType {
    Primary,
    Default,
    Danger,
}

#[allow(dead_code)]
impl CardBuilder {
    pub fn new() -> Self {
        Self {
            header: None,
            elements: Vec::new(),
        }
    }

    pub fn with_header(mut self, title: impl Into<String>) -> Self {
        self.header = Some(CardHeader {
            title: title.into(),
            template: None,
        });
        self
    }

    pub fn add_markdown(mut self, text: impl Into<String>) -> Self {
        self.elements.push(CardElement::Markdown(text.into()));
        self
    }

    pub fn add_divider(mut self) -> Self {
        self.elements.push(CardElement::Divider);
        self
    }

    pub fn add_action(
        mut self,
        text: impl Into<String>,
        value: impl Into<String>,
        action_type: ActionType,
    ) -> Self {
        if let Some(CardElement::Action(actions)) = self.elements.last_mut() {
            actions.push(CardAction {
                text: text.into(),
                value: value.into(),
                action_type,
            });
        } else {
            self.elements.push(CardElement::Action(vec![CardAction {
                text: text.into(),
                value: value.into(),
                action_type,
            }]));
        }
        self
    }

    pub fn build(&self) -> serde_json::Value {
        let mut body = serde_json::json!({
            "config": {
                "wide_screen_mode": true
            },
            "elements": []
        });

        if let Some(header) = &self.header {
            body["header"] = serde_json::json!({
                "title": {
                    "tag": "plain_text",
                    "content": header.title
                }
            });
        }

        let mut elements = Vec::new();
        for elem in &self.elements {
            match elem {
                CardElement::Markdown(text) => {
                    elements.push(serde_json::json!({
                        "tag": "markdown",
                        "content": text
                    }));
                }
                CardElement::Divider => {
                    elements.push(serde_json::json!({
                        "tag": "hr"
                    }));
                }
                CardElement::Field { label, value } => {
                    elements.push(serde_json::json!({
                        "tag": "div",
                        "fields": [
                            {
                                "is_short": true,
                                "text": {
                                    "tag": "lark_md",
                                    "content": format!("**{label}**")
                                }
                            },
                            {
                                "is_short": true,
                                "text": {
                                    "tag": "lark_md",
                                    "content": value
                                }
                            }
                        ]
                    }));
                }
                CardElement::Action(actions) => {
                    let mut action_list = Vec::new();
                    for action in actions {
                        let button_type = match action.action_type {
                            ActionType::Primary => "primary",
                            ActionType::Default => "default",
                            ActionType::Danger => "danger",
                        };
                        action_list.push(serde_json::json!({
                            "tag": "button",
                            "text": {
                                "tag": "plain_text",
                                "content": action.text
                            },
                            "type": button_type,
                            "value": action.value
                        }));
                    }
                    elements.push(serde_json::json!({
                        "tag": "action",
                        "actions": action_list
                    }));
                }
            }
        }

        body["elements"] = serde_json::json!(elements);
        body
    }
}

impl Default for CardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_builder() {
        let card = CardBuilder::new()
            .with_header("权限请求")
            .add_markdown("请批准以下操作")
            .add_divider()
            .add_action("允许", "allow", ActionType::Primary)
            .add_action("拒绝", "deny", ActionType::Danger)
            .build();

        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("权限请求"));
        assert!(json.contains("允许"));
    }
}
