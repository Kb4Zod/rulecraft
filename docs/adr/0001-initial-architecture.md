# ADR 0001: Initial Architecture

## Status

Accepted

## Date

2024-02-19

## Context

We need to build a D&D 2024 rules lookup application with the following requirements:
- Fast rule searches
- AI-powered scenario rulings
- Browser-based bookmarks
- Docker deployment
- No user authentication initially

## Decision

### Backend: Rust + Axum

**Chosen**: Rust with Axum web framework

**Alternatives Considered**:
- Python + FastAPI: Easier development, slower runtime
- Go + Gin: Good performance, less expressive types
- Node.js + Express: Large ecosystem, type safety concerns

**Rationale**:
- Excellent performance for search operations
- Type safety reduces runtime errors
- Axum is modern, ergonomic, and well-maintained
- Single binary deployment
- Memory safety without GC pauses

### Frontend: HTMX + Askama

**Chosen**: Server-rendered HTML with HTMX for interactivity

**Alternatives Considered**:
- React/Vue SPA: More complex, larger bundle
- Svelte: Good but adds build complexity
- Plain HTML: No interactivity

**Rationale**:
- Minimal JavaScript payload (~14KB)
- Progressive enhancement
- Server-rendered = better SEO (if needed)
- Askama provides compile-time template checking
- Simpler mental model (hypermedia)

### Database: SQLite

**Chosen**: SQLite with FTS5 full-text search

**Alternatives Considered**:
- PostgreSQL: More features, operational overhead
- MongoDB: Document model, less mature full-text
- Elasticsearch: Overkill for this scale

**Rationale**:
- Zero configuration
- Single file database
- FTS5 provides excellent full-text search
- Perfect for single-server deployment
- Easy backup (copy file)

### AI: Claude API

**Chosen**: Anthropic Claude API for scenario rulings

**Alternatives Considered**:
- OpenAI GPT-4: Similar quality, different alignment
- Local LLM: Lower quality, complex deployment
- No AI: Misses key feature

**Rationale**:
- High quality responses
- Anthropic's safety focus aligns with accurate rulings
- Simple HTTP API
- Good rate limits for MVP scale

### Bookmarks: Browser Local Storage

**Chosen**: Client-side storage with export/import

**Alternatives Considered**:
- Server-side storage: Requires authentication
- IndexedDB: More complex API
- Cookies: Size limitations

**Rationale**:
- No server-side state needed
- Works offline
- User controls their data
- Export/import for backup

### Deployment: Docker

**Chosen**: Docker with docker-compose

**Alternatives Considered**:
- Bare metal: Less portable
- Kubernetes: Overkill
- Serverless: Complex for Rust

**Rationale**:
- Reproducible builds
- Easy local development
- Single container simplicity
- Optional Qdrant container for vector search

## Consequences

### Positive

- Fast, type-safe backend
- Minimal frontend complexity
- Simple deployment
- No operational database to manage
- Good developer experience

### Negative

- Rust learning curve for contributors
- SQLite limits horizontal scaling
- FTS5 less sophisticated than Elasticsearch
- Client-side bookmarks not synced across devices

### Risks

- SQLite may need replacement if data grows significantly
- HTMX may feel limiting for complex interactions
- Rust compile times slow iteration

## Follow-up

- Consider vector search (Qdrant) for semantic search in M2
- Evaluate PostgreSQL if multi-user features added
- Monitor Claude API costs as usage grows
