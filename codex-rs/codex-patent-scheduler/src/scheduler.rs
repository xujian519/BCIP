use crate::cron::CronExpression;
use chrono::DateTime;
use chrono::Utc;
use notify::Event;
use notify::RecursiveMode;
use notify::Watcher;
use rand::Rng;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::error;
use tracing::info;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CronTask {
    pub id: String,
    pub cron: String,
    pub prompt: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub last_fired_at: Option<DateTime<Utc>>,
    pub recurring: bool,
    pub enabled: bool,
    pub jitter_ms: u32,
}

impl CronTask {
    pub fn validated(&self) -> Result<CronExpression, SchedulerError> {
        CronExpression::parse(&self.cron).map_err(|e| SchedulerError::CronParseError(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("文件操作失败: {0}")]
    IoError(String),
    #[error("Cron 解析失败: {0}")]
    CronParseError(String),
    #[error("锁文件错误: {0}")]
    LockError(String),
}

#[derive(Debug)]
pub struct CronScheduler {
    tasks: Vec<CronTask>,
    task_file: PathBuf,
    #[allow(dead_code)]
    lock_file: PathBuf,
    watcher: Option<notify::RecommendedWatcher>,
}

impl CronScheduler {
    pub fn new(data_dir: &Path) -> Result<Self, SchedulerError> {
        std::fs::create_dir_all(data_dir).map_err(|e| SchedulerError::IoError(e.to_string()))?;

        let task_file = data_dir.join("scheduled_tasks.json");
        let lock_file = data_dir.join("scheduler.lock");

        let tasks = if task_file.exists() {
            let content = std::fs::read_to_string(&task_file)
                .map_err(|e| SchedulerError::IoError(e.to_string()))?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(Self {
            tasks,
            task_file,
            lock_file,
            watcher: None,
        })
    }

    pub fn init_watcher(&mut self) -> Result<(), SchedulerError> {
        let _task_file = self.task_file.clone();
        let mut watcher =
            notify::recommended_watcher(move |res: Result<Event, notify::Error>| match res {
                Ok(event) => {
                    if event.kind.is_modify() {
                        info!("任务文件已变更，将重新加载");
                    }
                }
                Err(e) => {
                    error!(%e, "文件监视错误");
                }
            })
            .map_err(|e| SchedulerError::IoError(e.to_string()))?;

        if let Some(parent) = self.task_file.parent() {
            watcher
                .watch(parent, RecursiveMode::NonRecursive)
                .map_err(|e| SchedulerError::IoError(e.to_string()))?;
        }

        self.watcher = Some(watcher);
        Ok(())
    }

    pub fn add_task(&mut self, mut task: CronTask) -> Result<(), SchedulerError> {
        if task.id.is_empty() {
            task.id = uuid_simple();
        }
        task.created_at = Utc::now();

        CronExpression::parse(&task.cron)
            .map_err(|e| SchedulerError::CronParseError(e.to_string()))?;

        self.tasks.push(task);
        self.persist()
    }

    pub fn remove_task(&mut self, id: &str) -> Result<(), SchedulerError> {
        self.tasks.retain(|t| t.id != id);
        self.persist()
    }

    pub fn list_tasks(&self) -> &[CronTask] {
        &self.tasks
    }

    pub async fn run_loop(tasks: Arc<Mutex<Vec<CronTask>>>, on_fire: impl Fn(&CronTask)) -> ! {
        let check_interval = tokio::time::Duration::from_secs(1);

        loop {
            tokio::time::sleep(check_interval).await;
            let now = Utc::now();

            let fired = {
                let mut tasks_lock = tasks.lock().await;
                let mut fired = Vec::new();
                for task in tasks_lock.iter_mut() {
                    if !task.enabled {
                        continue;
                    }

                    let expr = match CronExpression::parse(&task.cron) {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    let from = task.last_fired_at.unwrap_or(now);
                    if let Some(next) = expr.next_run(from)
                        && next <= now
                    {
                        let jitter = if task.jitter_ms > 0 {
                            Some(rand::rng().random_range(0..task.jitter_ms))
                        } else {
                            None
                        };

                        info!(
                            task_id = %task.id,
                            task_name = %task.name,
                            "触发定时任务"
                        );

                        on_fire(task);
                        task.last_fired_at = Some(now);

                        if !task.recurring {
                            task.enabled = false;
                        }

                        fired.push(jitter);
                    }
                }
                fired
            };

            for ms in fired.into_iter().flatten() {
                tokio::time::sleep(tokio::time::Duration::from_millis(ms as u64)).await;
            }
        }
    }

    pub fn detect_missed_tasks(&self, now: DateTime<Utc>) -> Vec<&CronTask> {
        self.tasks
            .iter()
            .filter(|t| t.enabled)
            .filter(|t| {
                if let Some(last) = t.last_fired_at {
                    let expr = CronExpression::parse(&t.cron).ok();
                    expr.and_then(|e| e.next_run(last))
                        .map(|next| next < now)
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .collect()
    }

    fn persist(&self) -> Result<(), SchedulerError> {
        let json = serde_json::to_string_pretty(&self.tasks)
            .map_err(|e| SchedulerError::IoError(e.to_string()))?;

        std::fs::write(&self.task_file, json).map_err(|e| SchedulerError::IoError(e.to_string()))
    }

    pub fn reload(&mut self) -> Result<(), SchedulerError> {
        if self.task_file.exists() {
            let content = std::fs::read_to_string(&self.task_file)
                .map_err(|e| SchedulerError::IoError(e.to_string()))?;
            self.tasks = serde_json::from_str(&content).unwrap_or_default();
        }
        Ok(())
    }
}

fn uuid_simple() -> String {
    use rand::Rng;
    let mut rng = rand::rng();
    let chars: Vec<char> = "abcdef0123456789".chars().collect();
    (0..16)
        .map(|_| chars[rng.random_range(0..chars.len())])
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_list_tasks() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let task = CronTask {
            id: String::new(),
            cron: "0 9 * * *".into(),
            prompt: "test prompt".into(),
            name: "测试任务".into(),
            description: "每天 9:00 执行".into(),
            created_at: Utc::now(),
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };

        sched.add_task(task).unwrap();
        assert_eq!(sched.list_tasks().len(), 1);
        assert!(!sched.list_tasks()[0].id.is_empty());
    }

    #[test]
    fn test_invalid_cron_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let task = CronTask {
            id: "test".into(),
            cron: "invalid cron".into(),
            prompt: "test".into(),
            name: "test".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };

        assert!(sched.add_task(task).is_err());
    }
}
