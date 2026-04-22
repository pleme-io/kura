;; kura checks.lisp — workspace coherence driver
;; Follows tatara/checks.lisp pattern: domain registry integrity + cross-domain refs

;; ── Domain registry: all 8 keywords must resolve ──────────────────────

(defprovider :name "check-zen"
  :kind        Zen
  :api-key-env "OPENCODE_API_KEY"
  :model       "opencode/claude-sonnet-4-20250514"
  :priority    10)

(defagent check-coder
  :kind         Coder
  :provider     "check-zen"
  :max-turns    10
  :auto-approve #f)

(deftool check-bash
  :kind      Shell
  :guardrail #t)

(defplugin check-guardrail
  :kind    Hook
  :phase   PreToolUse
  :command "guardrail check")

(defkeymap check-default
  :bindings ((:key "enter" :action SubmitInput)
             (:key "q" :action Quit)))

(defsession check-session
  :name    "check"
  :provider "check-zen")

(defzen :model "opencode/claude-sonnet-4-20250514"
  :base-url    "https://opencode.ai/zen/v1"
  :api-key-env "OPENCODE_API_KEY")

(defopenai :name "check-openai"
  :base-url    "https://api.openai.com/v1"
  :api-key-env "OPENAI_API_KEY"
  :model       "gpt-4o")
