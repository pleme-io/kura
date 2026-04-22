use kura_core::ToolDefinition;
use kura_core::ToolKind;
use std::collections::HashMap;

pub struct ToolExecutor {
    tools: HashMap<String, ToolEntry>,
}

struct ToolEntry {
    kind: ToolKind,
    definition: ToolDefinition,
    guardrail: bool,
    command: Option<String>,
    args: Vec<String>,
    mcp_server: Option<String>,
}

impl ToolExecutor {
    pub fn new() -> Self {
        let mut tools = HashMap::new();

        Self::register_builtin(
            &mut tools,
            "bash",
            ToolKind::Shell,
            "Execute shell commands",
            true,
        );
        Self::register_builtin(
            &mut tools,
            "file_read",
            ToolKind::FileRead,
            "Read file contents",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "file_write",
            ToolKind::FileWrite,
            "Write file contents",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "file_edit",
            ToolKind::FileEdit,
            "Edit file with exact string replacement",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "glob",
            ToolKind::Glob,
            "Search files by name pattern",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "grep",
            ToolKind::Grep,
            "Search file contents by regex",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "git",
            ToolKind::Git,
            "Execute git commands",
            true,
        );
        Self::register_builtin(
            &mut tools,
            "web_fetch",
            ToolKind::WebFetch,
            "Fetch content from URL",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "web_search",
            ToolKind::WebSearch,
            "Search the web",
            false,
        );
        Self::register_builtin(
            &mut tools,
            "code_search",
            ToolKind::CodeSearch,
            "Semantic code search",
            false,
        );

        Self { tools }
    }

    fn register_builtin(
        tools: &mut HashMap<String, ToolEntry>,
        name: &str,
        kind: ToolKind,
        description: &str,
        guardrail: bool,
    ) {
        let input_schema = Self::schema_for_kind(&kind);
        tools.insert(
            name.to_string(),
            ToolEntry {
                kind,
                definition: ToolDefinition {
                    name: name.to_string(),
                    description: description.to_string(),
                    input_schema,
                },
                guardrail,
                command: None,
                args: vec![],
                mcp_server: None,
            },
        );
    }

    pub fn register_tool(
        &mut self,
        name: String,
        kind: ToolKind,
        description: String,
        input_schema: serde_json::Value,
        guardrail: bool,
        command: Option<String>,
        args: Vec<String>,
        mcp_server: Option<String>,
    ) {
        self.tools.insert(
            name.clone(),
            ToolEntry {
                kind,
                definition: ToolDefinition {
                    name,
                    description,
                    input_schema,
                },
                guardrail,
                command,
                args,
                mcp_server,
            },
        );
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|e| e.definition.clone()).collect()
    }

    pub async fn execute(&self, name: &str, input: serde_json::Value) -> anyhow::Result<String> {
        let entry = self
            .tools
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("unknown tool: {}", name))?;

        match entry.kind {
            ToolKind::Shell => self.execute_shell(&input, entry.guardrail).await,
            ToolKind::FileRead => self.execute_file_read(&input).await,
            ToolKind::FileWrite => self.execute_file_write(&input).await,
            ToolKind::FileEdit => self.execute_file_edit(&input).await,
            ToolKind::Glob => self.execute_glob(&input).await,
            ToolKind::Grep => self.execute_grep(&input).await,
            ToolKind::Git => self.execute_git(&input).await,
            ToolKind::WebFetch => self.execute_web_fetch(&input).await,
            ToolKind::WebSearch => self.execute_web_search(&input).await,
            ToolKind::CodeSearch => self.execute_code_search(&input).await,
            ToolKind::Mcp => self.execute_mcp(&input, &entry.mcp_server).await,
            ToolKind::Custom => {
                self.execute_custom(&input, entry.command.as_deref(), &entry.args)
                    .await
            }
        }
    }

    fn schema_for_kind(kind: &ToolKind) -> serde_json::Value {
        match kind {
            ToolKind::Shell | ToolKind::Git => serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string", "description": "The command to execute" },
                    "workdir": { "type": "string", "description": "Working directory" }
                },
                "required": ["command"]
            }),
            ToolKind::FileRead => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path" },
                    "offset": { "type": "integer", "description": "Line offset (0-based)" },
                    "limit": { "type": "integer", "description": "Max lines to read" }
                },
                "required": ["path"]
            }),
            ToolKind::FileWrite => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path" },
                    "content": { "type": "string", "description": "Content to write" }
                },
                "required": ["path", "content"]
            }),
            ToolKind::FileEdit => serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Absolute file path" },
                    "old_string": { "type": "string", "description": "Text to find" },
                    "new_string": { "type": "string", "description": "Replacement text" }
                },
                "required": ["path", "old_string", "new_string"]
            }),
            ToolKind::Glob => serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Glob pattern" },
                    "path": { "type": "string", "description": "Directory to search" }
                },
                "required": ["pattern"]
            }),
            ToolKind::Grep => serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": { "type": "string", "description": "Regex pattern" },
                    "path": { "type": "string", "description": "Directory to search" },
                    "include": { "type": "string", "description": "File pattern filter" }
                },
                "required": ["pattern"]
            }),
            ToolKind::Mcp => serde_json::json!({
                "type": "object",
                "properties": {
                    "method": { "type": "string" },
                    "params": { "type": "object" }
                },
                "required": ["method"]
            }),
            _ => serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query" }
                },
                "required": ["query"]
            }),
        }
    }
}
