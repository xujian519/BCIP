use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

static LINK_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"\[\[([^\]|#]+)(?:#([^\]|]+))?(?:\|[^\]]+)?\]\]").expect("link regex")
});

#[derive(Debug, Clone)]
pub struct WikiLink {
    pub source_file: String,
    pub target_file: String,
    pub anchor: Option<String>,
}

/// LLM Wiki [[双链]] 图
pub struct LinkGraph {
    links: Vec<WikiLink>,
    by_source: HashMap<String, Vec<WikiLink>>,
    by_target: HashMap<String, Vec<WikiLink>>,
}

impl LinkGraph {
    pub fn build(root_dir: &str) -> Result<Self, String> {
        let mut links = Vec::new();
        let mut by_source: HashMap<String, Vec<WikiLink>> = HashMap::new();
        let mut by_target: HashMap<String, Vec<WikiLink>> = HashMap::new();

        scan_dir(Path::new(root_dir), Path::new(root_dir), &mut links)?;

        for link in &links {
            by_source
                .entry(link.source_file.clone())
                .or_default()
                .push(link.clone());
            by_target
                .entry(link.target_file.clone())
                .or_default()
                .push(link.clone());
        }

        Ok(Self {
            links,
            by_source,
            by_target,
        })
    }

    pub fn total_links(&self) -> usize {
        self.links.len()
    }

    pub fn targets_of(&self, source_file: &str) -> Vec<&WikiLink> {
        self.by_source
            .get(source_file)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn backlinks_to(&self, target_file: &str) -> Vec<&WikiLink> {
        self.by_target
            .get(target_file)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    pub fn search_by_concept(&self, keyword: &str) -> Vec<&WikiLink> {
        let lower = keyword.to_lowercase();
        self.links
            .iter()
            .filter(|l| {
                l.target_file.to_lowercase().contains(&lower)
                    || l.source_file.to_lowercase().contains(&lower)
            })
            .collect()
    }

    pub fn all_links(&self) -> &[WikiLink] {
        &self.links
    }
}

fn scan_dir(dir: &Path, root: &Path, links: &mut Vec<WikiLink>) -> Result<(), String> {
    for entry in std::fs::read_dir(dir).map_err(|e| format!("read dir: {e}"))? {
        let entry = entry.map_err(|e| format!("entry: {e}"))?;
        let path = entry.path();
        if path.is_dir() && path.file_name().is_some_and(|n| n != ".git") {
            scan_dir(&path, root, links)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let source = path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            for cap in LINK_RE.captures_iter(&content) {
                let target = cap
                    .get(1)
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let anchor = cap.get(2).map(|m| m.as_str().to_string());
                if !target.is_empty() {
                    links.push(WikiLink {
                        source_file: source.clone(),
                        target_file: target,
                        anchor,
                    });
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_regex() {
        let text = "参见 [[专利实务/创造性/概述]] 和 [[法律法规/专利法-2020#第22条|专利法第22条]]";
        let caps: Vec<_> = LINK_RE.captures_iter(text).collect();
        assert_eq!(caps.len(), 2);
        assert_eq!(&caps[0][1], "专利实务/创造性/概述");
        assert_eq!(&caps[1][1], "法律法规/专利法-2020");
        assert_eq!(&caps[1][2], "第22条");
    }

    #[test]
    fn test_link_graph_build() {
        let graph = LinkGraph::build("../codex-patent-assets");
        assert!(graph.is_ok());
        let graph = graph.unwrap();
        assert!(
            graph.total_links() >= 100,
            "Expected >=100 links, got {}",
            graph.total_links()
        );
    }

    #[test]
    fn test_link_graph_search_by_concept() {
        let graph = LinkGraph::build("../codex-patent-assets").unwrap();
        let links = graph.search_by_concept("创造性");
        assert!(!links.is_empty(), "should find links mentioning 创造性");
    }
}
