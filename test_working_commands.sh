#!/bin/bash

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

DIEM_CMD="./target/debug/diem"
TEST_DIR="$HOME/diem_test"
ARTIFACTORY_DIR="$TEST_DIR/artifactory"

echo -e "${BLUE}Testing working Diem CLI commands...${NC}"

# Create directory for artifactory if it doesn't exist
mkdir -p "$ARTIFACTORY_DIR"

# Create a sample artifactory
echo -e "${YELLOW}Creating sample artifactory file...${NC}"
cat > "$ARTIFACTORY_DIR/test_art.toml" << EOF
name = "Test Artifactory"
description = "Artifactory for testing diem"
public = true
maintainer = "Test User"
artifactory_handler_version = 1

[[apps]]
name = "hello"
version = "1.0.0"
license = "MIT"
description = "A simple hello world app"
app_handler_version = 0

[[apps.commands]]
command = "hello"
path = "bin/hello"

[[apps.packages]]
name = "hello"
version = "1.0.0"
sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
license = "MIT"
source = "hello_1.0.0.tar.gz"
dependencies = []
package_handler_version = 0
EOF

echo -e "${GREEN}Sample artifactory created at $ARTIFACTORY_DIR/test_art.toml${NC}"

# Test config show command
echo -e "\n${YELLOW}1. Showing configuration...${NC}"
$DIEM_CMD config show

# Test listing providers (should be empty)
echo -e "\n${YELLOW}2. Listing providers...${NC}"
$DIEM_CMD providers list

# Subscribe to the artifactory (fixed command)
echo -e "\n${YELLOW}3. Subscribing to an artifactory (may show 'already subscribed')...${NC}"
$DIEM_CMD artifactory unsubscribe "Test" || echo "No previous subscription to remove"
$DIEM_CMD artifactory subscribe "Test" "$ARTIFACTORY_DIR/test_art.toml" || echo "Could not subscribe to artifactory"

# List subscribed artifactories
echo -e "\n${YELLOW}4. Listing artifactories...${NC}"
$DIEM_CMD artifactory list

# List available apps
echo -e "\n${YELLOW}5. Listing available apps...${NC}"
$DIEM_CMD list

echo -e "\n${GREEN}Testing completed!${NC}"