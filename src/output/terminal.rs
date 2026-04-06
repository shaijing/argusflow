use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl OutputFormatter for TerminalFormatter {
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
        "txt"
    }
}

impl Default for TerminalFormatter {
    fn default() -> Self {
        Self::new()
    }
}
