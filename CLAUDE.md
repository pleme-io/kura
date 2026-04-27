# Kura (鞍) — the Rust+Lisp Agentic Coding Harness

> **★★★ CSE / Knowable Construction.** This repo operates under
> **Constructive Substrate Engineering** — canonical specification at
> [`pleme-io/theory/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md`](https://github.com/pleme-io/theory/blob/main/CONSTRUCTIVE-SUBSTRATE-ENGINEERING.md).
> The Compounding Directive (operational rules: solve once, load-bearing
> fixes only, idiom-first, models stay current, direction beats velocity)
> is in the org-level pleme-io/CLAUDE.md ★★★ section. Read both before
> non-trivial changes. Multi-provider AI agent gateway; every domain
> (Provider/Session/Tool/Plugin/Keymap/Agent) is a `#[derive(TataraDomain)]`
> primitive — settings travel via shikumi YAML, agents are authored as Lisp.

> 鞍 (kura): the saddle. AI agents are powerful beasts. Kura is how you ride them.

## Architecture

```
kura (binary) — thin CLI, clap, tokio
  ├── kura-core — domain types via TataraDomain (ProviderSpec, SessionSpec, ToolSpec, PluginSpec, KeymapSpec, AgentSpec)
  ├── kura-provider — AI provider adapters (Zen first-class, OpenAI-compat streaming, multi-provider routing)
  ├── kura-agent — Think→Tool→Observe loop, session management, context windows, Lisp-authored agents
  ├── kura-tool — shell (guardrail-gated), file ops, MCP bridge, search, Git operations
  ├── kura-tui — crossterm renderer, Nord palette, Kitty graphics, Ghostty-optimized, Lisp-themeable
  └── kura-ghostty — Kitty Graphics Protocol, Kitty Keyboard Protocol, terminal detection, synced output
```

## Key Integrations

| Library | Purpose |
|---------|---------|
| `tatara-lisp` + `tatara-lisp-derive` | `#[derive(TataraDomain)]` — all domain types authorable as Lisp |
| `shikumi` | Config discovery/loading (YAML/TOML/Lisp/Nix), hot-reload, secrets |
| `irodori` | Nord color palette, base16 semantic aliases |
| `tatara-ui` | CLI UX: sigils, UiEvent stream, BLAKE3 run hashes |
| `kaname` | MCP scaffold |
| `mojiban` | Rich text / markdown rendering |
| `hasami` | Diff operations |
| `crossterm` | Terminal backend (raw mode, events, styled output) |

## Ghostty-Native Features

- **Kitty Graphics Protocol**: Render images inline (PNG/RGBA), z-index layering, unicode placeholders
- **Kitty Keyboard Protocol**: Full key event fidelity (press/repeat/release, all modifiers)
- **Synchronized Output (DECSET 2026)**: Atomic frame rendering, zero flicker
- **OSC 8 Hyperlinks**: Clickable links in conversation
- **OSC 52 Clipboard**: Copy-to-clipboard from TUI
- **Detection**: `TERM=xterm-ghostty`, `GHOSTTY_RESOURCES_DIR`, display-p3 colorspace

## Lisp Authoring Surface

Every domain type derives `TataraDomain`:

```lisp
;; providers — Zen is first-class
(defprovider :name "zen" :kind Zen :api-key-env "OPENCODE_API_KEY" :model "opencode/claude-sonnet-4-20250514" :priority 10)
(defzen :model "opencode/claude-opus-4-6" :max-tokens 16384)

;; agents — authored in lisp
(defagent coder :kind Coder :provider "zen" :max-turns 50 :thinking-budget "high")
(defagent reviewer :kind Reviewer :provider "zen" :auto-approve #t)

;; tools — extend the tool surface
(deftool kubernetes :kind Mcp :mcp-server "kubernetes" :description "K8s operations via MCP")
(deftool terraform :kind Mcp :mcp-server "terraform" :guardrail #t)

;; plugins — hooks, transformers, MCP bridges
(defplugin guardrail :kind Hook :phase PreToolUse :command "guardrail check")
(defplugin nordify :kind Transformer :lisp-transform "(rewrite-theme :style nord)")

;; keymaps — vim-inspired, Ghostty-aware
(defkeymap default
  :bindings ((:key "enter" :action SubmitInput)
             (:key "esc" :action CancelInput)
             (:key "ctrl-up" :action CycleProvider)
             (:key "t" :action ToggleThinking :mode "conversation")
             (:key "q" :action Quit :mode "conversation")))
```

## Nix / Blackmatter Integration

- **flake.nix**: `rust-tool-release-flake.nix` — 4-target GitHub release + HM module
- **blackmatter-kura**: Home-manager module under `blackmatter.components.kura`
- **anvil**: Self-registers MCP servers via `hmHelpers.mkAnvilRegistration`
- **guardrail**: Pre-tool-use hook via blackmatter-claude pattern
- **Config**: `~/.config/kura/kura.yaml` via shikumi (YAML/TOML/Lisp/Nix auto-detect)
- **Lisp init**: `~/.config/kura/init.lisp` loaded at startup

## Commands

| Command | What |
|---------|------|
| `nix run .#` | Launch TUI |
| `nix run .#release` | GitHub release (4 targets) |
| `nix run .#bump` | Version bump |
| `nix run .#check-all` | fmt + clippy + test |
| `kura launch` | Launch TUI |
| `kura sessions` | List sessions |
| `kura config show` | Show current config |
| `kura check` | Domain registry coherence check |

## TUI Keybindings

| Key | Action | Mode |
|-----|--------|------|
| `i` / `Tab` | Focus input | conversation |
| `Enter` | Submit input | input |
| `Esc` | Unfocus / cancel | any |
| `j` / `k` / `↑` / `↓` | Scroll conversation | conversation |
| `t` | Toggle thinking display | conversation |
| `o` | Toggle tool output display | conversation |
| `n` | New session | conversation |
| `Ctrl+Up` | Cycle provider | input |
| `y` / `n` | Approve/deny tool | approval |
| `q` | Quit | conversation |

## Conventions

- Rust edition 2024, minimum 1.85.0, MIT license
- Release profile: `codegen-units = 1`, `lto = true`, `opt-level = "z"`, `strip = true`
- `#[lints.clippy] pedantic = "warn"`
- All domain types: `#[derive(TataraDomain)]` + `serde::Serialize + Deserialize`
- Content addressing: BLAKE3 of canonical JSON via `KuraDomain::content_id()`
- Config: shikumi `ConfigStore<KuraConfig>` with hot-reload
- Tracing to stderr (stdout is the terminal surface)
