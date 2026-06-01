use anyhow::Result;
use std::env;
use std::path::PathBuf;

mod db;
mod decision_parser;
mod ipc_parser;
mod judgment_parser;
mod models;
mod utils;

use db::KgDatabase;

fn main() -> Result<()> {
    let kb_dir = env::var("BCIP_ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("../codex-patent-assets"));
    let raw_dir: PathBuf = env::var("BCIP_RAW_DIR").map(PathBuf::from).map_err(|_| {
        anyhow::anyhow!("BCIP_RAW_DIR 环境变量未设置（指向宝宸知识库 Raw 数据目录）")
    })?;
    let ipc_dir: PathBuf = env::var("IPC_DIR").map(PathBuf::from).map_err(|_| {
        anyhow::anyhow!("IPC_DIR 环境变量未设置（指向 IPC 分类表 extracted_text 目录）")
    })?;

    let db_path = kb_dir.join("patent_kg.db");

    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║      Patent KG Sync Tool v0.2.0                          ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!();
    println!("数据库: {}", db_path.display());

    if !db_path.exists() {
        anyhow::bail!("数据库不存在: {}", db_path.display());
    }

    let mut db = KgDatabase::open(&db_path)?;
    let (nodes_before, edges_before, types_before) = db.get_stats()?;
    println!("同步前: {} 节点, {} 边", nodes_before, edges_before);

    // ── Phase 1: IPC 2026.01 升级 ──
    println!("\n[1/5] IPC 2026.01 数据升级...");
    if ipc_dir.exists() {
        let ipc_entries = ipc_parser::parse_ipc_files(&ipc_dir)?;
        println!("      解析到 {} 条 IPC 条目", ipc_entries.len());

        let tx = db.begin_transaction()?;
        KgDatabase::purge_old_ipc_versions(&tx)?;
        let ipc_count = KgDatabase::insert_ipc_entries(&tx, &ipc_entries)?;
        println!("      写入 {} 条 IPC 数据", ipc_count);

        let node_count = KgDatabase::insert_ipc_nodes(&tx, &ipc_entries)?;
        let edge_count = KgDatabase::insert_ipc_edges(&tx, &ipc_entries)?;
        println!("      IPC 节点: {}, 边: {}", node_count, edge_count);
        tx.commit()?;
    } else {
        println!("      IPC 目录不存在，跳过: {}", ipc_dir.display());
    }

    // ── Phase 2: 复审决定入库 ──
    println!("\n[2/5] 复审决定入库...");
    let decisions_dir = raw_dir.join("无效复审决定");
    if decisions_dir.exists() {
        let decisions = decision_parser::parse_decisions(&decisions_dir)?;

        let tx = db.begin_transaction()?;
        KgDatabase::insert_decision_nodes(&tx, &decisions)?;
        KgDatabase::insert_decision_clause_edges(&tx, &decisions)?;
        KgDatabase::insert_decision_ipc_edges(&tx, &decisions)?;
        tx.commit()?;
    } else {
        println!("      复审决定目录不存在，跳过");
    }

    // ── Phase 3: 判决入库 ──
    println!("\n[3/5] 判决文书入库...");

    // 指导性判决
    let guiding_dir = raw_dir.join("指导性专利判决文书_md");
    if guiding_dir.exists() {
        let guiding = judgment_parser::parse_guiding_judgments(&guiding_dir)?;
        let tx = db.begin_transaction()?;
        KgDatabase::insert_judgment_nodes(&tx, &guiding)?;
        KgDatabase::insert_judgment_clause_edges(&tx, &guiding)?;
        KgDatabase::insert_judgment_concept_edges(&tx, &guiding)?;
        tx.commit()?;
    }

    // 一般判决
    let general_dir = raw_dir.join("专利判决");
    if general_dir.exists() {
        let general = judgment_parser::parse_general_judgments(&general_dir)?;
        let tx = db.begin_transaction()?;
        KgDatabase::insert_judgment_nodes(&tx, &general)?;
        KgDatabase::insert_judgment_clause_edges(&tx, &general)?;
        tx.commit()?;
    }

    // ── Phase 4: 统计 ──
    println!("\n[4/5] 统计...");
    let (nodes_after, edges_after, types_after) = db.get_stats()?;

    println!("\n═══════════════════════════════════════════════════════════");
    println!("                    同步完成统计");
    println!("═══════════════════════════════════════════════════════════");
    println!(
        "  节点: {} → {} (+{})",
        nodes_before,
        nodes_after,
        nodes_after - nodes_before
    );
    println!(
        "  边:   {} → {} (+{})",
        edges_before,
        edges_after,
        edges_after - edges_before
    );
    println!();
    println!("  节点类型分布:");

    let mut types: Vec<_> = types_after.iter().collect();
    types.sort_by(|a, b| b.1.cmp(a.1));
    for (node_type, count) in &types {
        let before = types_before.get(*node_type).copied().unwrap_or(0);
        let delta = **count - before;
        if delta > 0 {
            println!("    {:<25} {:>6} (+{})", node_type, count, delta);
        } else {
            println!("    {:<25} {:>6}", node_type, count);
        }
    }

    // ── Phase 5: 三角查询验证 ──
    println!("\n[5/5] 三角查询验证...");
    verify_triangle_queries(&db);

    println!("\n═══════════════════════════════════════════════════════════");
    Ok(())
}

fn verify_triangle_queries(db: &KgDatabase) {
    let ipc_decisions = db.count_ipc_decision_edges();
    println!("      IPC→Decision 映射: {} 条", ipc_decisions);

    let decision_clauses = db.count_edges_by_relation("APPLIES");
    println!("      Decision→Clause 边: {} 条", decision_clauses);

    let judgment_clauses = db.count_edges_by_relation("CITES");
    println!("      Judgment→Clause 边: {} 条", judgment_clauses);

    let ipc_nodes = db.count_nodes_by_type("IPC");
    println!("      IPC 小类节点: {} 个", ipc_nodes);

    println!("      ✓ 三角架构验证完成");
}
