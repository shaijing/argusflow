//! 命令处理函数

use super::{CommandContext, Commands};
use crate::{SearchParams, SourceKind, Paper, PdfDownloader};
use anyhow::Result;

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
        Commands::Save { title, arxiv_id, ss_id } => save_paper(ctx, title, arxiv_id, ss_id),
        Commands::List { limit } => list_papers(ctx, *limit),
        Commands::LocalSearch { query, limit } => local_search(ctx, query, *limit),
        Commands::Config => show_config(ctx),
        Commands::Sources => show_sources(ctx),
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

fn save_paper(ctx: &CommandContext, title: &str, arxiv_id: &Option<String>, ss_id: &Option<String>) -> Result<()> {
    let mut paper = Paper::new(title.to_string());
    if let Some(arxiv) = arxiv_id {
        paper = paper.with_arxiv_id(arxiv.clone());
    }
    if let Some(ss) = ss_id {
        paper = paper.with_semantic_scholar_id(ss.clone());
    }

    let id = ctx.db.insert_paper(&paper)?;
    println!("论文已保存，ID: {}", id);
    Ok(())
}

fn list_papers(ctx: &CommandContext, limit: usize) -> Result<()> {
    let papers = ctx.db.list_papers(limit as i64)?;

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

fn local_search(ctx: &CommandContext, query: &str, limit: usize) -> Result<()> {
    let papers = ctx.db.search_papers(query, limit as i64)?;

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