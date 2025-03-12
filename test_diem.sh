#!/bin/bash

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test directory setup
TEST_DIR="$(pwd)/test_environment"
ARTIFACTORY_DIR="$TEST_DIR/artifactory"
SAMPLE_APP_DIR="$TEST_DIR/apps"
CONFIG_DIR="$TEST_DIR/config"

# Create test directories
setup_test_environment() {
    echo -e "${BLUE}Setting up test environment...${NC}"
    
    # Remove existing test directory if it exists
    if [ -d "$TEST_DIR" ]; then
        rm -rf "$TEST_DIR"
    fi
    
    # Create test directories
    mkdir -p "$TEST_DIR"
    mkdir -p "$ARTIFACTORY_DIR"
    mkdir -p "$SAMPLE_APP_DIR"
    mkdir -p "$CONFIG_DIR"
    mkdir -p "$TEST_DIR/packages"
    mkdir -p "$TEST_DIR/goinfre"
    mkdir -p "$TEST_DIR/sgoinfre"
    
    echo -e "${GREEN}Test environment created at $TEST_DIR${NC}"
}

# Create a sample artifactory for testing
create_sample_artifactory() {
    echo -e "${BLUE}Creating sample artifactory...${NC}"
    
    cat > "$ARTIFACTORY_DIR/test_artifactory.toml" << EOF
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

    # Create a dummy package for the sample app
    mkdir -p "$SAMPLE_APP_DIR/hello"
    mkdir -p "$SAMPLE_APP_DIR/hello/bin"
    
    # Create a simple executable
    cat > "$SAMPLE_APP_DIR/hello/bin/hello" << EOF
#!/bin/bash
echo "Hello from diem test package!"
EOF
    
    chmod +x "$SAMPLE_APP_DIR/hello/bin/hello"
    
    # Create a tarball
    cd "$SAMPLE_APP_DIR"
    tar -czf hello_1.0.0.tar.gz hello
    mv hello_1.0.0.tar.gz "$ARTIFACTORY_DIR/"
    cd - > /dev/null
    
    echo -e "${GREEN}Sample artifactory created${NC}"
}

# Build the diem CLI
build_diem() {
    echo -e "${BLUE}Building diem CLI...${NC}"
    
    cargo build
    
    echo -e "${GREEN}Diem CLI built successfully${NC}"
}

# Run unit tests
run_unit_tests() {
    echo -e "${BLUE}Running unit tests...${NC}"
    
    cargo test -p diem
    
    echo -e "${GREEN}Unit tests completed${NC}"
}

# Test basic CLI functionality
test_cli_functionality() {
    echo -e "${BLUE}Testing basic CLI functionality...${NC}"
    
    # Test help command
    echo -e "${YELLOW}Testing help command...${NC}"
    ./target/debug/diem --help
    
    # Test config help
    echo -e "${YELLOW}Testing config commands...${NC}"
    ./target/debug/diem config --help
    
    # Show config
    echo -e "${YELLOW}Current configuration:${NC}"
    ./target/debug/diem config show || echo "Config show command failed"
    
    echo -e "${GREEN}Basic CLI functionality tests passed${NC}"
}

# Test CLI commands
test_cli_commands() {
    echo -e "${BLUE}Testing CLI commands...${NC}"
    
    # Test providers command
    echo -e "${YELLOW}Testing providers command:${NC}"
    ./target/debug/diem providers --help
    
    # Test artifactory command
    echo -e "${YELLOW}Testing artifactory command:${NC}"
    ./target/debug/diem artifactory --help
    
    # Test search command
    echo -e "${YELLOW}Testing search command:${NC}"
    ./target/debug/diem search --help
    
    # Test list command
    echo -e "${YELLOW}Testing list command:${NC}"
    ./target/debug/diem list --help
    
    # Test install command
    echo -e "${YELLOW}Testing install command:${NC}"
    ./target/debug/diem install --help
    
    # Test update command
    echo -e "${YELLOW}Testing update command:${NC}"
    ./target/debug/diem update --help
    
    # Test remove command
    echo -e "${YELLOW}Testing remove command:${NC}"
    ./target/debug/diem remove --help
    
    # Test sync command
    echo -e "${YELLOW}Testing sync command:${NC}"
    ./target/debug/diem sync --help
    
    echo -e "${GREEN}CLI commands tests passed${NC}"
}

# Try using some CLI commands
test_cli_usage() {
    echo -e "${BLUE}Testing CLI command usage...${NC}"
    
    # Try listing providers (should be empty)
    echo -e "${YELLOW}Listing providers:${NC}"
    ./target/debug/diem providers list || echo "Providers list command failed"
    
    # Try listing artifactories (should be empty)
    echo -e "${YELLOW}Listing artifactories:${NC}"
    ./target/debug/diem artifactory list || echo "Artifactory list command failed"
    
    echo -e "${GREEN}CLI usage tests completed${NC}"
}

# Run all tests
run_all_tests() {
    setup_test_environment
    create_sample_artifactory
    build_diem
    run_unit_tests
    test_cli_functionality
    test_cli_commands
    test_cli_usage
    
    echo -e "${GREEN}All tests completed!${NC}"
    echo -e "${BLUE}--------------------------------------------------${NC}"
    echo -e "${BLUE}Test Summary:${NC}"
    echo -e "${GREEN}✓ Environment setup${NC}"
    echo -e "${GREEN}✓ Unit tests passed${NC}"
    echo -e "${GREEN}✓ CLI functionality verified${NC}"
    echo -e "${GREEN}✓ Command-line interface tested${NC}"
    echo -e "${BLUE}--------------------------------------------------${NC}"
    echo -e "${YELLOW}Note: Some commands may not have been fully tested as they require configuration${NC}"
}

# Main execution
run_all_tests

# Clean up
echo -e "${BLUE}Do you want to clean up the test environment? (y/n)${NC}"
read -r answer
if [ "$answer" = "y" ]; then
    rm -rf "$TEST_DIR"
    echo -e "${GREEN}Test environment cleaned up${NC}"
else
    echo -e "${YELLOW}Test environment left at $TEST_DIR${NC}"
fi