# ArgusFlow 全面优化设计

## 背景

ArgusFlow 是文献搜索整理 CLI 工具，当前支持 arXiv 和 Semantic Scholar 数据源，使用 SeaORM + SQLite 存储论文数据。

**主要使用场景**：研究分析工具，构建引用网络，分析论文关系

**数据规模**：中等规模 (500-5000篇)

**目标**：性能优化、功能完善、输出增强、引用网络支持

## 设计概览

```
src/
├── output/           # 新增：输出格式抽象层
│   ├── mod.rs
│   ├── terminal.rs   # 终端文本输出
│   ├── json.rs       # JSON 输出
│   ├── bibtex.rs     # BibTeX 格式
│   └── markdown.rs   # Markdown 表格
├── citation/         # 新增：引用网络模块
│   ├── mod.rs
│   ├── graph.rs      # 引用图数据结构
│   ├── crawler.rs    # 引用网络爬取
│   └── stats.rs      # 引用统计分析
├── db/               # 修改：性能优化
│   ├── database.rs   # 消除 N+1，添加批量操作
│   └── migration/    # 新增索引
└── command/          # 修改：新增命令和参数
    ├── mod.rs        # 添加 --format 全局参数
    └── handlers.rs   # 调用 output formatter
```

---

## 第一部分：输出层抽象

### 1.1 OutputFormatter Trait

```rust
// src/output/mod.rs
pub trait OutputFormatter: Send + Sync {
    fn format_papers(&self, papers: &[Paper]) -> String;
    fn format_paper_detail(&self, paper: &Paper) -> String;
    fn format_citations(&self, citations: &[(Paper, Vec<Author>)], direction: CitationDirection) -> String;
    fn format_stats(&self, stats: &CitationStats) -> String;
    fn extension(&self) -> &'static str;
}

pub enum CitationDirection {
    Citing,    // 被哪些论文引用
    Cited,     // 引用了哪些论文
}

pub enum OutputFormat {
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
}
```

### 1.2 各格式实现要点

**TerminalFormatter**：
- 保持当前输出样式
- 添加颜色支持（可选，检测 tty）
- 分页显示（使用 pager crate）

**JsonFormatter**：
- 使用 `serde_json::to_string_pretty`
- 结构化输出，包含所有字段

**BibtexFormatter**：
- 标准 BibTeX 格式
- 支持 `@article` 和 `@misc` 类型
- 自动生成 citation key（作者+年份）

**MarkdownFormatter**：
- 表格格式显示列表
- YAML front matter 显示详情

### 1.3 CLI 参数

```bash
# 全局参数
--format <FORMAT>    # terminal, json, bibtex, markdown
--output <FILE>      # 输出到文件

# 示例
cargo run -- --format json list -l 10
cargo run -- export --format bibtex --output papers.bib
```

---

## 第二部分：数据库性能优化

### 2.1 消除 N+1 查询

**当前问题**：
```rust
// list_papers: 1 + N 次查询
for model in models {
    let authors = self.get_paper_authors(model.id).await?;  // N 次
}
```

**优化方案**：

```rust
pub async fn list_papers(&self, limit: i64) -> Result<Vec<Paper>> {
    // 方案 A：使用 find_with_related (SeaORM 原生)
    let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
        .order_by_desc(papers::Column::CreatedAt)
        .limit(limit as u64)
        .find_with_related(authors::Entity)
        .all(&self.conn)
        .await?;
    
    results.into_iter()
        .map(|(paper, authors)| {
            Ok(Paper {
                id: Some(paper.id),
                title: paper.title,
                // ... 其他字段
                authors: authors.into_iter()
                    .map(|a| Author { id: Some(a.id), name: a.name, semantic_scholar_id: a.semantic_scholar_id })
                    .collect(),
                // ...
            })
        })
        .collect()
}

pub async fn search_papers(&self, query: &str, limit: i64) -> Result<Vec<Paper>> {
    papers::Entity::find()
        .filter(papers::Column::Title.contains(query))
        .order_by_desc(papers::Column::CitationCount)
        .limit(limit as u64)
        .find_with_related(authors::Entity)
        .all(&self.conn)
        .await?
        .into_iter()
        .map(|(p, a)| model_to_paper(p, a))
        .collect()
}
```

### 2.2 新增批量操作

