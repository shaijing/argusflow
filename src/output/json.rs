use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct JsonFormatter;

impl OutputFormatter for JsonFormatter {
    fn format_papers(&self, _papers: &[Paper]) -> String {
        String::new()
    }

    fn format_paper_detail(&self, _paper: &Paper) -> String {
        String::new()
    }

    fn format_citations(&self, _citations: &[(Paper, Vec<Author>)], _direction: CitationDirection) -> String {
        String::new()
    }

    fn format_stats(&self, _stats: &crate::citation::CitationStats) -> String {
        String::new()
    }

    fn extension(&self) -> &'static str {
        "json"
    }
}
