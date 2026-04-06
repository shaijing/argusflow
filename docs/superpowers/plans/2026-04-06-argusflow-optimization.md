# ArgusFlow 全面优化实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 优化 ArgusFlow CLI 工具，实现输出格式抽象、数据库性能优化、引用网络功能、CLI 功能完善。

**Architecture:** 引入 OutputFormatter trait 抽象输出层，使用 SeaORM find_with_related 消除 N+1 查询，新增 CitationGraph 数据结构和爬取功能。

**Tech Stack:** Rust, SeaORM, SQLite, clap, serde, tokio

---

## 文件结构

```
src/
├── output/              # 新增模块
│   ├── mod.rs           # OutputFormatter trait + OutputFormat enum
│   ├── terminal.rs      # TerminalFormatter 实现
│   ├── json.rs          # JsonFormatter 实现
│   ├── bibtex.rs        # BibtexFormatter 实现
│   └── markdown.rs      # MarkdownFormatter 实现
├── citation/            # 新增模块
│   ├── mod.rs           # 模块导出
│   ├── graph.rs         # CitationGraph 数据结构
│   ├── stats.rs         # CitationStats 统计
│   └── crawler.rs       # CitationCrawler 爬取
├── db/
│   ├── database.rs      # 修改：优化查询，添加新方法
│   └── migration/       # 新增索引 migration
│       └── m20240101_000005_add_indexes.rs
├── command/
│   ├── mod.rs           # 修改：新增全局参数和命令
│   ├── handlers.rs      # 修改：使用 formatter，添加新命令处理
│   └── context.rs       # 无需修改
├── lib.rs               # 修改：添加新模块导出
└── models/
    └── paper.rs         # 无需修改
```

---

## Task 1: 输出层基础 - OutputFormatter Trait

**Files:**
- Create: `src/output/mod.rs`
- Modify: `src/lib.rs:1-13`

- [ ] **Step 1: 创建 output 模块和 trait 定义**

```rust
// src/output/mod.rs
//! 输出格式抽象层

mod terminal;
mod json;
mod bibtex;
mod markdown;

pub use terminal::TerminalFormatter;
pub use json::JsonFormatter;
pub use bibtex::BibtexFormatter;
pub use markdown::MarkdownFormatter;

use crate::models::{Author, Paper};

/// 引用方向
#[derive(Debug, Clone, Copy)]
pub enum CitationDirection {
    /// 被哪些论文引用
    Citing,
    /// 引用了哪些论文
    Cited,
}

/// 输出格式类型
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Terminal,
    Json,
    Bibtex,
    Markdown,
}

impl OutputFormat {
    pub fn formatter(&self) -> Box<dyn OutputFormatter> {
        match self {
            Self::Terminal => Box::new(TerminalFormatter::new()),
            Self::Json => Box::new(JsonFormatter),
            Self::Bibtex => Box::new(BibtexFormatter),
            Self::Markdown => Box::new(MarkdownFormatter),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Terminal => "txt",
            Self::Json => "json",
            Self::Bibtex => "bib",
            Self::Markdown => "md",
        }
    }
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "terminal" | "text" => Ok(Self::Terminal),
            "json" => Ok(Self::Json),
            "bibtex" | "bib" => Ok(Self::Bibtex),
            "markdown" | "md" => Ok(Self::Markdown),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// 输出格式化器 trait
pub trait OutputFormatter: Send + Sync {
    /// 格式化论文列表
    fn format_papers(&self, papers: &[Paper]) -> String;

    /// 格式化单篇论文详情
    fn format_paper_detail(&self, paper: &Paper) -> String;

    /// 格式化引用列表
    fn format_citations(
        &self,
        citations: &[(Paper, Vec<Author>)],
        direction: CitationDirection,
    ) -> String;

    /// 格式化引用统计
    fn format_stats(&self, stats: &crate::citation::CitationStats) -> String;

    /// 获取文件扩展名
    fn extension(&self) -> &'static str;
}
```

- [ ] **Step 2: 更新 lib.rs 导出 output 模块**

```rust
// src/lib.rs
pub mod command;
pub mod config;
pub mod db;
pub mod models;
pub mod pdf;
pub mod source;
pub mod output;      // 新增
pub mod citation;    // 新增（稍后实现）

pub use command::{Cli, CommandContext};
pub use config::*;
pub use db::*;
pub use models::*;
pub use pdf::*;
pub use source::*;
pub use output::*;   // 新增
```

- [ ] **Step 3: 创建空的子模块文件（占位）**

```rust
// src/output/terminal.rs
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
```

```rust
// src/output/json.rs
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
```

```rust
// src/output/bibtex.rs
use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct BibtexFormatter;

impl OutputFormatter for BibtexFormatter {
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
        "bib"
    }
}
```

```rust
// src/output/markdown.rs
use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct MarkdownFormatter;

impl OutputFormatter for MarkdownFormatter {
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
        "md"
    }
}
```

- [ ] **Step 4: 创建 citation 模块占位（避免编译错误）**

```rust
// src/citation/mod.rs
//! 引用网络模块

mod graph;
mod stats;
mod crawler;

pub use graph::CitationGraph;
pub use stats::CitationStats;
pub use crawler::{CitationCrawler, CrawlDirection};
```

```rust
// src/citation/graph.rs
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
```

```rust
// src/citation/stats.rs
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
```

```rust
// src/citation/crawler.rs
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
```

- [ ] **Step 5: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过（可能有未使用警告）

- [ ] **Step 6: 提交**

```bash
git add src/output/ src/citation/ src/lib.rs
git commit -m "feat(output): add output formatter trait and citation module skeleton

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: 实现 TerminalFormatter

**Files:**
- Modify: `src/output/terminal.rs`

- [ ] **Step 1: 实现 format_papers 方法**

```rust
// src/output/terminal.rs
use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct TerminalFormatter;

