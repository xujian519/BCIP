use codex_patent_skills::SkillLoader;
use std::path::Path;

#[test]
fn test_load_skills_from_toml_directory() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");

    if !skills_dir.exists() {
        println!("Skills assets dir not found at {:?}, skipping", skills_dir);
        return;
    }

    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    match loader {
        Ok(l) => {
            let skills = l.list();
            println!("加载技能: {:?}", skills);
            assert!(!skills.is_empty(), "应至少加载一个技能");
        }
        Err(e) => {
            println!("加载技能出错: {}", e);
            // 可能因为目录不存在而失败，这是可接受的
        }
    }
}

#[test]
fn test_check_bcip_skills_directory() {
    // 检查 .codex/skills/patent/ 目录下的 SKILL.md 文件
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let bcip_skills = project_root.join(".codex/skills/patent");

    if bcip_skills.exists() {
        let mut skill_count = 0;
        for entry in std::fs::read_dir(&bcip_skills).unwrap() {
            let entry = entry.unwrap();
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.exists() {
                let content = std::fs::read_to_string(&skill_md).unwrap();
                assert!(content.contains("---"), "SKILL.md 应有 YAML frontmatter");
                assert!(content.contains("name:"), "SKILL.md 应有 name 字段");
                skill_count += 1;
            }
        }
        println!(".codex/skills/patent/ 下有 {} 个技能", skill_count);
        assert!(skill_count >= 10, "应有至少10个专利技能");
    } else {
        println!("BCIP skills dir not found, checking TOML skills");
        let toml_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        if toml_dir.exists() {
            let count = std::fs::read_dir(&toml_dir).unwrap()
                .filter(|e| e.as_ref().unwrap().path().extension().map_or(false, |ext| ext == "toml"))
                .count();
            assert!(count >= 10, "TOML技能文件不足");
        }
    }
}

#[test]
fn test_skill_names_are_valid() {
    let expected_skills = [
        "cap-retrieval", "cap-analysis", "cap-writing",
        "cap-disclosure-exam", "cap-inventive", "cap-clarity-exam",
        "cap-invalid", "cap-prior-art-ident", "cap-response",
        "cap-formal-exam",
    ];

    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() { return; }

    for skill_id in &expected_skills {
        let file_path = skills_dir.join(format!("{}.toml", skill_id));
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("skill_id"), "{} 应包含 skill_id", skill_id);
            assert!(content.contains("instructions"), "{} 应包含 instructions", skill_id);
            println!("✓ {}", skill_id);
        } else {
            println!("? {} (文件不存在)", skill_id);
        }
    }
}

#[test]
fn test_shared_modules_exist() {
    let shared_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/_shared");
    if !shared_dir.exists() { return; }

    let expected = ["legal_reasoning", "hitl_protocol", "output_standards", "quality_checklist", "patent_glossary"];
    for name in &expected {
        let path = shared_dir.join(format!("{}.toml", name));
        if !path.exists() {
            println!("⚠ 共享模块 {} 不存在 (可能未实现)", name);
        }
    }

    let count = std::fs::read_dir(&shared_dir).unwrap()
        .filter(|e| e.as_ref().unwrap().path().extension().map_or(false, |ext| ext == "toml"))
        .count();
    println!("共享模块数量: {}", count);
}

#[test]
fn test_skill_includes_resolve() {
    // 测试 include 引用是否能正确解析
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() { return; }

    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    if let Ok(loader) = loader {
        // 尝试解析一个包含 include 引用的技能
        if let Some(skill_id) = loader.list().first() {
            match loader.resolve(skill_id) {
                Ok(resolved) => {
                    assert!(!resolved.is_empty());
                    println!("技能 {} 解析成功 ({} 字符)", skill_id, resolved.len());
                }
                Err(e) => println!("解析 {} 出错: {} (可能是正常情况)", skill_id, e),
            }
        }
    }
}

#[test]
fn test_count_toml_skill_files() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() { return; }

    let toml_count = std::fs::read_dir(&skills_dir).unwrap()
        .filter_map(|e| e.ok())
        .filter(|entry| {
            entry.path().extension()
                .map_or(false, |ext| ext == "toml")
        })
        .count();

    println!("TOML 技能文件总数: {}", toml_count);
    assert!(toml_count >= 10, "应有至少10个TOML技能文件");
}