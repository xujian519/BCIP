use thiserror::Error;

#[derive(Error, Debug)]
pub enum PatentError {
    #[error("kg error: {0}")]
    KnowledgeGraph(String),
    #[error("law db error: {0}")]
    LawDb(String),
    #[error("search error: {0}")]
    Search(String),
    #[error("claim parse error: {0}")]
    ClaimParse(String),
    #[error("rule engine error: {0}")]
    RuleEngine(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("not found: {0}")]
    NotFound(String),
}
