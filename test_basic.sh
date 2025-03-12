#!/bin/bash

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Use absolute paths for testing
HOME_DIR="$HOME"
TEST_DIR="$HOME_DIR/diem_test"
CONFIG_DIR="$TEST_DIR/.config/diem"
GOINFRE_DIR="$TEST_DIR/goinfre"
SGOINFRE_DIR="$TEST_DIR/sgoinfre"
INSTALL_DIR="$TEST_DIR/install"
ARTIFACTORY_DIR="$TEST_DIR/artifactory"

echo -e "${BLUE}Setting up real test environment with absolute paths...${NC}"

# Clean up any existing test directory
if [ -d "$TEST_DIR" ]; then
    rm -rf "$TEST_DIR"
fi

# Create the test directories with proper permissions
mkdir -p "$TEST_DIR"
mkdir -p "$CONFIG_DIR"
mkdir -p "$GOINFRE_DIR"
mkdir -p "$SGOINFRE_DIR"
mkdir -p "$INSTALL_DIR"
mkdir -p "$ARTIFACTORY_DIR"

echo -e "${GREEN}Test directories created at $TEST_DIR${NC}"

# Create a sample artifactory file
cat > "$ARTIFACTORY_DIR/sample.toml" << EOF
name = "Test Artifactory"
description = "Test artifactory for diem"
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

echo -e "${GREEN}Sample artifactory file created at $ARTIFACTORY_DIR/sample.toml${NC}"

# Build the diem CLI
echo -e "${BLUE}Building diem CLI...${NC}"
cargo build
echo -e "${GREEN}Diem CLI built successfully${NC}"

# Run a basic end-to-end test focusing on config commands
echo -e "${BLUE}Running basic end-to-end test...${NC}"

# Force the use of our test directory for config
export XDG_CONFIG_HOME="$TEST_DIR/.config"

# 1. Set the goinfre directory
echo -e "${YELLOW}Setting goinfre directory...${NC}"
./target/debug/diem config set-goinfre "$GOINFRE_DIR"
echo

# 2. Set the sgoinfre directory
echo -e "${YELLOW}Setting sgoinfre directory...${NC}"
./target/debug/diem config set-sgoinfre "$SGOINFRE_DIR"
echo

# 3. Show the config and verify the directories are set
echo -e "${YELLOW}Showing configuration to verify settings...${NC}"
./target/debug/diem config show
echo

# 4. Try to list providers
echo -e "${YELLOW}Listing providers (should be empty)...${NC}"
./target/debug/diem providers list
echo

# 5. Try to list artifactories
echo -e "${YELLOW}Listing artifactories (should be empty)...${NC}"
./target/debug/diem artifactory list
echo

# 6. Subscribe to an artifactory
echo -e "${YELLOW}Subscribing to sample artifactory...${NC}"
./target/debug/diem artifactory subscribe --name "Test" --source "$ARTIFACTORY_DIR/sample.toml"
echo

# 7. List artifactories again (should show the one we added)
echo -e "${YELLOW}Listing artifactories (should show 'Test')...${NC}"
./target/debug/diem artifactory list
echo

# 8. List available apps
echo -e "${YELLOW}Listing available apps from the test artifactory...${NC}"
./target/debug/diem list
echo

echo -e "${GREEN}Basic end-to-end test completed!${NC}"
echo -e "${BLUE}--------------------------------------------------${NC}"
echo -e "${BLUE}Test Directory: $TEST_DIR${NC}"
echo -e "${BLUE}--------------------------------------------------${NC}"

# Clean up
echo -e "${BLUE}Do you want to clean up the test environment? (y/n)${NC}"
read -r answer
if [ "$answer" = "y" ]; then
    rm -rf "$TEST_DIR"
    echo -e "${GREEN}Test environment cleaned up${NC}"
else
    echo -e "${YELLOW}Test environment left at $TEST_DIR${NC}"
fi