impl TerminalFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl TerminalFormatter {
    fn format_identifier(paper: &Paper) -> String {
        let mut parts = Vec::new();
        if let Some(arxiv_id) = &paper.arxiv_id {
            if !arxiv_id.is_empty() {
                parts.push(format!("arXiv: {}", arxiv_id));
            }
        }
        if let Some(ss_id) = &paper.semantic_scholar_id {
            if !ss_id.is_empty() {
                parts.push(format!("SS: {}", ss_id));
            }
        }
        parts.join(" | ")
    }

    fn format_authors(authors: &[Author]) -> String {
        if authors.is_empty() {
            return "N/A".to_string();
        }
        let names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
        if names.len() <= 3 {
            names.join(", ")
        } else {
            format!("{}, et al.", names[0])
        }
    }
}

impl OutputFormatter for TerminalFormatter {
    fn format_papers(&self, papers: &[Paper]) -> String {
        let mut output = format!("共 {} 篇论文:\n", papers.len());

        for paper in papers {
            output.push_str("\n---\n");
            output.push_str(&format!("ID: {}\n", paper.id.unwrap_or(0)));
            output.push_str(&format!("标题: {}\n", paper.title));

            let identifier = Self::format_identifier(paper);
            if !identifier.is_empty() {
                output.push_str(&format!("{}\n", identifier));
            }

            output.push_str(&format!("作者: {}\n", Self::format_authors(&paper.authors)));
            output.push_str(&format!("引用数: {}\n", paper.citation_count));
        }

        output
    }

    fn format_paper_detail(&self, paper: &Paper) -> String {
        let mut output = String::new();

        output.push_str(&format!("[DB ID: {}]\n", paper.id.unwrap_or(0)));
        output.push_str(&format!("标题: {}\n", paper.title));

        let identifier = Self::format_identifier(paper);
        if !identifier.is_empty() {
            output.push_str(&format!("{}\n", identifier));
        }

        output.push_str(&format!("作者: {}\n", Self::format_authors(&paper.authors)));
        output.push_str(&format!("DOI: {}\n", paper.doi.as_deref().unwrap_or("N/A")));
        output.push_str(&format!("引用数: {}\n", paper.citation_count));
        output.push_str(&format!("PDF: {}\n", paper.pdf_url.as_deref().unwrap_or("N/A")));
        output.push_str(&format!("\n摘要:\n{}\n", paper.abstract_text.as_deref().unwrap_or("N/A")));

        output
    }

    fn format_citations(&self, citations: &[(Paper, Vec<Author>)], direction: CitationDirection) -> String {
        let header = match direction {
            CitationDirection::Citing => format!("该论文被以下 {} 篇论文引用:", citations.len()),
            CitationDirection::Cited => format!("该论文引用了以下 {} 篇论文:", citations.len()),
        };

        let mut output = header;

        for (paper, authors) in citations {
            output.push_str("\n---\n");
            output.push_str(&format!("标题: {}\n", paper.title));

            if let Some(ss_id) = &paper.semantic_scholar_id {
                if !ss_id.is_empty() {
                    output.push_str(&format!("SS ID: {}\n", ss_id));
                }
            }

            output.push_str(&format!("作者: {}\n", Self::format_authors(authors)));
        }

        output
    }

