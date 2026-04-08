//! Citation relationship entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "citations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub citing_paper_id: i64,
    #[sea_orm(primary_key)]
    pub cited_paper_id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}