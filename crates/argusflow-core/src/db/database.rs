//! Database operations using SeaORM

use anyhow::Result;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Database as SeaDatabase, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use std::path::Path;

use crate::db::entity::{authors, citations, paper_authors, papers};
use crate::db::migration::Migrator;
use crate::models::{Author, Citation, Paper};
use sea_orm_migration::MigratorTrait;

pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    pub async fn new(path: &Path) -> Result<Self> {
        let db_url = format!("sqlite://{}?mode=rwc", path.display());
        let conn = SeaDatabase::connect(&db_url).await?;

        // Run migrations
        Migrator::up(&conn, None).await?;

        Ok(Self { conn })
    }

    pub async fn new_in_memory() -> Result<Self> {
        let conn = SeaDatabase::connect("sqlite::memory:").await?;
        Migrator::up(&conn, None).await?;
        Ok(Self { conn })
    }

    // Paper CRUD
    pub async fn insert_paper(&self, paper: &Paper) -> Result<i64> {
        let paper_model = papers::ActiveModel {
            title: Set(paper.title.clone()),
            abstract_text: Set(paper.abstract_text.clone()),
            arxiv_id: Set(paper.arxiv_id.clone()),
            semantic_scholar_id: Set(paper.semantic_scholar_id.clone()),
            doi: Set(paper.doi.clone()),
            pdf_url: Set(paper.pdf_url.clone()),
            local_pdf_path: Set(paper.local_pdf_path.clone()),
            publication_date: Set(paper.publication_date.clone()),
            venue: Set(paper.venue.clone()),
            citation_count: Set(paper.citation_count),
            created_at: Set(paper.created_at),
            updated_at: Set(paper.updated_at),
            ..Default::default()
        };

        let result = paper_model.insert(&self.conn).await?;
        let paper_id = result.id;

        // Insert authors and link them
        for (order, author) in paper.authors.iter().enumerate() {
            let author_id = self.insert_author(author).await?;
            self.link_paper_author(paper_id, author_id, order as i32).await?;
        }

        Ok(paper_id)
    }

    pub async fn get_paper_by_id(&self, id: i64) -> Result<Option<Paper>> {
        let paper_model = papers::Entity::find_by_id(id)
            .one(&self.conn)
            .await?;

        match paper_model {
            Some(model) => {
                let authors = self.get_paper_authors(id).await?;
                Ok(Some(model_to_paper(model, authors)))
            }
            None => Ok(None),
        }
    }

    pub async fn get_paper_by_arxiv_id(&self, arxiv_id: &str) -> Result<Option<Paper>> {
        let paper_model = papers::Entity::find()
            .filter(papers::Column::ArxivId.eq(arxiv_id))
            .one(&self.conn)
            .await?;

        match paper_model {
            Some(model) => {
                let authors = self.get_paper_authors(model.id).await?;
                Ok(Some(model_to_paper(model, authors)))
            }
            None => Ok(None),
        }
    }

    pub async fn get_paper_by_semantic_scholar_id(&self, ss_id: &str) -> Result<Option<Paper>> {
        let paper_model = papers::Entity::find()
            .filter(papers::Column::SemanticScholarId.eq(ss_id))
            .one(&self.conn)
            .await?;

        match paper_model {
            Some(model) => {
                let authors = self.get_paper_authors(model.id).await?;
                Ok(Some(model_to_paper(model, authors)))
            }
            None => Ok(None),
        }
    }

    pub async fn update_paper(&self, paper: &Paper) -> Result<()> {
        let id = paper.id.ok_or_else(|| anyhow::anyhow!("Paper id required"))?;

        let paper_model = papers::ActiveModel {
            id: Set(id),
            title: Set(paper.title.clone()),
            abstract_text: Set(paper.abstract_text.clone()),
            arxiv_id: Set(paper.arxiv_id.clone()),
            semantic_scholar_id: Set(paper.semantic_scholar_id.clone()),
            doi: Set(paper.doi.clone()),
            pdf_url: Set(paper.pdf_url.clone()),
            local_pdf_path: Set(paper.local_pdf_path.clone()),
            publication_date: Set(paper.publication_date.clone()),
            venue: Set(paper.venue.clone()),
            citation_count: Set(paper.citation_count),
            updated_at: Set(Utc::now()),
            created_at: Set(paper.created_at),
        };

        paper_model.update(&self.conn).await?;
        Ok(())
    }

    pub async fn list_papers(&self, limit: i64) -> Result<Vec<Paper>> {
        use sea_orm::QuerySelect;

        let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
            .order_by_desc(papers::Column::CreatedAt)
            .limit(limit as u64)
            .find_with_related(authors::Entity)
            .all(&self.conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(paper, author_models)| {
                let authors = author_models
                    .into_iter()
                    .map(|a| Author {
                        id: Some(a.id),
                        name: a.name,
                        semantic_scholar_id: a.semantic_scholar_id,
                        orcid: None,
                    })
                    .collect();

                Paper {
                    id: Some(paper.id),
                    title: paper.title,
                    abstract_text: paper.abstract_text,
                    arxiv_id: paper.arxiv_id,
                    semantic_scholar_id: paper.semantic_scholar_id,
                    doi: paper.doi,
                    pdf_url: paper.pdf_url,
                    local_pdf_path: paper.local_pdf_path,
                    publication_date: paper.publication_date,
                    venue: paper.venue,
                    citation_count: paper.citation_count,
                    authors,
                    created_at: paper.created_at,
                    updated_at: paper.updated_at,
                }
            })
            .collect())
    }

    // Author CRUD
    pub async fn insert_author(&self, author: &Author) -> Result<i64> {
        // Check if author exists by semantic_scholar_id
        if let Some(ss_id) = &author.semantic_scholar_id {
            if !ss_id.is_empty() {
                let existing = authors::Entity::find()
                    .filter(authors::Column::SemanticScholarId.eq(ss_id))
                    .one(&self.conn)
                    .await?;

                if let Some(model) = existing {
                    return Ok(model.id);
                }
            }
        }

        let author_model = authors::ActiveModel {
            name: Set(author.name.clone()),
            semantic_scholar_id: Set(author.semantic_scholar_id.clone()),
            ..Default::default()
        };

        let result = author_model.insert(&self.conn).await?;
        Ok(result.id)
    }

    pub async fn link_paper_author(&self, paper_id: i64, author_id: i64, order: i32) -> Result<()> {
        let link = paper_authors::ActiveModel {
            paper_id: Set(paper_id),
            author_id: Set(author_id),
            author_order: Set(order),
        };

        link.insert(&self.conn).await?;
        Ok(())
    }

    pub async fn get_paper_authors(&self, paper_id: i64) -> Result<Vec<Author>> {
        let links = paper_authors::Entity::find()
            .filter(paper_authors::Column::PaperId.eq(paper_id))
            .order_by_asc(paper_authors::Column::AuthorOrder)
            .all(&self.conn)
            .await?;

        let mut authors = Vec::new();
        for link in links {
            let author_model = authors::Entity::find_by_id(link.author_id)
                .one(&self.conn)
                .await?;

            if let Some(model) = author_model {
                authors.push(Author {
                    id: Some(model.id),
                    name: model.name,
                    semantic_scholar_id: model.semantic_scholar_id,
                    orcid: None,
                });
            }
        }

        Ok(authors)
    }

    // Citation operations
    pub async fn insert_citation(&self, citation: &Citation) -> Result<()> {
        let citation_model = citations::ActiveModel {
            citing_paper_id: Set(citation.citing_paper_id),
            cited_paper_id: Set(citation.cited_paper_id),
        };

        citation_model.insert(&self.conn).await?;
        Ok(())
    }

    pub async fn get_citations(&self, paper_id: i64) -> Result<Vec<i64>> {
        let citations = citations::Entity::find()
            .filter(citations::Column::CitingPaperId.eq(paper_id))
            .all(&self.conn)
            .await?;

        Ok(citations.into_iter().map(|c| c.cited_paper_id).collect())
    }

    pub async fn get_cited_by(&self, paper_id: i64) -> Result<Vec<i64>> {
        let citations = citations::Entity::find()
            .filter(citations::Column::CitedPaperId.eq(paper_id))
            .all(&self.conn)
            .await?;

        Ok(citations.into_iter().map(|c| c.citing_paper_id).collect())
    }

    // Search papers by title
    pub async fn search_papers(&self, query: &str, limit: i64) -> Result<Vec<Paper>> {
        let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
            .filter(papers::Column::Title.contains(query))
            .order_by_desc(papers::Column::CitationCount)
            .limit(limit as u64)
            .find_with_related(authors::Entity)
            .all(&self.conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(paper, author_models)| {
                let authors = author_models
                    .into_iter()
                    .map(|a| Author {
                        id: Some(a.id),
                        name: a.name,
                        semantic_scholar_id: a.semantic_scholar_id,
                        orcid: None,
                    })
                    .collect();

                Paper {
                    id: Some(paper.id),
                    title: paper.title,
                    abstract_text: paper.abstract_text,
                    arxiv_id: paper.arxiv_id,
                    semantic_scholar_id: paper.semantic_scholar_id,
                    doi: paper.doi,
                    pdf_url: paper.pdf_url,
                    local_pdf_path: paper.local_pdf_path,
                    publication_date: paper.publication_date,
                    venue: paper.venue,
                    citation_count: paper.citation_count,
                    authors,
                    created_at: paper.created_at,
                    updated_at: paper.updated_at,
                }
            })
            .collect())
    }

    // === New methods for optimization ===

    /// Delete a paper (cascade deletes paper_authors links)
    pub async fn delete_paper(&self, id: i64) -> Result<bool> {
        // First delete paper_author links
        paper_authors::Entity::delete_many()
            .filter(paper_authors::Column::PaperId.eq(id))
            .exec(&self.conn)
            .await?;

        // Delete citation links
        citations::Entity::delete_many()
            .filter(citations::Column::CitingPaperId.eq(id))
            .exec(&self.conn)
            .await?;
        citations::Entity::delete_many()
            .filter(citations::Column::CitedPaperId.eq(id))
            .exec(&self.conn)
            .await?;

        // Delete the paper
        let result = papers::Entity::delete_by_id(id)
            .exec(&self.conn)
            .await?;

        Ok(result.rows_affected > 0)
    }

    /// Get total paper count
    pub async fn count_papers(&self) -> Result<i64> {
        let count = papers::Entity::find()
            .count(&self.conn)
            .await?;
        Ok(count as i64)
    }

    /// Get top cited papers
    pub async fn top_cited_papers(&self, limit: i64) -> Result<Vec<Paper>> {
        let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
            .order_by_desc(papers::Column::CitationCount)
            .limit(limit as u64)
            .find_with_related(authors::Entity)
            .all(&self.conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(paper, author_models)| {
                let authors = author_models
                    .into_iter()
                    .map(|a| Author {
                        id: Some(a.id),
                        name: a.name,
                        semantic_scholar_id: a.semantic_scholar_id,
                        orcid: None,
                    })
                    .collect();

                Paper {
                    id: Some(paper.id),
                    title: paper.title,
                    abstract_text: paper.abstract_text,
                    arxiv_id: paper.arxiv_id,
                    semantic_scholar_id: paper.semantic_scholar_id,
                    doi: paper.doi,
                    pdf_url: paper.pdf_url,
                    local_pdf_path: paper.local_pdf_path,
                    publication_date: paper.publication_date,
                    venue: paper.venue,
                    citation_count: paper.citation_count,
                    authors,
                    created_at: paper.created_at,
                    updated_at: paper.updated_at,
                }
            })
            .collect())
    }

    /// Get papers added in the last N days
    pub async fn recent_papers(&self, days: i64) -> Result<Vec<Paper>> {
        let cutoff = Utc::now() - chrono::Duration::days(days);
        let cutoff_naive = cutoff.naive_utc();

        let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
            .filter(papers::Column::CreatedAt.gte(cutoff_naive.and_utc()))
            .order_by_desc(papers::Column::CreatedAt)
            .find_with_related(authors::Entity)
            .all(&self.conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(paper, author_models)| {
                let authors = author_models
                    .into_iter()
                    .map(|a| Author {
                        id: Some(a.id),
                        name: a.name,
                        semantic_scholar_id: a.semantic_scholar_id,
                        orcid: None,
                    })
                    .collect();

                Paper {
                    id: Some(paper.id),
                    title: paper.title,
                    abstract_text: paper.abstract_text,
                    arxiv_id: paper.arxiv_id,
                    semantic_scholar_id: paper.semantic_scholar_id,
                    doi: paper.doi,
                    pdf_url: paper.pdf_url,
                    local_pdf_path: paper.local_pdf_path,
                    publication_date: paper.publication_date,
                    venue: paper.venue,
                    citation_count: paper.citation_count,
                    authors,
                    created_at: paper.created_at,
                    updated_at: paper.updated_at,
                }
            })
            .collect())
    }

    /// Search papers by author name
    pub async fn search_by_author(&self, name: &str, limit: i64) -> Result<Vec<Paper>> {
        // Find authors matching the name
        let matching_authors = authors::Entity::find()
            .filter(authors::Column::Name.contains(name))
            .all(&self.conn)
            .await?;

        let author_ids: Vec<i64> = matching_authors.iter().map(|a| a.id).collect();

        if author_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Find papers by these authors
        let links = paper_authors::Entity::find()
            .filter(paper_authors::Column::AuthorId.is_in(author_ids))
            .all(&self.conn)
            .await?;

        let paper_ids: Vec<i64> = links.iter().map(|l| l.paper_id).collect();

        if paper_ids.is_empty() {
            return Ok(Vec::new());
        }

        let results: Vec<(papers::Model, Vec<authors::Model>)> = papers::Entity::find()
            .filter(papers::Column::Id.is_in(paper_ids))
            .order_by_desc(papers::Column::CitationCount)
            .limit(limit as u64)
            .find_with_related(authors::Entity)
            .all(&self.conn)
            .await?;

        Ok(results
            .into_iter()
            .map(|(paper, author_models)| {
                let authors = author_models
                    .into_iter()
                    .map(|a| Author {
                        id: Some(a.id),
                        name: a.name,
                        semantic_scholar_id: a.semantic_scholar_id,
                        orcid: None,
                    })
                    .collect();

                Paper {
                    id: Some(paper.id),
                    title: paper.title,
                    abstract_text: paper.abstract_text,
                    arxiv_id: paper.arxiv_id,
                    semantic_scholar_id: paper.semantic_scholar_id,
                    doi: paper.doi,
                    pdf_url: paper.pdf_url,
                    local_pdf_path: paper.local_pdf_path,
                    publication_date: paper.publication_date,
                    venue: paper.venue,
                    citation_count: paper.citation_count,
                    authors,
                    created_at: paper.created_at,
                    updated_at: paper.updated_at,
                }
            })
            .collect())
    }
}

fn model_to_paper(model: papers::Model, authors: Vec<Author>) -> Paper {
    Paper {
        id: Some(model.id),
        title: model.title,
        abstract_text: model.abstract_text,
        arxiv_id: model.arxiv_id,
        semantic_scholar_id: model.semantic_scholar_id,
        doi: model.doi,
        pdf_url: model.pdf_url,
        local_pdf_path: model.local_pdf_path,
        publication_date: model.publication_date,
        venue: model.venue,
        citation_count: model.citation_count,
        authors,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}