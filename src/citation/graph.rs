use std::collections::HashMap;
use crate::models::Paper;

pub struct CitationGraph {
    papers: HashMap<i64, Paper>,
    citations: HashMap<i64, Vec<i64>>,
    cited_by: HashMap<i64, Vec<i64>>,
}

impl CitationGraph {
    pub fn new() -> Self {
        Self {
            papers: HashMap::new(),
            citations: HashMap::new(),
            cited_by: HashMap::new(),
        }
    }
}

impl Default for CitationGraph {
    fn default() -> Self {
        Self::new()
    }
}
