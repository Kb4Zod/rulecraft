# RuleCraft VPS Deployment Guide

This guide covers deploying RuleCraft to any Ubuntu 24.04 LTS KVM VPS with full security hardening. Examples use DigitalOcean, but Hostinger VPS, Linode, Vultr, and Hetzner all work — the setup script is host-agnostic.

For a friends-only MVP deployment, also see [Appendix A: Friends-only Access via Cloudflare Access](#appendix-a-friends-only-access-via-cloudflare-access) at the end of this document.

## Prerequisites

- A VPS provider account (DigitalOcean, Hostinger, Linode, Vultr, Hetzner — anything that offers Ubuntu 24.04 with full root access)
- Domain name (e.g., `rulecraft.hughscottjr.com`)
- Claude API key from Anthropic
- SSH key pair for secure access

## 1. Create a VPS

### Recommended Specs
- **Size**: ~$5–7/mo tier (1 vCPU, 1 GB RAM, 20+ GB SSD)
- **Region**: Choose closest to your users
- **Image**: Ubuntu 24.04 LTS (Clean OS — not a pre-installed control panel image)
- **Options**: Enable monitoring + IPv6 if available, and add your SSH public key during creation

### Host-specific notes

**DigitalOcean:** Basic $6/mo droplet, region NYC1 or SFO2.

**Hostinger:** hPanel → VPS → choose "KVM 1" (~$5–7/mo on annual prepay). OS template: **Ubuntu 24.04 LTS (Clean OS)** — *not* a panel image like hPanel-preinstalled. Paste your SSH public key into the *SSH Keys* section before first boot. Note the IPv4 address shown in hPanel once provisioned.

**Any other provider:** As long as it gives root SSH on a clean Ubuntu 24.04 VM, the rest of this guide applies unchanged.

### Initial Setup

```bash
# SSH into your new VPS (replace YOUR_VPS_IP)
ssh root@YOUR_VPS_IP
```

## 2. VPS Hardening

Run these commands to secure your VPS:

### 2.1 System Updates

```bash
apt update && apt upgrade -y
apt install -y ufw fail2ban curl git
```

### 2.2 Create Non-Root User

```bash
# Create user
adduser rulecraft
usermod -aG sudo rulecraft

# Copy SSH key to new user
mkdir -p /home/rulecraft/.ssh
cp /root/.ssh/authorized_keys /home/rulecraft/.ssh/
chown -R rulecraft:rulecraft /home/rulecraft/.ssh
chmod 700 /home/rulecraft/.ssh
chmod 600 /home/rulecraft/.ssh/authorized_keys
```

### 2.3 Disable Root SSH & Password Auth

```bash
# Edit SSH config
nano /etc/ssh/sshd_config
```

Set these values:
```
PermitRootLogin no
PasswordAuthentication no
PubkeyAuthentication yes
```

```bash
# Restart SSH
systemctl restart sshd
```

**IMPORTANT**: Test SSH login as `rulecraft` user before logging out!

```bash
# From your local machine
ssh rulecraft@YOUR_VPS_IP
```

### 2.4 Configure Firewall (UFW)

```bash
# Allow SSH, HTTP, HTTPS
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp

# Enable firewall
ufw enable

# Verify
ufw status
```

### 2.5 Configure fail2ban

```bash
# Create jail config
cat > /etc/fail2ban/jail.local << 'EOF'
[DEFAULT]
bantime = 1h
findtime = 10m
maxretry = 5

[sshd]
enabled = true
port = ssh
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
bantime = 24h
EOF

# Restart fail2ban
systemctl restart fail2ban
systemctl enable fail2ban
```

### 2.6 Enable Automatic Security Updates

```bash
apt install -y unattended-upgrades
dpkg-reconfigure -plow unattended-upgrades
```

## 3. Install Docker

```bash
# Install Docker
curl -fsSL https://get.docker.com | sh

# Add user to docker group
usermod -aG docker rulecraft

# Start Docker
systemctl enable docker
systemctl start docker

# Install Docker Compose
apt install -y docker-compose-plugin
```

Log out and back in for group changes to take effect.

## 4. Domain & DNS Setup

### Option A: Cloudflare (Recommended — required for friends-only Access gating, see Appendix A)

1. Add your domain to Cloudflare (Free plan is sufficient). Cloudflare will assign two nameservers.
2. Update nameservers at your registrar:
   - **Hostinger registrar:** hPanel → Domains → pick domain → *DNS / Nameservers* → switch to **Custom nameservers** and paste Cloudflare's two values. Propagation: 5–30 min typical, up to 24 h worst case. The domain stays *registered* with Hostinger; only DNS moves.
   - **Other registrars:** the equivalent setting is usually called "Nameservers" or "DNS".
3. Add DNS records in Cloudflare:
   - For a subdomain deployment (e.g., `rulecraft.hughscottjr.com`): one `A` record `rulecraft` → `YOUR_VPS_IP` (proxied — orange cloud).
   - For a root deployment: `A` record `@` → `YOUR_VPS_IP` (proxied), plus optional `A` record `www` → `YOUR_VPS_IP` (proxied).
4. SSL/TLS settings:
   - Mode: Full (strict)
   - Enable "Always Use HTTPS"
   - Enable "Automatic HTTPS Rewrites"

### Option B: Direct DNS (not compatible with Cloudflare Access)

1. At your domain registrar, add:
   - `A` record: `@` → `YOUR_VPS_IP`
   - `A` record: `www` → `YOUR_VPS_IP`

## 5. Deploy RuleCraft

### 5.1 Clone Repository

```bash
# As rulecraft user
cd ~
git clone https://github.com/YOUR_USERNAME/rulecraft.git
cd rulecraft
```

### 5.2 Configure Environment

```bash
# Create secure .env file
sudo mkdir -p /etc/rulecraft
sudo touch /etc/rulecraft/.env
sudo chmod 600 /etc/rulecraft/.env
sudo chown rulecraft:rulecraft /etc/rulecraft/.env

# Edit the file
nano /etc/rulecraft/.env
```

Add your secrets:
```env
CLAUDE_API_KEY=sk-ant-api03-YOUR_KEY_HERE
ADMIN_API_KEY=YOUR_SECURE_RANDOM_KEY
AI_RATE_LIMIT_PER_HOUR=5
SEARCH_RATE_LIMIT_PER_MINUTE=30
```

Generate a secure admin key:
```bash
openssl rand -hex 32
```

### 5.3 Configure Caddyfile

```bash
# Edit Caddyfile with your domain
cd docker
nano Caddyfile
```

The committed [Caddyfile](../../docker/Caddyfile) is **already pre-filled** with `rulecraft.hughscottjr.com`. Skip this sub-section unless you're deploying under a different domain — in which case, edit line 4 of the Caddyfile.

### 5.4 Create Symlink to Secrets

```bash
# Symlink .env for docker-compose
ln -s /etc/rulecraft/.env ~/rulecraft/docker/.env
```

### 5.5 Build and Deploy

```bash
cd ~/rulecraft/docker

# Build the image
docker compose -f docker-compose.prod.yml build

# Start services
docker compose -f docker-compose.prod.yml up -d

# Import YAML rules into the container's database
docker compose -f docker-compose.prod.yml exec rulecraft ./import_rules --rules-dir /app/rules

# Check status
docker compose -f docker-compose.prod.yml ps
docker compose -f docker-compose.prod.yml logs -f
```

## 6. Setup Backups

### 6.1 Create Backup Script

```bash
sudo mkdir -p /opt/backups
sudo chown rulecraft:rulecraft /opt/backups

cat > /opt/backups/backup-rulecraft.sh << 'EOF'
#!/bin/bash
BACKUP_DIR=/opt/backups
DATE=$(date +%Y%m%d_%H%M%S)
CONTAINER_DATA=/var/lib/docker/volumes/docker_rulecraft_data/_data

# Create backup
if [ -f "$CONTAINER_DATA/rulecraft.db" ]; then
    sqlite3 "$CONTAINER_DATA/rulecraft.db" ".backup '$BACKUP_DIR/rulecraft_$DATE.db'"
    echo "Backup created: rulecraft_$DATE.db"
else
    echo "Database not found at $CONTAINER_DATA/rulecraft.db"
    exit 1
fi

# Keep only last 7 days of backups
find $BACKUP_DIR -name "rulecraft_*.db" -mtime +7 -delete
echo "Cleaned old backups"
EOF

chmod +x /opt/backups/backup-rulecraft.sh
```

### 6.2 Schedule Daily Backups

```bash
# Add to crontab
crontab -e
```

Add this line:
```
0 3 * * * /opt/backups/backup-rulecraft.sh >> /var/log/rulecraft-backup.log 2>&1
```

## 7. Monitoring

### 7.1 DigitalOcean Monitoring

DigitalOcean provides built-in monitoring when you enable it during droplet creation.

### 7.2 External Uptime Monitoring (Recommended)

Set up free monitoring with Better Stack (formerly Logtail/Better Uptime):

1. Sign up at https://betterstack.com
2. Add a new monitor:
   - URL: `https://YOUR_DOMAIN/health`
   - Check interval: 5 minutes
   - Alert via: Email/Slack

### 7.3 View Logs

```bash
# Application logs
docker compose -f docker-compose.prod.yml logs -f rulecraft

# Caddy access logs
docker compose -f docker-compose.prod.yml logs -f caddy

# System logs
journalctl -u docker -f
```

## 8. Maintenance

### Update Application

```bash
cd ~/rulecraft
git pull
cd docker
docker compose -f docker-compose.prod.yml build
docker compose -f docker-compose.prod.yml up -d
```

### Restart Services

```bash
cd ~/rulecraft/docker
docker compose -f docker-compose.prod.yml restart
```

### View Resource Usage

```bash
docker stats
```

## 9. Verification Checklist

After deployment, verify:

- [ ] HTTPS working (visit `https://rulecraft.hughscottjr.com`)
- [ ] Health check responding (`curl https://rulecraft.hughscottjr.com/health`)
- [ ] Rate limiting active (test rapid requests to `/scenario/ask`)
- [ ] Admin endpoint protected (test `POST /api/rules` without key)
- [ ] SSH key-only auth working
- [ ] Firewall blocking other ports (`nmap YOUR_VPS_IP`)
- [ ] fail2ban running (`sudo fail2ban-client status sshd`)
- [ ] Backups running (check `/opt/backups/`)
- [ ] Monitoring alerting (trigger test alert)

## 10. Troubleshooting

### Container Won't Start

```bash
# Check logs
docker compose -f docker-compose.prod.yml logs rulecraft

# Check if port is in use
sudo lsof -i :3000
```

### SSL Certificate Issues

```bash
# Check Caddy logs
docker compose -f docker-compose.prod.yml logs caddy

# Force certificate renewal
docker compose -f docker-compose.prod.yml exec caddy caddy reload
```

### Database Issues

```bash
# Check database file exists
docker compose -f docker-compose.prod.yml exec rulecraft ls -la /app/data/

# Test database
docker compose -f docker-compose.prod.yml exec rulecraft sqlite3 /app/data/rulecraft.db ".tables"
```

## Cost Summary

| Item | Monthly Cost |
|------|--------------|
| VPS (Hostinger KVM 1 on annual prepay, or DigitalOcean Basic) | $5–7 |
| Domain (already owned) | $0 incremental |
| Cloudflare (DNS + Access Zero Trust) | Free |
| Claude API | $5–20 |
| Better Stack monitoring | Free tier |
| **Total** | **~$10–27/month** |

---

## Appendix A: Friends-only Access via Cloudflare Access

For an MVP deployment that's only shared with a handful of people (instead of being fully public), place Cloudflare Access in front of the app. It's free for up to 50 users and requires zero code changes — friends log in via Google or email magic link before they can reach the app.

### A.1 Prerequisites

- Domain is on Cloudflare DNS (see section 4, Option A).
- Zero Trust enabled on your Cloudflare account (free tier, no credit card). Cloudflare dashboard → *Zero Trust* → follow the one-time onboarding.

### A.2 Configure an authentication method

Zero Trust → **Settings → Authentication → Login methods → Add new**. Pick one:

- **One-time PIN** (email magic link) — zero friend setup; everyone has email. Simplest for mixed audiences.
- **Google** — nicer UX if all friends have Gmail.

You can enable both and let friends choose.

### A.3 Create the Access application

Zero Trust → **Access → Applications → Add an application → Self-hosted**:

- **Application name:** `RuleCraft`
- **Session duration:** 24 hours (or longer — friends shouldn't have to log in often)
- **Application domain:** `rulecraft.hughscottjr.com` (or whichever host you're deploying to)
- **Identity providers:** include the method(s) enabled in A.2

### A.4 Define the allowlist policy

On the same application:

- **Policy name:** `Friends`
- **Action:** Allow
- **Include rule:** *Emails* → add each friend's email address individually (easiest for small groups).
  - Alternative: *Emails ending in* if you want to allow a whole domain (e.g., `@hughscottjr.com`).

### A.5 Bypass policy for `/health`

The Caddy healthcheck hits `/health` without a session cookie, so it needs to skip Cloudflare Access. Add a second application:

- **Application type:** Self-hosted
- **Name:** `RuleCraft Health`
- **Application domain:** `rulecraft.hughscottjr.com/health` (path-scoped)
- **Policy action:** **Bypass**, **Everyone**

Cloudflare evaluates the more specific path application first, so only `/health` is unauthenticated.

### A.6 Verification

From your laptop (not logged in):

```bash
curl -I https://rulecraft.hughscottjr.com/
# → HTTP/2 302, Location: https://<tenant>.cloudflareaccess.com/...

curl -I https://rulecraft.hughscottjr.com/health
# → HTTP/2 200
```

Then open `https://rulecraft.hughscottjr.com` in a browser → you should land on the Cloudflare Access login page, enter an allowlisted email, receive a magic link (or sign in with Google), and land on the RuleCraft search page.

### A.7 Adding or removing friends later

Zero Trust → **Access → Applications → RuleCraft → Policies → Friends → Edit** → update the email list. Changes take effect on the next login (existing sessions persist until expiry).
