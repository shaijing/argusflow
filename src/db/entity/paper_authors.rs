//! Paper-Author junction table entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "paper_authors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub paper_id: i64,
    #[sea_orm(primary_key)]
    pub author_id: i64,
    pub author_order: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::papers::Entity",
        from = "Column::PaperId",
        to = "super::papers::Column::Id"
    )]
    Papers,
    #[sea_orm(
        belongs_to = "super::authors::Entity",
        from = "Column::AuthorId",
        to = "super::authors::Column::Id"
    )]
    Authors,
}

impl Related<super::papers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Papers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}