use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};
use crate::persistence::save_system::{SaveResult, SaveError, SaveSlot, SaveMetadata};

/// Save rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRotationConfig {
    pub max_saves_per_slot: usize,
    pub max_total_saves: usize,
    pub max_age_days: u64,
    pub compress_old_saves: bool,
    pub backup_before_rotation: bool,
    pub rotation_strategy: RotationStrategy,
}

impl Default for SaveRotationConfig {
    fn default() -> Self {
        SaveRotationConfig {
            max_saves_per_slot: 5,
            max_total_saves: 50,
            max_age_days: 30,
            compress_old_saves: true,
            backup_before_rotation: true,
            rotation_strategy: RotationStrategy::TimeBasedWithCount,
        }
    }
}

/// Save rotation strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationStrategy {
    /// Keep only the most recent N saves per slot
    CountBased,
    /// Keep saves newer than N days
    TimeBased,
    /// Combination of time and count limits
    TimeBasedWithCount,
    /// Keep saves based on importance (manual saves kept longer)
    ImportanceBased,
}

/// Save file information for rotation
#[derive(Debug, Clone)]
pub struct SaveFileInfo {
    pub path: PathBuf,
    pub slot_id: u32,
    pub metadata: SaveMetadata,
    pub file_size: u64,
    pub is_autosave: bool,
    pub is_manual: bool,
    pub age_days: u64,
    pub importance_score: u32,
}

/// Save rotation system
pub struct SaveRotationSystem {
    config: SaveRotationConfig,
    save_directory: PathBuf,
    backup_directory: PathBuf,
}

impl SaveRotationSystem {
    pub fn new<P: AsRef<Path>>(
        save_directory: P,
        config: SaveRotationConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let save_dir = save_directory.as_ref().to_path_buf();
        let backup_dir = save_dir.join("backups");
        
        // Create directories
        fs::create_dir_all(&save_dir)?;
        fs::create_dir_all(&backup_dir)?;
        
        Ok(SaveRotationSystem {
            config,
            save_directory: save_dir,
            backup_directory: backup_dir,
        })
    }

    /// Perform save rotation based on configuration
    pub fn rotate_saves(&self) -> SaveResult<RotationResult> {
        let save_files = self.scan_save_files()?;
        let mut result = RotationResult::new();

        match self.config.rotation_strategy {
            RotationStrategy::CountBased => {
                result = self.rotate_by_count(save_files)?;
            },
            RotationStrategy::TimeBased => {
                result = self.rotate_by_time(save_files)?;
            },
            RotationStrategy::TimeBasedWithCount => {
                let mut temp_result = self.rotate_by_time(save_files.clone())?;
                result.merge(temp_result);
                
                // Apply count-based rotation to remaining files
                let remaining_files = self.scan_save_files()?;
                temp_result = self.rotate_by_count(remaining_files)?;
                result.merge(temp_result);
            },
            RotationStrategy::ImportanceBased => {
                result = self.rotate_by_importance(save_files)?;
            },
        }

        // Compress old saves if configured
        if self.config.compress_old_saves {
            self.compress_old_saves(&mut result)?;
        }

        Ok(result)
    }

