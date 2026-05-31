use crate::scheduler::CronTask;

pub enum PatentCronTemplate {
    WeeklyPriorArtSearch {
        search_query: String,
        ipc_class: String,
    },
    DailyOaDeadlineCheck {
        docket_path: std::path::PathBuf,
    },
    WeeklyPortfolioReport {
        patent_ids: Vec<String>,
    },
    DailyLegalStatusMonitor {
        patent_numbers: Vec<String>,
    },
}

impl PatentCronTemplate {
    pub fn to_task(&self) -> CronTask {
        match self {
            Self::WeeklyPriorArtSearch {
                search_query,
                ipc_class,
            } => CronTask {
                id: String::new(),
                cron: "0 9 * * 1".into(),
                prompt: format!(
                    "检索 IPC 分类 {ipc_class} 下关于 {search_query} 的最新公开专利，生成现有技术报告。",
                ),
                name: "每周现有技术检索".into(),
                description: format!("IPC: {ipc_class}, 关键词: {search_query}"),
                created_at: chrono::Utc::now(),
                last_fired_at: None,
                recurring: true,
                enabled: true,
                jitter_ms: 60000,
            },
            Self::DailyOaDeadlineCheck { docket_path } => CronTask {
                id: String::new(),
                cron: "0 8 * * 1-5".into(),
                prompt: format!(
                    "检查 {docket_path:?} 中所有待答复 OA 的剩余期限，生成当日 OA 期限提醒清单，按紧迫度排序。",
                ),
                name: "OA 期限检查".into(),
                description: "工作日每天早上 8:00 检查 OA 答复期限".into(),
                created_at: chrono::Utc::now(),
                last_fired_at: None,
                recurring: true,
                enabled: true,
                jitter_ms: 30000,
            },
            Self::WeeklyPortfolioReport { patent_ids } => CronTask {
                id: String::new(),
                cron: "0 10 * * 1".into(),
                prompt: format!(
                    "生成专利组合健康报告，覆盖 {} 件专利：{}。\n\
                     分析：1) 法律状态概览 2) 审查进度 3) 费用临近期限 4) 风险评估",
                    patent_ids.len(),
                    patent_ids.join(", "),
                ),
                name: "专利组合周报".into(),
                description: "每周一生成专利组合健康报告".into(),
                created_at: chrono::Utc::now(),
                last_fired_at: None,
                recurring: true,
                enabled: true,
                jitter_ms: 120000,
            },
            Self::DailyLegalStatusMonitor { patent_numbers } => CronTask {
                id: String::new(),
                cron: "0 7 * * *".into(),
                prompt: format!(
                    "监控以下专利的法律状态变更：{}。\n\
                     如有状态变更（公开/实审/授权/驳回/无效/届满），生成变更通知。",
                    patent_numbers.join(", "),
                ),
                name: "法律状态监控".into(),
                description: "每天监控目标专利法律状态变更".into(),
                created_at: chrono::Utc::now(),
                last_fired_at: None,
                recurring: true,
                enabled: true,
                jitter_ms: 0,
            },
        }
    }
}
