(defzen :model "opencode/claude-opus-4-6"
  :base-url    "https://opencode.ai/zen/v1"
  :api-key-env "OPENCODE_API_KEY"
  :max-tokens  16384
  :temperature 0.0)

(defagent coder-opus
  :kind            Coder
  :provider        "zen"
  :model           "opencode/claude-opus-4-6"
  :max-turns       100
  :thinking-budget "high"
  :auto-approve    #f)
