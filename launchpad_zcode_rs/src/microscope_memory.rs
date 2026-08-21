use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MemoryEntry {
    pub timestamp: u64,
    pub speaker: String,
    pub content: String,
    pub tag: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MicroscopeMemoryStore {
    pub memories: Vec<MemoryEntry>,
    pub user_name: String,
    pub agent_name: String,
}

impl MicroscopeMemoryStore {
    pub fn new() -> Self {
        Self {
            memories: Vec::new(),
            user_name: "Máté Róbert".to_string(),
            agent_name: "Jules".to_string(),
        }
    }

    pub fn load_bincode<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        if !path.as_ref().exists() {
            return Ok(Self::new());
        }
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        let store: Self = bincode::deserialize(&buffer)?;
        Ok(store)
    }

    pub fn save_bincode<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let encoded: Vec<u8> = bincode::serialize(self)?;
        let mut file = File::create(path)?;
        file.write_all(&encoded)?;
        Ok(())
    }

    pub fn add_memory(&mut self, speaker: &str, content: &str, tag: &str) {
        let entry = MemoryEntry {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            speaker: speaker.to_string(),
            content: content.to_string(),
            tag: tag.to_string(),
        };
        self.memories.push(entry);
    }
}
