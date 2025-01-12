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
    });
}
