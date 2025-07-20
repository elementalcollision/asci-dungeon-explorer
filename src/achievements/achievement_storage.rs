use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};
use crate::achievements::achievement_system::{
    AchievementSaveData, AchievementSystem, UnlockedAchievement, AchievementProgress,
};

/// Achievement storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementStorageConfig {
    pub storage_directory: PathBuf,
    pub backup_enabled: bool,
    pub backup_count: usize,
    pub auto_save: bool,
    pub auto_save_interval_seconds: u64,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

impl Default for AchievementStorageConfig {
    fn default() -> Self {
        AchievementStorageConfig {
            storage_directory: PathBuf::from("./achievements"),
            backup_enabled: true,
            backup_count: 5,
            auto_save: true,
            auto_save_interval_seconds: 300, // 5 minutes
            compression_enabled: false,
            encryption_enabled: false,
        }
    }
}

/// Achievement storage errors
#[derive(Debug, Clone)]
pub enum AchievementStorageError {
    IoError(String),
    SerializationError(String),
    CorruptedData(String),
    BackupError(String),
    ConfigError(String),
}

impl std::fmt::Display for AchievementStorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AchievementStorageError::IoError(msg) => write!(f, "IO Error: {}", msg),
            AchievementStorageError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            AchievementStorageError::CorruptedData(msg) => write!(f, "Corrupted Data: {}", msg),
            AchievementStorageError::BackupError(msg) => write!(f, "Backup Error: {}", msg),
            AchievementStorageError::ConfigError(msg) => write!(f, "Config Error: {}", msg),
        }
    }
}

impl std::error::Error for AchievementStorageError {}

pub type StorageResult<T> = Result<T, AchievementStorageError>;

/// Achievement storage metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementStorageMetadata {
    pub version: String,
    pub created_at: u64,
    pub last_modified: u64,
    pub player_id: String,
    pub total_achievements: usize,
    pub unlocked_count: usize,
    pub total_points: u32,
    pub checksum: String,
}

/// Achievement storage file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementStorageFile {
    pub metadata: AchievementStorageMetadata,
    pub data: AchievementSaveData,
}

/// Persistent achievement storage system
pub struct AchievementStorage {
    config: AchievementStorageConfig,
    last_save: std::time::Instant,
}

impl AchievementStorage {
    pub fn new(config: AchievementStorageConfig) -> StorageResult<Self> {
        // Create storage directory if it doesn't exist
        fs::create_dir_all(&config.storage_directory)
            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

        // Create backup directory if backups are enabled
        if config.backup_enabled {
            let backup_dir = config.storage_directory.join("backups");
            fs::create_dir_all(&backup_dir)
                .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;
        }

        Ok(AchievementStorage {
            config,
            last_save: std::time::Instant::now(),
        })
    }

    /// Save achievement data to storage
    pub fn save_achievements(
        &mut self,
        achievement_system: &AchievementSystem,
        player_id: &str,
    ) -> StorageResult<()> {
        let save_data = achievement_system.export_data();
        let statistics = achievement_system.get_statistics();

        // Create metadata
        let metadata = AchievementStorageMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            player_id: player_id.to_string(),
            total_achievements: statistics.total_achievements,
            unlocked_count: statistics.unlocked_achievements,
            total_points: statistics.earned_points,
            checksum: self.calculate_checksum(&save_data)?,
        };

        let storage_file = AchievementStorageFile {
            metadata,
            data: save_data,
        };

        // Create backup if enabled
        if self.config.backup_enabled {
            self.create_backup(player_id)?;
        }

        // Save to main file
        let file_path = self.get_storage_file_path(player_id);
        self.write_storage_file(&file_path, &storage_file)?;

