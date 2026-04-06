//! Create citations table migration

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Citations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Citations::CitingPaperId)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Citations::CitedPaperId)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(Citations::CitingPaperId)
                            .col(Citations::CitedPaperId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_citations_citing")
                            .from(Citations::Table, Citations::CitingPaperId)
                            .to(Papers::Table, Papers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_citations_cited")
                            .from(Citations::Table, Citations::CitedPaperId)
                            .to(Papers::Table, Papers::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Citations::Table).to_owned()).await
    }
}

#[derive(DeriveIden)]
enum Citations {
    Table,
    CitingPaperId,
    CitedPaperId,
}

#[derive(DeriveIden)]
enum Papers {
    Table,
    Id,
}