    fn format_stats(&self, stats: &crate::citation::CitationStats) -> String {
        let mut output = String::new();

        output.push_str("=== 引用统计 ===\n\n");
        output.push_str(&format!("论文总数: {}\n", stats.total_papers));
        output.push_str(&format!("引用关系数: {}\n", stats.total_citation_edges));
        output.push_str(&format!("平均引用数: {:.2}\n", stats.average_citations));
        output.push_str(&format!("H-Index: {}\n", stats.h_index));
        output.push_str(&format!("最大引用数: {}\n", stats.max_citations));

        output.push_str("\n=== 引用最多论文 Top 10 ===\n");
        for (i, (paper, count)) in stats.most_cited_papers.iter().take(10).enumerate() {
            output.push_str(&format!("\n{}. {} (引用: {})\n", i + 1, paper.title, count));
        }

        if !stats.isolated_papers.is_empty() {
            output.push_str(&format!("\n孤立节点（无引用关系）: {} 篇\n", stats.isolated_papers.len()));
        }

        output
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
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/output/terminal.rs
git commit -m "feat(output): implement TerminalFormatter

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 3: 实现 JsonFormatter

**Files:**
- Modify: `src/output/json.rs`

- [ ] **Step 1: 实现 JsonFormatter**

```rust
// src/output/json.rs
use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};
use serde_json::json;

pub struct JsonFormatter;

impl JsonFormatter {
    fn paper_to_json(paper: &Paper) -> serde_json::Value {
        json!({
            "id": paper.id,
            "title": paper.title,
            "abstract": paper.abstract_text,
            "arxiv_id": paper.arxiv_id,
            "semantic_scholar_id": paper.semantic_scholar_id,
            "doi": paper.doi,
            "pdf_url": paper.pdf_url,
            "local_pdf_path": paper.local_pdf_path,
            "publication_date": paper.publication_date,
            "venue": paper.venue,
            "citation_count": paper.citation_count,
            "authors": paper.authors.iter().map(|a| json!({
                "id": a.id,
                "name": a.name,
                "semantic_scholar_id": a.semantic_scholar_id
            })).collect::<Vec<_>>(),
            "created_at": paper.created_at.to_rfc3339(),
            "updated_at": paper.updated_at.to_rfc3339()
        })
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_papers(&self, papers: &[Paper]) -> String {
        let json = json!({
            "total": papers.len(),
            "papers": papers.iter().map(Self::paper_to_json).collect::<Vec<_>>()
        });
        serde_json::to_string_pretty(&json).unwrap_or_default()
    }

    fn format_paper_detail(&self, paper: &Paper) -> String {
        serde_json::to_string_pretty(&Self::paper_to_json(paper)).unwrap_or_default()
    }

    fn format_citations(&self, citations: &[(Paper, Vec<Author>)], direction: CitationDirection) -> String {
        let dir_str = match direction {
            CitationDirection::Citing => "citing",
            CitationDirection::Cited => "cited",
        };

        let json = json!({
            "direction": dir_str,
            "total": citations.len(),
            "papers": citations.iter().map(|(paper, authors)| {
                json!({
                    "paper": Self::paper_to_json(paper),
                    "authors": authors.iter().map(|a| json!({
                        "id": a.id,
                        "name": a.name,
                        "semantic_scholar_id": a.semantic_scholar_id
                    })).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });

        serde_json::to_string_pretty(&json).unwrap_or_default()
    }

    fn format_stats(&self, stats: &crate::citation::CitationStats) -> String {
        let json = json!({
            "total_papers": stats.total_papers,
            "total_citation_edges": stats.total_citation_edges,
            "average_citations": stats.average_citations,
            "h_index": stats.h_index,
            "max_citations": stats.max_citations,
            "most_cited_papers": stats.most_cited_papers.iter().map(|(paper, count)| {
                json!({
                    "paper": Self::paper_to_json(paper),
                    "citation_count": count
                })
            }).collect::<Vec<_>>(),
            "isolated_papers_count": stats.isolated_papers.len()
        });

        serde_json::to_string_pretty(&json).unwrap_or_default()
    }

    fn extension(&self) -> &'static str {
        "json"
    }
}
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/output/json.rs
git commit -m "feat(output): implement JsonFormatter

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 4: 实现 BibtexFormatter

**Files:**
- Modify: `src/output/bibtex.rs`

- [ ] **Step 1: 实现 BibtexFormatter**

```rust
// src/output/bibtex.rs
use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct BibtexFormatter;

impl BibtexFormatter {
    /// 生成 citation key: 第一作者姓氏 + 年份
    fn generate_key(paper: &Paper) -> String {
        let author_part = if let Some(first_author) = paper.authors.first() {
            // 取姓氏（假设最后一个单词是姓氏）
            first_author.name
                .split_whitespace()
                .last()
                .unwrap_or(&first_author.name)
                .to_lowercase()
                .replace(|c: char| !c.is_alphanumeric(), "")
        } else {
            "unknown".to_string()
        };

        let year_part = paper.publication_date
            .as_ref()
            .and_then(|d| d.split('-').next())
            .unwrap_or("nodate");

        format!("{}{}", author_part, year_part)
    }

    /// 转义 BibTeX 特殊字符
    fn escape_bibtex(s: &str) -> String {
        s.replace('&', "\\&")
            .replace('%', "\\%")
            .replace('$', "\\$")
            .replace('#', "\\#")
            .replace('_', "\\_")
            .replace('{', "\\{")
            .replace('}', "\\}")
    }

    fn format_authors(authors: &[Author]) -> String {
        authors
            .iter()
            .map(|a| a.name.clone())
            .collect::<Vec<_>>()
            .join(" and ")
    }
}

impl OutputFormatter for BibtexFormatter {
    fn format_papers(&self, papers: &[Paper]) -> String {
        papers
            .iter()
            .map(|p| self.format_paper_detail(p))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn format_paper_detail(&self, paper: &Paper) -> String {
        let key = Self::generate_key(paper);
        let entry_type = if paper.arxiv_id.is_some() {
            "misc"
        } else {
            "article"
        };

        let mut output = format!("@{}{{{},\n", entry_type, key);
        output.push_str(&format!("  title = {{{}}},\n", Self::escape_bibtex(&paper.title)));

        if !paper.authors.is_empty() {
            output.push_str(&format!("  author = {{{}}},\n", Self::format_authors(&paper.authors)));
        }

        if let Some(year) = &paper.publication_date {
            if let Some(y) = year.split('-').next() {
                output.push_str(&format!("  year = {{{}}},\n", y));
            }
        }

        if let Some(venue) = &paper.venue {
            output.push_str(&format!("  journal = {{{}}},\n", venue));
        }

        if let Some(doi) = &paper.doi {
            output.push_str(&format!("  doi = {{{}}},\n", doi));
        }

        if let Some(arxiv) = &paper.arxiv_id {
            output.push_str(&format!("  eprint = {{{}}},\n", arxiv));
            output.push_str("  archiveprefix = {arXiv},\n");
        }

        if let Some(url) = &paper.pdf_url {
            output.push_str(&format!("  url = {{{}}},\n", url));
        }

        output.push_str("}");
        output
    }

    fn format_citations(&self, citations: &[(Paper, Vec<Author>)], _direction: CitationDirection) -> String {
        citations
            .iter()
            .map(|(paper, _)| self.format_paper_detail(paper))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn format_stats(&self, _stats: &crate::citation::CitationStats) -> String {
        "% Citation statistics not available in BibTeX format\n".to_string()
    }

    fn extension(&self) -> &'static str {
        "bib"
    }
}
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/output/bibtex.rs
git commit -m "feat(output): implement BibtexFormatter

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 5: 实现 MarkdownFormatter

**Files:**
- Modify: `src/output/markdown.rs`

- [ ] **Step 1: 实现 MarkdownFormatter**

```rust
// src/output/markdown.rs
use super::{CitationDirection, OutputFormatter};
use crate::models::{Author, Paper};

pub struct MarkdownFormatter;

impl MarkdownFormatter {
    fn format_authors(authors: &[Author]) -> String {
        if authors.is_empty() {
            return "N/A".to_string();
        }
        let names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
        names.join(", ")
    }

    fn escape_markdown(s: &str) -> String {
        s.replace('|', "\\|")
            .replace('\n', " ")
    }
}

impl OutputFormatter for MarkdownFormatter {
    fn format_papers(&self, papers: &[Paper]) -> String {
        let mut output = String::from("# Paper List\n\n");
        output.push_str("| ID | Title | Authors | Citations |\n");
        output.push_str("|----|-------|---------|----------|\n");

        for paper in papers {
            let title = Self::escape_markdown(&paper.title);
            let authors = Self::escape_markdown(&Self::format_authors(&paper.authors));
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                paper.id.unwrap_or(0),
                title,
                authors,
                paper.citation_count
            ));
        }

        output
    }

    fn format_paper_detail(&self, paper: &Paper) -> String {
        let mut output = String::new();

        output.push_str(&format!("# {}\n\n", paper.title));

        output.push_str("## Metadata\n\n");
        output.push_str(&format!("- **ID:** {}\n", paper.id.unwrap_or(0)));

        if let Some(arxiv_id) = &paper.arxiv_id {
            if !arxiv_id.is_empty() {
                output.push_str(&format!("- **arXiv:** {}\n", arxiv_id));
            }
        }

        if let Some(ss_id) = &paper.semantic_scholar_id {
            if !ss_id.is_empty() {
                output.push_str(&format!("- **Semantic Scholar:** {}\n", ss_id));
            }
        }

        if let Some(doi) = &paper.doi {
            output.push_str(&format!("- **DOI:** {}\n", doi));
        }

        output.push_str(&format!("- **Authors:** {}\n", Self::format_authors(&paper.authors)));
        output.push_str(&format!("- **Citations:** {}\n", paper.citation_count));

        if let Some(venue) = &paper.venue {
            output.push_str(&format!("- **Venue:** {}\n", venue));
        }

        if let Some(url) = &paper.pdf_url {
            output.push_str(&format!("- **PDF:** {}\n", url));
        }

        if let Some(abs) = &paper.abstract_text {
            output.push_str("\n## Abstract\n\n");
            output.push_str(abs);
            output.push('\n');
        }

        output
    }

    fn format_citations(&self, citations: &[(Paper, Vec<Author>)], direction: CitationDirection) -> String {
        let title = match direction {
            CitationDirection::Citing => "# Citing Papers",
            CitationDirection::Cited => "# Cited Papers",
        };

        let mut output = format!("{}\n\n", title);
        output.push_str("| Title | Authors |\n");
        output.push_str("|-------|--------|\n");

        for (paper, authors) in citations {
            let title = Self::escape_markdown(&paper.title);
            let author_str = Self::escape_markdown(&Self::format_authors(authors));
            output.push_str(&format!("| {} | {} |\n", title, author_str));
        }

        output
    }

    fn format_stats(&self, stats: &crate::citation::CitationStats) -> String {
        let mut output = String::from("# Citation Statistics\n\n");

        output.push_str("## Overview\n\n");
        output.push_str(&format!("- **Total Papers:** {}\n", stats.total_papers));
        output.push_str(&format!("- **Total Citation Edges:** {}\n", stats.total_citation_edges));
        output.push_str(&format!("- **Average Citations:** {:.2}\n", stats.average_citations));
        output.push_str(&format!("- **H-Index:** {}\n", stats.h_index));
        output.push_str(&format!("- **Max Citations:** {}\n", stats.max_citations));

        output.push_str("\n## Most Cited Papers\n\n");
        output.push_str("| Rank | Title | Citations |\n");
        output.push_str("|------|-------|----------|\n");

        for (i, (paper, count)) in stats.most_cited_papers.iter().take(10).enumerate() {
            let title = Self::escape_markdown(&paper.title);
            output.push_str(&format!("| {} | {} | {} |\n", i + 1, title, count));
        }

        output
    }

    fn extension(&self) -> &'static str {
        "md"
    }
}
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/output/markdown.rs
git commit -m "feat(output): implement MarkdownFormatter

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 6: 数据库优化 - 消除 N+1 查询

**Files:**
- Modify: `src/db/database.rs:134-148, 242-257`

- [ ] **Step 1: 优化 list_papers 方法**

找到 `list_papers` 方法，替换为：

```rust
pub async fn list_papers(&self, limit: i64) -> Result<Vec<Paper>> {
    use sea_orm::QuerySelect;

    let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
        .order_by_desc(papers::Column::CreatedAt)
        .limit(limit as u64)
        .find_with_related(authors::Entity)
        .all(&self.conn)
        .await?;

    Ok(results
        .into_iter()
        .map(|(paper, author_models)| {
            let authors = author_models
                .into_iter()
                .map(|a| Author {
                    id: Some(a.id),
                    name: a.name,
                    semantic_scholar_id: a.semantic_scholar_id,
                })
                .collect();

            Paper {
                id: Some(paper.id),
                title: paper.title,
                abstract_text: paper.abstract_text,
                arxiv_id: paper.arxiv_id,
                semantic_scholar_id: paper.semantic_scholar_id,
                doi: paper.doi,
                pdf_url: paper.pdf_url,
                local_pdf_path: paper.local_pdf_path,
                publication_date: paper.publication_date,
                venue: paper.venue,
                citation_count: paper.citation_count,
                authors,
                created_at: paper.created_at,
                updated_at: paper.updated_at,
            }
        })
        .collect())
}
```

- [ ] **Step 2: 优化 search_papers 方法**

找到 `search_papers` 方法，替换为：

```rust
pub async fn search_papers(&self, query: &str, limit: i64) -> Result<Vec<Paper>> {
    let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
        .filter(papers::Column::Title.contains(query))
        .order_by_desc(papers::Column::CitationCount)
        .limit(limit as u64)
        .find_with_related(authors::Entity)
        .all(&self.conn)
        .await?;

    Ok(results
        .into_iter()
        .map(|(paper, author_models)| {
            let authors = author_models
                .into_iter()
                .map(|a| Author {
                    id: Some(a.id),
                    name: a.name,
                    semantic_scholar_id: a.semantic_scholar_id,
                })
                .collect();

            Paper {
                id: Some(paper.id),
                title: paper.title,
                abstract_text: paper.abstract_text,
                arxiv_id: paper.arxiv_id,
                semantic_scholar_id: paper.semantic_scholar_id,
                doi: paper.doi,
                pdf_url: paper.pdf_url,
                local_pdf_path: paper.local_pdf_path,
                publication_date: paper.publication_date,
                venue: paper.venue,
                citation_count: paper.citation_count,
                authors,
                created_at: paper.created_at,
                updated_at: paper.updated_at,
            }
        })
        .collect())
}
```

- [ ] **Step 3: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src/db/database.rs
git commit -m "perf(db): eliminate N+1 queries using find_with_related

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 7: 数据库新增方法

**Files:**
- Modify: `src/db/database.rs` (添加新方法)

- [ ] **Step 1: 添加删除论文方法**

在 `impl Database` 块末尾添加：

```rust
/// 删除论文
pub async fn delete_paper(&self, id: i64) -> Result<bool> {
    let result = papers::Entity::delete_by_id(id)
        .exec(&self.conn)
        .await?;
    Ok(result.rows_affected > 0)
}

/// 获取论文总数
pub async fn count_papers(&self) -> Result<i64> {
    let count = papers::Entity::find()
        .count(&self.conn)
        .await?;
    Ok(count as i64)
}

/// 按引用数排序获取 Top N
pub async fn top_cited_papers(&self, limit: i64) -> Result<Vec<Paper>> {
    let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
        .order_by_desc(papers::Column::CitationCount)
        .limit(limit as u64)
        .find_with_related(authors::Entity)
        .all(&self.conn)
        .await?;

    Ok(results
        .into_iter()
        .map(|(paper, author_models)| {
            let authors = author_models
                .into_iter()
                .map(|a| Author {
                    id: Some(a.id),
                    name: a.name,
                    semantic_scholar_id: a.semantic_scholar_id,
                })
                .collect();

            Paper {
                id: Some(paper.id),
                title: paper.title,
                abstract_text: paper.abstract_text,
                arxiv_id: paper.arxiv_id,
                semantic_scholar_id: paper.semantic_scholar_id,
                doi: paper.doi,
                pdf_url: paper.pdf_url,
                local_pdf_path: paper.local_pdf_path,
                publication_date: paper.publication_date,
                venue: paper.venue,
                citation_count: paper.citation_count,
                authors,
                created_at: paper.created_at,
                updated_at: paper.updated_at,
            }
        })
        .collect())
}

/// 按作者搜索
pub async fn search_by_author(&self, name: &str, limit: i64) -> Result<Vec<Paper>> {
    let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
        .inner_join(authors::Entity)
        .inner_join(paper_authors::Entity)
        .filter(authors::Column::Name.contains(name))
        .order_by_desc(papers::Column::CitationCount)
        .limit(limit as u64)
        .find_with_related(authors::Entity)
        .all(&self.conn)
        .await?;

    Ok(results
        .into_iter()
        .map(|(paper, author_models)| {
            let authors = author_models
                .into_iter()
                .map(|a| Author {
                    id: Some(a.id),
                    name: a.name,
                    semantic_scholar_id: a.semantic_scholar_id,
                })
                .collect();

            Paper {
                id: Some(paper.id),
                title: paper.title,
                abstract_text: paper.abstract_text,
                arxiv_id: paper.arxiv_id,
                semantic_scholar_id: paper.semantic_scholar_id,
                doi: paper.doi,
                pdf_url: paper.pdf_url,
                local_pdf_path: paper.local_pdf_path,
                publication_date: paper.publication_date,
                venue: paper.venue,
                citation_count: paper.citation_count,
                authors,
                created_at: paper.created_at,
                updated_at: paper.updated_at,
            }
        })
        .collect())
}
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/db/database.rs
git commit -m "feat(db): add delete_paper, count_papers, top_cited_papers, search_by_author

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 8: CLI 全局参数 - 添加 --format 和 --output

**Files:**
- Modify: `src/command/mod.rs:13-32`

- [ ] **Step 1: 添加全局参数到 Cli 结构体**

找到 `Cli` 结构体，添加新参数：

```rust
#[derive(Parser)]
#[command(name = "argusflow")]
#[command(about = "文献搜索整理工具", long_about = None)]
pub struct Cli {
    /// PDF 存储路径
    #[arg(long, global = true)]
    pub pdf_dir: Option<std::path::PathBuf>,

    /// 数据库路径
    #[arg(long, global = true)]
    pub db_path: Option<std::path::PathBuf>,

    /// Semantic Scholar API Key
    #[arg(long, global = true)]
    pub ss_api_key: Option<String>,

    /// HTTP/HTTPS 代理地址 (例如: http://127.0.0.1:7890)
    #[arg(short, long, global = true)]
    pub proxy: Option<String>,

    /// 输出格式 (terminal, json, bibtex, markdown)
    #[arg(long, global = true, default_value = "terminal")]
    pub format: String,

    /// 输出文件路径
    #[arg(short, long, global = true)]
    pub output: Option<std::path::PathBuf>,

    /// 跳过本地缓存
    #[arg(long, global = true)]
    pub no_cache: bool,

    #[command(subcommand)]
    pub command: Commands,
}
```

- [ ] **Step 2: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 3: 提交**

```bash
git add src/command/mod.rs
git commit -m "feat(cli): add --format, --output, --no-cache global arguments

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 9: CLI 新增命令定义

**Files:**
- Modify: `src/command/mod.rs:34-129`

- [ ] **Step 1: 添加新命令到 Commands enum**

在现有命令后添加：

```rust
    /// 删除论文
    Delete {
        #[arg(short, long)]
        id: i64,
    },

    /// 更新论文信息（从源重新获取）
    Update {
        #[arg(short, long)]
        id: i64,
    },

    /// 导出论文列表
    Export {
        #[arg(short, long, default_value = "bibtex")]
        format: String,
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        #[arg(short, long)]
        query: Option<String>,
    },

    /// 生成引用关系图
    CitationGraph {
        #[arg(short = 'i', long)]
        paper_id: String,
        #[arg(short, long, default_value = "dot")]
        format: String,
        #[arg(short, long)]
        output: Option<std::path::PathBuf>,
        #[arg(short, long, default_value = "2")]
        depth: usize,
    },

    /// 爬取引用网络
    CrawlCitations {
        #[arg(short = 'i', long)]
        paper_id: String,
        #[arg(short, long, default_value = "1")]
        depth: usize,
        #[arg(long, default_value = "100")]
        max: usize,
        #[arg(short, long, default_value = "both")]
        direction: String,
    },

    /// 显示引用统计
    CitationStats,

    /// 同步引用数（从 Semantic Scholar 更新）
    SyncCitations {
        #[arg(short, long, default_value = "50")]
        batch: usize,
    },
```

- [ ] **Step 2: 改进 List 命令参数**

找到 `List` 命令，替换为：

```rust
    /// 列出数据库中的论文
    List {
        #[arg(short, long, default_value = "20")]
        limit: usize,
        /// 排序方式: created, citation, title
        #[arg(short, long, default_value = "created")]
        sort: String,
        /// 排序方向: asc, desc
        #[arg(long, default_value = "desc")]
        order: String,
    },
```

- [ ] **Step 3: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 4: 提交**

```bash
git add src/command/mod.rs
git commit -m "feat(cli): add Delete, Update, Export, CitationGraph, CrawlCitations, CitationStats, SyncCitations commands

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 10: 实现 handlers 新命令

**Files:**
- Modify: `src/command/handlers.rs`

- [ ] **Step 1: 添加命令路由**

在 `execute` 函数的 match 中添加新分支：

```rust
    match command {
        // ... 现有命令 ...
        Commands::Delete { id } => delete_paper(ctx, *id).await,
        Commands::Update { id } => update_paper(ctx, *id).await,
        Commands::Export { format, output, query } => export_papers(ctx, format, output, query).await,
        Commands::CitationGraph { paper_id, format, output, depth } => {
            generate_citation_graph(ctx, paper_id, format, output, *depth).await
        }
        Commands::CrawlCitations { paper_id, depth, max, direction } => {
            crawl_citations(ctx, paper_id, *depth, *max, direction).await
        }
        Commands::CitationStats => show_citation_stats(ctx).await,
        Commands::SyncCitations { batch } => sync_citations(ctx, *batch).await,
    }
```

- [ ] **Step 2: 添加新命令处理函数**

在文件末尾添加：

```rust
async fn delete_paper(ctx: &CommandContext, id: i64) -> Result<()> {
    match ctx.db.delete_paper(id).await? {
        true => println!("论文 ID {} 已删除", id),
        false => println!("论文 ID {} 不存在", id),
    }
    Ok(())
}

async fn update_paper(ctx: &CommandContext, id: i64) -> Result<()> {
    // 获取现有论文
    let paper = ctx.db.get_paper_by_id(id).await?
        .ok_or_else(|| anyhow::anyhow!("论文 ID {} 不存在", id))?;

    // 确定数据源
    let source = if paper.arxiv_id.is_some() {
        ctx.manager.get(SourceKind::Arxiv)
    } else {
        ctx.manager.get(SourceKind::SemanticScholar)
    };

    let source = source.ok_or_else(|| anyhow::anyhow!("没有可用的数据源"))?;

    // 获取标识符
    let identifier = paper.arxiv_id.as_ref()
        .or(paper.semantic_scholar_id.as_ref())
        .ok_or_else(|| anyhow::anyhow!("论文没有有效的标识符"))?;

    // 重新获取数据
    let updated = source.get_by_identifier(identifier).await?
        .ok_or_else(|| anyhow::anyhow!("无法从源获取论文"))?;

    // 更新数据库
    let mut updated = updated;
    updated.id = Some(id);
    ctx.db.update_paper(&updated).await?;

    println!("论文 ID {} 已更新", id);
    Ok(())
}

async fn export_papers(
    ctx: &CommandContext,
    format: &str,
    output: &Option<std::path::PathBuf>,
    query: &Option<String>,
) -> Result<()> {
    use std::str::FromStr;
    use crate::output::OutputFormat;

    let output_format = OutputFormat::from_str(format)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    let papers = if let Some(q) = query {
        ctx.db.search_papers(q, 1000).await?
    } else {
        ctx.db.list_papers(1000).await?
    };

    let formatter = output_format.formatter();
    let content = formatter.format_papers(&papers);

    if let Some(path) = output {
        std::fs::write(path, content)?;
        println!("已导出到 {}", path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

async fn generate_citation_graph(
    ctx: &CommandContext,
    paper_id: &str,
    format: &str,
    output: &Option<std::path::PathBuf>,
    depth: usize,
) -> Result<()> {
    use crate::citation::{CitationCrawler, CrawlDirection};

    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    let crawler = CitationCrawler::new(source)
        .with_depth(depth)
        .with_max_papers(100);

    let graph = crawler.crawl(paper_id, CrawlDirection::Both).await?;

    let content = match format {
        "json" => graph.to_json()?,
        _ => graph.to_dot(),
    };

    if let Some(path) = output {
        std::fs::write(path, content)?;
        println!("引用图已导出到 {}", path.display());
    } else {
        println!("{}", content);
    }

    Ok(())
}

async fn crawl_citations(
    ctx: &CommandContext,
    paper_id: &str,
    depth: usize,
    max: usize,
    direction: &str,
) -> Result<()> {
    use crate::citation::{CitationCrawler, CrawlDirection};

    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    let crawl_dir = match direction {
        "citations" => CrawlDirection::Citations,
        "references" => CrawlDirection::References,
        _ => CrawlDirection::Both,
    };

    let crawler = CitationCrawler::new(source)
        .with_depth(depth)
        .with_max_papers(max);

    println!("开始爬取引用网络...");
    let graph = crawler.crawl(paper_id, crawl_dir).await?;

    // 保存到数据库
    let mut saved = 0;
    for paper in graph.papers() {
        if paper.id.is_none() {
            ctx.db.insert_paper(paper).await?;
            saved += 1;
        }
    }

    println!("爬取完成，发现 {} 篇论文，新保存 {} 篇", graph.papers().count(), saved);
    Ok(())
}

async fn show_citation_stats(ctx: &CommandContext) -> Result<()> {
    use crate::citation::CitationStats;

    // 从数据库构建引用图
    let papers = ctx.db.list_papers(5000).await?;

    if papers.is_empty() {
        println!("数据库中没有论文");
        return Ok(());
    }

    let stats = CitationStats::from_papers(&papers);

    // 使用默认终端格式输出
    let formatter = crate::output::TerminalFormatter::new();
    println!("{}", formatter.format_stats(&stats));

    Ok(())
}

async fn sync_citations(ctx: &CommandContext, batch: usize) -> Result<()> {
    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    let papers = ctx.db.list_papers(5000).await?;
    let mut updated = 0;

    for chunk in papers.chunks(batch) {
        for paper in chunk {
            if let Some(ss_id) = &paper.semantic_scholar_id {
                if let Some(updated_paper) = source.get_by_id(ss_id).await? {
                    if let Some(id) = paper.id {
                        ctx.db.update_paper(&Paper {
                            id: Some(id),
                            citation_count: updated_paper.citation_count,
                            ..paper.clone()
                        }).await?;
                        updated += 1;
                    }
                }
            }
        }
        println!("已处理 {}/{} 篇论文", updated, papers.len());
    }

    println!("同步完成，更新了 {} 篇论文的引用数", updated);
    Ok(())
}
```

- [ ] **Step 3: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过（可能有未使用警告）

- [ ] **Step 4: 提交**

```bash
git add src/command/handlers.rs
git commit -m "feat(cli): implement Delete, Update, Export, CitationGraph, CrawlCitations, CitationStats, SyncCitations handlers

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 11: 完善 CitationGraph 和 CitationStats

**Files:**
- Modify: `src/citation/graph.rs`
- Modify: `src/citation/stats.rs`
- Modify: `src/citation/crawler.rs`

- [ ] **Step 1: 完善 CitationGraph**

```rust
// src/citation/graph.rs
use std::collections::{HashMap, HashSet};
use crate::models::Paper;

/// 引用图
#[derive(Default)]
pub struct CitationGraph {
    /// 论文节点
    papers: HashMap<i64, Paper>,
    /// 引用边: paper_id -> 该论文引用的论文ID列表
    citations: HashMap<i64, Vec<i64>>,
    /// 反向引用: paper_id -> 引用该论文的论文ID列表
    cited_by: HashMap<i64, Vec<i64>>,
}

impl CitationGraph {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加论文节点
    pub fn add_paper(&mut self, paper: Paper) {
        if let Some(id) = paper.id {
            self.papers.insert(id, paper);
        }
    }

    /// 添加引用关系
    pub fn add_citation(&mut self, citing_id: i64, cited_id: i64) {
        self.citations
            .entry(citing_id)
            .or_default()
            .push(cited_id);
        self.cited_by
            .entry(cited_id)
            .or_default()
            .push(citing_id);
    }

    /// 获取论文的引用（该论文引用了哪些论文）
    pub fn get_citations(&self, paper_id: i64) -> Option<&[i64]> {
        self.citations.get(&paper_id).map(|v| v.as_slice())
    }

    /// 获取论文被引（哪些论文引用了该论文）
    pub fn get_cited_by(&self, paper_id: i64) -> Option<&[i64]> {
        self.cited_by.get(&paper_id).map(|v| v.as_slice())
    }

    /// 获取所有论文
    pub fn papers(&self) -> impl Iterator<Item = &Paper> {
        self.papers.values()
    }

    /// 获取引用数（被引次数）
    pub fn citation_count(&self, paper_id: i64) -> usize {
        self.cited_by.get(&paper_id).map(|v| v.len()).unwrap_or(0)
    }

    /// 导出为 Graphviz DOT 格式
    pub fn to_dot(&self) -> String {
        let mut output = String::from("digraph citations {\n");
        output.push_str("  rankdir=LR;\n");
        output.push_str("  node [shape=box];\n\n");

        // 节点
        for (id, paper) in &self.papers {
            let label: String = paper.title.chars().take(50).collect();
            output.push_str(&format!("  {} [label=\"{}\"];\n", id, label));
        }

        // 边
        output.push_str("\n");
        for (citing_id, cited_ids) in &self.citations {
            for cited_id in cited_ids {
                output.push_str(&format!("  {} -> {};\n", citing_id, cited_id));
            }
        }

        output.push_str("}\n");
        output
    }

    /// 导出为 JSON 图结构
    pub fn to_json(&self) -> anyhow::Result<String> {
        use serde_json::json;

        let nodes: Vec<_> = self.papers.values().collect();
        let edges: Vec<(i64, i64)> = self.citations.iter()
            .flat_map(|(from, tos)| tos.iter().map(move |to| (*from, *to)))
            .collect();

        let json = json!({
            "nodes": nodes,
            "edges": edges
        });

        serde_json::to_string_pretty(&json)
            .map_err(|e| anyhow::anyhow!(e))
    }
}
```

- [ ] **Step 2: 完善 CitationStats**

```rust
// src/citation/stats.rs
use crate::models::Paper;

/// 引用统计数据
#[derive(Default)]
pub struct CitationStats {
    pub total_papers: usize,
    pub total_citation_edges: usize,
    pub average_citations: f64,
    pub h_index: i32,
    pub max_citations: i64,
    pub most_cited_papers: Vec<(Paper, i64)>,
    pub isolated_papers: Vec<Paper>,
}

impl CitationStats {
    /// 从论文列表计算统计（基于本地 citation_count 字段）
    pub fn from_papers(papers: &[Paper]) -> Self {
        if papers.is_empty() {
            return Self::default();
        }

        let total_papers = papers.len();

        // 按 citation_count 排序
        let mut sorted: Vec<_> = papers.iter()
            .map(|p| (p, p.citation_count))
            .collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));

        let max_citations = sorted.first().map(|(_, c)| *c).unwrap_or(0);
        let total_citations: i64 = papers.iter().map(|p| p.citation_count).sum();
        let average_citations = total_citations as f64 / total_papers as f64;

        // 计算 H-Index
        let mut h_index = 0i32;
        for (i, (_, count)) in sorted.iter().enumerate() {
            if (*count as usize) >= i + 1 {
                h_index = (i + 1) as i32;
            } else {
                break;
            }
        }

        // Top 10 引用最多
        let most_cited_papers: Vec<(Paper, i64)> = sorted.iter()
            .take(10)
            .map(|(p, c)| ((*p).clone(), *c))
            .collect();

        // 孤立节点（引用数为 0）
        let isolated_papers: Vec<Paper> = papers.iter()
            .filter(|p| p.citation_count == 0)
            .cloned()
            .collect();

        Self {
            total_papers,
            total_citation_edges: total_citations as usize,
            average_citations,
            h_index,
            max_citations,
            most_cited_papers,
            isolated_papers,
        }
    }
}
```

- [ ] **Step 3: 完善 CitationCrawler**

```rust
// src/citation/crawler.rs
use std::sync::Arc;
use std::collections::HashSet;
use crate::source::PaperSource;
use crate::models::Paper;
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

