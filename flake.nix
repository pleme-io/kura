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
      lib = nixpkgs.lib;

      toolOutputs = (import "${substrate}/lib/rust-workspace-release-flake.nix" {
        inherit nixpkgs crate2nix flake-utils devenv;
      }) {
        toolName = "kura";
        packageName = "kura";
        src = self;
        repo = "pleme-io/kura";
        module = {
          description = "Kura — the Rust+Lisp agentic coding harness";
          hmNamespace = "blackmatter.components";

          extraHmOptions = {
            defaultProvider = lib.mkOption {
              type = lib.types.str;
              default = "zen";
              description = "Default AI provider.";
            };

            defaultModel = lib.mkOption {
              type = lib.types.str;
              default = "opencode/claude-sonnet-4-20250514";
              description = "Default model ID.";
            };

            ghostty.optimize = lib.mkOption {
              type = lib.types.bool;
              default = true;
              description = "Enable Ghostty-specific optimizations.";
            };

            mcp.extraServers = lib.mkOption {
              type = lib.types.attrs;
              default = {};
              description = "Additional MCP servers beyond anvil defaults.";
            };

            guardrail.enable = lib.mkOption {
              type = lib.types.bool;
              default = true;
              description = "Enable guardrail safety checks.";
            };

            lisp.initScript = lib.mkOption {
              type = lib.types.nullOr lib.types.path;
              default = null;
              description = "Path to init.lisp for startup declarations.";
            };

            lisp.plugins = lib.mkOption {
              type = lib.types.listOf lib.types.path;
              default = [];
              description = "Paths to .lisp plugin files.";
            };

            settings = lib.mkOption {
              type = lib.types.attrs;
              default = {};
              description = "Extra settings.";
            };
          };

          extraHmConfig = cfg:
            let
              kuraYaml = lib.generators.toYAML {} ({
                defaultProvider = cfg.defaultProvider;
                defaultModel = cfg.defaultModel;
                ghostty = { optimize = cfg.ghostty.optimize; };
                guardrail = { enable = cfg.guardrail.enable; };
              } // cfg.settings);
            in
              lib.mkMerge [
                {
                  xdg.configFile."kura/kura.yaml".text = kuraYaml;

                  blackmatter.components.anvil.agents.kura = {
                    enable = true;
                    configPath = ".config/kura/mcp.json";
                    configFormat = "mcpjson";
                  };
                }

                (lib.mkIf (cfg.lisp.initScript != null) {
                  xdg.configFile."kura/init.lisp".source = cfg.lisp.initScript;
                })

                # Plugins: original used pkgs.linkFarm to expose a single
                # `kura/plugins` directory symlink. extraHmConfig has no
                # access to pkgs, so emit individual home.file entries
                # keyed by index. Functionally equivalent at runtime.
                (lib.mkIf (builtins.length cfg.lisp.plugins > 0) {
                  home.file = lib.listToAttrs (lib.imap0 (i: path: {
                    name = ".config/kura/plugins/plugin-${toString i}.lisp";
                    value = { source = path; };
                  }) cfg.lisp.plugins);
                })
              ];
        };
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
