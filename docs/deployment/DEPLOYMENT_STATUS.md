# RuleCraft VPS Deployment - Implementation Status

**Last Updated:** 2026-02-21
**Project Location:** `D:\AI_Home\Workspace\40_Projects\rulecraft`

---

## Project Context

RuleCraft is a D&D 2024 rules lookup and AI-powered scenario ruling assistant. This document tracks the progress of deploying it to a VPS for public internet access.

## Deployment Requirements (Confirmed)

| Requirement | Decision |
|-------------|----------|
| Access Model | Fully public - all features available to anyone |
| Claude API Budget | $10-20/month (~500-1000 requests) |
| VPS Provider | DigitalOcean |
| Domain | Purchase needed (suggested: rulecraft.app) |
| Rate Limiting | Strict - 5 AI requests/IP/hour |

---

## Implementation Progress

### Phase 1: Critical Security Fixes - COMPLETE

| Task | Status | Files Modified |
|------|--------|----------------|
| Protect POST /api/rules | ✅ Done | `src/routes/rules.rs` |
| Add rate limiting | ✅ Done | `src/middleware/rate_limit.rs`, `src/routes/scenario.rs`, `src/routes/search.rs` |
| Add input validation | ✅ Done | `src/routes/rules.rs` |
| Add request logging | ✅ Done | `src/main.rs` |

**Security Improvements:**
- Admin API key authentication via `X-Admin-Key` header
- Per-IP rate limiting: 5 AI/hour, 30 search/minute
- Input validation with field length limits
- tower-http request/response logging

### Phase 2: Infrastructure Setup - NOT STARTED

| Task | Status | Notes |
|------|--------|-------|
| Purchase domain | ⬜ Pending | Suggested: rulecraft.app, dndrules.app |
| Create DigitalOcean droplet | ⬜ Pending | $6/mo Basic (1 vCPU, 1GB RAM) |
| Configure Cloudflare | ⬜ Pending | Free tier for DDoS + SSL |
| VPS hardening | ⬜ Pending | Script ready: `scripts/vps-setup.sh` |

### Phase 3: Deployment Configuration - COMPLETE

| Task | Status | Files Created |
|------|--------|---------------|
| Production Docker Compose | ✅ Done | `docker/docker-compose.prod.yml` |
| Caddyfile | ✅ Done | `docker/Caddyfile` |
| Dockerfile improvements | ✅ Done | `docker/Dockerfile` (pinned Rust version) |

### Phase 4: Documentation - COMPLETE

| Task | Status | Files Created |
|------|--------|---------------|
| VPS setup guide | ✅ Done | `docs/deployment/VPS_SETUP.md` |
| Setup script | ✅ Done | `scripts/vps-setup.sh` |
| Environment template | ✅ Done | `.env.example` (updated) |

---

## Files Modified/Created This Session

### New Files
```
src/middleware/mod.rs
src/middleware/rate_limit.rs
docker/docker-compose.prod.yml
docker/Caddyfile
docs/deployment/VPS_SETUP.md
docs/deployment/DEPLOYMENT_STATUS.md (this file)
scripts/vps-setup.sh
templates/scenario/error.html
```

### Modified Files
```
Cargo.toml                 - Added dependencies
.env.example               - Added new config options
src/config.rs              - Added admin_api_key, rate limits
src/main.rs                - Added middleware, logging
src/lib.rs                 - Added middleware module
src/routes/mod.rs          - Added rate_limiter to AppState
src/routes/rules.rs        - Auth + validation
src/routes/scenario.rs     - Rate limiting + validation
src/routes/search.rs       - Rate limiting
docker/Dockerfile          - Pinned Rust version
```

---

## New Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ADMIN_API_KEY` | none | Required for POST /api/rules |
| `AI_RATE_LIMIT_PER_HOUR` | 5 | Max AI requests per IP per hour |
| `SEARCH_RATE_LIMIT_PER_MINUTE` | 30 | Max search requests per IP per minute |

---

## Next Steps to Complete Deployment

### Immediate (To Deploy)

1. **Purchase Domain**
   - Recommended registrars: Cloudflare, Namecheap, Porkbun
   - Suggested names: `rulecraft.app`, `dndrules.app`

2. **Create DigitalOcean Droplet**
   ```
   Size: Basic $6/mo
   Image: Ubuntu 24.04 LTS
   Region: NYC1 or SFO2
   Options: Monitoring, IPv6
   ```

3. **Run VPS Setup Script**
   ```bash
   ssh root@YOUR_DROPLET_IP
   curl -fsSL https://raw.githubusercontent.com/Kb4Zod/rulecraft/main/scripts/vps-setup.sh | bash
   ```

4. **Configure Cloudflare**
   - Add domain
   - Set DNS A records to droplet IP
   - Enable proxy (orange cloud)
   - SSL mode: Full (strict)

5. **Deploy Application**
   ```bash
   ssh rulecraft@YOUR_DROPLET_IP
   cd ~/rulecraft/docker
   # Edit Caddyfile - replace YOUR_DOMAIN
   # Edit /etc/rulecraft/.env - add CLAUDE_API_KEY
   docker compose -f docker-compose.prod.yml up -d
   ```

### Post-Deployment

- [ ] Verify HTTPS working
- [ ] Test rate limiting
- [ ] Configure monitoring (Better Stack)
- [ ] Test backup script
- [ ] Rotate Claude API key (may be exposed)

---

## Estimated Costs

| Item | Monthly |
|------|---------|
| DigitalOcean Droplet | $6 |
| Domain (yearly/12) | ~$1 |
| Cloudflare | Free |
| Claude API | $10-20 |
| Monitoring | Free |
| **Total** | **~$17-27** |

---

## Deployment Readiness Score

| Before | After |
|--------|-------|
| 3/10 | 7/10 |

**Remaining to reach 10/10:**
- VPS provisioning and hardening
- Domain + DNS configuration
- SSL certificate activation
- Monitoring setup
- Backup verification

---

## Session Transcript

Full conversation transcript available at:
```
C:\Users\hughs\.claude\projects\D--AI-Home\0606f090-37a3-4df6-9b7d-325a0e5039c6.jsonl
```

---

*Document created: 2026-02-21*
