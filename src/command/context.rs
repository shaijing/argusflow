//! 命令执行上下文

use crate::{
    Config, Database, PdfDownloader,
    SourceManager, SourceBuilder,
};
use anyhow::Result;

/// 命令执行所需的上下文
pub struct CommandContext {
    pub config: Config,
    pub db: Database,
    pub manager: SourceManager,
}

impl CommandContext {
    /// 从 CLI 参数构建上下文
    pub fn from_cli(
        pdf_dir: Option<std::path::PathBuf>,
        db_path: Option<std::path::PathBuf>,
        ss_api_key: Option<String>,
        proxy: Option<String>,
    ) -> Result<Self> {
        let mut config = Config::default();
        if let Some(path) = pdf_dir {
            config.pdf_storage_path = path;
        }
        if let Some(path) = db_path {
            config.db_path = path;
        }
        if let Some(key) = ss_api_key {
            config.semantic_scholar_api_key = Some(key);
        }
        if let Some(p) = proxy {
            config.proxy = Some(p);
        }

        config.ensure_dirs()?;

        let db = Database::new(&config.db_path)?;
        let manager = build_manager(&config)?;

        Ok(Self { config, db, manager })
    }

    /// 获取 PDF 下载器
    pub fn pdf_downloader(&self) -> Result<PdfDownloader> {
        PdfDownloader::new_with_proxy(self.config.proxy.as_deref())
    }

    /// 缓存论文到数据库
    pub async fn cache_paper(&self, paper: &crate::Paper) -> Result<i64> {
        let cached = if let Some(arxiv_id) = &paper.arxiv_id {
            if !arxiv_id.is_empty() {
                self.db.get_paper_by_arxiv_id(arxiv_id)?
            } else {
                None
            }
        } else if let Some(ss_id) = &paper.semantic_scholar_id {
            if !ss_id.is_empty() {
                self.db.get_paper_by_semantic_scholar_id(ss_id)?
            } else {
                None
            }
        } else {
            None
        };

        match cached {
            Some(existing) => Ok(existing.id.unwrap()),
            None => self.db.insert_paper(paper),
        }
    }
}

fn build_manager(config: &Config) -> Result<SourceManager> {
    let mut manager = SourceManager::new();

    let mut arxiv_builder = SourceBuilder::new()
        .timeout(30)
        .max_retries(3);

    if let Some(ref proxy) = config.proxy {
        if !proxy.is_empty() {
            arxiv_builder = arxiv_builder.proxy(proxy);
        }
    }
    manager.register(arxiv_builder.build_arxiv()?);

    let mut ss_builder = SourceBuilder::new()
        .timeout(30)
        .max_retries(5);

    if let Some(ref key) = config.semantic_scholar_api_key {
        if !key.is_empty() {
            ss_builder = ss_builder.api_key(key);
        }
    }
    if let Some(ref proxy) = config.proxy {
        if !proxy.is_empty() {
            ss_builder = ss_builder.proxy(proxy);
        }
    }
    manager.register(ss_builder.build_semantic_scholar()?);

    Ok(manager)
}