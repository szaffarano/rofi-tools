{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";

    treefmt-nix = {
      url = "github:numtide/treefmt-nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    treefmt-nix,
    pre-commit-hooks,
  }:
    flake-utils.lib.eachSystem [flake-utils.lib.system.x86_64-linux] (
      system:
        with nixpkgs.legacyPackages.${system}; let
          # The workspace Cargo.lock pulls clipcat-base/client/proto from a git tag
          # (not crates.io), so Nix needs an explicit output hash for that source.
          # All three crates share the same git rev, hence the same hash.
          clipcatGitHash = "sha256-t9uDwpV2bLMNp2mxa8HnSCnk+oSKrEPo2r136M0f/Ww=";
          cargoLock = {
            lockFile = ./Cargo.lock;
            outputHashes = {
              "clipcat-base-0.27.0" = clipcatGitHash;
              "clipcat-client-0.27.0" = clipcatGitHash;
              "clipcat-proto-0.27.0" = clipcatGitHash;
            };
          };
          # Read each binary's version from its own crate manifest so release-please
          # version bumps flow into the flake automatically (no hardcoded version).
          crateVersion = path: (builtins.fromTOML (builtins.readFile path)).package.version;
        in {
          packages = {
            rofi-cliphist = rustPlatform.buildRustPackage {
              pname = "rofi-cliphist";
              version = crateVersion ./crates/rofi-cliphist/Cargo.toml;

              src = lib.cleanSource ./.;
              inherit cargoLock;

              cargoBuildFlags = ["-p" "rofi-cliphist"];
              cargoTestFlags = ["-p" "rofi-cliphist"];

              meta = with lib; {
                description = "Clipboard history tool for Rofi using cliphist";
                homepage = "https://github.com/szaffarano/rofi-tools";
                license = licenses.mit;
                mainProgram = "rofi-cliphist";
                maintainers = [];
              };
            };

            rofi-clipcat = rustPlatform.buildRustPackage {
              pname = "rofi-clipcat";
              version = crateVersion ./crates/rofi-clipcat/Cargo.toml;

              src = lib.cleanSource ./.;
              inherit cargoLock;

              # clipcat-proto compiles .proto files at build time via protoc.
              nativeBuildInputs = [protobuf];

              cargoBuildFlags = ["-p" "rofi-clipcat"];
              cargoTestFlags = ["-p" "rofi-clipcat"];

              meta = with lib; {
                description = "Clipboard history tool for Rofi using clipcat";
                homepage = "https://github.com/szaffarano/rofi-tools";
                license = licenses.mit;
                mainProgram = "rofi-clipcat";
                maintainers = [];
              };
            };
          };

          formatter = let
            treefmt = treefmt-nix.lib.evalModule nixpkgs.legacyPackages.${system} ./treefmt.nix;
          in
            treefmt.config.build.wrapper;

          packages.default = self.packages.${system}.rofi-cliphist;

          checks = {
            pre-commit-check = pre-commit-hooks.lib.${system}.run {
              src = self;
              hooks = {
                alejandra.enable = true;
                deadnix.enable = true;
                end-of-file-fixer.enable = true;
                markdownlint.enable = true;
                mixed-line-endings.enable = true;
                shfmt = {
                  enable = true;
                  entry = "${lib.getExe pkgs.shfmt} -w -l -s -i 2";
                };
                statix.enable = true;
                trim-trailing-whitespace.enable = true;
              };
            };
          };

          devShells.default = mkShell {
            inherit (self.checks.${pkgs.stdenv.hostPlatform.system}.pre-commit-check) shellHook;

            inputsFrom = builtins.attrValues self.packages.${system};
            packages = [
              # rust
              cargo-bloat
              cargo-edit
              cargo-info
              cargo-outdated
              cargo-udeps
              cargo-watch
              clippy
              rust-analyzer

              # rofi
              rofi
              cairo
              pango
              pkg-config

              # clipcat gRPC codegen (clipcat-proto build script)
              protobuf

              # tools
              git
              jq
              curl
            ];

            env = {
              RUST_BACKTRACE = "1";
              RUSTFLAGS = "--cfg rofi_next";
            };
          };
        }
    );
}
