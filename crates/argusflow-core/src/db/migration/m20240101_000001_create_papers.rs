//! Create papers table migration

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Papers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Papers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Papers::Title).text().not_null())
                    .col(ColumnDef::new(Papers::AbstractText).text())
                    .col(ColumnDef::new(Papers::ArxivId).text().unique_key())
                    .col(ColumnDef::new(Papers::SemanticScholarId).text().unique_key())
                    .col(ColumnDef::new(Papers::Doi).text())
                    .col(ColumnDef::new(Papers::PdfUrl).text())
                    .col(ColumnDef::new(Papers::LocalPdfPath).text())
                    .col(ColumnDef::new(Papers::PublicationDate).text())
                    .col(ColumnDef::new(Papers::Venue).text())
                    .col(ColumnDef::new(Papers::CitationCount).integer().default(0))
                    .col(ColumnDef::new(Papers::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Papers::UpdatedAt).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_papers_arxiv")
                    .table(Papers::Table)
                    .col(Papers::ArxivId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_papers_ss")
                    .table(Papers::Table)
                    .col(Papers::SemanticScholarId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_papers_doi")
                    .table(Papers::Table)
                    .col(Papers::Doi)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Papers::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Papers {
    Table,
    Id,
    Title,
    AbstractText,
    ArxivId,
    SemanticScholarId,
    Doi,
    PdfUrl,
    LocalPdfPath,
    PublicationDate,
    Venue,
    CitationCount,
    CreatedAt,
    UpdatedAt,
}