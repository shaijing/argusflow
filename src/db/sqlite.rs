use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use std::path::Path;

use crate::models::{Author, Citation, Paper};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS papers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    abstract_text TEXT,
    arxiv_id TEXT UNIQUE,
    semantic_scholar_id TEXT UNIQUE,
    doi TEXT,
    pdf_url TEXT,
    local_pdf_path TEXT,
    publication_date TEXT,
    venue TEXT,
    citation_count INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS authors (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    semantic_scholar_id TEXT UNIQUE
);

CREATE TABLE IF NOT EXISTS paper_authors (
    paper_id INTEGER NOT NULL,
    author_id INTEGER NOT NULL,
    author_order INTEGER NOT NULL,
    PRIMARY KEY (paper_id, author_id),
    FOREIGN KEY (paper_id) REFERENCES papers(id),
    FOREIGN KEY (author_id) REFERENCES authors(id)
);

CREATE TABLE IF NOT EXISTS citations (
    citing_paper_id INTEGER NOT NULL,
    cited_paper_id INTEGER NOT NULL,
    PRIMARY KEY (citing_paper_id, cited_paper_id),
    FOREIGN KEY (citing_paper_id) REFERENCES papers(id),
    FOREIGN KEY (cited_paper_id) REFERENCES papers(id)
);

