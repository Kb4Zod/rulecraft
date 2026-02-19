# Rulecraft

D&D 2024 rules lookup and AI-powered scenario ruling assistant.

## Features

- **Rules Lookup** - Search and browse D&D 2024 rules with full-text and semantic search
- **AI Rulings** - Ask scenario questions and get AI-powered rulings with rule citations
- **Bookmarks** - Save frequently referenced rules to browser local storage
- **Offline-First** - SQLite database for fast local access

## Tech Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust + Axum |
| Frontend | HTMX + Askama templates |
| Database | SQLite (FTS5 full-text search) |
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
cargo run

# Visit http://localhost:3000
```

### Docker Deployment

```bash
# Build and run with Docker Compose
cd docker
docker-compose up --build

# With vector search (Qdrant)
docker-compose --profile vector-search up --build
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

## Usage

### Rules Search

Navigate to `/search` and enter keywords. The search uses SQLite FTS5 for fast full-text matching.

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

Rules can be added directly to SQLite or via the migration files in `migrations/`.

### Code Style

- Rust: Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Templates: Use consistent indentation in Askama templates
- CSS: BEM-style naming where appropriate

## License

MIT
