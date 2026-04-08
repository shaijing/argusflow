# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

ArgusFlow is a CLI tool for literature/paper search and organization. It integrates with multiple academic paper sources (arXiv, Semantic Scholar, OpenAlex) to search, fetch, and manage research papers locally.

## Workspace Structure

```
argusflow/
├── Cargo.toml              # Workspace definition
├── crates/
│   ├── argusflow-core/     # Core library (can be published to crates.io)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs
│   │       ├── core/       # ArgusFlow main entry point
│   │       ├── models/     # Paper, Author, Citation
│   │       ├── source/     # PaperSource trait, implementations
│   │       ├── db/         # Database layer (SeaORM)
│   │       ├── pdf/        # PDF downloader
│   │       ├── citation/   # Citation graph utilities
│   │       └── output/     # Output formatters
│   └── argusflow-cli/      # CLI application
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── command/    # CLI commands and handlers
```

## Build Commands

```bash
cargo build                  # Build the entire workspace
cargo test                   # Run all unit tests
cargo test -p argusflow-core # Run only core library tests
cargo run -- <command>       # Run CLI with a command
cargo clippy                 # Run linter
```

## Running the CLI

```bash
# Search papers
cargo run -- arxiv-search -q "machine learning" -l 10
cargo run -- ss-search -q "attention mechanism" -l 10
cargo run -- oa-search -q "deep learning" -l 10
cargo run -- search -q "transformers" -l 10  # smart search across all sources

# Get paper by identifier (auto-detects source)
cargo run -- get -i "2301.00001"        # arXiv ID
cargo run -- get -i "doi:10.1234/test" # DOI
cargo run -- get -i "ss:abc123"         # Semantic Scholar ID

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

### Core Library (`argusflow-core`)

The core library provides all paper management functionality without CLI dependencies.

**Entry Point**: `ArgusFlow` struct with `ArgusFlowBuilder` for configuration.

**Source Abstraction** (`source/mod.rs`):
- `PaperSource` trait defines the interface for all paper sources
- `SourceManager` handles multi-source coordination with smart routing
- `SourceBuilder` for constructing sources with config (proxy, API key, timeout)

**Data Models** (`models/`):
- `Paper`: Core entity with title, abstract, IDs, PDF URL, authors, citation count
- `Author`: Author with optional Semantic Scholar ID
- `Citation`: Paper-to-paper citation relationship

**Database Layer** (`db/`):
- SeaORM for async SQLite operations
- Entity definitions in `db/entity/`
- Migrations in `db/migration/`

### CLI Application (`argusflow-cli`)

Thin wrapper around `argusflow-core`:
- `command/mod.rs`: Command definitions via clap
- `command/handlers.rs`: Command execution logic
- `command/context.rs`: `CommandContext` wraps `ArgusFlow` core

### Key Patterns

1. **Async trait**: `PaperSource` uses `async_trait` for async methods
2. **Builder pattern**: `ArgusFlowBuilder`, `SourceBuilder`, `Paper` use builder methods
3. **Smart routing**: `SourceManager.smart_search()` queries all registered sources
4. **Capability-based dispatch**: Check `SourceCapabilities` before calling methods
5. **Inline tests**: Unit tests are in `#[cfg(test)]` modules within source files

## Adding a New Paper Source

1. Create a new file in `crates/argusflow-core/src/source/` (e.g., `crossref.rs`)
2. Implement `PaperSource` trait with all required methods
3. Add module in `crates/argusflow-core/src/source/mod.rs` and re-export
4. Register in `ArgusFlowBuilder::build()` in `core/argusflow.rs`

## Configuration

Config stored as TOML, with defaults in `dirs::data_dir()/argusflow/`. Key fields:
- `pdf_storage_path`: Where PDFs are downloaded
- `db_path`: SQLite database location
- `semantic_scholar_api_key`: Optional API key for higher rate limits
- `proxy`: HTTP proxy for network requests