use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::value::Value;
use crate::error::PslError;
use std::collections::HashMap;

/// Serializable game state for saving/loading
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    /// Current position in story execution
    pub current_index: usize,
    /// Global variables (character data, containers)
    pub globals: HashMap<String, Value>,
    /// Current chapter
    pub current_chapter: Option<String>,
    /// Current section
    pub current_section: Option<String>,
    /// Game metadata
    pub metadata: GameMetadata,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameMetadata {
    /// Timestamp when save was created
    pub timestamp: String,
    /// Name of the story file
    pub story_file: String,
    /// Player-provided save name
    pub save_name: String,
}

impl GameState {
    /// Create a new game state
    pub fn new(story_file: String, save_name: String) -> Self {
        GameState {
            current_index: 0,
            globals: HashMap::new(),
            current_chapter: None,
            current_section: None,
            metadata: GameMetadata {
                timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                story_file,
                save_name,
            },
        }
    }

    /// Save the game state to a file
    pub fn save(&self, path: &str) -> Result<(), PslError> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| PslError::RuntimeError(format!("Failed to serialize game state: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| PslError::RuntimeError(format!("Failed to write save file: {}", e)))?;

        Ok(())
    }

    /// Load a game state from a file
    pub fn load(path: &str) -> Result<Self, PslError> {
        let json = fs::read_to_string(path)
            .map_err(|e| PslError::RuntimeError(format!("Failed to read save file: {}", e)))?;

        serde_json::from_str(&json)
            .map_err(|e| PslError::RuntimeError(format!("Failed to deserialize game state: {}", e)))
    }

    /// Check if a save file exists
    pub fn exists(path: &str) -> bool {
        Path::new(path).exists()
    }

    /// List all save files in a directory
    pub fn list_saves(dir: &str) -> Result<Vec<GameState>, PslError> {
        let paths = fs::read_dir(dir)
            .map_err(|e| PslError::RuntimeError(format!("Failed to read saves directory: {}", e)))?;

        let mut saves = Vec::new();
        for entry in paths {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "json") {
                    if let Ok(state) = Self::load(path.to_str().unwrap_or("")) {
                        saves.push(state);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.metadata.timestamp.cmp(&a.metadata.timestamp));
        Ok(saves)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_game_state() {
        let state = GameState::new("test.psl".to_string(), "My Save".to_string());
        assert_eq!(state.metadata.story_file, "test.psl");
        assert_eq!(state.metadata.save_name, "My Save");
        assert_eq!(state.current_index, 0);
    }

    #[test]
    fn test_save_and_load() {
        let mut state = GameState::new("test.psl".to_string(), "Test Save".to_string());
        state.current_index = 42;
        state.current_chapter = Some("chapter1".to_string());

        let path = "/tmp/test_save.json";
        state.save(path).expect("Failed to save");

        let loaded = GameState::load(path).expect("Failed to load");
        assert_eq!(loaded.current_index, 42);
        assert_eq!(loaded.current_chapter, Some("chapter1".to_string()));

        // Cleanup
        fs::remove_file(path).ok();
    }
}
