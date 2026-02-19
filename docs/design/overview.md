# Architecture Overview

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  HTMX       │  │  Bookmarks  │  │  Static Assets          │  │
│  │  (14KB)     │  │  (LocalStorage)│ │  (CSS/JS)              │  │
│  └──────┬──────┘  └─────────────┘  └─────────────────────────┘  │
└─────────┼───────────────────────────────────────────────────────┘
          │ HTTP (HTML fragments)
          ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Axum Web Server                              │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Routes Layer                          │    │
│  │  /           → Index                                     │    │
│  │  /rules      → Rules List                                │    │
│  │  /rules/:id  → Rule Detail                               │    │
│  │  /search     → Search Results                            │    │
│  │  /scenario   → Scenario Form                             │    │
│  │  /health     → Health Check                              │    │
│  └──────────────────────┬──────────────────────────────────┘    │
│                         │                                        │
│  ┌──────────────────────┼──────────────────────────────────┐    │
│  │              Application Layer                           │    │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐ │    │
│  │  │ Models  │  │ Search  │  │   AI    │  │  Templates  │ │    │
│  │  │ (Rule)  │  │ (FTS5)  │  │(Claude) │  │  (Askama)   │ │    │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └──────┬──────┘ │    │
│  └───────┼────────────┼────────────┼──────────────┼────────┘    │
│          │            │            │              │              │
│  ┌───────▼────────────▼────────────┼──────────────┼────────┐    │
│  │              Data Layer         │              │         │    │
│  │  ┌─────────────────────┐        │              │         │    │
│  │  │     SQLite DB       │        │   Templates  │         │    │
│  │  │  ┌─────────────┐    │        │   (HTML)     │         │    │
│  │  │  │   rules     │    │        │              │         │    │
│  │  │  ├─────────────┤    │        │              │         │    │
│  │  │  │  rules_fts  │    │        │              │         │    │
│  │  │  │   (FTS5)    │    │        │              │         │    │
│  │  │  └─────────────┘    │        │              │         │    │
│  │  └─────────────────────┘        │              │         │    │
│  └─────────────────────────────────┼──────────────┼─────────┘    │
│                                    │              │              │
└────────────────────────────────────┼──────────────┼──────────────┘
                                     │              │
                    ┌────────────────▼──────┐       │
                    │    Claude API         │       │
                    │  (api.anthropic.com)  │       │
                    └───────────────────────┘       │
                                                    │
                    ┌───────────────────────────────▼──────────────┐
                    │               File System                     │
                    │  templates/   static/   rulecraft.db          │
                    └──────────────────────────────────────────────┘
```

## Components

### Web Layer

**Axum** handles HTTP routing with typed extractors and async handlers.

| Route | Method | Handler | Description |
|-------|--------|---------|-------------|
| `/` | GET | `index` | Home page |
| `/rules` | GET | `list_rules` | Browse all rules |
| `/rules/:id` | GET | `get_rule` | Single rule detail |
| `/search` | GET | `search_rules` | Search with query param |
| `/scenario` | GET | `scenario_form` | AI ruling form |
| `/scenario/ask` | POST | `ask_scenario` | Submit question to AI |
| `/health` | GET | `health` | Health check |
| `/static/*` | GET | ServeDir | Static assets |

### Template Layer

**Askama** provides compile-time checked HTML templates.

```
templates/
├── base.html              # Base layout with nav/footer
├── index.html             # Home page
├── rules/
│   ├── list.html          # Rules listing
│   ├── detail.html        # Single rule view
│   └── search.html        # Search results
├── scenario/
│   ├── ask.html           # Question form
│   └── response.html      # AI ruling display
└── partials/
    ├── rule_card.html     # Reusable rule card
    └── bookmark_btn.html  # Bookmark button
```

### Data Layer

**SQLite** with FTS5 full-text search extension.

```sql
-- Main rules table
CREATE TABLE rules (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    category TEXT NOT NULL,
    subcategory TEXT,
    content TEXT NOT NULL,
    source TEXT NOT NULL,
    page INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Full-text search index
CREATE VIRTUAL TABLE rules_fts USING fts5(
    title, content, category,
    content='rules'
);
```

### AI Layer

**Claude API** integration for scenario rulings.

```rust
// System prompt enforces 2024 rules focus
let system_prompt = "You are a D&D 2024 rules expert...";

// Relevant rules included for context
let request = ClaudeRequest {
    model: "claude-sonnet-4-20250514",
    max_tokens: 1024,
    messages: [{ role: "user", content: question }],
    system: system_prompt,
};
```

### Client Layer

**HTMX** for dynamic updates without heavy JavaScript.

```html
<!-- Form submits via HTMX, response replaces target -->
<form hx-post="/scenario/ask" hx-target="#response">
    <textarea name="question"></textarea>
    <button type="submit">Get Ruling</button>
</form>
<div id="response"></div>
```

**Local Storage** for bookmarks (no server-side state).

```javascript
// Bookmarks stored in browser
localStorage.setItem('rulecraft_bookmarks', JSON.stringify({
    'rule-id-1': { title: 'Sneak Attack', addedAt: '...' }
}));
```

## Data Flow

### Rule Search

```
1. User enters query in search form
2. Browser sends GET /search?q=query
3. Axum extracts query parameter
4. fulltext::search() queries FTS5
5. Results passed to SearchResultsTemplate
6. Askama renders HTML
7. Response sent to browser
```

### AI Scenario Ruling

```
1. User submits question form
2. HTMX sends POST /scenario/ask
3. Axum extracts form data
4. Search for relevant rules (context)
5. Build Claude API request with rules context
6. Send to api.anthropic.com
7. Parse response
8. Render ScenarioResponseTemplate
9. HTMX replaces target div
```

## Deployment

### Docker Compose

```yaml
services:
  rulecraft:
    build: .
    ports: ["3000:3000"]
    volumes: [rulecraft_data:/app/data]
    environment:
      - DATABASE_URL=sqlite:./data/rulecraft.db
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}

  # Optional vector search
  qdrant:
    image: qdrant/qdrant
    profiles: [vector-search]
```

## Future Enhancements

1. **Vector Search** - Qdrant integration for semantic search
2. **Rule Import** - Bulk import from structured sources
3. **User Accounts** - Optional server-side bookmark sync
4. **Offline PWA** - Service worker for offline access
5. **Mobile App** - Tauri or React Native wrapper
