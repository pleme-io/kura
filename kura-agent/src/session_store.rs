use crate::context::Conversation;
use std::path::PathBuf;

pub struct SessionStore {
    base_dir: PathBuf,
}

impl SessionStore {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn default_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("kura")
            .join("sessions")
    }

    pub async fn save(&self, id: &str, conversation: &Conversation) -> anyhow::Result<()> {
        tokio::fs::create_dir_all(&self.base_dir).await?;
        let path = self.base_dir.join(format!("{}.json", id));
        let json = serde_json::to_string_pretty(&conversation.messages())?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    pub async fn load(&self, id: &str) -> anyhow::Result<Conversation> {
        let path = self.base_dir.join(format!("{}.json", id));
        let json = tokio::fs::read_to_string(path).await?;
        let messages: Vec<crate::context::Message> = serde_json::from_str(&json)?;
        let mut conv = Conversation::new(None);
        for msg in messages {
            conv.add_message(msg.role, msg.content);
        }
        Ok(conv)
    }

    pub async fn list(&self) -> anyhow::Result<Vec<String>> {
        let mut entries = tokio::fs::read_dir(&self.base_dir).await?;
        let mut ids = vec![];
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub async fn delete(&self, id: &str) -> anyhow::Result<()> {
        let path = self.base_dir.join(format!("{}.json", id));
        tokio::fs::remove_file(path).await?;
        Ok(())
    }
}
