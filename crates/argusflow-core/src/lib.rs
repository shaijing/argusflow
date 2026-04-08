//! ArgusFlow Core Library
//!
//! A library for paper search, management, and citation analysis.

pub mod citation;
pub mod config;
pub mod core;
pub mod db;
pub mod models;
pub mod output;
pub mod pdf;
pub mod source;

// Re-export main types
pub use config::Config;
pub use core::{ArgusFlow, ArgusFlowBuilder, GraphFormat, SortBy};
pub use models::{Author, Citation, Paper, PaperAuthor};
pub use output::OutputFormat;
pub use source::{PaperSource, SearchParams, SearchResult, SourceCapabilities, SourceError, SourceKind, SourceManager};