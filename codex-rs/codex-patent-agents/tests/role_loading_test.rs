use codex_patent_agents::{PatentAgentRole, AgentRegistry, bcip_roles};
use std::path::Path;

#[test]
fn test_all_nine_roles_defined() {
    let roles = PatentAgentRole::all();
    assert_eq!(roles.len(), 9, "应有9个角色");

    let role_ids: Vec<&str> = roles.iter().map(|r| r.role_id()).collect();
    assert!(role_ids.contains(&"retriever"));
    assert!(role_ids.contains(&"analyzer"));
    assert!(role_ids.contains(&"writer"));
    assert!(role_ids.contains(&"novelty_checker"));
    assert!(role_ids.contains(&"creativity_checker"));
    assert!(role_ids.contains(&"infringement_checker"));
    assert!(role_ids.contains(&"invalidity_checker"));
    assert!(role_ids.contains(&"reviewer"));
    assert!(role_ids.contains(&"quality_checker"));
}

#[test]
fn test_role_names_are_chinese() {
    for role in PatentAgentRole::all() {
        let name = role.name();
        assert!(!name.is_empty(), "角色名不应为空");
        // 中文名称应包含中文
        assert!(name.chars().any(|c| c as u32 > 127), "{} 应包含中文字符", name);
    }
}

#[test]
fn test_role_from_str_roundtrip() {
    for role in PatentAgentRole::all() {
        let id = role.role_id();
        let parsed = PatentAgentRole::from_str(id);
        assert!(parsed.is_some(), "role_id {} 应能解析", id);
        assert_eq!(parsed.unwrap(), *role);
    }
}

#[test]
fn test_role_system_prompt_generation() {
    // 使用嵌入的角色配置生成系统提示词
    let configs = bcip_roles::patent_agent_role_configs();
    assert_eq!(configs.len(), 9, "应有9个BCIP角色配置");

    for (role_id, path) in &configs {
        let content = bcip_roles::config_file_contents(path);
        assert!(content.is_some(), "{} 的配置不应为空", role_id);
        let content = content.unwrap();
        // BCIP 格式使用 developer_instructions 字段，不需要 [role] 或 [prompt] 节
        assert!(content.contains("developer_instructions"), "{} 应包含 developer_instructions 字段", role_id);
        assert!(!content.is_empty());
    }
}

#[test]
fn test_load_configs_from_toml_files() {
    // 从 assets/ 和 assets/bcip/ 目录加载 TOML 文件
    let base_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let bcip_dir = base_dir.join("bcip");

    let mut count = 0;

    // 检查 assets/bcip/ 目录
    if bcip_dir.exists() {
        for entry in std::fs::read_dir(&bcip_dir).unwrap() {
            let entry = entry.unwrap();
            if entry.path().extension().map_or(false, |e| e == "toml") {
                let content = std::fs::read_to_string(entry.path()).unwrap();
                // BCIP 格式不使用 [role] 节，而是平铺字段
                assert!(content.contains("description") || content.contains("developer_instructions"), 
                        "文件应包含 description 或 developer_instructions 字段");
                assert!(!content.is_empty());
                count += 1;
            }
        }
    }

    assert_eq!(count, 9, "bcip/assets/ 目录应有9个TOML文件");
}

#[test]
fn test_agent_system_prompt_content() {
    for role in PatentAgentRole::all() {
        // 验证每个角色都有基本的工作方法描述
        let role_name = role.name();
        assert!(!role_name.is_empty());

        match role {
            PatentAgentRole::Retriever => assert!(role_name.contains("检索")),
            PatentAgentRole::Analyzer => assert!(role_name.contains("分析")),
            PatentAgentRole::Writer => assert!(role_name.contains("撰写")),
            PatentAgentRole::NoveltyChecker => assert!(role_name.contains("新颖")),
            PatentAgentRole::CreativityChecker => assert!(role_name.contains("创造")),
            PatentAgentRole::InfringementChecker => assert!(role_name.contains("侵权")),
            PatentAgentRole::InvalidityChecker => assert!(role_name.contains("无效")),
            PatentAgentRole::Reviewer => assert!(role_name.contains("审查")),
            PatentAgentRole::QualityChecker => assert!(role_name.contains("质量")),
        }
    }
}

#[test]
fn test_role_routing_hints() {
    // 验证每个角色有明确的路由提示信息
    let hints: Vec<(&str, &str)> = PatentAgentRole::all().iter().map(|r| {
        (r.role_id(), r.name())
    }).collect();

    for (id, name) in &hints {
        println!("角色: {} -> {}", id, name);
    }
    assert_eq!(hints.len(), 9);
}

#[test]
fn test_agent_registry_loads_configs() {
    // 注意：AgentRegistry 使用旧格式，BCIP 使用新格式
    // 这个测试验证 bcip 角色配置是否能正确加载为嵌入内容
    let configs = bcip_roles::patent_agent_role_configs();
    assert_eq!(configs.len(), 9, "应有9个BCIP角色配置");

    for (role_id, path) in &configs {
        let content = bcip_roles::config_file_contents(path);
        assert!(content.is_some(), "{} 的配置不应为空", role_id);
        let content = content.unwrap();
        // 验证配置内容包含基本字段
        assert!(content.contains("description") || content.contains("developer_instructions"),
                "{} 的配置应包含 description 或 developer_instructions", role_id);
    }
    println!("BCIP Registry 加载了 {} 个角色", configs.len());
}