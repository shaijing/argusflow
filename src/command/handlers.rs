//! 命令处理函数

use super::{CommandContext, Commands};
use crate::{CitationStats, OutputFormat, SearchParams, SourceKind, Paper, PdfDownloader};
use anyhow::Result;
use std::fs::File;
use std::io::Write;

pub async fn execute(ctx: &CommandContext, command: &Commands) -> Result<()> {
    match command {
        Commands::ArxivSearch { query, limit } => search_arxiv(ctx, query, *limit).await,
        Commands::SsSearch { query, limit } => search_ss(ctx, query, *limit).await,
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

        // New commands
        Commands::Delete { id } => delete_paper(ctx, *id).await,
        Commands::Update { id } => update_paper(ctx, *id).await,
        Commands::Export { format, output, query } => export_papers(ctx, format, output, query).await,
        Commands::CitationGraph { paper_id, format, output, depth } => citation_graph(ctx, paper_id, format, output, *depth).await,
        Commands::CrawlCitations { paper_id, depth, max, direction } => crawl_citations(ctx, paper_id, *depth, *max, direction).await,
        Commands::CitationStats => citation_stats(ctx).await,
        Commands::SyncCitations { batch } => sync_citations(ctx, *batch).await,
    }
}

async fn search_arxiv(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let source = ctx.manager.get(SourceKind::Arxiv)
        .ok_or_else(|| anyhow::anyhow!("arXiv 源未注册"))?;

    let params = SearchParams {
        query: query.to_string(),
        limit,
        ..Default::default()
    };

    let result = source.search(&params).await?;

    println!("找到 {} 篇论文:", result.papers.len());
    for paper in &result.papers {
        let paper_id = ctx.cache_paper(paper).await?;

        println!("\n---");
        println!("[DB ID: {}] [来源: {}]", paper_id, source.name());
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
    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    let params = SearchParams {
        query: query.to_string(),
        limit,
        ..Default::default()
    };

    let result = source.search(&params).await?;

    println!("找到 {} 篇论文:", result.papers.len());
    for paper in &result.papers {
        let paper_id = ctx.cache_paper(paper).await?;

        println!("\n---");
        println!("[DB ID: {}] [来源: {}]", paper_id, source.name());
        println!("标题: {}", paper.title);
        println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
        print_authors(&paper.authors);
        println!("引用数: {}", paper.citation_count);
        println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    }

    Ok(())
}

async fn smart_search(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let results = ctx.manager.smart_search(query, limit).await?;

    println!("找到 {} 篇论文:", results.len());
    for (kind, paper) in &results {
        let paper_id = ctx.cache_paper(paper).await?;

        println!("\n---");
        println!("[DB ID: {}] [来源: {}]", paper_id, kind);
        println!("标题: {}", paper.title);
        print_identifier(paper);
        print_authors(&paper.authors);
        println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    }

    Ok(())
}

async fn get_by_identifier(ctx: &CommandContext, id: &str) -> Result<()> {
    match ctx.manager.fetch_by_identifier(id).await? {
        Some((kind, paper)) => {
            let paper_id = ctx.cache_paper(&paper).await?;
            print_paper_detail(paper_id, &kind, &paper);
        }
        None => println!("未找到论文"),
    }
    Ok(())
}

async fn get_arxiv(ctx: &CommandContext, id: &str) -> Result<()> {
    let source = ctx.manager.get(SourceKind::Arxiv)
        .ok_or_else(|| anyhow::anyhow!("arXiv 源未注册"))?;

    match source.get_by_identifier(id).await? {
        Some(paper) => {
            let paper_id = ctx.cache_paper(&paper).await?;
            print_paper_detail(paper_id, source.name(), &paper);
        }
        None => println!("未找到论文"),
    }
    Ok(())
}

async fn get_ss(ctx: &CommandContext, id: &str) -> Result<()> {
    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    match source.get_by_identifier(id).await? {
        Some(paper) => {
            let paper_id = ctx.cache_paper(&paper).await?;
            print_paper_detail(paper_id, source.name(), &paper);
        }
        None => println!("未找到论文"),
    }
    Ok(())
}

async fn get_citations(ctx: &CommandContext, paper_id: &str, limit: usize) -> Result<()> {
    let citations = ctx.manager.get_citations(paper_id, limit).await?;

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
    let refs = ctx.manager.get_references(paper_id, limit).await?;

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
    let downloader = ctx.pdf_downloader()?;

    if id.starts_with("http") {
        let filename = PdfDownloader::extract_filename(id)
            .unwrap_or_else(|| "paper.pdf".to_string());
        let dest = ctx.config.pdf_storage_path.join(filename);
        println!("下载 {} -> {}", id, dest.display());
        downloader.download(id, &dest).await?;
        println!("下载完成: {}", dest.display());
    } else {
        let dest = ctx.config.pdf_path(id);
        println!("下载 arXiv:{} -> {}", id, dest.display());
        downloader.download_arxiv_pdf(id, &dest).await?;
        println!("下载完成: {}", dest.display());
    }
    Ok(())
}

async fn save_paper(ctx: &CommandContext, title: &str, arxiv_id: &Option<String>, ss_id: &Option<String>) -> Result<()> {
    let mut paper = Paper::new(title.to_string());
    if let Some(arxiv) = arxiv_id {
        paper = paper.with_arxiv_id(arxiv.clone());
    }
    if let Some(ss) = ss_id {
        paper = paper.with_semantic_scholar_id(ss.clone());
    }

    let id = ctx.db.insert_paper(&paper).await?;
    println!("论文已保存，ID: {}", id);
    Ok(())
}

async fn list_papers(ctx: &CommandContext, limit: usize, sort: &str, order: &str) -> Result<()> {
    let papers = match sort {
        "citation" => ctx.db.top_cited_papers(limit as i64).await?,
        _ => ctx.db.list_papers(limit as i64).await?,
    };

    // Apply order reversal if needed
    let papers = if order == "asc" {
        papers.into_iter().rev().collect::<Vec<_>>()
    } else {
        papers
    };

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
        "author" => ctx.db.search_by_author(query, limit as i64).await?,
        _ => ctx.db.search_papers(query, limit as i64).await?,
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
    println!("PDF 存储路径: {}", ctx.config.pdf_storage_path.display());
    println!("数据库路径: {}", ctx.config.db_path.display());
    println!("引用深度: {}", ctx.config.citation_depth);
    println!(
        "API Key: {}",
        ctx.config.semantic_scholar_api_key.clone().unwrap_or_else(|| "未设置".to_string())
    );
    println!("代理: {}", ctx.config.proxy.clone().unwrap_or_else(|| "未设置".to_string()));
    Ok(())
}

fn show_sources(ctx: &CommandContext) -> Result<()> {
    println!("已注册的论文源:");
    for kind in ctx.manager.list_sources() {
        if let Some(source) = ctx.manager.get(kind) {
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

// === New command handlers ===

async fn delete_paper(ctx: &CommandContext, id: i64) -> Result<()> {
    match ctx.db.delete_paper(id).await? {
        true => println!("论文 {} 已删除", id),
        false => println!("论文 {} 不存在", id),
    }
    Ok(())
}

async fn update_paper(ctx: &CommandContext, id: i64) -> Result<()> {
    // Get existing paper
    let paper = ctx.db.get_paper_by_id(id).await?
        .ok_or_else(|| anyhow::anyhow!("论文 {} 不存在", id))?;

    // Try to fetch updated info from Semantic Scholar
    if let Some(ss_id) = &paper.semantic_scholar_id {
        let source = ctx.manager.get(SourceKind::SemanticScholar)
            .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

        if let Some(updated) = source.get_by_id(ss_id).await? {
            let mut paper = paper.clone();
            paper.citation_count = updated.citation_count;
            paper.updated_at = chrono::Utc::now();
            ctx.db.update_paper(&paper).await?;
            println!("论文 {} 已更新，引用数: {}", id, paper.citation_count);
        } else {
            println!("无法从 Semantic Scholar 获取更新");
        }
    } else {
        println!("论文没有 Semantic Scholar ID，无法更新");
    }
    Ok(())
}

async fn export_papers(ctx: &CommandContext, format: &str, output: &Option<std::path::PathBuf>, query: &Option<String>) -> Result<()> {
    // Get papers to export
    let papers = if let Some(q) = query {
        ctx.db.search_papers(q, 1000).await?
    } else {
        ctx.db.list_papers(1000).await?
    };

    // Create formatter
    let output_format: OutputFormat = format.parse()
        .map_err(|e: String| anyhow::anyhow!("{}", e))?;
    let formatter = output_format.formatter();

    let content = formatter.format_papers(&papers);

    // Output
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
    // Build citation graph from database
    use crate::CitationGraph;

    let mut graph = CitationGraph::new();

    // Get papers from database that have citation relationships
    let papers = ctx.db.list_papers(1000).await?;
    for paper in papers {
        if let Some(id) = paper.id {
            graph.add_paper(paper);

            // Get citations (papers this paper cites)
            let cited_ids = ctx.db.get_citations(id).await?;
            for cited_id in cited_ids {
                graph.add_citation(id, cited_id);
            }

            // Get cited_by (papers citing this paper)
            let citing_ids = ctx.db.get_cited_by(id).await?;
            for citing_id in citing_ids {
                graph.add_citation(citing_id, id);
            }
        }
    }

    let content = match format {
        "dot" => graph.to_dot(),
        "json" => graph.to_json()?,
        _ => return Err(anyhow::anyhow!("不支持格式: {}", format)),
    };

    if let Some(path) = output {
        let mut file = File::create(path)?;
        file.write_all(content.as_bytes())?;
        println!("已导出引用图到 {}", path.display());
    } else {
        println!("{}", content);
    }
    Ok(())
}

async fn crawl_citations(ctx: &CommandContext, paper_id: &str, depth: usize, max: usize, direction: &str) -> Result<()> {
    use crate::{CitationCrawler, CrawlDirection};

    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    let crawler = CitationCrawler::new(source.clone(), depth, max);

    let dir = match direction {
        "citations" => CrawlDirection::Citations,
        "references" => CrawlDirection::References,
        _ => CrawlDirection::Both,
    };

    println!("开始爬取引用网络 (深度: {}, 最大: {})...", depth, max);
    let graph = crawler.crawl(paper_id, dir).await?;

    // Save papers to database
    let count = graph.papers().count();
    println!("爬取完成，共 {} 篇论文", count);

    for paper in graph.papers() {
        ctx.db.insert_paper(paper).await?;
    }

    println!("已保存到数据库");
    Ok(())
}

async fn citation_stats(ctx: &CommandContext) -> Result<()> {
    use crate::CitationGraph;

    let mut graph = CitationGraph::new();

    let papers = ctx.db.list_papers(1000).await?;
    for paper in papers {
        if let Some(id) = paper.id {
            graph.add_paper(paper);

            let cited_ids = ctx.db.get_citations(id).await?;
            for cited_id in cited_ids {
                graph.add_citation(id, cited_id);
            }

            let citing_ids = ctx.db.get_cited_by(id).await?;
            for citing_id in citing_ids {
                graph.add_citation(citing_id, id);
            }
        }
    }

    let stats = CitationStats::from_graph(&graph);

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
    let source = ctx.manager.get(SourceKind::SemanticScholar)
        .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

    let papers = ctx.db.list_papers(1000).await?;

    let mut updated = 0;
    let mut failed = 0;

    println!("开始同步引用数...");

    for paper in papers.iter().take(batch) {
        if let Some(ss_id) = &paper.semantic_scholar_id {
            if paper.id.is_some() {
                if let Some(updated_paper) = source.get_by_id(ss_id).await? {
                    let mut p = paper.clone();
                    p.citation_count = updated_paper.citation_count;
                    p.updated_at = chrono::Utc::now();
                    ctx.db.update_paper(&p).await?;
                    updated += 1;
                    println!("更新 {} -> 引用数: {}", paper.title, p.citation_count);
                } else {
                    failed += 1;
                }
            }
        }
    }

    println!("同步完成: 更新 {}, 失败 {}", updated, failed);
    Ok(())
}

// 辅助函数
fn print_identifier(paper: &Paper) {
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

fn print_paper_detail(db_id: i64, source: impl std::fmt::Display, paper: &Paper) {
    println!("[DB ID: {}] [来源: {}]", db_id, source);
    println!("标题: {}", paper.title);
    print_identifier(paper);
    print_authors(&paper.authors);
    println!("DOI: {}", paper.doi.as_deref().unwrap_or("N/A"));
    println!("引用数: {}", paper.citation_count);
    println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
    println!("\n摘要:\n{}", paper.abstract_text.as_deref().unwrap_or("N/A"));
}