# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ArgusFlow is a CLI tool for literature/paper search and organization. It integrates with multiple academic paper sources (arXiv, Semantic Scholar) to search, fetch, and manage research papers locally.

## Build Commands

```bash
cargo build                  # Build the project
cargo test                   # Run all unit tests (tests are inline in source files)
cargo test --lib             # Run library tests only
cargo run -- <command>       # Run CLI with a command
cargo clippy                 # Run linter
```

## Running the CLI

```bash
# Search papers
cargo run -- arxiv-search -q "machine learning" -l 10
cargo run -- ss-search -q "attention mechanism" -l 10
cargo run -- search -q "transformers" -l 10  # smart search across all sources

# Get paper by identifier (auto-detects source)
cargo run -- get -i "2301.00001"        # arXiv ID
cargo run -- get -i "doi:10.1234/test" # DOI
cargo run -- get -i "ss:abc123"        # Semantic Scholar ID

# Citation graph
cargo run -- citations -i "paper-id" -l 50
cargo run -- references -i "paper-id" -l 50

# Local database operations
cargo run -- list -l 20
cargo run -- local-search -q "keyword" -l 20

# Download PDFs
cargo run -- download -i "2301.00001"
cargo run -- download -i "https://arxiv.org/pdf/2301.00001"

# Configuration (optional proxy for network requests)
cargo run -- --proxy "http://127.0.0.1:7890" search -q "test"
cargo run -- --ss-api-key "YOUR_KEY" ss-search -q "test"
```

## Architecture

### Source Abstraction (`src/source/mod.rs`)

The `PaperSource` trait defines the interface for all paper sources. Each source implements:
- `search()` - Search papers by query
- `get_by_id()` / `get_by_identifier()` - Fetch by specific ID
- `get_citations()` / `get_references()` - Citation graph traversal
- `capabilities()` - Returns `SourceCapabilities` struct indicating supported features

Key types:
- `SourceKind` enum: Arxiv, SemanticScholar, Crossref, OpenAlex, Pubmed, etc.
- `Identifier` enum: Parses identifiers like `arxiv:2301.00001`, `doi:10.xxxx`, `ss:abc123`
- `SourceManager`: Registry for sources with smart routing
- `SourceBuilder`: Builder pattern for constructing sources with config (proxy, API key, timeout)

### Data Models (`src/models/`)

- `Paper`: Core paper entity with title, abstract, IDs (arXiv, SS, DOI), PDF URL, authors, citation count
- `Author`: Author with optional Semantic Scholar ID
- `PaperAuthor`: Paper-Author relationship with ordering
- `Citation`: Paper-to-paper citation relationship

### Database Layer (`src/db/`)

Uses SeaORM for async SQLite operations:

- `src/db/entity/`: SeaORM entity definitions (papers, authors, paper_authors, citations)
- `src/db/migration/`: Database migrations using sea-orm-migration
- `src/db/database.rs`: `Database` struct with async CRUD operations

Tables: `papers`, `authors`, `paper_authors` (junction), `citations`. Authors are stored separately and linked via `paper_authors` with ordering. All database operations are async.

### CLI (`src/command/`)

- `mod.rs`: Command definitions via clap's `Subcommand`
- `handlers.rs`: Command execution logic
- `context.rs`: `CommandContext` holds config, db, and source manager

### Key Patterns

1. **Async trait**: `PaperSource` uses `async_trait` for async methods
2. **Builder pattern**: `SourceBuilder` and `Paper` use builder methods (`.with_arxiv_id()`, etc.)
3. **Smart routing**: `SourceManager.smart_search()` queries all registered sources; `fetch_by_identifier()` routes by identifier type
4. **Capability-based dispatch**: Check `SourceCapabilities` before calling citation/reference methods
5. **Inline tests**: Unit tests are in `#[cfg(test)]` modules within source files

## Adding a New Paper Source

1. Create a new file in `src/source/` (e.g., `crossref.rs`)
2. Implement `PaperSource` trait with all required methods
3. Add module in `src/source/mod.rs` and re-export
4. Register in `build_manager()` in `src/command/context.rs`

## Configuration

Config stored as TOML, with defaults in `dirs::data_dir()/argusflow/`. Key fields:
- `pdf_storage_path`: Where PDFs are downloaded
- `db_path`: SQLite database location
- `semantic_scholar_api_key`: Optional API key for higher rate limits
- `proxy`: HTTP proxy for network requests