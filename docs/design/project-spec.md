# Rulecraft Project Specification

## Overview

**Rulecraft** is a D&D 2024 rules lookup and AI-powered scenario ruling assistant designed for DMs and players who need quick, accurate rules references during gameplay.

## Problem Statement

D&D 2024 introduced significant rules changes from the 2014 edition. Players and DMs need:
- Quick access to rule lookups during sessions
- Clarification on how rules apply to specific scenarios
- A way to bookmark frequently referenced rules
- Distinction between RAW (Rules as Written) and RAI (Rules as Intended)

## Goals

1. **Speed** - Sub-100ms rule lookups
2. **Accuracy** - Only cite official 2024 rules
3. **Usability** - Minimal UI, keyboard-friendly
4. **Offline Capable** - Works without internet (except AI features)
5. **Portable** - Single Docker container deployment

## Non-Goals

- User accounts or authentication (MVP)
- Full rule text storage (copyright concerns)
- Character sheet management
- Combat tracking
- Virtual tabletop features

## User Stories

### As a DM

1. I want to search for a rule by keyword so I can resolve disputes quickly
2. I want to ask scenario questions so I can make fair rulings
3. I want to bookmark common rules so I can reference them repeatedly
4. I want citations with page numbers so players can verify

### As a Player

1. I want to look up my class features so I understand my abilities
2. I want to understand conditions so I know their effects
3. I want to save my most-used rules for quick reference

## Functional Requirements

### FR-1: Rule Search

- Full-text search across rule titles and content
- Filter by category (Combat, Spellcasting, etc.)
- Display source and page number
- Support for D&D terminology (AC, DC, etc.)

### FR-2: Rule Display

- Show full rule content
- Display source book and page
- Category and subcategory tags
- Related rules links (future)

### FR-3: AI Rulings

- Accept natural language scenario questions
- Provide ruling with RAW/RAI distinction
- Cite relevant rules
- Acknowledge uncertainty when appropriate

### FR-4: Bookmarks

- Save rules to browser local storage
- View bookmarked rules list
- Export/import bookmarks as JSON
- Works offline

## Technical Requirements

### TR-1: Performance

- Page load < 1 second
- Search response < 100ms
- AI ruling < 5 seconds

### TR-2: Reliability

- Graceful degradation without AI
- Offline support for search/bookmarks
- Error messages for API failures

### TR-3: Security

- No sensitive data storage
- HTTPS in production
- Input sanitization
- Rate limiting (future)

### TR-4: Compatibility

- Modern browsers (Chrome 90+, Firefox 88+, Safari 14+)
- Mobile responsive
- Keyboard accessible

## Data Model

### Rule

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| title | String | Rule name |
| category | String | Primary category |
| subcategory | String? | Optional subcategory |
| content | Text | Rule description/summary |
| source | String | Source book name |
| page | Int? | Page number |
| created_at | DateTime | Creation timestamp |
| updated_at | DateTime | Last update timestamp |

### Categories

- Combat
- Spellcasting
- Abilities
- Conditions
- Equipment
- Exploration
- Social
- Resting
- Class Features
- Feats
- Backgrounds
- Races/Species

## API Design

### REST Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Home page |
| GET | `/rules` | List all rules |
| GET | `/rules/:id` | Single rule detail |
| GET | `/search?q=` | Search rules |
| GET | `/scenario` | Scenario question form |
| POST | `/scenario/ask` | Submit scenario question |
| GET | `/health` | Health check |

### Response Formats

All responses are HTML (server-rendered with HTMX).
No JSON API in MVP (future consideration).

## UI Design

### Pages

1. **Home** - Hero, search bar, feature cards
2. **Rules List** - Filterable rule cards
3. **Rule Detail** - Full rule content, bookmark button
4. **Search Results** - Matching rules with excerpts
5. **Scenario** - Question form, AI response

### Theme

- Dark mode default (easier on eyes during sessions)
- Purple accent color (D&D aesthetic)
- Minimal, focused UI
- Large touch targets for mobile

## Milestones

### M1: MVP Foundation

- [x] Project scaffold
- [ ] Basic Axum server
- [ ] SQLite database
- [ ] Rules CRUD
- [ ] Full-text search

### M2: AI Integration

- [ ] Claude API client
- [ ] Scenario endpoint
- [ ] Response formatting
- [ ] Error handling

### M3: Bookmarks

- [ ] Local storage module
- [ ] Bookmark UI
- [ ] Export/import

### M4: Polish

- [ ] Responsive design
- [ ] Accessibility
- [ ] Docker deployment
- [ ] Documentation

## Success Metrics

1. **Usability** - Rule lookup in < 3 clicks
2. **Performance** - Search < 100ms (p95)
3. **Reliability** - 99% uptime
4. **Adoption** - 100+ weekly active users

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Copyright claims | High | Only store summaries, not full text |
| Claude API costs | Medium | Rate limiting, caching |
| Rules changes | Low | Regular database updates |
| Scope creep | Medium | Strict MVP definition |

## Open Questions

1. How to handle rules updates when WotC publishes errata?
2. Should we support multiple rule sources (PHB, DMG, etc.)?
3. Vector search: Qdrant vs. SQLite-vss vs. LanceDB?

## Appendix

### Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Language | Rust | Performance, safety |
| Web | Axum | Modern, async, ergonomic |
| Templates | Askama | Type-safe, compile-time |
| Database | SQLite | Simple, embedded, FTS5 |
| Frontend | HTMX | Minimal JS, hypermedia |
| AI | Claude API | Quality, Anthropic alignment |
| Deploy | Docker | Portable, reproducible |

### References

- [D&D 2024 Player's Handbook](https://www.dndbeyond.com/)
- [Axum Documentation](https://docs.rs/axum)
- [HTMX Documentation](https://htmx.org/)
- [Askama Templates](https://djc.github.io/askama/)
- [SQLite FTS5](https://www.sqlite.org/fts5.html)
