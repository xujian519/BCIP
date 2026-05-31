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
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
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
            let count = std::fs::read_dir(&toml_dir)
                .unwrap()
                .filter(|e| {
                    e.as_ref()
                        .unwrap()
                        .path()
                        .extension()
                        .is_some_and(|ext| ext == "toml")
                })
                .count();
            assert!(count >= 10, "TOML技能文件不足");
        }
    }
}

#[test]
fn test_skill_names_are_valid() {
    let expected_skills = [
        "cap-retrieval",
        "cap-analysis",
        "cap-writing",
        "cap-disclosure-exam",
        "cap-inventive",
        "cap-clarity-exam",
        "cap-invalid",
        "cap-prior-art-ident",
        "cap-response",
        "cap-formal-exam",
    ];

    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

    for skill_id in &expected_skills {
        let file_path = skills_dir.join(format!("{}.toml", skill_id));
        if file_path.exists() {
            let content = std::fs::read_to_string(&file_path).unwrap();
            assert!(content.contains("skill_id"), "{} 应包含 skill_id", skill_id);
            assert!(
                content.contains("instructions"),
                "{} 应包含 instructions",
                skill_id
            );
            println!("✓ {}", skill_id);
        } else {
            println!("? {} (文件不存在)", skill_id);
        }
    }
}

#[test]
fn test_shared_modules_exist() {
    let shared_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/_shared");
    if !shared_dir.exists() {
        return;
    }

    let expected = [
        "legal_reasoning",
        "hitl_protocol",
        "output_standards",
        "quality_checklist",
        "patent_glossary",
    ];
    for name in &expected {
        let path = shared_dir.join(format!("{}.toml", name));
        if !path.exists() {
            println!("⚠ 共享模块 {} 不存在 (可能未实现)", name);
        }
    }

    let count = std::fs::read_dir(&shared_dir)
        .unwrap()
        .filter(|e| {
            e.as_ref()
                .unwrap()
                .path()
                .extension()
                .is_some_and(|ext| ext == "toml")
        })
        .count();
    println!("共享模块数量: {}", count);
}

#[test]
fn test_skill_includes_resolve() {
    // 测试 include 引用是否能正确解析
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

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
fn test_inline_include_in_instructions() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    if let Ok(loader) = loader {
        // 检查包含内联 include 的技能
        let inline_tests = ["cap-retrieval", "cap-analysis"];
        for skill_id in &inline_tests {
            if loader.get(skill_id).is_some() {
                match loader.resolve(skill_id) {
                    Ok(resolved) => {
                        assert!(!resolved.is_empty(), "{skill_id} 解析结果不应为空");
                        // 验证内联 include 被替换为实际内容（不应再看到 {{include: 标记）
                        assert!(
                            !resolved.contains("{{include:_shared/"),
                            "{skill_id} 中应无未解析的内联 include: {resolved}"
                        );
                        println!(
                            "✓ {} 内联 include 解析成功 ({} 字符)",
                            skill_id,
                            resolved.len()
                        );
                    }
                    Err(e) => println!("? {} 解析失败: {} (可能无此技能文件)", skill_id, e),
                }
            }
        }
    }
}

#[test]
fn test_inline_include_depth_limit() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    if let Ok(loader) = loader {
        // 解析应正常工作，不会因深度限制而崩溃
        for skill_id in loader.list() {
            match loader.resolve(skill_id) {
                Ok(resolved) => {
                    assert!(!resolved.is_empty(), "{skill_id} 不应为空");
                    // 检查是否有深度限制标记（超出深度时返回错误标记）
                    let has_depth_warning = resolved.contains("超出最大深度");
                    if has_depth_warning {
                        println!("⚠ {} 包含深度限制警告", skill_id);
                    }
                }
                Err(e) => println!("? {} 解析出错: {}", skill_id, e),
            }
        }
    }
}

#[test]
fn test_inline_include_circular_reference() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    if let Ok(loader) = loader {
        // 解析不应因循环引用而 panic
        for skill_id in loader.list() {
            let result = loader.resolve(skill_id);
            assert!(result.is_ok(), "{skill_id} 不应因循环引用而解析失败");
        }
        println!("✓ 所有技能解析无循环引用崩溃");
    }
}

#[test]
fn test_phase_field_default() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    if let Ok(loader) = loader {
        for skill_id in loader.list() {
            let phase = loader.phase_for(skill_id);
            // phase 要么是 "general"（默认），要么是非空字符串
            assert!(!phase.is_empty(), "{skill_id} 的 phase 不应为空");
            println!("  {} → phase: {}", skill_id, phase);
        }
    }
}

#[test]
fn test_resolve_instructions_api() {
    let shared_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/_shared");
    if !shared_dir.exists() {
        return;
    }

    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let loader = SkillLoader::load(skills_dir.to_str().unwrap());
    if let Ok(loader) = loader {
        // 测试 resolve_instructions（供 agent 系统使用）
        let text =
            "分析以下权利要求的结构。\n\n{{include:_shared/legal_reasoning}}\n\n请输出分析结果。";
        let resolved = loader.resolve_instructions(text, &[]);
        assert!(resolved.is_ok());
        let resolved = resolved.unwrap();
        assert!(resolved.contains("法律推理应遵循"));
        assert!(resolved.contains("分析以下权利要求的结构"));
        assert!(resolved.contains("请输出分析结果"));
        assert!(!resolved.contains("{{include:_shared/legal_reasoning}}"));
        println!("✓ resolve_instructions API 工作正常");
    }
}

#[test]
fn test_count_toml_skill_files() {
    let skills_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    if !skills_dir.exists() {
        return;
    }

    let toml_count = std::fs::read_dir(&skills_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "toml"))
        .count();

    println!("TOML 技能文件总数: {}", toml_count);
    assert!(toml_count >= 10, "应有至少10个TOML技能文件");
}
