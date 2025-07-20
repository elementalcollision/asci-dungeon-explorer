use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::dialogue_system_trait::{DialogueEntry, DialogueContext};

/// Dialogue history entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub entry: DialogueEntry,
    pub context_snapshot: Option<DialogueContext>,
    pub timestamp: u64,
    pub location: String,
    pub tags: Vec<String>,
    pub importance: u8, // 0-100
}

/// Dialogue history for a character
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueHistory {
    pub character_id: String,
    pub entries: VecDeque<HistoryEntry>,
    pub max_entries: usize,
    pub metadata: HashMap<String, String>,
    pub creation_date: u64,
    pub last_updated: u64,
}

impl DialogueHistory {
    /// Create a new dialogue history
    pub fn new(character_id: &str, max_entries: usize) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        DialogueHistory {
            character_id: character_id.to_string(),
            entries: VecDeque::new(),
            max_entries,
            metadata: HashMap::new(),
            creation_date: now,
            last_updated: now,
        }
    }
    
    /// Add an entry to the dialogue history
    pub fn add_entry(
        &mut self,
        entry: DialogueEntry,
        context: Option<&DialogueContext>,
        location: &str,
        tags: Vec<String>,
        importance: u8,
    ) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let history_entry = HistoryEntry {
            entry,
            context_snapshot: context.cloned(),
            timestamp: now,
            location: location.to_string(),
            tags,
            importance,
        };
        
        self.entries.push_back(history_entry);
        self.last_updated = now;
        
        // Trim history if it exceeds max entries
        while self.entries.len() > self.max_entries {
            // Remove least important entries first
            if let Some(min_importance_idx) = self.find_least_important_entry() {
                self.entries.remove(min_importance_idx);
            } else {
                // If no importance data, remove oldest
                self.entries.pop_front();
            }
        }
    }
    
    /// Find the index of the least important entry
    fn find_least_important_entry(&self) -> Option<usize> {
        if self.entries.is_empty() {
            return None;
        }
        
        let mut min_importance = 101; // Higher than max importance
        let mut min_idx = 0;
        
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.importance < min_importance {
                min_importance = entry.importance;
                min_idx = i;
            }
        }
        
        Some(min_idx)
    }
    
    /// Get entries by tag
    pub fn get_entries_by_tag(&self, tag: &str) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|entry| entry.tags.iter().any(|t| t == tag))
            .collect()
    }
    
    /// Get entries by location
    pub fn get_entries_by_location(&self, location: &str) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|entry| entry.location == location)
            .collect()
    }
    
    /// Get entries by importance threshold
    pub fn get_entries_by_importance(&self, min_importance: u8) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|entry| entry.importance >= min_importance)
            .collect()
    }
    
    /// Get entries by time range
    pub fn get_entries_by_time_range(&self, start: u64, end: u64) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .collect()
    }
    
    /// Get the most recent entries
    pub fn get_recent_entries(&self, count: usize) -> Vec<&HistoryEntry> {
        self.entries.iter()
            .rev()
            .take(count)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }
    
    /// Clear the dialogue history
    pub fn clear(&mut self) {
        self.entries.clear();
        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }
    
    /// Save the dialogue history to a file
    pub fn save_to_file(&self, path: &Path) -> io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
    
    /// Load the dialogue history from a file
    pub fn load_from_file(path: &Path) -> io::Result<Self> {
        let json = fs::read_to_string(path)?;
        let history: DialogueHistory = serde_json::from_str(&json)?;
        Ok(history)
    }
    
    /// Summarize the dialogue history
    pub fn summarize(&self) -> String {
        let mut summary = String::new();
        
        if self.entries.is_empty() {
            return "No dialogue history available.".to_string();
        }
        
        // Count entries by speaker
        let mut speaker_counts: HashMap<String, usize> = HashMap::new();
        for entry in &self.entries {
            *speaker_counts.entry(entry.entry.speaker.clone()).or_insert(0) += 1;
        }
        
        // Add speaker statistics
        summary.push_str("Dialogue Summary:\n");
        summary.push_str(&format!("Total entries: {}\n", self.entries.len()));
        summary.push_str("Speakers:\n");
        
        for (speaker, count) in speaker_counts {
            summary.push_str(&format!("- {}: {} entries\n", speaker, count));
        }
        
        // Add most recent conversation snippet
        if let Some(recent) = self.get_recent_entries(3).last() {
            summary.push_str("\nMost recent dialogue:\n");
            summary.push_str(&format!("{}: {}\n", recent.entry.speaker, recent.entry.text));
        }
        
        // Add most important entries
        let important_entries = self.get_entries_by_importance(80);
        if !important_entries.is_empty() {
            summary.push_str("\nKey dialogue moments:\n");
            
            for entry in important_entries.iter().take(3) {
                summary.push_str(&format!("- {}: {}\n", entry.entry.speaker, entry.entry.text));
            }
        }
        
        summary
    }
}

