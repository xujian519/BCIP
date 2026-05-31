//! 任务定义。

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    AgentCall,
    ToolCall,
    QualityCheck,
    HumanApproval,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub task_type: TaskType,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
}

impl Task {
    pub fn new(id: impl Into<String>, name: impl Into<String>, task_type: TaskType) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            task_type,
            input: serde_json::json!({}),
        }
    }

    pub fn with_input(mut self, input: serde_json::Value) -> Self {
        self.input = input;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("task-1", "分析任务", TaskType::AgentCall);
        assert_eq!(task.id, "task-1");
        assert_eq!(task.name, "分析任务");
        assert!(matches!(task.task_type, TaskType::AgentCall));
    }

    #[test]
    fn test_task_with_input() {
        let input = serde_json::json!({"prompt": "分析专利"});
        let task = Task::new("task-2", "检索任务", TaskType::ToolCall).with_input(input.clone());
        assert_eq!(task.input, input);
    }

    #[test]
    fn test_task_serialize() {
        let task = Task::new("task-3", "质量检查", TaskType::QualityCheck);
        let json = serde_json::to_string(&task).unwrap();
        println!("Serialized: {}", json);
        assert!(json.contains("task-3"));
        assert!(json.contains("quality_check"));
    }

    #[test]
    fn test_task_deserialize() {
        let json = r#"{
            "id": "task-4",
            "name": "审批任务",
            "type": "human_approval",
            "input": {"title": "需要审批"}
        }"#;
        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.id, "task-4");
        assert_eq!(task.name, "审批任务");
        assert!(matches!(task.task_type, TaskType::HumanApproval));
        assert_eq!(task.input["title"], "需要审批");
    }

    #[test]
    fn test_task_serialize_deserialize_round_trip() {
        let original = Task::new("task-5", "测试", TaskType::AgentCall)
            .with_input(serde_json::json!({"data": "test"}));
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: Task = serde_json::from_str(&json).unwrap();
        assert_eq!(original.id, deserialized.id);
        assert_eq!(original.name, deserialized.name);
        assert_eq!(original.input, deserialized.input);
    }

    #[test]
    fn test_task_result() {
        let result = TaskResult {
            task_id: "task-1".to_string(),
            success: true,
            output: Some(serde_json::json!({"status": "completed"})),
            error: None,
        };
        assert_eq!(result.task_id, "task-1");
        assert!(result.success);
        assert!(result.output.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_task_result_failure() {
        let result = TaskResult {
            task_id: "task-2".to_string(),
            success: false,
            output: None,
            error: Some("执行失败".to_string()),
        };
        assert_eq!(result.task_id, "task-2");
        assert!(!result.success);
        assert!(result.output.is_none());
        assert_eq!(result.error, Some("执行失败".to_string()));
    }
}