    /// 从种子论文爬取引用网络
    pub async fn crawl(
        &self,
        seed_paper_id: &str,
        direction: CrawlDirection,
    ) -> anyhow::Result<CitationGraph> {
        let mut graph = CitationGraph::new();
        let mut visited = HashSet::new();
        let mut queue = vec![(seed_paper_id.to_string(), 0usize)];

        while let Some((paper_id, depth)) = queue.pop() {
            if visited.contains(&paper_id) || depth > self.max_depth {
                continue;
            }

            if graph.papers().count() >= self.max_papers {
                break;
            }

            visited.insert(paper_id.clone());

            // 获取论文
            let paper = match self.source.get_by_identifier(&paper_id).await? {
                Some(p) => p,
                None => continue,
            };

            let paper_db_id = paper.id.unwrap_or_else(|| {
                // 使用 hash 作为临时 ID
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                paper_id.hash(&mut hasher);
                hasher.finish() as i64
            });

            let mut paper = paper;
            if paper.id.is_none() {
                paper.id = Some(paper_db_id);
            }

            graph.add_paper(paper);

            // 获取引用/被引
            if depth < self.max_depth {
                match direction {
                    CrawlDirection::Citations | CrawlDirection::Both => {
                        if let Ok(citations) = self.source.get_citations(&paper_id, 20).await {
                            for (citing_paper, _) in citations {
                                if let Some(id) = &citing_paper.semantic_scholar_id {
                                    queue.push((id.clone(), depth + 1));
                                }
                            }
                        }
                    }
                    CrawlDirection::References | CrawlDirection::Both => {
                        if let Ok(references) = self.source.get_references(&paper_id, 20).await {
                            for (cited_paper, _) in references {
                                if let Some(id) = &cited_paper.semantic_scholar_id {
                                    queue.push((id.clone(), depth + 1));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(graph)
    }
}
```

- [ ] **Step 4: 运行编译验证**

Run: `cargo build 2>&1`
Expected: 编译通过

- [ ] **Step 5: 提交**

```bash
git add src/citation/
git commit -m "feat(citation): implement CitationGraph, CitationStats, CitationCrawler

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 12: 集成测试与验证

**Files:**
- 无新增文件

- [ ] **Step 1: 运行所有测试**

Run: `cargo test 2>&1`
Expected: 所有测试通过

- [ ] **Step 2: 测试基本功能**

Run: `cargo run -- list -l 5 2>&1`
Expected: 显示论文列表

- [ ] **Step 3: 测试 JSON 输出**

Run: `cargo run -- --format json list -l 3 2>&1`
Expected: JSON 格式输出

- [ ] **Step 4: 测试导出功能**

Run: `cargo run -- export --format bibtex 2>&1 | head -20`
Expected: BibTeX 格式输出

- [ ] **Step 5: 测试引用统计**

Run: `cargo run -- citation-stats 2>&1`
Expected: 显示引用统计

- [ ] **Step 6: 最终提交**

```bash
git add -A
git commit -m "feat: complete ArgusFlow optimization

- Add output formatter layer (terminal, json, bibtex, markdown)
- Optimize database queries (eliminate N+1)
- Add citation network module
- Add new CLI commands (delete, update, export, citation-graph, etc.)

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## 验证清单

- [ ] `cargo test` - 所有测试通过
- [ ] `cargo run -- list -l 100` - N+1 优化生效
- [ ] `cargo run -- --format json list -l 10` - JSON 输出正常
- [ ] `cargo run -- export --format bibtex` - BibTeX 导出正常
- [ ] `cargo run -- citation-stats` - 引用统计正常
- [ ] `cargo run -- delete -i 1` - 删除功能正常
- [ ] `cargo clippy` - 无警告