use crate::models::Paper;

pub struct CitationStats {
    pub total_papers: usize,
    pub total_citation_edges: usize,
    pub average_citations: f64,
    pub h_index: i32,
    pub max_citations: i64,
    pub most_cited_papers: Vec<(Paper, i64)>,
    pub isolated_papers: Vec<Paper>,
}

impl Default for CitationStats {
    fn default() -> Self {
        Self {
            total_papers: 0,
            total_citation_edges: 0,
            average_citations: 0.0,
            h_index: 0,
            max_citations: 0,
            most_cited_papers: Vec::new(),
            isolated_papers: Vec::new(),
        }
    }
}
