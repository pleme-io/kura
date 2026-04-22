(defprovider :name "zen"
  :kind        Zen
  :api-key-env "OPENCODE_API_KEY"
  :model       "opencode/claude-sonnet-4-20250514"
  :priority    10)

(defprovider :name "openai"
  :kind        OpenAi
  :base-url    "https://api.openai.com/v1"
  :api-key-env "OPENAI_API_KEY"
  :model       "gpt-4o"
  :priority    5)

(defprovider :name "ollama"
  :kind        Ollama
  :base-url    "http://localhost:11434/v1"
  :model       "llama3"
  :priority    1)

(defagent coder
  :kind            Coder
  :provider        "zen"
  :max-turns       50
  :thinking-budget "high"
  :auto-approve    #f)

(defagent reviewer
  :kind         Reviewer
  :provider     "zen"
  :auto-approve #t
  :max-turns    10)

(defagent explorer
  :kind     Explorer
  :provider "zen"
  :max-turns 20)

(deftool kubernetes
  :kind        Mcp
  :mcp-server  "kubernetes"
  :description "K8s operations via MCP")

(deftool zoekt
  :kind        Mcp
  :mcp-server  "zoekt"
  :description "Trigram code search via MCP")

(deftool codesearch
  :kind        Mcp
  :mcp-server  "codesearch"
  :description "Semantic code search via MCP")

(defplugin guardrail
  :kind    Hook
  :phase   PreToolUse
  :command "guardrail check")

(defplugin nordify
  :kind            Transformer
  :lisp-transform  "(rewrite-theme :style nord)")

(defkeymap default
  :bindings ((:key "enter"    :action SubmitInput)
             (:key "esc"      :action CancelInput)
             (:key "ctrl-up"  :action CycleProvider)
             (:key "t"        :action ToggleThinking    :mode "conversation")
             (:key "o"        :action ToggleToolOutput   :mode "conversation")
             (:key "n"        :action NewSession         :mode "conversation")
             (:key "q"        :action Quit               :mode "conversation")
             (:key "y"        :action ApproveTool        :mode "approval")
             (:key "n"        :action DenyTool           :mode "approval")))
