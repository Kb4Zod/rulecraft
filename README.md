# Rulecraft

D&D 2024 rules lookup and AI-powered scenario ruling assistant.

## Features

- **Rules Lookup** - Search and browse D&D 2024 rules with SQLite FTS5 and fuzzy fallback
- **AI Rulings** - Ask scenario questions and get Claude-powered rulings with rule citations
- **Oracle Vector Retrieval** - Optional Qdrant + OpenAI embeddings add semantic recall to scenario context
- **Bookmarks** - Save frequently referenced rules to browser local storage
- **Offline-First** - SQLite database for fast local access

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust + Axum |
| Frontend | HTMX + Askama templates |
| Database | SQLite (FTS5 full-text search) |
| Vector Search | Optional Qdrant with OpenAI `text-embedding-3-small` embeddings |
| AI | Claude API for scenario rulings |
| Deployment | Docker |

## Quick Start

### Prerequisites

- Rust 1.75+ (`rustup update`)
- SQLite 3.x
- Docker (optional, for containerized deployment)

### Development

```bash
# Clone and enter project
cd rulecraft

# Copy environment file
cp .env.example .env

# Edit .env and add your Claude API key
# CLAUDE_API_KEY=your-key-here

# Build and run
cargo run --bin rulecraft

# Visit http://localhost:3000
```

### Docker Deployment

```bash
# Build and run with Docker Compose
cd docker
docker-compose up --build -d

# Import YAML rules into the container's database
docker exec rulecraft ./import_rules --rules-dir /app/rules

# With vector search (Qdrant)
docker compose --profile vector-search up --build -d
docker compose exec rulecraft ./index_vectors
```

## Project Structure

```
rulecraft/
├── src/
│   ├── main.rs           # Entry point
│   ├── config.rs         # Configuration
│   ├── routes/           # HTTP route handlers
│   ├── models/           # Data models
│   ├── db/               # Database operations
│   ├── search/           # Search (FTS + vector)
│   └── ai/               # Claude API integration
├── templates/            # Askama HTML templates
├── static/               # CSS, JS, images
├── migrations/           # SQL migrations
├── docker/               # Docker configuration
├── agent/                # AI agent instructions
└── docs/                 # Documentation
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite connection string | `sqlite:./rulecraft.db` |
| `CLAUDE_API_KEY` | Anthropic API key | (required for AI rulings) |
| `CLAUDE_MODEL` | Claude model to use | `claude-sonnet-4-20250514` |
| `PORT` | Server port | `3000` |
| `ADMIN_API_KEY` | Protects admin write endpoints | (required for admin writes) |
| `AI_RATE_LIMIT_PER_HOUR` | AI requests per IP per hour | `5` |
| `SEARCH_RATE_LIMIT_PER_MINUTE` | Search requests per IP per minute | `30` |
| `VECTOR_SEARCH_ENABLED` | Enables Oracle vector retrieval | `false` |
| `OPENAI_API_KEY` | OpenAI key for embeddings | (required for vector search) |
| `OPENAI_EMBEDDING_MODEL` | Embedding model | `text-embedding-3-small` |
| `OPENAI_EMBEDDING_DIMENSION` | Embedding vector size | `1536` |
| `QDRANT_URL` | Qdrant endpoint | `http://localhost:6333` |
| `QDRANT_COLLECTION` | Qdrant collection name | `rulecraft_rules_openai_small_v1` |
| `VECTOR_TOP_K` | Vector hits requested per Oracle query | `10` |
| `VECTOR_SCORE_THRESHOLD` | Minimum vector score used in Oracle context | `0.35` |
| `ORACLE_MAX_CONTEXT_RULES` | Max rules injected into Oracle prompt | `10` |

## Usage

### Rules Search

Navigate to `/search` and enter keywords. The search uses SQLite FTS5 for fast full-text matching, then falls back to fuzzy SQL matching if FTS finds nothing.

### Oracle Vector Search

Vector retrieval is optional and currently used by the Oracle scenario flow. The Oracle always keeps FTS5 results first, then adds Qdrant semantic matches that clear the configured score threshold.

Local setup:

```bash
# Start Qdrant
cd docker
docker compose --profile vector-search up -d qdrant

# Add OPENAI_API_KEY and VECTOR_SEARCH_ENABLED=true to .env, then index
cd ..
cargo run --bin index_vectors -- --fail-fast
cargo run --bin index_vectors

# Restart the app after changing .env
cargo run --bin rulecraft
```

### Scenario Questions

1. Go to `/scenario`
2. Enter your D&D rules question
3. Get an AI-powered ruling with relevant rule citations

Example questions:
- "Can a rogue use Sneak Attack with advantage but no allies nearby?"
- "Does Counterspell work on legendary actions?"
- "How does concentration work when taking damage?"

### Bookmarks

Click the bookmark button on any rule to save it to your browser's local storage. Access bookmarks from the navigation bar.

## Development

### Running Tests

```bash
cargo test
```

### Adding Rules

Rules can be added via the admin routes or by editing YAML files in `data/rules/` and running:

```bash
cargo run --bin import_rules
```

If vector search is enabled, rerun:

```bash
cargo run --bin index_vectors
```

### Code Style

- Rust: Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Templates: Use consistent indentation in Askama templates
- CSS: BEM-style naming where appropriate

## License

MIT
