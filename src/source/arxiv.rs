//! arXiv 论文源实现

use crate::models::{Author, Paper};
use crate::source::*;
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::sleep;

const ARXIV_API_URL: &str = "http://export.arxiv.org/api/query";

/// arXiv 论文源
pub struct ArxivSource {
    client: reqwest::Client,
    config: SourceConfig,
}

impl ArxivSource {
    pub fn new(config: SourceConfig) -> Result<Self, SourceError> {
        let mut builder = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.timeout));

        if let Some(proxy_url) = &config.proxy {
            let proxy = reqwest::Proxy::all(proxy_url)
                .map_err(|e| SourceError::Network(e.to_string()))?;
            builder = builder.proxy(proxy);
        }

        let client = builder.build()
            .map_err(|e| SourceError::Network(e.to_string()))?;

        Ok(Self { client, config })
    }

    fn build_search_url(&self, params: &SearchParams) -> String {
        let sort_by = params.sort_by.as_deref().unwrap_or("relevance");
        let sort_order = params.sort_order.as_deref().unwrap_or("descending");

        format!(
            "{}?search_query={}&start={}&max_results={}&sortBy={}&sortOrder={}",
            ARXIV_API_URL,
            Self::url_encode(&params.query),
            params.offset,
            params.limit,
            sort_by,
            sort_order
        )
    }

    fn url_encode(s: &str) -> String {
        s.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                    c.to_string()
                } else {
                    format!("%{:02X}", c as u32)
                }
            })
            .collect()
    }

    async fn fetch_with_retry(&self, url: &str) -> Result<String, SourceError> {
        let mut last_error = None;

        for _ in 0..self.config.max_retries {
            let response = self.client.get(url).send().await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    return resp.text().await.map_err(|e| SourceError::Network(e.to_string()));
                }
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 429 {
                        sleep(Duration::from_millis(self.config.retry_delay)).await;
                        last_error = Some(SourceError::RateLimit { retry_after: None });
                    } else {
                        last_error = Some(SourceError::Network(format!("HTTP {}", status)));
                    }
                }
                Err(e) => {
                    last_error = Some(SourceError::Network(e.to_string()));
                }
            }

            sleep(Duration::from_millis(self.config.retry_delay)).await;
        }

        Err(last_error.unwrap_or(SourceError::Network("Max retries exceeded".into())))
    }

    fn parse_response(&self, xml: &str) -> Result<Vec<Paper>, SourceError> {
        let mut papers = Vec::new();

        for entry in xml.split("<entry>").skip(1) {
            if let Some(end) = entry.find("</entry>") {
                let entry_xml = &entry[..end];
                if let Some(paper) = self.parse_entry(entry_xml) {
                    papers.push(paper);
                }
            }
        }

        Ok(papers)
    }

    fn parse_entry(&self, xml: &str) -> Option<Paper> {
        let title = self.extract_tag(xml, "title")?;
        let summary = self.extract_tag(xml, "summary").unwrap_or_default();
        let id = self.extract_tag(xml, "id")?;
        let doi = self.extract_tag(xml, "arxiv:doi");

        let arxiv_id = id
            .strip_prefix("http://arxiv.org/abs/")
            .or_else(|| id.strip_prefix("https://arxiv.org/abs/"))
            .map(|s| s.to_string());

        let pdf_url = self.extract_pdf_link(xml);

        Some(Paper::new(title.trim().to_string())
            .with_abstract(summary.trim().to_string())
            .with_arxiv_id(arxiv_id.unwrap_or_default())
            .with_doi(doi.unwrap_or_default())
            .with_pdf_url(pdf_url.unwrap_or_default()))
    }

    fn extract_tag(&self, xml: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);

        let start = xml.find(&start_tag)?;
        let content_start = start + start_tag.len();
        let end = xml[content_start..].find(&end_tag)?;

        Some(xml[content_start..content_start + end].to_string())
    }

    fn extract_pdf_link(&self, xml: &str) -> Option<String> {
        for link in xml.split("<link ") {
            if link.contains("title=\"pdf\"") || link.contains("type=\"application/pdf\"") {
                if let Some(href_start) = link.find("href=\"") {
                    let rest = &link[href_start + 6..];
                    if let Some(end) = rest.find("\"") {
                        return Some(rest[..end].to_string());
                    }
                }
            }
        }
        None
    }
}

#[async_trait]
impl PaperSource for ArxivSource {
    fn kind(&self) -> SourceKind {
        SourceKind::Arxiv
    }

    fn name(&self) -> &str {
        "arXiv"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            search: true,
            get_by_id: true,
            citations: false,
            references: false,
            authors: false,
            pdf_download: true,
        }
    }

    async fn search(&self, params: &SearchParams) -> Result<SearchResult, SourceError> {
        let url = self.build_search_url(params);
        let xml = self.fetch_with_retry(&url).await?;
        let papers = self.parse_response(&xml)?;

        Ok(SearchResult {
            total: Some(papers.len()),
            has_more: papers.len() == params.limit,
            papers,
        })
    }

    async fn get_by_identifier(&self, identifier: &str) -> Result<Option<Paper>, SourceError> {
        let id = match Identifier::parse(identifier) {
            Identifier::Arxiv(id) => id,
            Identifier::Url(url) => {
                // 从 URL 提取 arXiv ID
                url.strip_prefix("https://arxiv.org/abs/")
                    .or_else(|| url.strip_prefix("http://arxiv.org/abs/"))
                    .map(|s| s.to_string())
                    .unwrap_or(url)
            }
            _ => identifier.to_string(),
        };

        let params = SearchParams {
            query: format!("id:{}", id),
            limit: 1,
            ..Default::default()
        };

        let result = self.search(&params).await?;
        Ok(result.papers.into_iter().next())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Paper>, SourceError> {
        self.get_by_identifier(id).await
    }

    async fn get_citations(&self, _paper_id: &str, _limit: usize) -> Result<Vec<(Paper, Vec<Author>)>, SourceError> {
        Err(SourceError::Other("arXiv 不支持获取引用关系".into()))
    }

    async fn get_references(&self, _paper_id: &str, _limit: usize) -> Result<Vec<(Paper, Vec<Author>)>, SourceError> {
        Err(SourceError::Other("arXiv 不支持获取参考文献".into()))
    }

    async fn health_check(&self) -> Result<bool, SourceError> {
        let url = format!("{}?search_query=test&max_results=1", ARXIV_API_URL);
        self.fetch_with_retry(&url).await?;
        Ok(true)
    }
}