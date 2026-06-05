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

/// 定时任务定义。
#[derive(serde::Serialize, serde::Deserialize)]
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

/// 调度器错误类型。
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("文件操作失败: {0}")]
    IoError(String),
    #[error("Cron 解析失败: {0}")]
    CronParseError(String),
    #[error("锁文件错误: {0}")]
    LockError(String),
}

/// Cron 调度器，管理定时任务的注册、持久化和触发。
pub struct CronScheduler {
    tasks: Vec<CronTask>,
    task_file: PathBuf,
    #[allow(dead_code)] // 预留给进程锁机制
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
        loop {
            let now = Utc::now();

            let sleep_duration = {
                let tasks_lock = tasks.lock().await;
                let mut nearest = None;

                for task in tasks_lock.iter() {
                    if !task.enabled {
                        continue;
                    }

                    if let Ok(expr) = CronExpression::parse(&task.cron)
                        && let Some(next) = expr.next_run(now)
                    {
                        let dur = (next - now)
                            .to_std()
                            .unwrap_or(std::time::Duration::from_secs(1));
                        nearest = Some(match nearest {
                            Some(prev) if dur < prev => dur,
                            None => dur,
                            Some(prev) => prev,
                        });
                    }
                }

                nearest.unwrap_or(std::time::Duration::from_secs(60))
            };

            tokio::time::sleep(sleep_duration).await;

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

                    let from = task
                        .last_fired_at
                        .unwrap_or(now - chrono::TimeDelta::minutes(1));
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
    use crate::cron::CronField;

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

    #[test]
    fn test_remove_task() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let task = CronTask {
            id: "remove-me".into(),
            cron: "0 9 * * *".into(),
            prompt: "test".into(),
            name: "删除测试".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };

        sched.add_task(task).unwrap();
        assert_eq!(sched.list_tasks().len(), 1);

        sched.remove_task("remove-me").unwrap();
        assert!(sched.list_tasks().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_task() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let result = sched.remove_task("nonexistent");
        assert!(result.is_ok());
        assert!(sched.list_tasks().is_empty());
    }

    #[test]
    fn test_auto_generated_id() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let task = CronTask {
            id: String::new(),
            cron: "0 9 * * *".into(),
            prompt: "test".into(),
            name: "自动ID".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };

        sched.add_task(task).unwrap();
        let id = &sched.list_tasks()[0].id;
        assert_eq!(id.len(), 16);
    }

    #[test]
    fn test_detect_missed_tasks() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let past = Utc::now() - chrono::Duration::hours(2);
        let task = CronTask {
            id: "missed".into(),
            cron: "* * * * *".into(),
            prompt: "test".into(),
            name: "过期任务".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: Some(past),
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };

        sched.add_task(task).unwrap();
        let missed = sched.detect_missed_tasks(Utc::now());
        assert_eq!(missed.len(), 1);
        assert_eq!(missed[0].id, "missed");
    }

    #[test]
    fn test_detect_no_missed_for_disabled() {
        let dir = tempfile::tempdir().unwrap();
        let mut sched = CronScheduler::new(dir.path()).unwrap();

        let past = Utc::now() - chrono::Duration::hours(2);
        let task = CronTask {
            id: "disabled".into(),
            cron: "* * * * *".into(),
            prompt: "test".into(),
            name: "禁用任务".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: Some(past),
            recurring: true,
            enabled: false,
            jitter_ms: 0,
        };

        sched.add_task(task).unwrap();
        let missed = sched.detect_missed_tasks(Utc::now());
        assert!(missed.is_empty());
    }

    #[test]
    fn test_reload_picks_up_new_tasks() {
        let dir = tempfile::tempdir().unwrap();
        let task_file = dir.path().join("scheduled_tasks.json");

        let sched = CronScheduler::new(dir.path()).unwrap();
        assert!(sched.list_tasks().is_empty());

        let now = Utc::now();
        let tasks = vec![CronTask {
            id: "reloaded".into(),
            cron: "0 9 * * *".into(),
            prompt: "reloaded".into(),
            name: "重新加载".into(),
            description: "test".into(),
            created_at: now,
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        }];
        let json = serde_json::to_string_pretty(&tasks).unwrap();
        std::fs::write(&task_file, &json).unwrap();

        let mut sched = CronScheduler::new(dir.path()).unwrap();
        sched.reload().unwrap();
        assert_eq!(sched.list_tasks().len(), 1);
        assert_eq!(sched.list_tasks()[0].id, "reloaded");
    }

    #[test]
    fn test_validated_ok() {
        let task = CronTask {
            id: "v1".into(),
            cron: "0 9 * * 1-5".into(),
            prompt: "test".into(),
            name: "验证".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };
        let expr = task.validated().unwrap();
        assert_eq!(expr.minute, CronField::Single(0));
    }

    #[test]
    fn test_validated_err() {
        let task = CronTask {
            id: "v2".into(),
            cron: "bad cron expr".into(),
            prompt: "test".into(),
            name: "无效".into(),
            description: "test".into(),
            created_at: Utc::now(),
            last_fired_at: None,
            recurring: true,
            enabled: true,
            jitter_ms: 0,
        };
        assert!(task.validated().is_err());
    }

    #[test]
    fn test_scheduler_error_display() {
        let err = SchedulerError::IoError("file not found".into());
        assert!(format!("{err}").contains("file not found"));

        let err = SchedulerError::CronParseError("bad expr".into());
        assert!(format!("{err}").contains("bad expr"));

        let err = SchedulerError::LockError("locked".into());
        assert!(format!("{err}").contains("locked"));
    }
}

#[cfg(test)]
#[path = "scheduler_concurrency_tests.rs"]
mod scheduler_concurrency_tests;
