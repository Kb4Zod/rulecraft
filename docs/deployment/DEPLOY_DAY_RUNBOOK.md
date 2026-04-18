# RuleCraft Deploy-Day Runbook

Single-sheet walkthrough for the MVP deploy. Use this when running through the deploy live. Full reference lives in [VPS_SETUP.md](VPS_SETUP.md); status tracking in [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md); decision context in the plan file at `C:\Users\hughs\.claude\plans\crispy-hopping-abelson.md`.

**Target URL:** `https://rulecraft.hughscottjr.com`
**Gate:** Cloudflare Access allowlist (friends-only MVP)
**Estimated wall-clock time:** 60–90 min, plus DNS propagation wait

---

## Things to have in hand before starting

- [ ] Hostinger login (domain + likely VPS)
- [ ] Cloudflare account (free tier; will enable Zero Trust during deploy)
- [ ] Anthropic Console login (to mint a Claude API key)
- [ ] SSH public key ready to paste (`cat ~/.ssh/id_ed25519.pub` or generate with `ssh-keygen -t ed25519`)
- [ ] List of friend emails to allowlist
- [ ] A password manager open (you'll stash an admin API key and the Cloudflare magic links)

---

## Step 1 — Provision the VPS (~10 min)

### Hostinger path
1. hPanel → **VPS** → Add VPS → **KVM 1** (annual prepay for best price).
2. OS template: **Ubuntu 24.04 LTS (Clean OS)**. *Not* a hPanel-preinstalled image.
3. Paste SSH public key into *SSH Keys* before the first boot.
4. Note the public IPv4 address once it's provisioned.

### DigitalOcean path (fallback)
1. Create → Droplet → Basic $6/mo → NYC1 or SFO2 → Ubuntu 24.04 LTS → attach SSH key → enable monitoring + IPv6.
2. Note the public IPv4.

---

## Step 2 — Harden + install Docker (~10 min)

SSH in as `root`:
```bash
ssh root@YOUR_VPS_IP
```

Run the bundled setup script. It prompts for a username (accept `rulecraft`) and a domain (enter `rulecraft.hughscottjr.com`):
```bash
curl -fsSL https://raw.githubusercontent.com/Kb4Zod/rulecraft/main/scripts/vps-setup.sh | bash
```

The script does: apt upgrade, creates `rulecraft` user with sudo, hardens sshd, configures UFW (22/80/443), installs fail2ban, installs Docker + compose plugin, enables unattended-upgrades.

**Before logging out**, open a second terminal and verify you can SSH in as `rulecraft`:
```bash
ssh rulecraft@YOUR_VPS_IP
```

---

## Step 3 — Move DNS to Cloudflare (~10 min active + propagation wait)

1. **Cloudflare dashboard** → Add site → `hughscottjr.com` → Free plan. Cloudflare shows two NS values (e.g., `arya.ns.cloudflare.com`, `tim.ns.cloudflare.com`).
2. **Hostinger hPanel** → Domains → `hughscottjr.com` → DNS/Nameservers → **Custom nameservers** → paste Cloudflare's two values → save.
3. Wait for propagation (5–30 min typical). Cloudflare emails when it's active. You can continue to Step 4 while you wait — just don't deploy yet.
4. Once active: Cloudflare → `hughscottjr.com` → DNS → Records:
   - **Type:** A, **Name:** `rulecraft`, **IPv4:** your VPS IP, **Proxy status:** Proxied (orange cloud). TTL: Auto.
5. **SSL/TLS** → Overview → **Full (strict)**. Edge Certificates → enable *Always Use HTTPS*, *Automatic HTTPS Rewrites*.

---

## Step 4 — Configure Cloudflare Access (friends-only gate) (~15 min)

Full details: [VPS_SETUP.md Appendix A](VPS_SETUP.md#appendix-a-friends-only-access-via-cloudflare-access).

1. Cloudflare dashboard → **Zero Trust** → complete the one-time onboarding (pick a team name — shows up in the login URL).
2. Zero Trust → **Settings → Authentication → Login methods** → Add: **One-time PIN** (email magic link). Optionally also Google.
3. Zero Trust → **Access → Applications → Add → Self-hosted**:
   - Name: `RuleCraft`
   - Session duration: 24 hours
   - Application domain: `rulecraft.hughscottjr.com`
   - Identity provider: the method(s) from step 2
4. Policy on that app:
   - Name: `Friends`, Action: **Allow**
   - Include: **Emails** → add each friend's email address (can add more later)
5. Add a second application for the healthcheck:
   - Type: Self-hosted
   - Name: `RuleCraft Health`
   - Application domain: `rulecraft.hughscottjr.com/health` (path-scoped — the `/health` suffix matters)
   - Policy: Action **Bypass**, Selector **Everyone**

---

## Step 5 — Deploy the app (~15 min)

SSH in as `rulecraft` (not root):
```bash
ssh rulecraft@YOUR_VPS_IP
```

Clone and configure:
```bash
# 5.1 Clone
cd ~
git clone https://github.com/Kb4Zod/rulecraft.git
cd rulecraft

# 5.2 Secure .env file (kept outside the repo dir)
sudo mkdir -p /etc/rulecraft
sudo touch /etc/rulecraft/.env
sudo chmod 600 /etc/rulecraft/.env
sudo chown rulecraft:rulecraft /etc/rulecraft/.env

# 5.3 Generate an admin API key (SAVE THIS to your password manager)
openssl rand -hex 32
```

Edit `/etc/rulecraft/.env`:
```bash
nano /etc/rulecraft/.env
```

Paste (with real values):
```env
CLAUDE_API_KEY=sk-ant-api03-...
CLAUDE_MODEL=claude-sonnet-4-20250514
ADMIN_API_KEY=<the hex string from step 5.3>
AI_RATE_LIMIT_PER_HOUR=5
SEARCH_RATE_LIMIT_PER_MINUTE=30
```

Symlink .env for docker-compose and deploy:
```bash
ln -s /etc/rulecraft/.env ~/rulecraft/docker/.env

cd ~/rulecraft/docker

# 5.4 Build + start (Caddyfile is already pre-filled — no edit needed)
docker compose -f docker-compose.prod.yml build
docker compose -f docker-compose.prod.yml up -d

# 5.5 Seed the database with bundled YAML rules
docker compose -f docker-compose.prod.yml exec rulecraft ./import_rules --rules-dir /app/rules

# 5.6 Check status
docker compose -f docker-compose.prod.yml ps
docker compose -f docker-compose.prod.yml logs -f rulecraft
# (Ctrl-C out of logs once you see "listening on 0.0.0.0:3000")
```

---

## Step 6 — Verify (~5 min)

On the VPS:
```bash
# Containers healthy
docker compose -f docker-compose.prod.yml ps
# Both rulecraft + rulecraft-caddy should show "Up (healthy)"

# Health endpoint responding inside the droplet
curl -f http://localhost:3000/health
# → 200 OK
```

From your laptop:
```bash
# Gate is live — should 302 to a cloudflareaccess.com URL
curl -I https://rulecraft.hughscottjr.com/

# Health bypass works — should be 200
curl -I https://rulecraft.hughscottjr.com/health
```

In your browser:
1. Visit `https://rulecraft.hughscottjr.com/` → Cloudflare Access login → enter your email → magic link in inbox → click link → land on search page.
2. Search `fireball` → see rule results.
3. Visit `/scenario`, ask a question → Claude responds (confirms API key wired).
4. Rapid-fire 6 scenario questions → 6th returns a rate-limit message (confirms defense-in-depth).

Admin endpoint sanity check:
```bash
# Should be blocked by Cloudflare Access (not 401 from the app)
curl -I -X POST https://rulecraft.hughscottjr.com/api/rules
```

---

## Step 7 — Backups (~5 min, optional but recommended)

On the VPS as `rulecraft`:
```bash
sudo mkdir -p /opt/backups
sudo chown rulecraft:rulecraft /opt/backups
```

Create `/opt/backups/backup-rulecraft.sh` (full content in [VPS_SETUP.md §6.1](VPS_SETUP.md#61-create-backup-script)), then:
```bash
chmod +x /opt/backups/backup-rulecraft.sh
crontab -e
# Add: 0 3 * * * /opt/backups/backup-rulecraft.sh >> /var/log/rulecraft-backup.log 2>&1
```

---

## Step 8 — Share with friends

1. Zero Trust → Access → Applications → RuleCraft → Policies → Friends → confirm the email list.
2. DM friends: "Go to `https://rulecraft.hughscottjr.com`, enter your email, click the magic link." First-time visit takes ~30 seconds; session lasts 24h.

---

## Rollback / "something's wrong"

| Symptom | Check |
|---------|-------|
| `https://rulecraft.hughscottjr.com` shows Cloudflare 522/523 | VPS down or firewall blocking — `ssh` in, `docker compose ps`, `ufw status` |
| Gets to Cloudflare Access but the login page errors | Zero Trust → Access → Applications → verify app domain exactly matches `rulecraft.hughscottjr.com` |
| Logged in but app shows 502 | Caddy can't reach the app — `docker compose logs rulecraft` |
| `/scenario` responds with errors | Missing/invalid `CLAUDE_API_KEY` in `/etc/rulecraft/.env` — re-edit, then `docker compose restart rulecraft` |
| `/health` 302s to Cloudflare login | Bypass policy misconfigured — the second Access application must be path-scoped to `/health` |
| Caddy can't get TLS cert | Check Cloudflare SSL mode is **Full (strict)** and DNS `A` record exists |

Container restart without rebuilding:
```bash
docker compose -f docker-compose.prod.yml restart
```

Full redeploy after a `git pull`:
```bash
cd ~/rulecraft && git pull
cd docker && docker compose -f docker-compose.prod.yml build && docker compose -f docker-compose.prod.yml up -d
```

---

## Post-deploy cleanup

- Flip unchecked items in [DEPLOYMENT_STATUS.md](DEPLOYMENT_STATUS.md) to ✅.
- If the Claude API key in your password manager was ever shared/emailed, rotate it at console.anthropic.com.
