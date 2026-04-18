#!/bin/bash
# RuleCraft VPS Setup Script
# Run this script on a fresh Ubuntu 24.04 VPS as root
# Usage: curl -fsSL https://raw.githubusercontent.com/YOUR_USER/rulecraft/main/scripts/vps-setup.sh | bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}RuleCraft VPS Setup Script${NC}"
echo -e "${GREEN}========================================${NC}"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo -e "${RED}Please run as root${NC}"
    exit 1
fi

# Prompt for configuration
read -p "Enter username to create (default: rulecraft): " USERNAME
USERNAME=${USERNAME:-rulecraft}

read -p "Enter your domain (e.g., rulecraft.hughscottjr.com): " DOMAIN
if [ -z "$DOMAIN" ]; then
    echo -e "${RED}Domain is required${NC}"
    exit 1
fi

echo ""
echo -e "${YELLOW}Configuration:${NC}"
echo "  Username: $USERNAME"
echo "  Domain: $DOMAIN"
echo ""
read -p "Continue? (y/n): " CONFIRM
if [ "$CONFIRM" != "y" ]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo -e "${GREEN}[1/7] Updating system...${NC}"
apt update && apt upgrade -y
apt install -y ufw fail2ban curl git sqlite3

echo ""
echo -e "${GREEN}[2/7] Creating user $USERNAME...${NC}"
if id "$USERNAME" &>/dev/null; then
    echo "User $USERNAME already exists"
else
    adduser --disabled-password --gecos "" $USERNAME
    usermod -aG sudo $USERNAME

    # Copy SSH keys from root
    if [ -d /root/.ssh ]; then
        mkdir -p /home/$USERNAME/.ssh
        cp /root/.ssh/authorized_keys /home/$USERNAME/.ssh/ 2>/dev/null || true
        chown -R $USERNAME:$USERNAME /home/$USERNAME/.ssh
        chmod 700 /home/$USERNAME/.ssh
        chmod 600 /home/$USERNAME/.ssh/authorized_keys 2>/dev/null || true
    fi
fi

echo ""
echo -e "${GREEN}[3/7] Configuring SSH security...${NC}"
# Backup original config
cp /etc/ssh/sshd_config /etc/ssh/sshd_config.bak

# Update SSH config
sed -i 's/#*PermitRootLogin.*/PermitRootLogin no/' /etc/ssh/sshd_config
sed -i 's/#*PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/#*PubkeyAuthentication.*/PubkeyAuthentication yes/' /etc/ssh/sshd_config

systemctl restart sshd

echo ""
echo -e "${GREEN}[4/7] Configuring firewall...${NC}"
ufw --force reset
ufw default deny incoming
ufw default allow outgoing
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable

echo ""
echo -e "${GREEN}[5/7] Configuring fail2ban...${NC}"
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

systemctl restart fail2ban
systemctl enable fail2ban

echo ""
echo -e "${GREEN}[6/7] Installing Docker...${NC}"
if command -v docker &> /dev/null; then
    echo "Docker already installed"
else
    curl -fsSL https://get.docker.com | sh
    usermod -aG docker $USERNAME
    systemctl enable docker
    systemctl start docker
fi

# Install Docker Compose plugin if not present
if ! docker compose version &> /dev/null; then
    apt install -y docker-compose-plugin
fi

echo ""
echo -e "${GREEN}[7/7] Creating directories and configs...${NC}"

# Create secrets directory
mkdir -p /etc/rulecraft
touch /etc/rulecraft/.env
chmod 600 /etc/rulecraft/.env
chown $USERNAME:$USERNAME /etc/rulecraft/.env

# Generate admin API key
ADMIN_KEY=$(openssl rand -hex 32)

# Create initial .env
cat > /etc/rulecraft/.env << EOF
# RuleCraft Production Configuration
# Generated: $(date)

# Claude API Key (REQUIRED - get from https://console.anthropic.com)
CLAUDE_API_KEY=

# Admin API Key for protected endpoints
ADMIN_API_KEY=$ADMIN_KEY

# Rate Limiting
AI_RATE_LIMIT_PER_HOUR=5
SEARCH_RATE_LIMIT_PER_MINUTE=30

# Claude Model
CLAUDE_MODEL=claude-sonnet-4-20250514
EOF

# Create backup directory
mkdir -p /opt/backups
chown $USERNAME:$USERNAME /opt/backups

# Create backup script
cat > /opt/backups/backup-rulecraft.sh << 'BACKUP_EOF'
#!/bin/bash
BACKUP_DIR=/opt/backups
DATE=$(date +%Y%m%d_%H%M%S)
CONTAINER_DATA=/var/lib/docker/volumes/docker_rulecraft_data/_data

if [ -f "$CONTAINER_DATA/rulecraft.db" ]; then
    sqlite3 "$CONTAINER_DATA/rulecraft.db" ".backup '$BACKUP_DIR/rulecraft_$DATE.db'"
    echo "$(date): Backup created: rulecraft_$DATE.db"
else
    echo "$(date): Database not found"
    exit 1
fi

find $BACKUP_DIR -name "rulecraft_*.db" -mtime +7 -delete
BACKUP_EOF

chmod +x /opt/backups/backup-rulecraft.sh

# Add backup cron job
(crontab -l 2>/dev/null | grep -v "backup-rulecraft"; echo "0 3 * * * /opt/backups/backup-rulecraft.sh >> /var/log/rulecraft-backup.log 2>&1") | crontab -

# Enable automatic security updates
apt install -y unattended-upgrades
echo 'Unattended-Upgrade::Automatic-Reboot "false";' >> /etc/apt/apt.conf.d/50unattended-upgrades

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}Setup Complete!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${YELLOW}Next Steps:${NC}"
echo ""
echo "1. TEST SSH ACCESS BEFORE LOGGING OUT:"
echo "   ssh $USERNAME@$(hostname -I | awk '{print $1}')"
echo ""
echo "2. Configure your Claude API key:"
echo "   sudo nano /etc/rulecraft/.env"
echo ""
echo "3. Clone and deploy RuleCraft:"
echo "   su - $USERNAME"
echo "   git clone https://github.com/YOUR_USER/rulecraft.git"
echo "   cd rulecraft/docker"
echo "   # Update Caddyfile with your domain: $DOMAIN"
echo "   ln -s /etc/rulecraft/.env .env"
echo "   docker compose -f docker-compose.prod.yml build"
echo "   docker compose -f docker-compose.prod.yml up -d"
echo ""
echo "4. Configure DNS to point $DOMAIN to this server's IP"
echo ""
echo -e "${YELLOW}Generated Admin API Key:${NC}"
echo "   $ADMIN_KEY"
echo "   (Also saved in /etc/rulecraft/.env)"
echo ""
echo -e "${GREEN}Security Configuration:${NC}"
echo "  - SSH: Key-only authentication"
echo "  - Firewall: Ports 22, 80, 443 only"
echo "  - fail2ban: Enabled for SSH"
echo "  - Updates: Automatic security updates enabled"
echo ""
