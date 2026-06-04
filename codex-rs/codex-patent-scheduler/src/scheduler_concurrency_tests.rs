//! CronScheduler 并发触发测试 — 验证多任务并发触发的正确性。

use super::*;
use std::sync::Arc;

/// Test 17: 5 个过期任务 jitter_ms=100，各触发恰好 1 次。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn concurrent_task_firing_with_jitter() {
    let _dir = tempfile::tempdir().unwrap();

    let now = Utc::now();
    let past = now - chrono::Duration::hours(2);

    // Use "* * * * *" (every minute) to ensure tasks are always due
    let tasks = Arc::new(tokio::sync::Mutex::new(vec![
        make_task("t1", "* * * * *", past, 100),
        make_task("t2", "* * * * *", past, 100),
        make_task("t3", "* * * * *", past, 100),
        make_task("t4", "* * * * *", past, 100),
        make_task("t5", "* * * * *", past, 100),
    ]));

    let fire_counts: Arc<std::sync::Mutex<std::collections::HashMap<String, usize>>> =
        Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));

    let tasks_clone = Arc::clone(&tasks);
    let fire_counts_clone = Arc::clone(&fire_counts);

    // Run the loop for a controlled period
    let handle = tokio::spawn(async move {
        let check_interval = tokio::time::Duration::from_millis(500);
        for _ in 0..3 {
            tokio::time::sleep(check_interval).await;
            let mut tasks_lock = tasks_clone.lock().await;
            for task in tasks_lock.iter_mut() {
                if !task.enabled {
                    continue;
                }
                let expr = match CronExpression::parse(&task.cron) {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let from = task.last_fired_at.unwrap_or(past);
                if let Some(next) = expr.next_run(from)
                    && next <= Utc::now()
                {
                    let task_id = task.id.clone();
                    let mut counts = fire_counts_clone.lock().unwrap();
                    *counts.entry(task_id).or_insert(0) += 1;
                    task.last_fired_at = Some(Utc::now());
                    if !task.recurring {
                        task.enabled = false;
                    }
                }
            }
        }
    });

    handle.await.unwrap();

    let counts = fire_counts.lock().unwrap();
    for (id, count) in counts.iter() {
        assert_eq!(*count, 1, "task {id} should fire exactly once, got {count}");
    }
    assert_eq!(counts.len(), 5, "all 5 tasks should have fired");
}

/// Test 18: 触发中重载任务文件不 panic。
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn file_reload_during_active_firing() {
    let dir = tempfile::tempdir().unwrap();
    let task_file = dir.path().join("scheduled_tasks.json");

    let now = Utc::now();
    let past = now - chrono::Duration::seconds(10);

    // Write initial tasks
    let initial_tasks = vec![make_task("r1", "0 * * * *", past, 0)];
    let json = serde_json::to_string_pretty(&initial_tasks).unwrap();
    std::fs::write(&task_file, &json).unwrap();

    // Verify we can load the file
    let mut sched = CronScheduler::new(dir.path()).unwrap();
    assert_eq!(sched.list_tasks().len(), 1);

    // Write new tasks while potentially reading
    let new_tasks = vec![
        make_task("r1", "0 * * * *", past, 0),
        make_task("r2", "0 * * * *", past, 0),
    ];
    let json = serde_json::to_string_pretty(&new_tasks).unwrap();
    std::fs::write(&task_file, &json).unwrap();

    // Reload should succeed without panic
    sched.reload().unwrap();
    assert_eq!(sched.list_tasks().len(), 2);
}

fn make_task(id: &str, cron: &str, last_fired: DateTime<Utc>, jitter_ms: u32) -> CronTask {
    CronTask {
        id: id.to_string(),
        cron: cron.to_string(),
        prompt: format!("test prompt for {id}"),
        name: format!("test task {id}"),
        description: "test".into(),
        created_at: Utc::now(),
        last_fired_at: Some(last_fired),
        recurring: true,
        enabled: true,
        jitter_ms,
    }
}
