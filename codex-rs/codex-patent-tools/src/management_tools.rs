use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PatentManageInput {
    pub action: String,
    pub patent_id: Option<String>,
    pub data: Option<serde_json::Value>,
}

pub struct ManagementTools;

impl ManagementTools {
    pub fn patent_manager(input: PatentManageInput) -> Result<serde_json::Value, String> {
        let states = [
            "draft",
            "filed",
            "published",
            "examined",
            "granted",
            "maintained",
            "expired",
        ];
        Ok(
            serde_json::json!({"action": input.action, "valid_states": states, "message": "Patent lifecycle: draft→filed→published→examined→granted→maintained→expired"}),
        )
    }

    pub fn template_library(template_type: &str) -> Result<serde_json::Value, String> {
        let templates: [(&str, &str, &str); 5] = [
            (
                "oa_response",
                "审查意见答复模板",
                "一、修改说明\n二、意见陈述\n三、结论",
            ),
            (
                "patent_application",
                "专利申请模板",
                "一、技术领域\n二、背景技术\n三、发明内容\n四、附图说明\n五、具体实施方式",
            ),
            (
                "invalidation",
                "无效宣告请求模板",
                "一、请求人信息\n二、专利信息\n三、无效理由\n四、证据清单\n五、详细陈述",
            ),
            (
                "reexamination",
                "复审请求模板",
                "一、复审请求人\n二、驳回决定\n三、复审理由\n四、证据",
            ),
            (
                "examination_opinion",
                "审查意见模板",
                "一、引用对比文件\n二、新颖性分析\n三、创造性分析\n四、其他问题",
            ),
        ];
        let found = templates
            .iter()
            .find(|(id, _, _)| id.contains(template_type))
            .map(|(_, n, c)| (*n, *c))
            .unwrap_or(("通用模板", ""));
        Ok(serde_json::json!({"template_name": found.0, "structure": found.1}))
    }

    pub fn trademark_analysis(mark: &str) -> Result<serde_json::Value, String> {
        let mut score: f64 = 70.0;
        let mut issues = Vec::new();
        if mark.len() < 2 {
            score -= 30.0;
            issues.push("商标过短");
        }
        if mark.len() > 20 {
            score -= 10.0;
            issues.push("商标过长");
        }
        let common = ["优质", "最佳", "第一", "超级", "最好"];
        if common.iter().any(|w| mark.contains(w)) {
            score -= 20.0;
            issues.push("含通用赞美词");
        }
        Ok(
            serde_json::json!({"registrability_score": score.max(0.0), "issues": issues, "recommendation": if score > 60.0 {"可注册"} else if score > 40.0 {"可能被驳回"} else {"难以注册"}}),
        )
    }

    pub fn process_chart(process_type: &str) -> Result<serde_json::Value, String> {
        let chart = match process_type {
            "application" => {
                "graph LR\n  A[发明构思] --> B[撰写申请]\n  B --> C[提交申请]\n  C --> D[初步审查]\n  D --> E[公布]\n  E --> F[实质审查]\n  F --> G[授权]"
            }
            "invalidation" => {
                "graph LR\n  A[发现无效理由] --> B[收集证据]\n  B --> C[撰写请求]\n  C --> D[提交请求]\n  D --> E[口审]\n  E --> F[决定]"
            }
            _ => "graph LR\n  A[开始] --> B[处理]\n  B --> C[完成]",
        };
        Ok(serde_json::json!({"process_type": process_type, "mermaid": chart}))
    }
}