        self.last_save = std::time::Instant::now();
        Ok(())
    }

    /// Load achievement data from storage
    pub fn load_achievements(
        &self,
        achievement_system: &mut AchievementSystem,
        player_id: &str,
    ) -> StorageResult<()> {
        let file_path = self.get_storage_file_path(player_id);
        
        if !file_path.exists() {
            // No save file exists, start with empty achievements
            return Ok(());
        }

        let storage_file = self.read_storage_file(&file_path)?;

        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&storage_file.data)?;
        if calculated_checksum != storage_file.metadata.checksum {
            return Err(AchievementStorageError::CorruptedData(
                "Checksum mismatch - data may be corrupted".to_string()
            ));
        }

        // Import data into achievement system
        achievement_system.import_data(storage_file.data);

        Ok(())
    }

    /// Check if auto-save should be performed
    pub fn should_auto_save(&self) -> bool {
        self.config.auto_save && 
        self.last_save.elapsed().as_secs() >= self.config.auto_save_interval_seconds
    }

    /// Perform auto-save if needed
    pub fn update_auto_save(
        &mut self,
        achievement_system: &AchievementSystem,
        player_id: &str,
    ) -> StorageResult<bool> {
        if self.should_auto_save() {
            self.save_achievements(achievement_system, player_id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Create a backup of the current save file
    fn create_backup(&self, player_id: &str) -> StorageResult<()> {
        let main_file = self.get_storage_file_path(player_id);
        
        if !main_file.exists() {
            return Ok(()); // Nothing to backup
        }

        let backup_dir = self.config.storage_directory.join("backups");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let backup_filename = format!("{}_{}.backup", player_id, timestamp);
        let backup_path = backup_dir.join(backup_filename);

        // Copy main file to backup
        fs::copy(&main_file, &backup_path)
            .map_err(|e| AchievementStorageError::BackupError(e.to_string()))?;

        // Clean up old backups
        self.cleanup_old_backups(player_id)?;

        Ok(())
    }

    /// Clean up old backup files
    fn cleanup_old_backups(&self, player_id: &str) -> StorageResult<()> {
        let backup_dir = self.config.storage_directory.join("backups");
        
        if !backup_dir.exists() {
            return Ok(());
        }

        // Get all backup files for this player
        let mut backup_files = Vec::new();
        let entries = fs::read_dir(&backup_dir)
            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| AchievementStorageError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(&format!("{}_", player_id)) && filename.ends_with(".backup") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            backup_files.push((path, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess backups
        if backup_files.len() > self.config.backup_count {
            for (path, _) in backup_files.iter().skip(self.config.backup_count) {
                fs::remove_file(path)
                    .map_err(|e| AchievementStorageError::BackupError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Get the storage file path for a player
    fn get_storage_file_path(&self, player_id: &str) -> PathBuf {
        self.config.storage_directory.join(format!("{}.achievements", player_id))
    }

    /// Write storage file to disk
    fn write_storage_file(&self, path: &Path, storage_file: &AchievementStorageFile) -> StorageResult<()> {
        let serialized = if self.config.compression_enabled {
            self.compress_data(&bincode::serialize(storage_file)
                .map_err(|e| AchievementStorageError::SerializationError(e.to_string()))?)?
        } else {
            bincode::serialize(storage_file)
                .map_err(|e| AchievementStorageError::SerializationError(e.to_string()))?
        };

        let final_data = if self.config.encryption_enabled {
            self.encrypt_data(&serialized)?
        } else {
            serialized
        };

        fs::write(path, final_data)
            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Read storage file from disk
    fn read_storage_file(&self, path: &Path) -> StorageResult<AchievementStorageFile> {
        let mut file_data = fs::read(path)
            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

        if self.config.encryption_enabled {
            file_data = self.decrypt_data(&file_data)?;
        }

        if self.config.compression_enabled {
            file_data = self.decompress_data(&file_data)?;
        }

        let storage_file: AchievementStorageFile = bincode::deserialize(&file_data)
            .map_err(|e| AchievementStorageError::SerializationError(e.to_string()))?;

        Ok(storage_file)
    }

    /// Calculate checksum for data integrity
    fn calculate_checksum(&self, data: &AchievementSaveData) -> StorageResult<String> {
        let serialized = bincode::serialize(data)
            .map_err(|e| AchievementStorageError::SerializationError(e.to_string()))?;
        
        // Simple checksum using CRC32 (in a real implementation, use a proper hash)
        let checksum = crc32fast::hash(&serialized);
        Ok(format!("{:08x}", checksum))
    }

    /// Compress data (placeholder implementation)
    fn compress_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // In a real implementation, use a compression library like flate2
        Ok(data.to_vec())
    }

    /// Decompress data (placeholder implementation)
    fn decompress_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // In a real implementation, use a compression library like flate2
        Ok(data.to_vec())
    }

    /// Encrypt data (placeholder implementation)
    fn encrypt_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // In a real implementation, use a proper encryption library
        Ok(data.to_vec())
    }

    /// Decrypt data (placeholder implementation)
    fn decrypt_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        // In a real implementation, use a proper encryption library
        Ok(data.to_vec())
    }

    /// Get storage statistics
    pub fn get_storage_statistics(&self, player_id: &str) -> StorageResult<StorageStatistics> {
        let file_path = self.get_storage_file_path(player_id);
        
        let file_size = if file_path.exists() {
            fs::metadata(&file_path)
                .map_err(|e| AchievementStorageError::IoError(e.to_string()))?
                .len()
        } else {
            0
        };

        // Count backup files
        let backup_dir = self.config.storage_directory.join("backups");
        let mut backup_count = 0;
        let mut backup_size = 0;

        if backup_dir.exists() {
            let entries = fs::read_dir(&backup_dir)
                .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

            for entry in entries {
                let entry = entry.map_err(|e| AchievementStorageError::IoError(e.to_string()))?;
                let path = entry.path();
                
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&format!("{}_", player_id)) && filename.ends_with(".backup") {
                        backup_count += 1;
                        if let Ok(metadata) = fs::metadata(&path) {
                            backup_size += metadata.len();
                        }
                    }
                }
            }
        }

        Ok(StorageStatistics {
            main_file_size: file_size,
            backup_count,
            backup_total_size: backup_size,
            storage_directory: self.config.storage_directory.clone(),
            last_save_elapsed: self.last_save.elapsed(),
            auto_save_enabled: self.config.auto_save,
            backup_enabled: self.config.backup_enabled,
            compression_enabled: self.config.compression_enabled,
            encryption_enabled: self.config.encryption_enabled,
        })
    }

    /// Export achievements to a custom file
    pub fn export_achievements(
        &self,
        achievement_system: &AchievementSystem,
        player_id: &str,
        export_path: &Path,
    ) -> StorageResult<()> {
        let save_data = achievement_system.export_data();
        let statistics = achievement_system.get_statistics();

        let metadata = AchievementStorageMetadata {
            version: env!("CARGO_PKG_VERSION").to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            player_id: player_id.to_string(),
            total_achievements: statistics.total_achievements,
            unlocked_count: statistics.unlocked_achievements,
            total_points: statistics.earned_points,
            checksum: self.calculate_checksum(&save_data)?,
        };

        let storage_file = AchievementStorageFile {
            metadata,
            data: save_data,
        };

        self.write_storage_file(export_path, &storage_file)?;
        Ok(())
    }

    /// Import achievements from a custom file
    pub fn import_achievements(
        &self,
        achievement_system: &mut AchievementSystem,
        import_path: &Path,
    ) -> StorageResult<()> {
        let storage_file = self.read_storage_file(import_path)?;

        // Verify checksum
        let calculated_checksum = self.calculate_checksum(&storage_file.data)?;
        if calculated_checksum != storage_file.metadata.checksum {
            return Err(AchievementStorageError::CorruptedData(
                "Imported file checksum mismatch".to_string()
            ));
        }

        achievement_system.import_data(storage_file.data);
        Ok(())
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AchievementStorageConfig) -> StorageResult<()> {
        // Validate new configuration
        if config.backup_count == 0 && config.backup_enabled {
            return Err(AchievementStorageError::ConfigError(
                "Backup count cannot be 0 when backups are enabled".to_string()
            ));
        }

        // Create new directories if needed
        fs::create_dir_all(&config.storage_directory)
            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

        if config.backup_enabled {
            let backup_dir = config.storage_directory.join("backups");
            fs::create_dir_all(&backup_dir)
                .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;
        }

        self.config = config;
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AchievementStorageConfig {
        &self.config
    }

    /// Repair corrupted achievement data
    pub fn repair_achievements(
        &self,
        achievement_system: &mut AchievementSystem,
        player_id: &str,
    ) -> StorageResult<bool> {
        // Try to load from backup files
        let backup_dir = self.config.storage_directory.join("backups");
        
        if !backup_dir.exists() {
            return Ok(false);
        }

        // Get all backup files for this player
        let mut backup_files = Vec::new();
        let entries = fs::read_dir(&backup_dir)
            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| AchievementStorageError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(&format!("{}_", player_id)) && filename.ends_with(".backup") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            backup_files.push((path, modified));
                        }
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Try to restore from the most recent valid backup
        for (backup_path, _) in backup_files {
            if let Ok(storage_file) = self.read_storage_file(&backup_path) {
                // Verify checksum
                if let Ok(calculated_checksum) = self.calculate_checksum(&storage_file.data) {
                    if calculated_checksum == storage_file.metadata.checksum {
                        // Valid backup found, restore it
                        achievement_system.import_data(storage_file.data);
                        
                        // Save as main file
                        let main_file = self.get_storage_file_path(player_id);
                        fs::copy(&backup_path, &main_file)
                            .map_err(|e| AchievementStorageError::IoError(e.to_string()))?;
                        
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false) // No valid backup found
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStatistics {
    pub main_file_size: u64,
    pub backup_count: usize,
    pub backup_total_size: u64,
    pub storage_directory: PathBuf,
    pub last_save_elapsed: std::time::Duration,
    pub auto_save_enabled: bool,
    pub backup_enabled: bool,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::achievements::achievement_system::AchievementSystem;

    fn create_test_storage() -> (AchievementStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config = AchievementStorageConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..AchievementStorageConfig::default()
        };
        let storage = AchievementStorage::new(config).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_creation() {
        let (storage, _temp_dir) = create_test_storage();
        assert!(storage.config.storage_directory.exists());
    }

    #[test]
    fn test_save_and_load_achievements() {
        let (mut storage, _temp_dir) = create_test_storage();
        let mut achievement_system = AchievementSystem::new();
        
        // Unlock an achievement
        achievement_system.increment_progress("first_kill", 1);
        
        // Save achievements
        storage.save_achievements(&achievement_system, "test_player").unwrap();
        
        // Create new system and load
        let mut new_system = AchievementSystem::new();
        storage.load_achievements(&mut new_system, "test_player").unwrap();
        
        // Verify achievement was loaded
        assert!(new_system.is_unlocked("first_kill"));
    }

    #[test]
    fn test_backup_creation() {
        let (mut storage, _temp_dir) = create_test_storage();
        let achievement_system = AchievementSystem::new();
        
        // Save achievements (should create backup)
        storage.save_achievements(&achievement_system, "test_player").unwrap();
        
        // Save again (should create another backup)
        storage.save_achievements(&achievement_system, "test_player").unwrap();
        
        let stats = storage.get_storage_statistics("test_player").unwrap();
        assert!(stats.backup_count > 0);
    }

    #[test]
    fn test_checksum_verification() {
        let (mut storage, _temp_dir) = create_test_storage();
        let achievement_system = AchievementSystem::new();
        
        // Save achievements
        storage.save_achievements(&achievement_system, "test_player").unwrap();
        
        // Manually corrupt the file
        let file_path = storage.get_storage_file_path("test_player");
        fs::write(&file_path, b"corrupted data").unwrap();
        
        // Try to load - should fail with checksum error
        let mut new_system = AchievementSystem::new();
        let result = storage.load_achievements(&mut new_system, "test_player");
        assert!(result.is_err());
    }

    #[test]
    fn test_auto_save() {
        let (mut storage, _temp_dir) = create_test_storage();
        let achievement_system = AchievementSystem::new();
        
        // Initially should not need auto-save
        assert!(!storage.should_auto_save());
        
        // Manually set last save time to trigger auto-save
        storage.last_save = std::time::Instant::now() - std::time::Duration::from_secs(400);
        assert!(storage.should_auto_save());
        
        // Perform auto-save
        let saved = storage.update_auto_save(&achievement_system, "test_player").unwrap();
        assert!(saved);
    }

    #[test]
    fn test_export_import() {
        let (mut storage, temp_dir) = create_test_storage();
        let mut achievement_system = AchievementSystem::new();
        
        // Unlock an achievement
        achievement_system.increment_progress("first_kill", 1);
        
        // Export achievements
        let export_path = temp_dir.path().join("export.achievements");
        storage.export_achievements(&achievement_system, "test_player", &export_path).unwrap();
        
        // Create new system and import
        let mut new_system = AchievementSystem::new();
        storage.import_achievements(&mut new_system, &export_path).unwrap();
        
        // Verify achievement was imported
        assert!(new_system.is_unlocked("first_kill"));
    }

    #[test]
    fn test_storage_statistics() {
        let (mut storage, _temp_dir) = create_test_storage();
        let achievement_system = AchievementSystem::new();
        
        // Save achievements
        storage.save_achievements(&achievement_system, "test_player").unwrap();
        
        let stats = storage.get_storage_statistics("test_player").unwrap();
        assert!(stats.main_file_size > 0);
        assert!(stats.auto_save_enabled);
        assert!(stats.backup_enabled);
    }

    #[test]
    fn test_repair_achievements() {
        let (mut storage, _temp_dir) = create_test_storage();
        let mut achievement_system = AchievementSystem::new();
        
        // Unlock an achievement and save
        achievement_system.increment_progress("first_kill", 1);
        storage.save_achievements(&achievement_system, "test_player").unwrap();
        
        // Corrupt the main file
        let file_path = storage.get_storage_file_path("test_player");
        fs::write(&file_path, b"corrupted data").unwrap();
        
        // Try to repair
        let mut new_system = AchievementSystem::new();
        let repaired = storage.repair_achievements(&mut new_system, "test_player").unwrap();
        assert!(repaired);
        
        // Verify achievement was restored
        assert!(new_system.is_unlocked("first_kill"));
    }

    #[test]
    fn test_config_update() {
        let (mut storage, temp_dir) = create_test_storage();
        
        let mut new_config = AchievementStorageConfig::default();
        new_config.storage_directory = temp_dir.path().to_path_buf();
        new_config.backup_count = 10;
        new_config.auto_save_interval_seconds = 600;
        
        storage.update_config(new_config).unwrap();
        
        assert_eq!(storage.config.backup_count, 10);
        assert_eq!(storage.config.auto_save_interval_seconds, 600);
    }
}