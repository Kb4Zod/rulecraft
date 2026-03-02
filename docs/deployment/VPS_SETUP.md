# RuleCraft VPS Deployment Guide

This guide covers deploying RuleCraft to a DigitalOcean droplet with full security hardening.

## Prerequisites

- DigitalOcean account
- Domain name (e.g., `rulecraft.app`)
- Claude API key from Anthropic
- SSH key pair for secure access

## 1. Create DigitalOcean Droplet

### Recommended Specs
- **Size**: Basic $6/mo (1 vCPU, 1GB RAM, 25GB SSD)
- **Region**: NYC1 or SFO2 (choose closest to users)
- **Image**: Ubuntu 24.04 LTS
- **Options**: Enable monitoring, IPv6, and add your SSH key

### Initial Setup

```bash
# SSH into your new droplet
ssh root@YOUR_DROPLET_IP
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
ssh rulecraft@YOUR_DROPLET_IP
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

### Option A: Cloudflare (Recommended)

1. Add your domain to Cloudflare
2. Update nameservers at your registrar
3. Add DNS records:
   - `A` record: `@` → `YOUR_DROPLET_IP` (proxied - orange cloud)
   - `A` record: `www` → `YOUR_DROPLET_IP` (proxied - orange cloud)
4. SSL/TLS settings:
   - Mode: Full (strict)
   - Enable "Always Use HTTPS"
   - Enable "Automatic HTTPS Rewrites"

### Option B: Direct DNS

1. At your domain registrar, add:
   - `A` record: `@` → `YOUR_DROPLET_IP`
   - `A` record: `www` → `YOUR_DROPLET_IP`

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

Replace `YOUR_DOMAIN` with your actual domain (e.g., `rulecraft.app`).

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

- [ ] HTTPS working (visit `https://YOUR_DOMAIN`)
- [ ] Health check responding (`curl https://YOUR_DOMAIN/health`)
- [ ] Rate limiting active (test rapid requests to `/scenario/ask`)
- [ ] Admin endpoint protected (test `POST /api/rules` without key)
- [ ] SSH key-only auth working
- [ ] Firewall blocking other ports (`nmap YOUR_DROPLET_IP`)
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
| DigitalOcean Droplet (Basic) | $6 |
| Domain (yearly / 12) | ~$1 |
| Cloudflare | Free |
| Claude API | $10-20 |
| Better Stack monitoring | Free tier |
| **Total** | **~$17-27/month** |
