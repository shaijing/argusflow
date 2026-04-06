//! CLI 命令定义和执行逻辑

mod context;
mod handlers;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub use context::CommandContext;

#[derive(Parser)]
#[command(name = "argusflow")]
#[command(about = "文献搜索整理工具", long_about = None)]
pub struct Cli {
    /// PDF 存储路径
    #[arg(long, global = true)]
    pub pdf_dir: Option<PathBuf>,

    /// 数据库路径
    #[arg(long, global = true)]
    pub db_path: Option<PathBuf>,

    /// Semantic Scholar API Key
    #[arg(long, global = true)]
    pub ss_api_key: Option<String>,

    /// HTTP/HTTPS 代理地址 (例如: http://127.0.0.1:7890)
    #[arg(short, long, global = true)]
    pub proxy: Option<String>,

    /// 输出格式: terminal, json, bibtex, markdown
    #[arg(long, global = true, default_value = "terminal")]
    pub format: String,

    /// 输出文件路径
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,

    /// 跳过本地缓存，强制从源获取
    #[arg(long, global = true)]
    pub no_cache: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 从 arXiv 搜索论文
    ArxivSearch {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 从 Semantic Scholar 搜索论文
    SsSearch {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 从 OpenAlex 搜索论文
    OaSearch {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 智能搜索（从所有源搜索）
    Search {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// 通过标识符获取论文（自动识别来源）
    Get {
        #[arg(short, long)]
        id: String,
    },

    /// 通过 arXiv ID 获取论文详情
    GetArxiv {
        #[arg(short, long)]
        id: String,
    },

    /// 通过 Semantic Scholar ID 获取论文详情
    GetSs {
        #[arg(short, long)]
        id: String,
    },

    /// 获取论文的引用关系
    Citations {
        #[arg(short = 'i', long)]
        paper_id: String,
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// 获取论文的参考文献
    References {
        #[arg(short = 'i', long)]
        paper_id: String,
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// 下载论文 PDF
    Download {
        #[arg(short, long)]
        id: String,
    },

    /// 保存论文到数据库
    Save {
        #[arg(short, long)]
        title: String,
        #[arg(short = 'a', long)]
        arxiv_id: Option<String>,
        #[arg(short = 's', long)]
        ss_id: Option<String>,
    },

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

    /// 搜索本地数据库
    LocalSearch {
        #[arg(short, long)]
        query: String,
        #[arg(short, long, default_value = "20")]
        limit: usize,
        /// 搜索字段: title, abstract, author, all
        #[arg(short, long, default_value = "all")]
        field: String,
    },

    /// 显示配置
    Config,

    /// 列出可用的论文源
    Sources,

    // === New commands ===

    /// 删除论文
    Delete {
        #[arg(short, long)]
        id: i64,
    },

    /// 更新论文信息（从源重新获取引用数等）
    Update {
        #[arg(short, long)]
        id: i64,
    },

    /// 导出论文列表
    Export {
        #[arg(short, long, default_value = "bibtex")]
        format: String,
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// 可选：只导出匹配查询的论文
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
        output: Option<PathBuf>,
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
}

/// 执行命令
pub async fn execute(ctx: &CommandContext, command: &Commands) -> anyhow::Result<()> {
    handlers::execute(ctx, command).await
}