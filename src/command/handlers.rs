//! 命令处理函数
//!
//! CLI 层 - 仅负责调用核心 API 和格式化输出

use super::{CommandContext, Commands};
use crate::{GraphFormat, OutputFormat, SortBy, SourceKind};
use anyhow::Result;
use std::fs::File;
use std::io::Write;

pub async fn execute(ctx: &CommandContext, command: &Commands) -> Result<()> {
    match command {
        Commands::ArxivSearch { query, limit } => search_arxiv(ctx, query, *limit).await,
        Commands::SsSearch { query, limit } => search_ss(ctx, query, *limit).await,
        Commands::OaSearch { query, limit } => search_oa(ctx, query, *limit).await,
        Commands::Search { query, limit } => smart_search(ctx, query, *limit).await,
        Commands::Get { id } => get_by_identifier(ctx, id).await,
        Commands::GetArxiv { id } => get_arxiv(ctx, id).await,
        Commands::GetSs { id } => get_ss(ctx, id).await,
        Commands::Citations { paper_id, limit } => get_citations(ctx, paper_id, *limit).await,
        Commands::References { paper_id, limit } => get_references(ctx, paper_id, *limit).await,
        Commands::Download { id } => download_pdf(ctx, id).await,
        Commands::Save { title, arxiv_id, ss_id } => save_paper(ctx, title, arxiv_id, ss_id).await,
        Commands::List { limit, sort, order } => list_papers(ctx, *limit, sort, order).await,
        Commands::LocalSearch { query, limit, field } => local_search(ctx, query, *limit, field).await,
        Commands::Config => show_config(ctx),
        Commands::Sources => show_sources(ctx),

        Commands::Delete { id } => delete_paper(ctx, *id).await,
        Commands::Update { id } => update_paper(ctx, *id).await,
        Commands::Export { format, output, query } => export_papers(ctx, format, output, query).await,
        Commands::CitationGraph { paper_id, format, output, depth } => citation_graph(ctx, paper_id, format, output, *depth).await,
        Commands::CrawlCitations { paper_id, depth, max, direction } => crawl_citations(ctx, paper_id, *depth, *max, direction).await,
        Commands::CitationStats => citation_stats(ctx).await,
        Commands::SyncCitations { batch } => sync_citations(ctx, *batch).await,
    }
}

// === 搜索命令 ===

async fn search_arxiv(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let papers = ctx.core.search_from(SourceKind::Arxiv, query, limit).await?;

    println!("找到 {} 篇论文:", papers.len());
    for paper in &papers {
        let paper_id = ctx.cache_paper(paper).await?;
        println!("\n---");
        println!("[DB ID: {}] [来源: arXiv]", paper_id);
        println!("标题: {}", paper.title);
        println!("arXiv ID: {}", paper.arxiv_id.as_deref().unwrap_or("N/A"));
        print_authors(&paper.authors);
        println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
        if let Some(abs) = &paper.abstract_text {
            println!("摘要: {}...", &abs[..200.min(abs.len())]);
        }
    }
    Ok(())
}

async fn search_ss(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let papers = ctx.core.search_from(SourceKind::SemanticScholar, query, limit).await?;

    println!("找到 {} 篇论文:", papers.len());
    for paper in &papers {
        let paper_id = ctx.cache_paper(paper).await?;
        println!("\n---");
        println!("[DB ID: {}] [来源: Semantic Scholar]", paper_id);
        println!("标题: {}", paper.title);
        println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
        print_authors(&paper.authors);
        println!("引用数: {}", paper.citation_count);
        println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    }
    Ok(())
}

