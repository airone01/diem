{
  description = "Flake for diem";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.bashInteractive ];
        buildInputs = with pkgs; [
          # Funny
          onefetch

          # Libs
          openssl
          lld
          stdenv.cc.cc.lib

          # Rust
          pkg-config
          rustc
          cargo
          gcc
          rustfmt
          clippy
        ];
        shellHook = with pkgs; ''
          onefetch --no-art
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.stdenv.cc.cc.lib}/lib:"
        '';
      };
      
      # Testing environment that simulates the 42 school environment
      devShells.test = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.bashInteractive ];
        buildInputs = with pkgs; [
          # Libs
          openssl
          lld
          stdenv.cc.cc.lib

          # Rust
          pkg-config
          rustc
          cargo
          gcc
          
          # Testing tools
          jq
          curl
          zip
          unzip
        ];
        shellHook = with pkgs; ''
          echo "Setting up 42 school test environment..."
          
          # Create 42 school-like directory structure
          export TEST_HOME="$PWD/test_home"
          export TEST_USER="student42"
          mkdir -p $TEST_HOME
          
          # Create /sgoinfre/username and /goinfre/username directories
          export SGOINFRE_ROOT="$PWD/sgoinfre"
          export GOINFRE_ROOT="$PWD/goinfre"
          export SGOINFRE_DIR="$SGOINFRE_ROOT/$TEST_USER"
          export GOINFRE_DIR="$GOINFRE_ROOT/$TEST_USER"
          mkdir -p $SGOINFRE_DIR/diem
          mkdir -p $GOINFRE_DIR/diem
          
          # Create symbolic links in the home directory
          mkdir -p $TEST_HOME
          ln -sf $SGOINFRE_DIR $TEST_HOME/sgoinfre
          ln -sf $GOINFRE_DIR $TEST_HOME/goinfre
          
          # Create shared artifactory directory
          export SHARED_ARTIFACTORY="$SGOINFRE_DIR/shared_artifactory"
          mkdir -p $SHARED_ARTIFACTORY
          
          # Create a basic artifactory for testing
          export TEST_ARTIFACTORY="$SHARED_ARTIFACTORY/test_artifactory.toml"
          
          # Create bin directory for symlinks
          mkdir -p $TEST_HOME/.local/bin
          
          # Override HOME for testing
          export HOME="$TEST_HOME"
          export PATH="$HOME/.local/bin:$PATH"
          
          echo "Test environment ready!"
          echo "sgoinfre dir:  $SGOINFRE_DIR"
          echo "goinfre dir:   $GOINFRE_DIR"
          echo "artifactories: $SHARED_ARTIFACTORY"
          echo ""
          echo "To use the test environment, run: 'cargo run -- <command>'"
          echo "For example: 'cargo run -- config show'"
          
          # Function to create a simple test artifactory
          setup_test_artifactory() {
            cat > $TEST_ARTIFACTORY << EOF
            name = "test_artifactory"
            description = "Test artifactory for diem"
            maintainer = "test_user"
            public = true
            artifactory_handler_version = 1
            
            [[apps]]
            name = "hello"
            version = "1.0.0"
            app_handler_version = 1
            
            [[apps.packages]]
            name = "hello_cli"
            source = "https://github.com/caarlos0/example/releases/download/v0.1.0/example_Linux_x86_64.tar.gz"
            sha256 = "723a1971d85a8f76e99096176591d42e24131a261f5d854edee986b3fd7f55a8"
            version = "0.1.0"
            
            [[apps.commands]]
            command = "hello"
            path = "example"
            EOF
            
            echo "Created test artifactory at $TEST_ARTIFACTORY"
          }
          
          export -f setup_test_artifactory
          echo "Run 'setup_test_artifactory' to create a test artifactory"
          
          # Set LD library path for linking
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.stdenv.cc.cc.lib}/lib:"
        '';
      };
    });
}