    /// Scan save directory for save files
    fn scan_save_files(&self) -> SaveResult<Vec<SaveFileInfo>> {
        let mut save_files = Vec::new();
        
        let entries = fs::read_dir(&self.save_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if let Some(extension) = path.extension() {
                if extension == "sav" || extension == "save" {
                    if let Ok(save_info) = self.analyze_save_file(&path) {
                        save_files.push(save_info);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        save_files.sort_by(|a, b| {
            b.metadata.last_modified.cmp(&a.metadata.last_modified)
        });

        Ok(save_files)
    }

    /// Analyze a save file to extract information
    fn analyze_save_file(&self, path: &Path) -> SaveResult<SaveFileInfo> {
        // Get file metadata
        let file_metadata = fs::metadata(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        let file_size = file_metadata.len();
        
        // Calculate age
        let modified_time = file_metadata.modified()
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        let age_days = SystemTime::now()
            .duration_since(modified_time)
            .unwrap_or_default()
            .as_secs() / (24 * 60 * 60);

        // Try to load save metadata (simplified - in real implementation would parse save file)
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        // Extract slot ID from filename (assuming format like "slot_0.sav")
        let slot_id = self.extract_slot_id_from_filename(filename);
        
        // Determine if it's an autosave
        let is_autosave = filename.contains("autosave") || filename.contains("auto");
        let is_manual = !is_autosave;
        
        // Create placeholder metadata (in real implementation, load from save file)
        let metadata = SaveMetadata::new(
            format!("Save {}", slot_id),
            "Player".to_string(),
        );
        
        // Calculate importance score
        let importance_score = self.calculate_importance_score(
            is_manual,
            is_autosave,
            age_days,
            &metadata,
        );

        Ok(SaveFileInfo {
            path: path.to_path_buf(),
            slot_id,
            metadata,
            file_size,
            is_autosave,
            is_manual,
            age_days,
            importance_score,
        })
    }

    /// Extract slot ID from filename
    fn extract_slot_id_from_filename(&self, filename: &str) -> u32 {
        // Simple extraction - in real implementation would be more robust
        if let Some(start) = filename.find("slot_") {
            let after_slot = &filename[start + 5..];
            if let Some(end) = after_slot.find('.') {
                let slot_str = &after_slot[..end];
                return slot_str.parse().unwrap_or(0);
            }
        }
        0
    }

    /// Calculate importance score for a save file
    fn calculate_importance_score(
        &self,
        is_manual: bool,
        is_autosave: bool,
        age_days: u64,
        metadata: &SaveMetadata,
    ) -> u32 {
        let mut score = 0;
        
        // Manual saves are more important
        if is_manual {
            score += 100;
        }
        
        // Recent saves are more important
        if age_days < 1 {
            score += 50;
        } else if age_days < 7 {
            score += 30;
        } else if age_days < 30 {
            score += 10;
        }
        
        // Higher level characters are more important
        score += metadata.character_level as u32;
        
        // Longer playtime is more important
        score += (metadata.playtime_seconds / 3600) as u32; // Hours of playtime
        
        score
    }

    /// Rotate saves by count
    fn rotate_by_count(&self, save_files: Vec<SaveFileInfo>) -> SaveResult<RotationResult> {
        let mut result = RotationResult::new();
        
        // Group saves by slot
        let mut saves_by_slot: std::collections::HashMap<u32, Vec<SaveFileInfo>> = 
            std::collections::HashMap::new();
        
        for save_file in save_files {
            saves_by_slot.entry(save_file.slot_id)
                .or_insert_with(Vec::new)
                .push(save_file);
        }
        
        // Process each slot
        for (slot_id, mut slot_saves) in saves_by_slot {
            // Sort by timestamp (newest first)
            slot_saves.sort_by(|a, b| {
                b.metadata.last_modified.cmp(&a.metadata.last_modified)
            });
            
            // Keep only the most recent saves
            if slot_saves.len() > self.config.max_saves_per_slot {
                let to_remove = slot_saves.split_off(self.config.max_saves_per_slot);
                
                for save_file in to_remove {
                    self.remove_save_file(&save_file, &mut result)?;
                }
            }
        }
        
        Ok(result)
    }

    /// Rotate saves by time
    fn rotate_by_time(&self, save_files: Vec<SaveFileInfo>) -> SaveResult<RotationResult> {
        let mut result = RotationResult::new();
        
        for save_file in save_files {
            if save_file.age_days > self.config.max_age_days {
                self.remove_save_file(&save_file, &mut result)?;
            }
        }
        
        Ok(result)
    }

    /// Rotate saves by importance
    fn rotate_by_importance(&self, mut save_files: Vec<SaveFileInfo>) -> SaveResult<RotationResult> {
        let mut result = RotationResult::new();
        
        // Sort by importance score (highest first)
        save_files.sort_by(|a, b| b.importance_score.cmp(&a.importance_score));
        
        // Keep only the most important saves
        if save_files.len() > self.config.max_total_saves {
            let to_remove = save_files.split_off(self.config.max_total_saves);
            
            for save_file in to_remove {
                self.remove_save_file(&save_file, &mut result)?;
            }
        }
        
        Ok(result)
    }

    /// Remove a save file (with optional backup)
    fn remove_save_file(
        &self,
        save_file: &SaveFileInfo,
        result: &mut RotationResult,
    ) -> SaveResult<()> {
        // Create backup if configured
        if self.config.backup_before_rotation {
            let backup_path = self.create_backup(&save_file.path)?;
            result.backed_up_files.push(backup_path);
        }
        
        // Remove the file
        fs::remove_file(&save_file.path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        result.deleted_files.push(save_file.path.clone());
        result.space_freed += save_file.file_size;
        
        Ok(())
    }

    /// Create backup of a save file
    fn create_backup(&self, save_path: &Path) -> SaveResult<PathBuf> {
        let filename = save_path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| SaveError::InvalidSaveFile("Invalid filename".to_string()))?;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let backup_filename = format!("{}_{}.backup", filename, timestamp);
        let backup_path = self.backup_directory.join(backup_filename);
        
        fs::copy(save_path, &backup_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        Ok(backup_path)
    }

    /// Compress old saves
    fn compress_old_saves(&self, result: &mut RotationResult) -> SaveResult<()> {
        // In a real implementation, this would compress save files
        // For now, just mark them as compressed
        result.compressed_files = result.backed_up_files.len();
        Ok(())
    }

    /// Get rotation statistics
    pub fn get_statistics(&self) -> SaveResult<RotationStatistics> {
        let save_files = self.scan_save_files()?;
        
        let total_files = save_files.len();
        let total_size: u64 = save_files.iter().map(|f| f.file_size).sum();
        let autosave_count = save_files.iter().filter(|f| f.is_autosave).count();
        let manual_save_count = save_files.iter().filter(|f| f.is_manual).count();
        
        let oldest_save_age = save_files.iter()
            .map(|f| f.age_days)
            .max()
            .unwrap_or(0);
        
        let newest_save_age = save_files.iter()
            .map(|f| f.age_days)
            .min()
            .unwrap_or(0);

        Ok(RotationStatistics {
            total_save_files: total_files,
            total_size_bytes: total_size,
            autosave_count,
            manual_save_count,
            oldest_save_age_days: oldest_save_age,
            newest_save_age_days: newest_save_age,
            backup_directory_size: self.calculate_backup_directory_size()?,
            config: self.config.clone(),
        })
    }

    /// Calculate backup directory size
    fn calculate_backup_directory_size(&self) -> SaveResult<u64> {
        let mut total_size = 0;
        
        if self.backup_directory.exists() {
            let entries = fs::read_dir(&self.backup_directory)
                .map_err(|e| SaveError::IoError(e.to_string()))?;
            
            for entry in entries {
                let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
                let metadata = entry.metadata()
                    .map_err(|e| SaveError::IoError(e.to_string()))?;
                total_size += metadata.len();
            }
        }
        
        Ok(total_size)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SaveRotationConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &SaveRotationConfig {
        &self.config
    }

    /// Clean up backup directory
    pub fn cleanup_backups(&self, max_age_days: u64) -> SaveResult<usize> {
        let mut deleted_count = 0;
        
        if !self.backup_directory.exists() {
            return Ok(0);
        }
        
        let entries = fs::read_dir(&self.backup_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        let cutoff_time = SystemTime::now() - std::time::Duration::from_secs(max_age_days * 24 * 60 * 60);
        
        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();
            
            let metadata = fs::metadata(&path)
                .map_err(|e| SaveError::IoError(e.to_string()))?;
            
            if let Ok(modified) = metadata.modified() {
                if modified < cutoff_time {
                    fs::remove_file(&path)
                        .map_err(|e| SaveError::IoError(e.to_string()))?;
                    deleted_count += 1;
                }
            }
        }
        
        Ok(deleted_count)
    }
}

/// Result of save rotation operation
#[derive(Debug, Clone)]
pub struct RotationResult {
    pub deleted_files: Vec<PathBuf>,
    pub backed_up_files: Vec<PathBuf>,
    pub compressed_files: usize,
    pub space_freed: u64,
}

impl RotationResult {
    pub fn new() -> Self {
        RotationResult {
            deleted_files: Vec::new(),
            backed_up_files: Vec::new(),
            compressed_files: 0,
            space_freed: 0,
        }
    }

    pub fn merge(&mut self, other: RotationResult) {
        self.deleted_files.extend(other.deleted_files);
        self.backed_up_files.extend(other.backed_up_files);
        self.compressed_files += other.compressed_files;
        self.space_freed += other.space_freed;
    }
}

/// Save rotation statistics
#[derive(Debug, Clone)]
pub struct RotationStatistics {
    pub total_save_files: usize,
    pub total_size_bytes: u64,
    pub autosave_count: usize,
    pub manual_save_count: usize,
    pub oldest_save_age_days: u64,
    pub newest_save_age_days: u64,
    pub backup_directory_size: u64,
    pub config: SaveRotationConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_save_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_save_rotation_system_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveRotationConfig::default();
        
        let system = SaveRotationSystem::new(temp_dir.path(), config);
        assert!(system.is_ok());
    }

    #[test]
    fn test_save_file_scanning() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveRotationConfig::default();
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        // Create test save files
        create_test_save_file(temp_dir.path(), "slot_0.sav", "test save data");
        create_test_save_file(temp_dir.path(), "autosave_1.sav", "autosave data");
        create_test_save_file(temp_dir.path(), "manual_save.sav", "manual save data");
        
        let save_files = system.scan_save_files().unwrap();
        assert_eq!(save_files.len(), 3);
    }

    #[test]
    fn test_slot_id_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveRotationConfig::default();
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        assert_eq!(system.extract_slot_id_from_filename("slot_5.sav"), 5);
        assert_eq!(system.extract_slot_id_from_filename("slot_0.save"), 0);
        assert_eq!(system.extract_slot_id_from_filename("autosave.sav"), 0); // Default
    }

    #[test]
    fn test_importance_score_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveRotationConfig::default();
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        let metadata = SaveMetadata {
            save_name: "Test".to_string(),
            player_name: "Player".to_string(),
            character_level: 10,
            current_depth: 5,
            playtime_seconds: 7200, // 2 hours
            created_at: chrono::Utc::now(),
            last_modified: chrono::Utc::now(),
        };
        
        let manual_score = system.calculate_importance_score(true, false, 0, &metadata);
        let autosave_score = system.calculate_importance_score(false, true, 0, &metadata);
        
        assert!(manual_score > autosave_score); // Manual saves should be more important
    }

    #[test]
    fn test_rotation_by_count() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SaveRotationConfig::default();
        config.max_saves_per_slot = 2;
        
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        // Create multiple save files for the same slot
        for i in 0..5 {
            create_test_save_file(temp_dir.path(), &format!("slot_0_{}.sav", i), "test data");
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
        }
        
        let save_files = system.scan_save_files().unwrap();
        let result = system.rotate_by_count(save_files).unwrap();
        
        assert!(result.deleted_files.len() > 0);
    }

    #[test]
    fn test_rotation_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveRotationConfig::default();
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        // Create test save files
        create_test_save_file(temp_dir.path(), "slot_0.sav", "test save data");
        create_test_save_file(temp_dir.path(), "autosave_1.sav", "autosave data");
        
        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_save_files, 2);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_backup_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SaveRotationConfig::default();
        config.backup_before_rotation = true;
        
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        let save_path = create_test_save_file(temp_dir.path(), "test.sav", "test data");
        let backup_path = system.create_backup(&save_path).unwrap();
        
        assert!(backup_path.exists());
        assert!(backup_path.file_name().unwrap().to_str().unwrap().contains("test.sav"));
    }

    #[test]
    fn test_backup_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveRotationConfig::default();
        let system = SaveRotationSystem::new(temp_dir.path(), config).unwrap();
        
        // Create a backup file
        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();
        create_test_save_file(&backup_dir, "old_backup.backup", "old backup data");
        
        let deleted_count = system.cleanup_backups(0).unwrap(); // Delete all backups
        assert!(deleted_count > 0);
    }
}