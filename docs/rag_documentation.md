# Rulecraft Oracle: Retrieval-Augmented Generation Architecture

## Overview

The Oracle feature is a Retrieval-Augmented Generation (RAG) flow for D&D 2024 scenario rulings. It retrieves local rule context first, then asks Claude to answer the user's question using that context.

The active retrieval stack is now hybrid:

1. SQLite FTS5 keyword retrieval remains the reliable backbone.
2. Optional Qdrant vector retrieval adds semantic recall when `VECTOR_SEARCH_ENABLED=true`.
3. Claude receives the merged rule context and produces the final ruling.

## Components

### User Interface

- Files: `templates/scenario/ask.html`, `templates/scenario/response.html`
- Role: accepts the user's scenario question and renders the Markdown answer plus referenced rules.

### Keyword Retrieval

- Files: `src/search/fulltext.rs`, `src/db/sqlite.rs`
- Role: sanitizes user text for SQLite FTS5, searches `rules_fts`, and returns ranked `Rule` rows from SQLite.
- Behavior: terms are joined with `OR`, which favors broad recall.

### Vector Retrieval

- Files: `src/search/hybrid.rs`, `src/search/openai_embeddings.rs`, `src/search/qdrant.rs`, `src/search/vector.rs`
- Role: embeds the user question with OpenAI `text-embedding-3-small`, searches Qdrant, filters by score threshold, and hydrates matching rule IDs from SQLite.
- Default: disabled unless `VECTOR_SEARCH_ENABLED=true`.
- Safety behavior: if OpenAI or Qdrant fails, the Oracle logs a warning and continues with FTS5-only context.

### Vector Indexing

- File: `src/bin/index_vectors.rs`
- Command: `cargo run --bin index_vectors`
- Role: reads all SQLite rules, builds one embedding per rule, and upserts those vectors into Qdrant.
- Reindexing is idempotent. If the Qdrant collection already exists, setup continues.

### Context Aggregation

- File: `src/search/hybrid.rs`
- Behavior:
  - FTS5 results are kept first.
  - Vector results are appended only when their score is at least `VECTOR_SCORE_THRESHOLD`.
  - Duplicate rule IDs are removed.
  - The final context is capped by `ORACLE_MAX_CONTEXT_RULES`.

### AI Generation

- File: `src/ai/claude.rs`
- Role: formats retrieved rules with title, content, source, and page number, injects them into Claude's system prompt, and sends the user's question.

## Operational Flow

1. The user submits a scenario to `POST /scenario/ask`.
2. The server validates input length and applies the AI rate limit.
3. FTS5 retrieves keyword matches from SQLite.
4. If vector search is enabled, OpenAI embeds the query and Qdrant returns semantic matches.
5. The hybrid layer merges FTS and vector results, filters low-confidence vector hits, dedupes IDs, and caps context size.
6. Claude receives the question plus retrieved rules and generates the ruling.
7. The Markdown answer is rendered to HTML, along with referenced rule links.

## Local Vector Setup

```bash
# Start Qdrant
cd docker
docker compose --profile vector-search up -d qdrant

# Add OPENAI_API_KEY and VECTOR_SEARCH_ENABLED=true to .env
cd ..
cargo run --bin index_vectors -- --fail-fast
cargo run --bin index_vectors

# Restart the app so .env is reloaded
cargo run --bin rulecraft
```

## Design Imperative

The goal is grounded accuracy over unconstrained generation.

- SQLite remains the canonical rule database.
- Qdrant stores vectors and lightweight payloads only.
- Exact and keyword matches stay ahead of semantic matches.
- Vector retrieval is optional and failure-tolerant.
- Source/page metadata comes from SQLite so citations remain verifiable.