CREATE INDEX IF NOT EXISTS idx_papers_arxiv ON papers(arxiv_id);
CREATE INDEX IF NOT EXISTS idx_papers_ss ON papers(semantic_scholar_id);
CREATE INDEX IF NOT EXISTS idx_papers_doi ON papers(doi);
";

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    // Paper CRUD
    pub fn insert_paper(&self, paper: &Paper) -> Result<i64> {
        let created_at = paper.created_at.to_rfc3339();
        let updated_at = paper.updated_at.to_rfc3339();

        let id = self.conn.execute(
            "INSERT INTO papers (title, abstract_text, arxiv_id, semantic_scholar_id, doi, pdf_url, local_pdf_path, publication_date, venue, citation_count, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                paper.title,
                paper.abstract_text,
                paper.arxiv_id,
                paper.semantic_scholar_id,
                paper.doi,
                paper.pdf_url,
                paper.local_pdf_path,
                paper.publication_date,
                paper.venue,
                paper.citation_count,
                created_at,
                updated_at,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_paper_by_id(&self, id: i64) -> Result<Option<Paper>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, abstract_text, arxiv_id, semantic_scholar_id, doi, pdf_url, local_pdf_path, publication_date, venue, citation_count, created_at, updated_at
             FROM papers WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(Paper {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                abstract_text: row.get(2)?,
                arxiv_id: row.get(3)?,
                semantic_scholar_id: row.get(4)?,
                doi: row.get(5)?,
                pdf_url: row.get(6)?,
                local_pdf_path: row.get(7)?,
                publication_date: row.get(8)?,
                venue: row.get(9)?,
                citation_count: row.get(10)?,
                created_at: row.get::<_, String>(11)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(12)?.parse().unwrap_or(Utc::now()),
            })
        });

        match result {
            Ok(paper) => Ok(Some(paper)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_paper_by_arxiv_id(&self, arxiv_id: &str) -> Result<Option<Paper>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, abstract_text, arxiv_id, semantic_scholar_id, doi, pdf_url, local_pdf_path, publication_date, venue, citation_count, created_at, updated_at
             FROM papers WHERE arxiv_id = ?1"
        )?;

        let result = stmt.query_row(params![arxiv_id], |row| {
            Ok(Paper {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                abstract_text: row.get(2)?,
                arxiv_id: row.get(3)?,
                semantic_scholar_id: row.get(4)?,
                doi: row.get(5)?,
                pdf_url: row.get(6)?,
                local_pdf_path: row.get(7)?,
                publication_date: row.get(8)?,
                venue: row.get(9)?,
                citation_count: row.get(10)?,
                created_at: row.get::<_, String>(11)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(12)?.parse().unwrap_or(Utc::now()),
            })
        });

        match result {
            Ok(paper) => Ok(Some(paper)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_paper_by_semantic_scholar_id(&self, ss_id: &str) -> Result<Option<Paper>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, abstract_text, arxiv_id, semantic_scholar_id, doi, pdf_url, local_pdf_path, publication_date, venue, citation_count, created_at, updated_at
             FROM papers WHERE semantic_scholar_id = ?1"
        )?;

        let result = stmt.query_row(params![ss_id], |row| {
            Ok(Paper {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                abstract_text: row.get(2)?,
                arxiv_id: row.get(3)?,
                semantic_scholar_id: row.get(4)?,
                doi: row.get(5)?,
                pdf_url: row.get(6)?,
                local_pdf_path: row.get(7)?,
                publication_date: row.get(8)?,
                venue: row.get(9)?,
                citation_count: row.get(10)?,
                created_at: row.get::<_, String>(11)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(12)?.parse().unwrap_or(Utc::now()),
            })
        });

        match result {
            Ok(paper) => Ok(Some(paper)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_paper(&self, paper: &Paper) -> Result<()> {
        let id = paper.id.ok_or_else(|| anyhow::anyhow!("Paper id required"))?;
        let updated_at = paper.updated_at.to_rfc3339();

        self.conn.execute(
            "UPDATE papers SET title=?1, abstract_text=?2, arxiv_id=?3, semantic_scholar_id=?4, doi=?5, pdf_url=?6, local_pdf_path=?7, publication_date=?8, venue=?9, citation_count=?10, updated_at=?11
             WHERE id=?12",
            params![
                paper.title,
                paper.abstract_text,
                paper.arxiv_id,
                paper.semantic_scholar_id,
                paper.doi,
                paper.pdf_url,
                paper.local_pdf_path,
                paper.publication_date,
                paper.venue,
                paper.citation_count,
                updated_at,
                id,
            ],
        )?;

        Ok(())
    }

    pub fn list_papers(&self, limit: i64) -> Result<Vec<Paper>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, abstract_text, arxiv_id, semantic_scholar_id, doi, pdf_url, local_pdf_path, publication_date, venue, citation_count, created_at, updated_at
             FROM papers ORDER BY created_at DESC LIMIT ?1"
        )?;

        let papers = stmt.query_map(params![limit], |row| {
            Ok(Paper {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                abstract_text: row.get(2)?,
                arxiv_id: row.get(3)?,
                semantic_scholar_id: row.get(4)?,
                doi: row.get(5)?,
                pdf_url: row.get(6)?,
                local_pdf_path: row.get(7)?,
                publication_date: row.get(8)?,
                venue: row.get(9)?,
                citation_count: row.get(10)?,
                created_at: row.get::<_, String>(11)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(12)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        papers.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    // Author CRUD
    pub fn insert_author(&self, author: &Author) -> Result<i64> {
        let id = if let Some(ss_id) = &author.semantic_scholar_id {
            self.conn.execute(
                "INSERT OR IGNORE INTO authors (name, semantic_scholar_id) VALUES (?1, ?2)",
                params![author.name, ss_id],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO authors (name, semantic_scholar_id) VALUES (?1, NULL)",
                params![author.name],
            )?;
        };

        // 如果有 semantic_scholar_id，通过它获取 ID
        if let Some(ss_id) = &author.semantic_scholar_id {
            let id: i64 = self.conn.query_row(
                "SELECT id FROM authors WHERE semantic_scholar_id = ?1",
                params![ss_id],
                |row| row.get(0),
            )?;
            Ok(id)
        } else {
            Ok(self.conn.last_insert_rowid())
        }
    }

    pub fn link_paper_author(&self, paper_id: i64, author_id: i64, order: i32) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO paper_authors (paper_id, author_id, author_order) VALUES (?1, ?2, ?3)",
            params![paper_id, author_id, order],
        )?;
        Ok(())
    }

    pub fn get_paper_authors(&self, paper_id: i64) -> Result<Vec<Author>> {
        let mut stmt = self.conn.prepare(
            "SELECT a.id, a.name, a.semantic_scholar_id
             FROM authors a
             JOIN paper_authors pa ON a.id = pa.author_id
             WHERE pa.paper_id = ?1
             ORDER BY pa.author_order"
        )?;

        let authors = stmt.query_map(params![paper_id], |row| {
            Ok(Author {
                id: Some(row.get(0)?),
                name: row.get(1)?,
                semantic_scholar_id: row.get(2)?,
            })
        })?;

        authors.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    // Citation operations
    pub fn insert_citation(&self, citation: &Citation) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO citations (citing_paper_id, cited_paper_id) VALUES (?1, ?2)",
            params![citation.citing_paper_id, citation.cited_paper_id],
        )?;
        Ok(())
    }

    pub fn get_citations(&self, paper_id: i64) -> Result<Vec<i64>> {
        // 获取该论文引用的其他论文
        let mut stmt = self.conn.prepare(
            "SELECT cited_paper_id FROM citations WHERE citing_paper_id = ?1"
        )?;

        let ids = stmt.query_map(params![paper_id], |row| row.get(0))?;
        ids.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    pub fn get_cited_by(&self, paper_id: i64) -> Result<Vec<i64>> {
        // 获取引用该论文的其他论文
        let mut stmt = self.conn.prepare(
            "SELECT citing_paper_id FROM citations WHERE cited_paper_id = ?1"
        )?;

        let ids = stmt.query_map(params![paper_id], |row| row.get(0))?;
        ids.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }

    // Search papers by title
    pub fn search_papers(&self, query: &str, limit: i64) -> Result<Vec<Paper>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, abstract_text, arxiv_id, semantic_scholar_id, doi, pdf_url, local_pdf_path, publication_date, venue, citation_count, created_at, updated_at
             FROM papers WHERE title LIKE ?1 ORDER BY citation_count DESC LIMIT ?2"
        )?;

        let papers = stmt.query_map(params![format!("%{}%", query), limit], |row| {
            Ok(Paper {
                id: Some(row.get(0)?),
                title: row.get(1)?,
                abstract_text: row.get(2)?,
                arxiv_id: row.get(3)?,
                semantic_scholar_id: row.get(4)?,
                doi: row.get(5)?,
                pdf_url: row.get(6)?,
                local_pdf_path: row.get(7)?,
                publication_date: row.get(8)?,
                venue: row.get(9)?,
                citation_count: row.get(10)?,
                created_at: row.get::<_, String>(11)?.parse().unwrap_or(Utc::now()),
                updated_at: row.get::<_, String>(12)?.parse().unwrap_or(Utc::now()),
            })
        })?;

        papers.collect::<Result<Vec<_>, _>>().map_err(|e| e.into())
    }
}