# Technical and Domain Constraints

## Scope Constraints

### D&D 2024 Only

- **Include**: Player's Handbook 2024, Dungeon Master's Guide 2024
- **Exclude**: 2014 edition rules (these differ significantly)
- **Exclude**: Homebrew or unofficial content
- **Exclude**: Older editions (3.5e, 4e, etc.)

### Rules Accuracy

- Always distinguish between RAW (Rules as Written) and RAI (Rules as Intended)
- When rules are ambiguous, state the ambiguity clearly
- Cite specific page numbers when possible
- If DM discretion is required, say so explicitly

## Technical Constraints

### Database

- SQLite single file database
- FTS5 for full-text search (built into SQLite)
- No external database services required
- Vector search via Qdrant is optional/future enhancement

### Authentication

- No user accounts in MVP
- Bookmarks stored in browser local storage only
- No server-side session management

### Performance

- Target < 100ms response time for searches
- Target < 3s for AI rulings (depends on Claude API)
- Minimize JavaScript payload (HTMX only, ~14KB)

### Deployment

- Docker-first deployment
- Single container for main application
- Optional Qdrant container for vector search
- Must work on localhost without internet (except AI features)

## Content Constraints

### Legal

- Do not include full rule text from copyrighted sources
- Include only:
  - Rule names and references
  - Page number citations
  - Brief summaries (fair use)
  - User-contributed rule explanations

### AI Responses

- Claude must not invent rules that don't exist
- Always cite actual rules when making rulings
- Acknowledge uncertainty when rules are unclear
- Recommend consulting the source books for full text

## API Constraints

### Claude API

- Use `claude-sonnet-4-20250514` by default (balance of speed/quality)
- Max 1024 tokens per response (sufficient for rulings)
- Include relevant rules in context for grounding
- System prompt enforces 2024 rules focus

### Rate Limiting

- No built-in rate limiting in MVP
- Rely on Claude API rate limits
- Future: Add request throttling

## Browser Compatibility

- Modern browsers only (Chrome, Firefox, Safari, Edge)
- ES6+ JavaScript required
- CSS Grid/Flexbox support required
- Local Storage API required

## Accessibility

- Semantic HTML structure
- Keyboard navigation support
- ARIA labels where appropriate
- Color contrast compliance (WCAG AA)
