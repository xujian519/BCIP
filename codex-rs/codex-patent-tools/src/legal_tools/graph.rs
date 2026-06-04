use codex_patent_knowledge::UnifiedSearch;

impl super::LegalTools {
    /// 图遍历查询
    pub fn graph_query(
        start_id: &str,
        relation_filter: Option<Vec<String>>,
        max_depth: usize,
    ) -> Result<serde_json::Value, String> {
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let filter: Option<Vec<&str>> = relation_filter
            .as_ref()
            .map(|v| v.iter().map(|s| s.as_str()).collect());
        let filter_ref = filter.as_deref();
        let edges = kg
            .traverse(start_id, filter_ref, max_depth)
            .map_err(|e| format!("图遍历失败: {e}"))?;
        serde_json::to_value(&edges).map_err(|e| e.to_string())
    }

    pub fn graph_neighbors(node_id: &str) -> Result<serde_json::Value, String> {
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let edges = kg.get_edges(node_id).map_err(|e| e.to_string())?;
        serde_json::to_value(&edges).map_err(|e| e.to_string())
    }

    /// 知识卡片搜索：按概念/关键词检索 Wiki 知识卡片
    pub fn card_search(query: &str, limit: usize) -> Result<serde_json::Value, String> {
        let card_mutex = UnifiedSearch::global()
            .card_index()
            .ok_or("知识卡片索引未初始化")?;
        let index = card_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;

        let by_keyword = index.search_by_keyword(query, limit);
        let by_concept = index.search_by_concept(query, limit);

        // 合并去重，概念搜索优先
        let mut seen_ids = std::collections::HashSet::new();
        let mut results = Vec::new();
        for card in by_concept.iter().chain(by_keyword.iter()) {
            if seen_ids.insert(card.id.clone()) {
                let content = index.load_content(card).unwrap_or_default();
                let preview = if content.len() > 300 {
                    &content[..300]
                } else {
                    &content
                };
                results.push(serde_json::json!({
                    "id": card.id,
                    "title": card.title,
                    "concept": card.concept,
                    "domain": card.domain,
                    "quality": card.quality,
                    "related_concepts": card.related_concepts,
                    "content_preview": preview,
                }));
                if results.len() >= limit {
                    break;
                }
            }
        }
        Ok(serde_json::json!({
            "query": query,
            "total": results.len(),
            "results": results,
        }))
    }

    /// 路径查找：在知识图谱中查找两个节点之间的最短路径
    pub fn find_path(
        from_id: &str,
        to_id: &str,
        max_depth: usize,
    ) -> Result<serde_json::Value, String> {
        let kg_mutex = UnifiedSearch::global().kg().ok_or("知识图谱未初始化")?;
        let kg = kg_mutex.lock().map_err(|e| format!("锁获取失败: {e}"))?;
        let paths = kg
            .find_path(from_id, to_id, max_depth)
            .map_err(|e| format!("路径查找失败: {e}"))?;
        Ok(serde_json::json!({
            "from": from_id,
            "to": to_id,
            "max_depth": max_depth,
            "paths": paths,
            "path_count": paths.len(),
        }))
    }
}
