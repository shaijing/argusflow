use anyhow::Result;
use clap::{Parser, Subcommand};
use argusflow::{
    Database, PdfDownloader, Config,
    SourceManager, SourceBuilder, SourceKind, SearchParams, SourceConfig,
};

#[derive(Parser)]
#[command(name = "argusflow")]
#[command(about = "文献搜索整理工具", long_about = None)]
struct Cli {
    /// PDF 存储路径
    #[arg(long, global = true)]
    pdf_dir: Option<std::path::PathBuf>,

    /// 数据库路径
    #[arg(long, global = true)]
    db_path: Option<std::path::PathBuf>,

    /// Semantic Scholar API Key
    #[arg(long, global = true)]
    ss_api_key: Option<String>,

    /// HTTP/HTTPS 代理地址 (例如: http://127.0.0.1:7890)
    #[arg(short, long, global = true)]
    proxy: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 从 arXiv 搜索论文
    ArxivSearch {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        /// 最大结果数
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 从 Semantic Scholar 搜索论文
    SsSearch {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        /// 最大结果数
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 智能搜索（从所有源搜索）
    Search {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        /// 最大结果数
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 通过标识符获取论文（自动识别来源）
    Get {
        /// 论文标识符 (arXiv ID / DOI / SS ID / URL)
        #[arg(short, long)]
        id: String,
    },

    /// 通过 arXiv ID 获取论文详情
    GetArxiv {
        /// arXiv ID
        #[arg(short, long)]
        id: String,
    },

    /// 通过 Semantic Scholar ID 获取论文详情
    GetSs {
        /// Semantic Scholar ID
        #[arg(short, long)]
        id: String,
    },

    /// 获取论文的引用关系
    Citations {
        /// 论文 ID（Semantic Scholar）
        #[arg(short = 'i', long)]
        paper_id: String,
        /// 最大结果数
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// 获取论文的参考文献
    References {
        /// 论文 ID（Semantic Scholar）
        #[arg(short = 'i', long)]
        paper_id: String,
        /// 最大结果数
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// 下载论文 PDF
    Download {
        /// 论文 ID（arXiv ID 或 URL）
        #[arg(short, long)]
        id: String,
    },

    /// 保存论文到数据库
    Save {
        /// 论文标题
        #[arg(short, long)]
        title: String,
        /// arXiv ID
        #[arg(short = 'a', long)]
        arxiv_id: Option<String>,
        /// Semantic Scholar ID
        #[arg(short = 's', long)]
        ss_id: Option<String>,
    },

    /// 列出数据库中的论文
    List {
        /// 限制数量
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// 搜索本地数据库
    LocalSearch {
        /// 搜索关键词
        #[arg(short, long)]
        query: String,
        /// 限制数量
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// 显示配置
    Config,

    /// 列出可用的论文源
    Sources,
}

fn build_source_config(cli: &Cli) -> SourceConfig {
    SourceConfig {
        api_key: cli.ss_api_key.clone(),
        proxy: cli.proxy.clone(),
        timeout: 30,
        max_retries: 5,
        retry_delay: 2000,
    }
}

async fn cache_paper(db: &Database, paper: &argusflow::Paper) -> Result<i64> {
    // 尝试通过 arxiv_id 或 semantic_scholar_id 查找
    let cached = if let Some(arxiv_id) = &paper.arxiv_id {
        if !arxiv_id.is_empty() {
            db.get_paper_by_arxiv_id(arxiv_id)?
        } else {
            None
        }
    } else if let Some(ss_id) = &paper.semantic_scholar_id {
        if !ss_id.is_empty() {
            db.get_paper_by_semantic_scholar_id(ss_id)?
        } else {
            None
        }
    } else {
        None
    };

    match cached {
        Some(existing) => Ok(existing.id.unwrap()),
        None => db.insert_paper(paper),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // 构建配置
    let mut config = Config::default();
    if let Some(ref pdf_dir) = cli.pdf_dir {
        config.pdf_storage_path = pdf_dir.clone();
    }
    if let Some(ref db_path) = cli.db_path {
        config.db_path = db_path.clone();
    }
    if let Some(ref key) = cli.ss_api_key {
        config.semantic_scholar_api_key = Some(key.clone());
    }
    if let Some(ref proxy) = cli.proxy {
        config.proxy = Some(proxy.clone());
    }

    config.ensure_dirs()?;

    // 在 match 之前构建 manager
    let source_config = build_source_config(&cli);
    let mut manager = SourceManager::new();

    // 注册 arXiv 源
    {
        let arxiv_source = SourceBuilder::new()
            .proxy(source_config.proxy.clone().unwrap_or_default())
            .timeout(30)
            .max_retries(3)
            .build_arxiv()?;
        manager.register(arxiv_source);
    }

    // 注册 Semantic Scholar 源
    {
        let ss_source = SourceBuilder::new()
            .api_key(source_config.api_key.clone().unwrap_or_default())
            .proxy(source_config.proxy.clone().unwrap_or_default())
            .timeout(30)
            .max_retries(5)
            .build_semantic_scholar()?;
        manager.register(ss_source);
    }

    let db = Database::new(&config.db_path)?;

    match cli.command {
        Commands::ArxivSearch { query, limit } => {
            let source = manager.get(SourceKind::Arxiv)
                .ok_or_else(|| anyhow::anyhow!("arXiv 源未注册"))?;

            let params = SearchParams {
                query,
                limit,
                ..Default::default()
            };

            let result = source.search(&params).await?;

            println!("找到 {} 篇论文:", result.papers.len());
            for paper in &result.papers {
                let paper_id = cache_paper(&db, paper).await?;

                println!("\n---");
                println!("[DB ID: {}] [来源: {}]", paper_id, source.name());
                println!("标题: {}", paper.title);
                println!("arXiv ID: {}", paper.arxiv_id.as_deref().unwrap_or("N/A"));
                println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
                if let Some(abs) = &paper.abstract_text {
                    println!("摘要: {}...", &abs[..200.min(abs.len())]);
                }
            }
        }

        Commands::SsSearch { query, limit } => {
            let source = manager.get(SourceKind::SemanticScholar)
                .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

            let params = SearchParams {
                query,
                limit,
                ..Default::default()
            };

            let result = source.search(&params).await?;

            println!("找到 {} 篇论文:", result.papers.len());
            for paper in &result.papers {
                let paper_id = cache_paper(&db, paper).await?;

                println!("\n---");
                println!("[DB ID: {}] [来源: {}]", paper_id, source.name());
                println!("标题: {}", paper.title);
                println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
                println!("引用数: {}", paper.citation_count);
                println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
            }
        }

        Commands::Search { query, limit } => {
            let results = manager.smart_search(&query, limit).await?;

            println!("找到 {} 篇论文:", results.len());
            for (kind, paper) in &results {
                let paper_id = cache_paper(&db, paper).await?;

                println!("\n---");
                println!("[DB ID: {}] [来源: {}]", paper_id, kind);
                println!("标题: {}", paper.title);
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
                println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
            }
        }

        Commands::Get { id } => {
            match manager.fetch_by_identifier(&id).await? {
                Some((kind, paper)) => {
                    let paper_id = cache_paper(&db, &paper).await?;

                    println!("[DB ID: {}] [来源: {}]", paper_id, kind);
                    println!("标题: {}", paper.title);
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
                    println!("DOI: {}", paper.doi.as_deref().unwrap_or("N/A"));
                    println!("引用数: {}", paper.citation_count);
                    println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
                    println!("\n摘要:\n{}", paper.abstract_text.as_deref().unwrap_or("N/A"));
                }
                None => println!("未找到论文"),
            }
        }

        Commands::GetArxiv { id } => {
            let source = manager.get(SourceKind::Arxiv)
                .ok_or_else(|| anyhow::anyhow!("arXiv 源未注册"))?;

            match source.get_by_identifier(&id).await? {
                Some(paper) => {
                    let paper_id = cache_paper(&db, &paper).await?;

                    println!("[DB ID: {}] [来源: {}]", paper_id, source.name());
                    println!("标题: {}", paper.title);
                    println!("arXiv ID: {}", paper.arxiv_id.as_deref().unwrap_or("N/A"));
                    println!("DOI: {}", paper.doi.as_deref().unwrap_or("N/A"));
                    println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
                    println!("\n摘要:\n{}", paper.abstract_text.as_deref().unwrap_or("N/A"));
                }
                None => println!("未找到论文"),
            }
        }

        Commands::GetSs { id } => {
            let source = manager.get(SourceKind::SemanticScholar)
                .ok_or_else(|| anyhow::anyhow!("Semantic Scholar 源未注册"))?;

            match source.get_by_identifier(&id).await? {
                Some(paper) => {
                    let paper_id = cache_paper(&db, &paper).await?;

                    println!("[DB ID: {}] [来源: {}]", paper_id, source.name());
                    println!("标题: {}", paper.title);
                    println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
                    println!("DOI: {}", paper.doi.as_deref().unwrap_or("N/A"));
                    println!("引用数: {}", paper.citation_count);
                    println!("PDF: {}", paper.pdf_url.as_deref().unwrap_or("N/A"));
                    println!("\n摘要:\n{}", paper.abstract_text.as_deref().unwrap_or("N/A"));
                }
                None => println!("未找到论文"),
            }
        }

        Commands::Citations { paper_id, limit } => {
            let citations = manager.get_citations(&paper_id, limit).await?;

            println!("该论文被以下 {} 篇论文引用:", citations.len());
            for (paper, authors) in citations {
                println!("\n---");
                println!("标题: {}", paper.title);
                println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
                let author_names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
                println!("作者: {}", author_names.join(", "));
            }
        }

        Commands::References { paper_id, limit } => {
            let refs = manager.get_references(&paper_id, limit).await?;

            println!("该论文引用了以下 {} 篇论文:", refs.len());
            for (paper, authors) in refs {
                println!("\n---");
                println!("标题: {}", paper.title);
                println!("SS ID: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
                let author_names: Vec<&str> = authors.iter().map(|a| a.name.as_str()).collect();
                println!("作者: {}", author_names.join(", "));
            }
        }

        Commands::Download { id } => {
            let proxy_ref = config.proxy.as_deref();
            let downloader = PdfDownloader::new_with_proxy(proxy_ref)?;

            if id.starts_with("http") {
                let filename = PdfDownloader::extract_filename(&id).unwrap_or_else(|| "paper.pdf".to_string());
                let dest = config.pdf_storage_path.join(filename);
                println!("下载 {} -> {}", id, dest.display());
                downloader.download(&id, &dest).await?;
                println!("下载完成: {}", dest.display());
            } else {
                let dest = config.pdf_path(&id);
                println!("下载 arXiv:{} -> {}", id, dest.display());
                downloader.download_arxiv_pdf(&id, &dest).await?;
                println!("下载完成: {}", dest.display());
            }
        }

        Commands::Save { title, arxiv_id, ss_id } => {
            let db = Database::new(&config.db_path)?;

            let mut paper = argusflow::Paper::new(title);
            if let Some(arxiv) = arxiv_id {
                paper = paper.with_arxiv_id(arxiv);
            }
            if let Some(ss) = ss_id {
                paper = paper.with_semantic_scholar_id(ss);
            }

            let id = db.insert_paper(&paper)?;
            println!("论文已保存，ID: {}", id);
        }

        Commands::List { limit } => {
            let db = Database::new(&config.db_path)?;
            let papers = db.list_papers(limit as i64)?;

            println!("共 {} 篇论文:", papers.len());
            for paper in papers {
                println!("\nID: {}", paper.id.unwrap());
                println!("标题: {}", paper.title);
                println!("arXiv: {}", paper.arxiv_id.as_deref().unwrap_or("N/A"));
                println!("SS: {}", paper.semantic_scholar_id.as_deref().unwrap_or("N/A"));
                println!("引用数: {}", paper.citation_count);
            }
        }

        Commands::LocalSearch { query, limit } => {
            let db = Database::new(&config.db_path)?;
            let papers = db.search_papers(&query, limit as i64)?;

            println!("找到 {} 篇论文:", papers.len());
            for paper in papers {
                println!("\nID: {}", paper.id.unwrap());
                println!("标题: {}", paper.title);
                println!("引用数: {}", paper.citation_count);
            }
        }

        Commands::Config => {
            println!("PDF 存储路径: {}", config.pdf_storage_path.display());
            println!("数据库路径: {}", config.db_path.display());
            println!("引用深度: {}", config.citation_depth);
            println!("API Key: {}", config.semantic_scholar_api_key.clone().unwrap_or_else(|| "未设置".to_string()));
            println!("代理: {}", config.proxy.clone().unwrap_or_else(|| "未设置".to_string()));
        }

        Commands::Sources => {
            println!("已注册的论文源:");
            for kind in manager.list_sources() {
                if let Some(source) = manager.get(kind) {
                    let caps = source.capabilities();
                    println!("\n{} ({})", source.name(), kind);
                    println!("  搜索: {}", if caps.search { "✓" } else { "✗" });
                    println!("  获取: {}", if caps.get_by_id { "✓" } else { "✗" });
                    println!("  引用: {}", if caps.citations { "✓" } else { "✗" });
                    println!("  参考文献: {}", if caps.references { "✓" } else { "✗" });
                    println!("  PDF下载: {}", if caps.pdf_download { "✓" } else { "✗" });
                }
            }
        }
    }

    Ok(())
}