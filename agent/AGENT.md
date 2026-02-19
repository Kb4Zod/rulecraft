# Agent Operating Instructions

## Mission

Build and maintain **Rulecraft**, a D&D 2024 rules lookup and AI-powered scenario ruling assistant.

## Project Context

This is a Rust web application using:
- **Axum** - Web framework
- **Askama** - Type-safe HTML templating
- **SQLite** - Database with FTS5 full-text search
- **HTMX** - Frontend interactivity
- **Claude API** - AI-powered scenario rulings

## Load Order

1. Read `agent/context/glossary.md` for D&D terminology
2. Read `agent/context/constraints.md` for technical constraints
3. Review `docs/design/overview.md` for architecture
4. Check `docs/adr/` for past decisions

## Key Commands

```bash
# Install dependencies
cargo build

# Run development server
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy

# Build for release
cargo build --release

# Docker build
cd docker && docker-compose up --build
```

## Code Conventions

### Rust

- Use `thiserror` for error types
- Prefer `?` operator for error propagation
- Keep handlers thin, logic in modules
- Use `tracing` for logging

### Templates

- Base template at `templates/base.html`
- Use Askama's `{% extends %}` and `{% include %}`
- Keep templates in `templates/` matching route structure

### Database

- Migrations in `migrations/`
- Use `sqlx` query macros for type safety
- FTS5 for full-text search

## Architecture Decisions

See `docs/adr/` for recorded decisions:
- ADR 0001: Initial architecture choices (Axum, SQLite, HTMX)

## Domain Knowledge

### D&D 2024 Rules

- Focus on Player's Handbook 2024 and Dungeon Master's Guide 2024
- Do not reference 2014 edition rules (these are different)
- Distinguish between RAW (Rules as Written) and RAI (Rules as Intended)
- When rules are ambiguous, acknowledge uncertainty

### Key Concepts

- **Advantage/Disadvantage** - Roll 2d20, take higher/lower
- **Proficiency Bonus** - Scales with level (+2 to +6)
- **DC (Difficulty Class)** - Target number for checks
- **AC (Armor Class)** - Target for attack rolls

See `agent/context/glossary.md` for full terminology.

## File Locations

| Purpose | Location |
|---------|----------|
| Route handlers | `src/routes/` |
| Database ops | `src/db/` |
| Models | `src/models/` |
| Search logic | `src/search/` |
| AI integration | `src/ai/` |
| HTML templates | `templates/` |
| Static assets | `static/` |
| Migrations | `migrations/` |
| Documentation | `docs/` |

## Testing

```bash
# All tests
cargo test

# Integration tests only
cargo test --test '*'

# With output
cargo test -- --nocapture
```

## Common Tasks

### Add a new rule to database

1. Add to `migrations/001_initial.sql` (for seed data)
2. Or use SQLite directly for runtime additions

### Add a new route

1. Create handler in `src/routes/<name>.rs`
2. Add module to `src/routes/mod.rs`
3. Register in router
4. Create template in `templates/`

### Update AI prompts

Edit system prompt in `src/ai/claude.rs`

## Troubleshooting

### Build fails

```bash
cargo clean
cargo build
```

### Database issues

```bash
rm rulecraft.db
cargo run  # Recreates with migrations
```

### Template not found

Ensure template path matches exactly in `#[template(path = "...")]`
