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
        with nixpkgs.legacyPackages.${system}; {
          packages = {
            rofi-cliphist = rustPlatform.buildRustPackage {
              pname = "rofi-cliphist";
              version = "0.4.1";

              src = lib.cleanSource ./.;
              cargoLock = {
                lockFile = ./Cargo.lock;
              };

              meta = with lib; {
                description = "Rofi extensions";
                homepage = "https://github.com/szaffarano/rofi-cliphist";
                license = licenses.mit;
                mainProgram = "rofi-cliphist";
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
            inherit (self.checks.${pkgs.system}.pre-commit-check) shellHook;

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
