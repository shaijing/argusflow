//! Database entity definitions for SeaORM

pub mod papers;
pub mod authors;
pub mod paper_authors;
pub mod citations;

pub use papers::Entity as Papers;
pub use authors::Entity as Authors;
pub use paper_authors::Entity as PaperAuthors;
pub use citations::Entity as Citations;