use std::sync::Arc;
use crate::source::PaperSource;
use super::CitationGraph;

pub enum CrawlDirection {
    Citations,
    References,
    Both,
}

pub struct CitationCrawler {
    source: Arc<dyn PaperSource>,
    max_depth: usize,
    max_papers: usize,
}

impl CitationCrawler {
    pub fn new(source: Arc<dyn PaperSource>) -> Self {
        Self {
            source,
            max_depth: 1,
            max_papers: 100,
        }
    }

    pub fn with_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_max_papers(mut self, max: usize) -> Self {
        self.max_papers = max;
        self
    }
}
