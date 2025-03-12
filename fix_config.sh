#!/bin/bash

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}Creating required test directories...${NC}"

# Create proper test directories
mkdir -p ~/diem_test/packages
mkdir -p ~/diem_test/sgoinfre
mkdir -p ~/diem_test/goinfre

# Update the config file
cat > ~/.config/diem/config.toml << EOF
packages = []
providers = []
install_dir = "/home/elagouch/diem_test/packages"
sgoinfre_dir = "/home/elagouch/diem_test/sgoinfre"
goinfre_dir = "/home/elagouch/diem_test/goinfre"
subscribed_artifactories = []
config_handler_version = 0
EOF

echo -e "${GREEN}Configuration updated to use test directories${NC}"
echo -e "${YELLOW}New config:${NC}"
cat ~/.config/diem/config.toml