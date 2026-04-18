# RuleCraft VPS Deployment - Implementation Status

**Last Updated:** 2026-04-18
**Project Location:** `S:\AI_Home\Workspace\40_Projects\rulecraft`

---

## Project Context

RuleCraft is a D&D 2024 rules lookup and AI-powered scenario ruling assistant. This document tracks the progress of deploying it as a friends-only MVP — a small allowlist of friends sign in via Cloudflare Access before reaching the app.

**Evolution of this plan:** The earlier version of this document targeted a fully-public launch on DigitalOcean with a purchased domain. The current MVP target is narrower (friends-only) and host-agnostic (Hostinger or DigitalOcean).

## Deployment Requirements (Confirmed)

| Requirement | Decision |
|-------------|----------|
| Access Model | Friends-only — Cloudflare Access allowlist (~10 emails) |
| Claude API Budget | $5–20/month (rate-limited + email-gated) |
| VPS Provider | Host-agnostic. Likely **Hostinger KVM 1** (domain already registered there). DigitalOcean Basic droplet is the fallback reference. |
| Domain | `hughscottjr.com` (owned, registered at Hostinger). Public URL: `rulecraft.hughscottjr.com`. Nameservers move to Cloudflare; registration stays at Hostinger. |
| Rate Limiting | Kept at 5 AI/IP/hour, 30 search/IP/min as defense-in-depth behind Cloudflare Access |

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
| Domain | ✅ Owned | `hughscottjr.com` registered at Hostinger — no purchase needed |
| Provision VPS | ⬜ Pending | Hostinger KVM 1 (~$5–7/mo on annual prepay) or DigitalOcean Basic ($6/mo) — Ubuntu 24.04 LTS, Clean OS |
| Move nameservers to Cloudflare | ⬜ Pending | From Hostinger hPanel → Domains → Custom nameservers |
| Configure Cloudflare DNS + SSL | ⬜ Pending | `A` record `rulecraft` → VPS IP (proxied); SSL Full (strict) |
| Enable Cloudflare Zero Trust | ⬜ Pending | Free tier, no credit card |
| Configure Cloudflare Access (friends-only gate) | ⬜ Pending | See VPS_SETUP.md Appendix A |
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

1. **Provision VPS** (Hostinger KVM 1 or DigitalOcean Basic)
   ```
   Size: ~$5–7/mo
   Image: Ubuntu 24.04 LTS (Clean OS)
   SSH key: added at provisioning time
   ```

2. **Run VPS Setup Script**
   ```bash
   ssh root@YOUR_VPS_IP
   curl -fsSL https://raw.githubusercontent.com/Kb4Zod/rulecraft/main/scripts/vps-setup.sh | bash
   ```

3. **Move DNS to Cloudflare**
   - Add `hughscottjr.com` to Cloudflare (Free plan)
   - In Hostinger hPanel → Domains → Custom nameservers → paste Cloudflare's two NS values
   - Wait for propagation (5–30 min typical)
   - Add `A` record `rulecraft` → VPS IP (proxied / orange cloud)
   - SSL mode: Full (strict); enable Always Use HTTPS

4. **Gate with Cloudflare Access** (friends-only MVP)
   - Zero Trust → Settings → Authentication → enable One-time PIN (email magic link) and/or Google
   - Zero Trust → Access → Applications → Self-hosted app for `rulecraft.hughscottjr.com`
   - Policy `Friends`: Allow, include specific friend emails
   - Second app at path `rulecraft.hughscottjr.com/health` → Bypass / Everyone
   - See VPS_SETUP.md Appendix A for full details

5. **Deploy Application**
   ```bash
   ssh rulecraft@YOUR_VPS_IP
   cd ~/rulecraft/docker
   # Caddyfile already pre-filled for rulecraft.hughscottjr.com — no edit needed
   # Edit /etc/rulecraft/.env - add CLAUDE_API_KEY and ADMIN_API_KEY
   docker compose -f docker-compose.prod.yml up -d
   docker compose -f docker-compose.prod.yml exec rulecraft ./import_rules --rules-dir /app/rules
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
| VPS (Hostinger KVM 1 annual prepay, or DigitalOcean Basic) | $5–7 |
| Domain (already owned at Hostinger) | $0 incremental |
| Cloudflare DNS + Access (free tier, ≤50 users) | Free |
| Claude API (friend-group volume, rate-limited) | $5–20 |
| Monitoring | Free |
| **Total** | **~$10–27** |

---

## Deployment Readiness Score

| Before | After (MVP scope) |
|--------|-------------------|
| 3/10 | 8/10 |

**Remaining to reach 10/10:**
- VPS provisioning and hardening
- Nameservers moved from Hostinger to Cloudflare
- Cloudflare Access application + Friends allowlist configured
- SSL certificate activation
- Backup cron verified
- (Deferred for post-MVP) Uptime monitoring via Better Stack

---

## Session Transcript

Full conversation transcript available at:
```
C:\Users\hughs\.claude\projects\D--AI-Home\0606f090-37a3-4df6-9b7d-325a0e5039c6.jsonl
```

---

*Document created: 2026-02-21*