```rust
impl Database {
    /// 批量插入论文（用于引用网络爬取）
    pub async fn insert_papers_batch(&self, papers: &[Paper]) -> Result<Vec<i64>> {
        // 使用事务批量插入
        let txn = self.conn.begin().await?;
        let mut ids = Vec::new();
        for paper in papers {
            // 插入论文和作者
            let id = self.insert_paper_in_txn(&txn, paper).await?;
            ids.push(id);
        }
        txn.commit().await?;
        Ok(ids)
    }
    
    /// 批量更新引用数
    pub async fn update_citation_counts(&self, updates: &[(i64, i64)]) -> Result<()> {
        let txn = self.conn.begin().await?;
        for (paper_id, count) in updates {
            papers::Entity::update_many()
                .col_expr(papers::Column::CitationCount, Expr::value(*count))
                .filter(papers::Column::Id.eq(*paper_id))
                .exec(&txn)
                .await?;
        }
        txn.commit().await?;
        Ok(())
    }
    
    /// 删除论文（级联删除关联）
    pub async fn delete_paper(&self, id: i64) -> Result<bool> {
        let result = papers::Entity::delete_by_id(id)
            .exec(&self.conn)
            .await?;
        Ok(result.rows_affected > 0)
    }
}
```

### 2.3 新增数据库索引

```rust
// 新增 migration: m20240101_000005_add_indexes
pub struct Migration;

impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 引用数排序索引
        manager.create_index(Index::create()
            .name("idx_papers_citation_count")
            .table(Papers::Table)
            .col(Papers::CitationCount)
            .to_owned()).await?;
        
        // 作者查询优化索引
        manager.create_index(Index::create()
            .name("idx_paper_authors_composite")
            .table(PaperAuthors::Table)
            .col(PaperAuthors::PaperId)
            .col(PaperAuthors::AuthorOrder)
            .to_owned()).await?;
        
        Ok(())
    }
}
```

### 2.4 新增查询方法

```rust
impl Database {
    /// 获取论文总数
    pub async fn count_papers(&self) -> Result<i64>;
    
    /// 按引用数排序获取 Top N
    pub async fn top_cited_papers(&self, limit: i64) -> Result<Vec<Paper>>;
    
    /// 获取最近 N 天添加的论文
    pub async fn recent_papers(&self, days: i64) -> Result<Vec<Paper>>;
    
    /// 按作者搜索
    pub async fn search_by_author(&self, name: &str, limit: i64) -> Result<Vec<Paper>>;
}
```

---

## 第三部分：引用网络模块

### 3.1 数据结构

```rust
// src/citation/graph.rs
use std::collections::{HashMap, HashSet};

/// 引用图
pub struct CitationGraph {
    /// 论文节点
    papers: HashMap<i64, Paper>,
    /// 引用边: paper_id -> 该论文引用的论文ID列表
    citations: HashMap<i64, Vec<i64>>,
    /// 反向引用: paper_id -> 引用该论文的论文ID列表
    cited_by: HashMap<i64, Vec<i64>>,
}

impl CitationGraph {
    pub fn new() -> Self;
    
    /// 添加论文节点
    pub fn add_paper(&mut self, paper: Paper);
    
    /// 添加引用关系
    pub fn add_citation(&mut self, citing_id: i64, cited_id: i64);
    
    /// 获取论文的引用（该论文引用了哪些论文）
    pub fn get_citations(&self, paper_id: i64) -> Option<&[i64]>;
    
    /// 获取论文被引（哪些论文引用了该论文）
    pub fn get_cited_by(&self, paper_id: i64) -> Option<&[i64]>;
    
    /// 获取所有论文
    pub fn papers(&self) -> impl Iterator<Item = &Paper>;
    
    /// 获取引用数
    pub fn citation_count(&self, paper_id: i64) -> usize;
}

// src/citation/stats.rs
/// 引用统计数据
pub struct CitationStats {
    pub total_papers: usize,
    pub total_citation_edges: usize,
    pub average_citations: f64,
    pub h_index: i32,
    pub max_citations: i64,
    pub most_cited_papers: Vec<(Paper, i64)>,
    /// 孤立节点（无引用关系的论文）
    pub isolated_papers: Vec<Paper>,
}

impl CitationStats {
    pub fn from_graph(graph: &CitationGraph) -> Self;
}
```

### 3.2 引用网络爬取

