//! Create paper_authors junction table migration

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PaperAuthors::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PaperAuthors::PaperId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PaperAuthors::AuthorId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PaperAuthors::AuthorOrder).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(PaperAuthors::PaperId)
                            .col(PaperAuthors::AuthorId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_authors_paper")
                            .from(PaperAuthors::Table, PaperAuthors::PaperId)
                            .to(Papers::Table, Papers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_paper_authors_author")
                            .from(PaperAuthors::Table, PaperAuthors::AuthorId)
                            .to(Authors::Table, Authors::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(PaperAuthors::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum PaperAuthors {
    Table,
    PaperId,
    AuthorId,
    AuthorOrder,
}

#[derive(DeriveIden)]
enum Papers {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Authors {
    Table,
    Id,
}