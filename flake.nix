{
  description = "Kura — the Rust+Lisp agentic coding harness, Ghostty-native";

  nixConfig.allow-import-from-derivation = true;

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    crate2nix.url = "github:nix-community/crate2nix";
    flake-utils.url = "github:numtide/flake-utils";
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    forge = {
      url = "github:pleme-io/forge";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    devenv = {
      url = "github:cachix/devenv";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, crate2nix, flake-utils, substrate, forge, devenv, ... }:
    let
      toolOutputs = (import "${substrate}/lib/rust-workspace-release-flake.nix" {
        inherit nixpkgs crate2nix flake-utils devenv;
      }) {
        toolName = "kura";
        packageName = "kura";
        src = self;
        repo = "pleme-io/kura";
      };

      checkOutputs = (import "${substrate}/lib/rust-workspace-release-flake.nix" {
        inherit nixpkgs crate2nix flake-utils devenv;
      }) {
        toolName = "kura-check";
        packageName = "kura-check";
        src = self;
        repo = "pleme-io/kura";
      };
    in
      toolOutputs // {
        homeManagerModules.default = import ./module {
          hmHelpers = import "${substrate}/lib/hm-service-helpers.nix" { lib = nixpkgs.lib; };
          mcpHelpers = import "${substrate}/lib/hm-mcp-helpers.nix" { lib = nixpkgs.lib; };
        };

        packages = nixpkgs.lib.genAttrs
          [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ]
          (system:
            (toolOutputs.packages.${system} or {})
            // (let co = checkOutputs.packages.${system} or {}; in {
              kura-check = co.kura-check or co.default or null;
            })
          );

        apps = nixpkgs.lib.genAttrs
          [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" ]
          (system:
            (toolOutputs.apps.${system} or {})
          );
      };
}