/// Manager for dialogue histories
pub struct DialogueHistoryManager {
    histories: HashMap<String, DialogueHistory>,
    history_directory: PathBuf,
}

impl DialogueHistoryManager {
    /// Create a new dialogue history manager
    pub fn new(history_directory: PathBuf) -> Self {
        DialogueHistoryManager {
            histories: HashMap::new(),
            history_directory,
        }
    }
    
    /// Initialize the dialogue history manager
    pub fn initialize(&mut self) -> io::Result<()> {
        // Create directory if it doesn't exist
        if !self.history_directory.exists() {
            fs::create_dir_all(&self.history_directory)?;
        }
        
        // Load all histories
        self.load_all_histories()?;
        
        Ok(())
    }
    
    /// Load all dialogue histories
    fn load_all_histories(&mut self) -> io::Result<()> {
        if !self.history_directory.exists() {
            return Ok(());
        }
        
        for entry in fs::read_dir(&self.history_directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(character_id) = stem.to_str() {
                        match DialogueHistory::load_from_file(&path) {
                            Ok(history) => {
                                self.histories.insert(character_id.to_string(), history);
                                info!("Loaded dialogue history for {}", character_id);
                            },
                            Err(e) => {
                                warn!("Failed to load dialogue history for {}: {}", character_id, e);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a dialogue history
    pub fn get_history(&self, character_id: &str) -> Option<&DialogueHistory> {
        self.histories.get(character_id)
    }
    
    /// Get a mutable reference to a dialogue history
    pub fn get_history_mut(&mut self, character_id: &str) -> Option<&mut DialogueHistory> {
        self.histories.get_mut(character_id)
    }
    
    /// Create a new dialogue history
    pub fn create_history(&mut self, character_id: &str, max_entries: usize) -> &mut DialogueHistory {
        let history = DialogueHistory::new(character_id, max_entries);
        self.histories.insert(character_id.to_string(), history);
        self.histories.get_mut(character_id).unwrap()
    }
    
    /// Get or create a dialogue history
    pub fn get_or_create_history(&mut self, character_id: &str, max_entries: usize) -> &mut DialogueHistory {
        if !self.histories.contains_key(character_id) {
            self.create_history(character_id, max_entries);
        }
        
        self.histories.get_mut(character_id).unwrap()
    }
    
    /// Add an entry to a dialogue history
    pub fn add_entry(
        &mut self,
        character_id: &str,
        entry: DialogueEntry,
        context: Option<&DialogueContext>,
        location: &str,
        tags: Vec<String>,
        importance: u8,
    ) {
        let history = self.get_or_create_history(character_id, 100);
        history.add_entry(entry, context, location, tags, importance);
        
        // Save the history
        let path = self.history_directory.join(format!("{}.json", character_id));
        if let Err(e) = history.save_to_file(&path) {
            warn!("Failed to save dialogue history for {}: {}", character_id, e);
        }
    }
    
    /// Save all dialogue histories
    pub fn save_all_histories(&self) -> io::Result<()> {
        for (character_id, history) in &self.histories {
            let path = self.history_directory.join(format!("{}.json", character_id));
            history.save_to_file(&path)?;
        }
        
        Ok(())
    }
    
    /// Delete a dialogue history
    pub fn delete_history(&mut self, character_id: &str) -> io::Result<()> {
        self.histories.remove(character_id);
        
        let path = self.history_directory.join(format!("{}.json", character_id));
        if path.exists() {
            fs::remove_file(path)?;
        }
        
        Ok(())
    }
    
    /// Get all character IDs with dialogue histories
    pub fn get_all_character_ids(&self) -> Vec<&str> {
        self.histories.keys().map(|s| s.as_str()).collect()
    }
    
    /// Search for dialogue entries containing a specific text
    pub fn search_dialogue(&self, query: &str) -> Vec<(&str, &HistoryEntry)> {
        let mut results = Vec::new();
        
        for (character_id, history) in &self.histories {
            for entry in &history.entries {
                if entry.entry.text.to_lowercase().contains(&query.to_lowercase()) {
                    results.push((character_id.as_str(), entry));
                }
            }
        }
        
        results
    }
}