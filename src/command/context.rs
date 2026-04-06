//! 命令执行上下文

use crate::{ArgusFlow, ArgusFlowBuilder, Config, SourceManager};
use anyhow::Result;

/// 命令执行所需的上下文
///
/// 这是 ArgusFlow 核心库的 CLI 包装器，提供便捷的 CLI 功能
pub struct CommandContext {
    /// ArgusFlow 核心实例
    pub core: ArgusFlow,
}

impl CommandContext {
    /// 从 CLI 参数构建上下文
    pub async fn from_cli(
        pdf_dir: Option<std::path::PathBuf>,
        db_path: Option<std::path::PathBuf>,
        ss_api_key: Option<String>,
        proxy: Option<String>,
    ) -> Result<Self> {
        let mut builder = ArgusFlowBuilder::new();

        if let Some(path) = pdf_dir {
            builder = builder.pdf_dir(path);
        }
        if let Some(path) = db_path {
            builder = builder.db_path(path);
        }
        if let Some(key) = ss_api_key {
            builder = builder.api_key(key);
        }
        if let Some(p) = proxy {
            builder = builder.proxy(p);
        }

        let core = builder.build().await?;
        Ok(Self { core })
    }

    /// 获取配置
    pub fn config(&self) -> &Config {
        self.core.config()
    }

    /// 获取源管理器
    pub fn manager(&self) -> &SourceManager {
        self.core.sources()
    }

    /// 缓存论文到数据库（便捷方法）
    pub async fn cache_paper(&self, paper: &crate::Paper) -> Result<i64> {
        self.core.save(paper).await
    }
}