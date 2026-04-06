//! Paper entity definition

use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "papers")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i64,
    pub title: String,
    pub abstract_text: Option<String>,
    pub arxiv_id: Option<String>,
    pub semantic_scholar_id: Option<String>,
    pub doi: Option<String>,
    pub pdf_url: Option<String>,
    pub local_pdf_path: Option<String>,
    pub publication_date: Option<String>,
    pub venue: Option<String>,
    pub citation_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}