```rust
// src/citation/crawler.rs
pub struct CitationCrawler {
    source: Arc<dyn PaperSource>,
    max_depth: usize,
    max_papers: usize,
}

impl CitationCrawler {
    pub fn new(source: Arc<dyn PaperSource>) -> Self;
    
    /// 从种子论文爬取引用网络
    /// direction: "citations" (被引) 或 "references" (引用)
    pub async fn crawl(
        &self,
        seed_paper_id: &str,
        direction: CrawlDirection,
    ) -> Result<CitationGraph>;
}

pub enum CrawlDirection {
    Citations,   // 爬取被哪些论文引用
    References,  // 爬取引用了哪些论文
    Both,        // 双向爬取
}
```

### 3.3 导出格式

```rust
impl CitationGraph {
    /// 导出为 Graphviz DOT 格式
    pub fn to_dot(&self) -> String {
        let mut output = String::from("digraph citations {\n");
        output.push_str("  rankdir=LR;\n");
        output.push_str("  node [shape=box];\n\n");
        
        // 节点
        for (id, paper) in &self.papers {
            let label = paper.title.chars().take(50).collect::<String>();
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
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&json!({
            "nodes": self.papers.values().collect::<Vec<_>>(),
            "edges": self.citations.iter()
                .flat_map(|(from, tos)| tos.iter().map(move |to| (*from, *to)))
                .collect::<Vec<_>>()
        })).map_err(|e| anyhow::anyhow!(e))
    }
}
```

---

## 第四部分：CLI 功能完善

### 4.1 新增命令

```rust
#[derive(Subcommand)]
pub enum Commands {
    // === 现有命令 ===
    ArxivSearch { ... },
    SsSearch { ... },
    Search { ... },
    Get { ... },
    GetArxiv { ... },
    GetSs { ... },
    Citations { ... },
    References { ... },
    Download { ... },
    Save { ... },
    List { ... },
    LocalSearch { ... },
    Config,
    Sources,
    
    // === 新增命令 ===
    
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
        output: Option<PathBuf>,
        #[arg(short, long)]
        query: Option<String>,  // 可选：只导出匹配的论文
    },
    
    /// 生成引用关系图
    CitationGraph {
        #[arg(short = 'i', long)]
        paper_id: String,
        #[arg(short, long, default_value = "dot")]
        format: String,  // dot, json
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(short, long, default_value = "2")]
        depth: usize,  // 图的深度
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
        direction: String,  // citations, references, both
    },
    
    /// 显示引用统计
    CitationStats,
    
    /// 同步引用数（从 Semantic Scholar 更新）
    SyncCitations {
        #[arg(short, long, default_value = "50")]
        batch: usize,
    },
}
```

### 4.2 全局参数

```rust
#[derive(Parser)]
pub struct Cli {
    // 现有参数
    #[arg(long, global = true)]
    pub pdf_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    pub db_path: Option<PathBuf>,
    #[arg(long, global = true)]
    pub ss_api_key: Option<String>,
    #[arg(short, long, global = true)]
    pub proxy: Option<String>,
    
    // 新增全局参数
    /// 输出格式
    #[arg(long, global = true, default_value = "terminal")]
    pub format: String,
    
    /// 输出文件
    #[arg(short, long, global = true)]
    pub output: Option<PathBuf>,
    
    /// 跳过本地缓存
    #[arg(long, global = true)]
    pub no_cache: bool,
}
```

### 4.3 改进列表命令

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

/// 搜索本地数据库
LocalSearch {
    #[arg(short, long)]
    query: String,
    #[arg(short, long, default_value = "20")]
    limit: usize,
    /// 搜索字段: title, abstract, all
    #[arg(short, long, default_value = "all")]
    field: String,
},
```

---

## 实现顺序

1. **输出层抽象** - 基础设施，其他功能依赖
2. **数据库优化** - 性能提升，后续功能受益
3. **CLI 功能完善** - 删除/更新/导出
4. **引用网络模块** - 独立模块，最后实现

## 验证计划

1. `cargo test` - 所有单元测试通过
2. `cargo run -- list -l 100` - 验证 N+1 优化效果
3. `cargo run -- --format json list -l 10` - 验证输出格式
4. `cargo run -- export --format bibtex` - 验证导出功能
5. `cargo run -- citation-stats` - 验证引用统计
6. `cargo run -- crawl-citations -i "xxx" --depth 1` - 验证爬取功能