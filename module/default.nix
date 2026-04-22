{ hmHelpers, mcpHelpers }:

{ lib, config, pkgs, ... }:
with lib;
let
  cfg = config.blackmatter.components.kura;

  anvilConfig =
    if builtins.hasAttr "anvil" (config.blackmatter.components or {})
    then config.blackmatter.components.anvil
    else { enable = false; generatedServers = {}; };
  anvilServers = if anvilConfig.enable then anvilConfig.generatedServers else {};

in {
  options.blackmatter.components.kura = {
    enable = mkEnableOption "Kura — the Rust+Lisp agentic coding harness";

    package = mkOption {
      type = types.package;
      default = pkgs.kura;
      description = "Kura package.";
    };

    defaultProvider = mkOption {
      type = types.str;
      default = "zen";
      description = "Default AI provider.";
    };

    defaultModel = mkOption {
      type = types.str;
      default = "opencode/claude-sonnet-4-20250514";
      description = "Default model ID.";
    };

    ghostty.optimize = mkOption {
      type = types.bool;
      default = true;
      description = "Enable Ghostty-specific optimizations.";
    };

    mcp.extraServers = mkOption {
      type = types.attrs;
      default = {};
      description = "Additional MCP servers beyond anvil defaults.";
    };

    guardrail.enable = mkOption {
      type = types.bool;
      default = true;
      description = "Enable guardrail safety checks.";
    };

    lisp.initScript = mkOption {
      type = types.nullOr types.path;
      default = null;
      description = "Path to init.lisp for startup declarations.";
    };

    lisp.plugins = mkOption {
      type = types.listOf types.path;
      default = [];
      description = "Paths to .lisp plugin files.";
    };

    settings = mkOption {
      type = types.attrs;
      default = {};
      description = "Extra settings.";
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      home.packages = [ cfg.package ];

      xdg.configFile."kura/kura.yaml".text = generators.toYAML {} ({
        defaultProvider = cfg.defaultProvider;
        defaultModel = cfg.defaultModel;
        ghostty = { optimize = cfg.ghostty.optimize; };
        guardrail = { enable = cfg.guardrail.enable; };
      } // cfg.settings);

      blackmatter.components.anvil.agents.kura = {
        enable = true;
        configPath = ".config/kura/mcp.json";
        configFormat = "mcpjson";
      };
    }

    (mkIf (cfg.lisp.initScript != null) {
      xdg.configFile."kura/init.lisp".source = cfg.lisp.initScript;
    })

    (mkIf (builtins.length cfg.lisp.plugins > 0) {
      xdg.configFile."kura/plugins" = {
        source = pkgs.linkFarm "kura-plugins"
          (imap0 (i: path: { name = "plugin-${toString i}.lisp"; inherit path; }) cfg.lisp.plugins);
      };
    })
  ]);
}
