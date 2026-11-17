{
  description = "flake for andrust markov chain";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";

    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";

    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.treefmt-nix.flakeModule ];

      systems = inputs.nixpkgs.lib.systems.flakeExposed;

      perSystem =
        {
          config,
          pkgs,
          self',
          ...
        }:
        {
          devShells = {
            default = pkgs.mkShell {
              inputsFrom = [
                config.treefmt.build.devShell
                self'.packages.default
              ];
              packages = [ pkgs.clippy ];
            };

            with-lsp = pkgs.mkShell {
              inputsFrom = [ self'.devShells.default ];
              packages = with pkgs; [
                nixd
                rust-analyzer
              ];
            };
          };

          packages = {
            default = self'.packages.andrust;
            andrust = (pkgs.callPackage inputs.naersk { }).buildPackage ./.;
          };

          treefmt.programs = {
            deadnix.enable = true;
            flake-edit.enable = true;
            nixf-diagnose.enable = true;
            nixfmt = {
              enable = true;
              package = pkgs.nixfmt-rs;
            };
            statix.enable = true;

            ruff-check.enable = true;
            rustfmt = {
              enable = true;
              edition = (fromTOML (builtins.readFile ./Cargo.toml)).package.edition;
            };
          };
        };
    };
}