async fn search_oa(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let papers = ctx.core.search_from(SourceKind::OpenAlex, query, limit).await?;

    println!("找到 {} 篇论文:", papers.len());
    for paper in &papers {
        let paper_id = ctx.cache_paper(paper).await?;
        println!("\n---");
        println!("[DB ID: {}] [来源: OpenAlex]", paper_id);
        println!("标题: {}", paper.title);
        println!("OpenAlex ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
        print_authors(&paper.authors);
        println!("引用数: {}", paper.citation_count);
        println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    }
    Ok(())
}

async fn smart_search(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let papers = ctx.core.search(query, limit).await?;

    println!("找到 {} 篇论文:", papers.len());
    for paper in &papers {
        let paper_id = ctx.cache_paper(paper).await?;
        println!("\n---");
        println!("[DB ID: {}]", paper_id);
        println!("标题: {}", paper.title);
        print_identifier(paper);
        print_authors(&paper.authors);
        println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    }
    Ok(())
}

async fn get_by_identifier(ctx: &CommandContext, id: &str) -> Result<()> {
    match ctx.core.fetch(id).await? {
        Some(paper) => {
            let paper_id = ctx.cache_paper(&paper).await?;
            print_paper_detail(paper_id, "auto-detected", &paper);
        }
        None => println!("未找到论文"),
    }
    Ok(())
}

async fn get_arxiv(ctx: &CommandContext, id: &str) -> Result<()> {
    match ctx.core.fetch(&format!("arxiv:{}", id)).await? {
        Some(paper) => {
            let paper_id = ctx.cache_paper(&paper).await?;
            print_paper_detail(paper_id, "arXiv", &paper);
        }
        None => println!("未找到论文"),
    }
    Ok(())
}

async fn get_ss(ctx: &CommandContext, id: &str) -> Result<()> {
    match ctx.core.fetch(&format!("ss:{}", id)).await? {
        Some(paper) => {
            let paper_id = ctx.cache_paper(&paper).await?;
            print_paper_detail(paper_id, "Semantic Scholar", &paper);
        }
        None => println!("未找到论文"),
    }
    Ok(())
}

async fn get_citations(ctx: &CommandContext, paper_id: &str, limit: usize) -> Result<()> {
    let citations = ctx.core.citations(paper_id, limit).await?;

    println!("该论文被以下 {} 篇论文引用:", citations.len());
    for (paper, authors) in citations {
        println!("\n---");
        println!("标题: {}", paper.title);
        println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
        let author_names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
        println!("作者: {}", author_names.join(", "));
    }
    Ok(())
}

async fn get_references(ctx: &CommandContext, paper_id: &str, limit: usize) -> Result<()> {
    let refs = ctx.core.references(paper_id, limit).await?;

    println!("该论文引用了以下 {} 篇论文:", refs.len());
    for (paper, authors) in refs {
        println!("\n---");
        println!("标题: {}", paper.title);
        println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
        let author_names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
        println!("作者: {}", author_names.join(", "));
    }
    Ok(())
}

async fn download_pdf(ctx: &CommandContext, id: &str) -> Result<()> {
    let dest = ctx.core.download_pdf(id).await?;
    println!("下载完成: {}", dest.display());
    Ok(())
}

async fn save_paper(ctx: &CommandContext, title: &str, arxiv_id: &Option<String>, ss_id: &Option<String>) -> Result<()> {
    let mut paper = crate::Paper::new(title.to_string());
    if let Some(arxiv) = arxiv_id {
        paper = paper.with_arxiv_id(arxiv.clone());
    }
    if let Some(ss) = ss_id {
        paper = paper.with_semantic_scholar_id(ss.clone());
    }

    let id = ctx.core.save(&paper).await?;
    println!("论文已保存，ID: {}", id);
    Ok(())
}

async fn list_papers(ctx: &CommandContext, limit: usize, sort: &str, order: &str) -> Result<()> {
    let sort_by = match sort {
        "citation" => SortBy::Citation,
        _ => SortBy::Created,
    };

    let mut papers = ctx.core.list(limit, sort_by).await?;

    if order == "asc" {
        papers.reverse();
    }

    println!("共 {} 篇论文:", papers.len());
    for paper in papers {
        println!("\nID: {}", paper.id.unwrap());
        println!("标题: {}", paper.title);
        println!("arXiv: {}", paper.arxiv_id.as_deref().unwrap_or("N/A"));
        println!("SS: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
        print_authors(&paper.authors);
        println!("引用数: {}", paper.citation_count);
    }
    Ok(())
}

async fn local_search(ctx: &CommandContext, query: &str, limit: usize, field: &str) -> Result<()> {
    let papers = match field {
        "author" => ctx.core.search_by_author(query, limit).await?,
        _ => ctx.core.search_local(query, limit).await?,
    };

    println!("找到 {} 篇论文:", papers.len());
    for paper in papers {
        println!("\nID: {}", paper.id.unwrap());
        println!("标题: {}", paper.title);
        println!("引用数: {}", paper.citation_count);
    }
    Ok(())
}

fn show_config(ctx: &CommandContext) -> Result<()> {
    let config = ctx.config();
    println!("PDF 存储路径: {}", config.pdf_storage_path.display());
    println!("数据库路径: {}", config.db_path.display());
    println!("引用深度: {}", config.citation_depth);
    println!(
        "API Key: {}",
        config.semantic_scholar_api_key.clone().unwrap_or_else(|| "未设置".to_string())
    );
    println!("代理: {}", config.proxy.clone().unwrap_or_else(|| "未设置".to_string()));
    Ok(())
}

fn show_sources(ctx: &CommandContext) -> Result<()> {
    println!("已注册的论文源:");
    for kind in ctx.manager().list_sources() {
        if let Some(source) = ctx.manager().get(kind) {
            let caps = source.capabilities();
            println!("\n{} ({})", source.name(), kind);
            println!("  搜索: {}", if caps.search { "✓" } else { "✗" });
            println!("  获取: {}", if caps.get_by_id { "✓" } else { "✗" });
            println!("  引用: {}", if caps.citations { "✓" } else { "✗" });
            println!("  参考文献: {}", if caps.references { "✓" } else { "✗" });
            println!("  PDF下载: {}", if caps.pdf_download { "✓" } else { "✗" });
        }
    }
    Ok(())
}

// === 新命令处理 ===

async fn delete_paper(ctx: &CommandContext, id: i64) -> Result<()> {
    match ctx.core.delete(id).await? {
        true => println!("论文 {} 已删除", id),
        false => println!("论文 {} 不存在", id),
    }
    Ok(())
}

async fn update_paper(ctx: &CommandContext, id: i64) -> Result<()> {
    match ctx.core.update(id).await? {
        true => println!("论文 {} 已更新", id),
        false => println!("论文 {} 无法更新（可能没有 Semantic Scholar ID）", id),
    }
    Ok(())
}

async fn export_papers(ctx: &CommandContext, format: &str, output: &Option<std::path::PathBuf>, query: &Option<String>) -> Result<()> {
    let papers = if let Some(q) = query {
        ctx.core.search_local(q, 1000).await?
    } else {
        ctx.core.list(1000, SortBy::Created).await?
    };

    let output_format: OutputFormat = format.parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;

    let content = ctx.core.export(&papers, output_format);

    if let Some(path) = output {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!("已导出 {} 篇论文到 {}", papers.len(), path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

async fn citation_graph(ctx: &CommandContext, _paper_id: &str, format: &str, output: &Option<std::path::PathBuf>, _depth: usize) -> Result<()> {
    let graph = ctx.core.build_citation_graph().await?;

    let graph_format = match format {
        "json" => GraphFormat::Json,
        _ => GraphFormat::Dot,
    };

    let content = ctx.core.export_citation_graph(&graph, graph_format)?;

    if let Some(path) = output {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!("已导出引用图到 {}", path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

async fn crawl_citations(_ctx: &CommandContext, _paper_id: &str, _depth: usize, _max: usize, _direction: &str) -> Result<()> {
    // TODO: 实现 crawl_citations
    println!("crawl-citations 功能尚未完全实现");
    Ok(())
}

async fn citation_stats(ctx: &CommandContext) -> Result<()> {
    let stats = ctx.core.citation_stats().await?;

    println!("=== 引用统计 ===");
    println!("论文总数: {}", stats.total_papers);
    println!("引用关系数: {}", stats.total_citation_edges);
    println!("平均引用数: {:.2}", stats.average_citations);
    println!("H-index: {}", stats.h_index);
    println!("最高引用数: {}", stats.max_citations);

    println!("\n引用最多的论文:");
    for (paper, count) in stats.most_cited_papers.iter().take(5) {
        println!("  {} (引用数: {})", paper.title, count);
    }

    if !stats.isolated_papers.is_empty() {
        println!("\n孤立论文 (无引用关系): {}", stats.isolated_papers.len());
    }
    Ok(())
}

async fn sync_citations(ctx: &CommandContext, batch: usize) -> Result<()> {
    println!("开始同步引用数...");
    let (updated, failed) = ctx.core.sync_citations(batch).await?;
    println!("同步完成: 更新 {}, 失败 {}", updated, failed);
    Ok(())
}

// === 辅助函数 ===

fn print_identifier(paper: &crate::Paper) {
    if let Some(arxiv_id) = &paper.arxiv_id {
        if !arxiv_id.is_empty() {
            println!("arXiv ID: {}", arxiv_id);
        }
    }
    if let Some(ss_id) = &paper.semantic_scholar_id {
        if !ss_id.is_empty() {
            println!("SS ID: {}", ss_id);
        }
    }
}

fn print_authors(authors: &[crate::Author]) {
    if !authors.is_empty() {
        let names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
        println!("作者: {}", names.join(", "));
    }
}

fn print_paper_detail(db_id: i64, source: impl std::fmt::Display, paper: &crate::Paper) {
    println!("[DB ID: {}] [来源: {}]", db_id, source);
    println!("标题: {}", paper.title);
    print_identifier(paper);
    print_authors(&paper.authors);
    println!("DOI: {}", paper.doi.as_deref().unwrap_or("N/A"));
    println!("引用数: {}", paper.citation_count);
    println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    println!("\n摘要:\n{}", paper.abstract_text.as_deref().unwrap_or("N/A"));
}