//! 无效决定支持模块。

use codex_patent_core::InvalidDecision;

/// 无效决定存储(内存中)
#[derive(Debug, Default)]
pub struct InvalidDecisionStore {
    decisions: Vec<InvalidDecision>,
}

impl InvalidDecisionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, decision: InvalidDecision) {
        self.decisions.push(decision);
    }

    pub fn search_by_patent(&self, patent_number: &str) -> Vec<&InvalidDecision> {
        self.decisions
            .iter()
            .filter(|d| d.patent_number == patent_number)
            .collect()
    }

    pub fn search_by_ground(&self, ground: &str) -> Vec<&InvalidDecision> {
        self.decisions
            .iter()
            .filter(|d| d.grounds.iter().any(|g| g.contains(ground)))
            .collect()
    }

    pub fn all(&self) -> &[InvalidDecision] {
        &self.decisions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_patent_core::DecisionType;

    #[test]
    fn test_store_operations() {
        let mut store = InvalidDecisionStore::new();
        store.add(InvalidDecision {
            id: "1".into(),
            patent_number: "CN123456".into(),
            decision_number: "WX2024-001".into(),
            decision_date: "2024-01-01".into(),
            decision_type: DecisionType::Invalid,
            grounds: vec!["新颖性".into()],
            conclusion: "全部无效".into(),
        });

        let results = store.search_by_patent("CN123456");
        assert_eq!(results.len(), 1);

        let results = store.search_by_ground("新颖性");
        assert_eq!(results.len(), 1);
    }
}
