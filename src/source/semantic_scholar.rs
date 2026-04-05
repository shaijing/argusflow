//! Semantic Scholar 论文源实现

use crate::models::{Author, Paper};
use crate::source::*;
use async_trait::async_trait;
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

const SS_API_URL: &str = "https://api.semanticscholar.org/graph/v1";

// API 响应结构
#[derive(Debug, Deserialize)]
struct SsSearchResponse {
    data: Vec<SsPaper>,
    total: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct SsPaper {
    #[serde(rename = "paperId")]
    paper_id: String,
    title: String,
    #[serde(rename = "abstract", default)]
    abstract_text: Option<String>,
    #[serde(default)]
    year: Option<i64>,
    #[serde(default)]
    venue: Option<String>,
    #[serde(rename = "citationCount", default)]
    citation_count: Option<i64>,
    #[serde(rename = "externalIds", default)]
    external_ids: Option<SsExternalIds>,
    #[serde(default)]
    authors: Vec<SsAuthor>,
    #[serde(rename = "openAccessPdf", default)]
    open_access_pdf: Option<SsOpenAccess>,
}

#[derive(Debug, Deserialize)]
struct SsExternalIds {
    #[serde(rename = "DOI")]
    doi: Option<String>,
    #[serde(rename = "ArXiv")]
    arxiv_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SsAuthor {
    #[serde(rename = "authorId")]
    author_id: Option<String>,
    name: String,
}

#[derive(Debug, Deserialize)]
struct SsOpenAccess {
    #[serde(rename = "url")]
    pdf_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SsCitationsResponse {
    data: Vec<SsCitation>,
}

#[derive(Debug, Deserialize)]
struct SsCitation {
    #[serde(rename = "citingPaper")]
    citing_paper: Option<SsCitingPaper>,
}

#[derive(Debug, Deserialize)]
struct SsCitingPaper {
    #[serde(rename = "paperId")]
    paper_id: String,
    title: Option<String>,
    #[serde(default)]
    authors: Vec<SsAuthor>,
}

#[derive(Debug, Deserialize)]
struct SsReferencesResponse {
    data: Vec<SsReference>,
}

#[derive(Debug, Deserialize)]
struct SsReference {
    #[serde(rename = "citedPaper")]
    cited_paper: Option<SsCitedPaper>,
}

#[derive(Debug, Deserialize)]
struct SsCitedPaper {
    #[serde(rename = "paperId")]
    paper_id: String,
    title: Option<String>,
    #[serde(default)]
    authors: Vec<SsAuthor>,
}

/// Semantic Scholar 论文源
pub struct SemanticScholarSource {
    client: reqwest::Client,
    config: SourceConfig,
}

impl SemanticScholarSource {
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

    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = &self.config.api_key {
            headers.insert("x-api-key", key.parse().unwrap());
        }
        headers
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

    async fn fetch_with_retry<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, SourceError> {
        let mut last_error = None;

        for _ in 0..self.config.max_retries {
            let response = self.client
                .get(url)
                .headers(self.build_headers())
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    return resp.json::<T>().await.map_err(|e| SourceError::Parse(e.to_string()));
                }
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 429 {
                        sleep(Duration::from_millis(self.config.retry_delay)).await;
                        last_error = Some(SourceError::RateLimit { retry_after: None });
                    } else if status.as_u16() == 404 {
                        return Err(SourceError::NotFound);
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

    fn ss_to_paper(&self, ss: SsPaper) -> Paper {
        let (doi, arxiv_id) = ss.external_ids
            .map(|ext| (ext.doi, ext.arxiv_id))
            .unwrap_or((None, None));

        let pdf_url = ss
            .open_access_pdf
            .and_then(|oa| oa.pdf_url)
            .or_else(|| arxiv_id.as_ref().map(|id| format!("https://arxiv.org/pdf/{}", id)));

        Paper::new(ss.title)
            .with_semantic_scholar_id(ss.paper_id)
            .with_abstract(ss.abstract_text.unwrap_or_default())
            .with_doi(doi.unwrap_or_default())
            .with_arxiv_id(arxiv_id.unwrap_or_default())
            .with_pdf_url(pdf_url.unwrap_or_default())
            .with_citation_count(ss.citation_count.unwrap_or(0))
    }
}

#[async_trait]
impl PaperSource for SemanticScholarSource {
    fn kind(&self) -> SourceKind {
        SourceKind::SemanticScholar
    }

    fn name(&self) -> &str {
        "Semantic Scholar"
    }

    fn capabilities(&self) -> SourceCapabilities {
        SourceCapabilities {
            search: true,
            get_by_id: true,
            citations: true,
            references: true,
            authors: true,
            pdf_download: false,
        }
    }

    async fn search(&self, params: &SearchParams) -> Result<SearchResult, SourceError> {
        let url = format!(
            "{}/paper/search?query={}&offset={}&limit={}&fields=paperId,title,abstract,year,venue,citationCount,externalIds,authors,openAccessPdf",
            SS_API_URL,
            Self::url_encode(&params.query),
            params.offset,
            params.limit
        );

        let response: SsSearchResponse = self.fetch_with_retry(&url).await?;
        let papers: Vec<Paper> = response.data.into_iter().map(|p| self.ss_to_paper(p)).collect();

        Ok(SearchResult {
            total: response.total.map(|t| t as usize),
            has_more: papers.len() == params.limit,
            papers,
        })
    }

    async fn get_by_identifier(&self, identifier: &str) -> Result<Option<Paper>, SourceError> {
        let id = match Identifier::parse(identifier) {
            Identifier::SemanticScholar(id) => id,
            Identifier::Arxiv(arxiv_id) => {
                return self.get_by_arxiv_id(&arxiv_id).await;
            }
            Identifier::Doi(doi) => {
                return self.get_by_doi(&doi).await;
            }
            _ => identifier.to_string(),
        };

        self.get_by_id(&id).await
    }

    async fn get_by_id(&self, paper_id: &str) -> Result<Option<Paper>, SourceError> {
        let url = format!(
            "{}/paper/{}?fields=paperId,title,abstract,year,venue,citationCount,externalIds,authors,openAccessPdf",
            SS_API_URL, paper_id
        );

        match self.fetch_with_retry::<SsPaper>(&url).await {
            Ok(paper) => Ok(Some(self.ss_to_paper(paper))),
            Err(SourceError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_citations(&self, paper_id: &str, limit: usize) -> Result<Vec<(Paper, Vec<Author>)>, SourceError> {
        let url = format!(
            "{}/paper/{}/citations?fields=paperId,title,authors&limit={}",
            SS_API_URL, paper_id, limit
        );

        let response: SsCitationsResponse = self.fetch_with_retry(&url).await?;

        Ok(response.data.into_iter().filter_map(|c| {
            if let Some(citing) = c.citing_paper {
                let paper = Paper::new(citing.title.unwrap_or_default())
                    .with_semantic_scholar_id(citing.paper_id);
                let authors = citing.authors.into_iter()
                    .map(|a| Author::new(a.name).with_semantic_scholar_id(a.author_id.unwrap_or_default()))
                    .collect();
                Some((paper, authors))
            } else {
                None
            }
        }).collect())
    }

    async fn get_references(&self, paper_id: &str, limit: usize) -> Result<Vec<(Paper, Vec<Author>)>, SourceError> {
        let url = format!(
            "{}/paper/{}/references?fields=paperId,title,authors&limit={}",
            SS_API_URL, paper_id, limit
        );

        let response: SsReferencesResponse = self.fetch_with_retry(&url).await?;

        Ok(response.data.into_iter().filter_map(|r| {
            if let Some(cited) = r.cited_paper {
                let paper = Paper::new(cited.title.unwrap_or_default())
                    .with_semantic_scholar_id(cited.paper_id);
                let authors = cited.authors.into_iter()
                    .map(|a| Author::new(a.name).with_semantic_scholar_id(a.author_id.unwrap_or_default()))
                    .collect();
                Some((paper, authors))
            } else {
                None
            }
        }).collect())
    }

    async fn health_check(&self) -> Result<bool, SourceError> {
        let url = format!("{}/paper/search?query=test&limit=1&fields=paperId", SS_API_URL);
        self.fetch_with_retry::<serde_json::Value>(&url).await?;
        Ok(true)
    }
}

impl SemanticScholarSource {
    async fn get_by_doi(&self, doi: &str) -> Result<Option<Paper>, SourceError> {
        let url = format!(
            "{}/paper/DOI:{}?fields=paperId,title,abstract,year,venue,citationCount,externalIds,authors,openAccessPdf",
            SS_API_URL, doi
        );

        match self.fetch_with_retry::<SsPaper>(&url).await {
            Ok(paper) => Ok(Some(self.ss_to_paper(paper))),
            Err(SourceError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    async fn get_by_arxiv_id(&self, arxiv_id: &str) -> Result<Option<Paper>, SourceError> {
        let url = format!(
            "{}/paper/arXiv:{}?fields=paperId,title,abstract,year,venue,citationCount,externalIds,authors,openAccessPdf",
            SS_API_URL, arxiv_id
        );

        match self.fetch_with_retry::<SsPaper>(&url).await {
            Ok(paper) => Ok(Some(self.ss_to_paper(paper))),
            Err(SourceError::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}