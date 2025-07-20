use serde::{Serialize, Deserialize};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write, BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::persistence::serialization::{SaveData, SerializationResult, SerializationError};

/// Save system errors
#[derive(Debug, Clone)]
pub enum SaveError {
    IoError(String),
    SerializationError(SerializationError),
    SlotNotFound(u32),
    InvalidSaveFile(String),
    PermissionDenied(String),
    DiskFull,
    CorruptedSave(String),
}

impl From<SerializationError> for SaveError {
    fn from(error: SerializationError) -> Self {
        SaveError::SerializationError(error)
    }
}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        SaveError::IoError(error.to_string())
    }
}

pub type SaveResult<T> = Result<T, SaveError>;

/// Save slot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSlot {
    pub slot_id: u32,
    pub metadata: SaveMetadata,
    pub file_path: PathBuf,
    pub is_occupied: bool,
    pub is_corrupted: bool,
    pub backup_available: bool,
}

/// Save file metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub save_name: String,
    pub player_name: String,
    pub character_level: i32,
    pub current_depth: i32,
    pub playtime_seconds: u64,
    pub created_at: u64,
    pub last_saved: u64,
    pub game_version: String,
    pub screenshot_path: Option<PathBuf>,
    pub achievements_count: u32,
    pub difficulty: String,
    pub seed: Option<u64>,
}

impl SaveMetadata {
    pub fn new(save_name: String, player_name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        SaveMetadata {
            save_name,
            player_name,
            character_level: 1,
            current_depth: 1,
            playtime_seconds: 0,
            created_at: now,
            last_saved: now,
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            screenshot_path: None,
            achievements_count: 0,
            difficulty: "Normal".to_string(),
            seed: None,
        }
    }

    pub fn formatted_playtime(&self) -> String {
        let hours = self.playtime_seconds / 3600;
        let minutes = (self.playtime_seconds % 3600) / 60;
        let seconds = self.playtime_seconds % 60;

        if hours > 0 {
            format!("{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", seconds)
        }
    }

    pub fn formatted_last_saved(&self) -> String {
        // In a real implementation, you'd format this as a human-readable date
        format!("Timestamp: {}", self.last_saved)
    }
}

/// Save file wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    pub metadata: SaveMetadata,
    pub data: SaveData,
    pub checksum: Option<String>,
}

impl SaveFile {
    pub fn new(metadata: SaveMetadata, data: SaveData) -> Self {
        let mut save_file = SaveFile {
            metadata,
            data,
            checksum: None,
        };
        save_file.calculate_checksum();
        save_file
    }

    pub fn calculate_checksum(&mut self) {
        // Simple checksum calculation (in practice, use a proper hash)
        let serialized = bincode::serialize(&self.data).unwrap_or_default();
        let checksum = format!("{:x}", md5::compute(&serialized));
        self.checksum = Some(checksum);
    }

    pub fn verify_checksum(&self) -> bool {
        if let Some(ref stored_checksum) = self.checksum {
            let serialized = bincode::serialize(&self.data).unwrap_or_default();
            let calculated_checksum = format!("{:x}", md5::compute(&serialized));
            *stored_checksum == calculated_checksum
        } else {
            false
        }
    }
}

/// Main save system
pub struct SaveSystem {
    save_directory: PathBuf,
    max_save_slots: u32,
    backup_count: u32,
    auto_backup: bool,
    compression_enabled: bool,
}

impl SaveSystem {
    pub fn new<P: AsRef<Path>>(save_directory: P) -> SaveResult<Self> {
        let save_dir = save_directory.as_ref().to_path_buf();
        
        // Create save directory if it doesn't exist
        create_dir_all(&save_dir)?;

        Ok(SaveSystem {
            save_directory: save_dir,
            max_save_slots: 10,
            backup_count: 3,
            auto_backup: true,
            compression_enabled: true,
        })
    }

    pub fn with_max_slots(mut self, max_slots: u32) -> Self {
        self.max_save_slots = max_slots;
        self
    }

    pub fn with_backup_count(mut self, backup_count: u32) -> Self {
        self.backup_count = backup_count;
        self
    }

    pub fn with_auto_backup(mut self, enabled: bool) -> Self {
        self.auto_backup = enabled;
        self
    }

    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression_enabled = enabled;
        self
    }

    /// Save game to a specific slot
    pub fn save_to_slot(&self, slot_id: u32, save_data: SaveData, metadata: SaveMetadata) -> SaveResult<()> {
        if slot_id >= self.max_save_slots {
            return Err(SaveError::SlotNotFound(slot_id));
        }

        let save_file = SaveFile::new(metadata, save_data);
        let file_path = self.get_save_file_path(slot_id);

        // Create backup if auto-backup is enabled
        if self.auto_backup && file_path.exists() {
            self.create_backup(slot_id)?;
        }

        // Write save file
        self.write_save_file(&file_path, &save_file)?;

        // Update slot metadata
        self.update_slot_metadata(slot_id, &save_file.metadata)?;

        Ok(())
    }

    /// Load game from a specific slot
    pub fn load_from_slot(&self, slot_id: u32) -> SaveResult<SaveFile> {
        if slot_id >= self.max_save_slots {
            return Err(SaveError::SlotNotFound(slot_id));
        }

        let file_path = self.get_save_file_path(slot_id);
        
        if !file_path.exists() {
            return Err(SaveError::SlotNotFound(slot_id));
        }

        let save_file = self.read_save_file(&file_path)?;

        // Verify checksum
        if !save_file.verify_checksum() {
            // Try to load from backup
            if let Ok(backup_file) = self.load_from_backup(slot_id) {
                return Ok(backup_file);
            }
            return Err(SaveError::CorruptedSave(format!("Slot {}", slot_id)));
        }

        Ok(save_file)
    }

    /// Get all save slots
    pub fn get_save_slots(&self) -> SaveResult<Vec<SaveSlot>> {
        let mut slots = Vec::new();

        for slot_id in 0..self.max_save_slots {
            let file_path = self.get_save_file_path(slot_id);
            let is_occupied = file_path.exists();
            let backup_available = self.get_backup_file_path(slot_id, 0).exists();

            let (metadata, is_corrupted) = if is_occupied {
                match self.load_slot_metadata(slot_id) {
                    Ok(meta) => (meta, false),
                    Err(_) => (
                        SaveMetadata::new("Corrupted Save".to_string(), "Unknown".to_string()),
                        true
                    ),
                }
            } else {
                (SaveMetadata::new("Empty Slot".to_string(), "".to_string()), false)
            };

            slots.push(SaveSlot {
                slot_id,
                metadata,
                file_path,
                is_occupied,
                is_corrupted,
                backup_available,
            });
        }

        Ok(slots)
    }

    /// Delete a save slot
    pub fn delete_slot(&self, slot_id: u32) -> SaveResult<()> {
        if slot_id >= self.max_save_slots {
            return Err(SaveError::SlotNotFound(slot_id));
        }

        let file_path = self.get_save_file_path(slot_id);
        
        if file_path.exists() {
            std::fs::remove_file(&file_path)?;
        }

        // Also remove backups
        for backup_index in 0..self.backup_count {
            let backup_path = self.get_backup_file_path(slot_id, backup_index);
            if backup_path.exists() {
                let _ = std::fs::remove_file(&backup_path); // Ignore errors for backups
            }
        }

        // Remove metadata file
        let metadata_path = self.get_metadata_file_path(slot_id);
        if metadata_path.exists() {
            let _ = std::fs::remove_file(&metadata_path);
        }

        Ok(())
    }

    /// Create a backup of a save slot
    pub fn create_backup(&self, slot_id: u32) -> SaveResult<()> {
        let source_path = self.get_save_file_path(slot_id);
        
        if !source_path.exists() {
            return Ok(()); // Nothing to backup
        }

        // Rotate existing backups
        for i in (1..self.backup_count).rev() {
            let from_path = self.get_backup_file_path(slot_id, i - 1);
            let to_path = self.get_backup_file_path(slot_id, i);
            
            if from_path.exists() {
                let _ = std::fs::rename(&from_path, &to_path); // Ignore errors
            }
        }

        // Create new backup
        let backup_path = self.get_backup_file_path(slot_id, 0);
        std::fs::copy(&source_path, &backup_path)?;

        Ok(())
    }

    /// Load from backup
    pub fn load_from_backup(&self, slot_id: u32) -> SaveResult<SaveFile> {
        for backup_index in 0..self.backup_count {
            let backup_path = self.get_backup_file_path(slot_id, backup_index);
            
            if backup_path.exists() {
                match self.read_save_file(&backup_path) {
                    Ok(save_file) => {
                        if save_file.verify_checksum() {
                            return Ok(save_file);
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Err(SaveError::CorruptedSave(format!("No valid backup for slot {}", slot_id)))
    }

    /// Get save directory info
    pub fn get_save_info(&self) -> SaveSystemInfo {
        let mut total_size = 0;
        let mut file_count = 0;

        if let Ok(entries) = std::fs::read_dir(&self.save_directory) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                }
            }
        }

        SaveSystemInfo {
            save_directory: self.save_directory.clone(),
            max_save_slots: self.max_save_slots,
            backup_count: self.backup_count,
            total_size_bytes: total_size,
            file_count,
            compression_enabled: self.compression_enabled,
            auto_backup_enabled: self.auto_backup,
        }
    }

    /// Cleanup old backups and temporary files
    pub fn cleanup(&self) -> SaveResult<u32> {
        let mut cleaned_files = 0;

        // Remove temporary files
        if let Ok(entries) = std::fs::read_dir(&self.save_directory) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(extension) = path.extension() {
                    if extension == "tmp" || extension == "bak" {
                        if std::fs::remove_file(&path).is_ok() {
                            cleaned_files += 1;
                        }
                    }
                }
            }
        }

        Ok(cleaned_files)
    }

    // Private helper methods

    fn get_save_file_path(&self, slot_id: u32) -> PathBuf {
        self.save_directory.join(format!("save_{:03}.dat", slot_id))
    }

    fn get_backup_file_path(&self, slot_id: u32, backup_index: u32) -> PathBuf {
        self.save_directory.join(format!("save_{:03}.bak{}", slot_id, backup_index))
    }

    fn get_metadata_file_path(&self, slot_id: u32) -> PathBuf {
        self.save_directory.join(format!("save_{:03}.meta", slot_id))
    }

    fn write_save_file(&self, file_path: &Path, save_file: &SaveFile) -> SaveResult<()> {
        let temp_path = file_path.with_extension("tmp");
        
        {
            let file = File::create(&temp_path)?;
            let mut writer = BufWriter::new(file);
            
            let data = if self.compression_enabled {
                self.compress_data(&bincode::serialize(save_file).map_err(|e| {
                    SaveError::IoError(format!("Serialization failed: {}", e))
                })?)?
            } else {
                bincode::serialize(save_file).map_err(|e| {
                    SaveError::IoError(format!("Serialization failed: {}", e))
                })?
            };
            
            writer.write_all(&data)?;
            writer.flush()?;
        }

        // Atomic rename
        std::fs::rename(&temp_path, file_path)?;
        
        Ok(())
    }

    fn read_save_file(&self, file_path: &Path) -> SaveResult<SaveFile> {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        let mut data = Vec::new();
        reader.read_to_end(&mut data)?;

        let decompressed_data = if self.compression_enabled {
            self.decompress_data(&data)?
        } else {
            data
        };

        let save_file: SaveFile = bincode::deserialize(&decompressed_data)
            .map_err(|e| SaveError::InvalidSaveFile(e.to_string()))?;

        Ok(save_file)
    }

    fn load_slot_metadata(&self, slot_id: u32) -> SaveResult<SaveMetadata> {
        let save_file = self.load_from_slot(slot_id)?;
        Ok(save_file.metadata)
    }

    fn update_slot_metadata(&self, slot_id: u32, metadata: &SaveMetadata) -> SaveResult<()> {
        let metadata_path = self.get_metadata_file_path(slot_id);
        let file = File::create(&metadata_path)?;
        let writer = BufWriter::new(file);
        
        bincode::serialize_into(writer, metadata)
            .map_err(|e| SaveError::IoError(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    fn compress_data(&self, data: &[u8]) -> SaveResult<Vec<u8>> {
        // Placeholder for compression - in practice, use a compression library
        Ok(data.to_vec())
    }

    fn decompress_data(&self, data: &[u8]) -> SaveResult<Vec<u8>> {
        // Placeholder for decompression - in practice, use a compression library
        Ok(data.to_vec())
    }
}

/// Save system information
#[derive(Debug, Clone)]
pub struct SaveSystemInfo {
    pub save_directory: PathBuf,
    pub max_save_slots: u32,
    pub backup_count: u32,
    pub total_size_bytes: u64,
    pub file_count: usize,
    pub compression_enabled: bool,
    pub auto_backup_enabled: bool,
}

impl SaveSystemInfo {
    pub fn formatted_size(&self) -> String {
        let size = self.total_size_bytes as f64;
        
        if size >= 1_073_741_824.0 {
            format!("{:.2} GB", size / 1_073_741_824.0)
        } else if size >= 1_048_576.0 {
            format!("{:.2} MB", size / 1_048_576.0)
        } else if size >= 1024.0 {
            format!("{:.2} KB", size / 1024.0)
        } else {
            format!("{} bytes", size)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_save_system() -> (SaveSystem, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let save_system = SaveSystem::new(temp_dir.path()).unwrap();
        (save_system, temp_dir)
    }

    #[test]
    fn test_save_system_creation() {
        let (save_system, _temp_dir) = create_test_save_system();
        assert_eq!(save_system.max_save_slots, 10);
        assert_eq!(save_system.backup_count, 3);
        assert!(save_system.auto_backup);
        assert!(save_system.compression_enabled);
    }

    #[test]
    fn test_save_system_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let save_system = SaveSystem::new(temp_dir.path()).unwrap()
            .with_max_slots(5)
            .with_backup_count(2)
            .with_auto_backup(false)
            .with_compression(false);

        assert_eq!(save_system.max_save_slots, 5);
        assert_eq!(save_system.backup_count, 2);
        assert!(!save_system.auto_backup);
        assert!(!save_system.compression_enabled);
    }

    #[test]
    fn test_save_and_load() {
        let (save_system, _temp_dir) = create_test_save_system();
        
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string());
        let metadata = SaveMetadata::new("Test Save".to_string(), "Test Player".to_string());
        
        // Save to slot 0
        save_system.save_to_slot(0, save_data.clone(), metadata.clone()).unwrap();
        
        // Load from slot 0
        let loaded_save = save_system.load_from_slot(0).unwrap();
        
        assert_eq!(loaded_save.data.game_name, save_data.game_name);
        assert_eq!(loaded_save.data.player_name, save_data.player_name);
        assert_eq!(loaded_save.metadata.save_name, metadata.save_name);
    }

    #[test]
    fn test_save_slots() {
        let (save_system, _temp_dir) = create_test_save_system();
        
        // Initially all slots should be empty
        let slots = save_system.get_save_slots().unwrap();
        assert_eq!(slots.len(), 10);
        assert!(slots.iter().all(|slot| !slot.is_occupied));
        
        // Save to slot 0
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string());
        let metadata = SaveMetadata::new("Test Save".to_string(), "Test Player".to_string());
        save_system.save_to_slot(0, save_data, metadata).unwrap();
        
        // Check slots again
        let slots = save_system.get_save_slots().unwrap();
        assert!(slots[0].is_occupied);
        assert!(slots[1..].iter().all(|slot| !slot.is_occupied));
    }

    #[test]
    fn test_delete_slot() {
        let (save_system, _temp_dir) = create_test_save_system();
        
        // Save to slot 0
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string());
        let metadata = SaveMetadata::new("Test Save".to_string(), "Test Player".to_string());
        save_system.save_to_slot(0, save_data, metadata).unwrap();
        
        // Verify it exists
        assert!(save_system.load_from_slot(0).is_ok());
        
        // Delete it
        save_system.delete_slot(0).unwrap();
        
        // Verify it's gone
        assert!(save_system.load_from_slot(0).is_err());
    }

    #[test]
    fn test_invalid_slot() {
        let (save_system, _temp_dir) = create_test_save_system();
        
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string());
        let metadata = SaveMetadata::new("Test Save".to_string(), "Test Player".to_string());
        
        // Try to save to invalid slot
        let result = save_system.save_to_slot(999, save_data, metadata);
        assert!(matches!(result, Err(SaveError::SlotNotFound(999))));
        
        // Try to load from invalid slot
        let result = save_system.load_from_slot(999);
        assert!(matches!(result, Err(SaveError::SlotNotFound(999))));
    }

    #[test]
    fn test_save_metadata() {
        let metadata = SaveMetadata::new("Test Save".to_string(), "Test Player".to_string());
        
        assert_eq!(metadata.save_name, "Test Save");
        assert_eq!(metadata.player_name, "Test Player");
        assert_eq!(metadata.character_level, 1);
        assert_eq!(metadata.current_depth, 1);
        assert_eq!(metadata.playtime_seconds, 0);
        assert_eq!(metadata.difficulty, "Normal");
    }

    #[test]
    fn test_formatted_playtime() {
        let mut metadata = SaveMetadata::new("Test".to_string(), "Player".to_string());
        
        metadata.playtime_seconds = 30;
        assert_eq!(metadata.formatted_playtime(), "30s");
        
        metadata.playtime_seconds = 90;
        assert_eq!(metadata.formatted_playtime(), "1m 30s");
        
        metadata.playtime_seconds = 3661;
        assert_eq!(metadata.formatted_playtime(), "1h 1m 1s");
    }

    #[test]
    fn test_save_file_checksum() {
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string());
        let metadata = SaveMetadata::new("Test Save".to_string(), "Test Player".to_string());
        
        let save_file = SaveFile::new(metadata, save_data);
        
        assert!(save_file.checksum.is_some());
        assert!(save_file.verify_checksum());
    }

    #[test]
    fn test_save_system_info() {
        let (save_system, _temp_dir) = create_test_save_system();
        
        let info = save_system.get_save_info();
        
        assert_eq!(info.max_save_slots, 10);
        assert_eq!(info.backup_count, 3);
        assert!(info.compression_enabled);
        assert!(info.auto_backup_enabled);
    }

    #[test]
    fn test_formatted_size() {
        let mut info = SaveSystemInfo {
            save_directory: PathBuf::new(),
            max_save_slots: 10,
            backup_count: 3,
            total_size_bytes: 1024,
            file_count: 1,
            compression_enabled: true,
            auto_backup_enabled: true,
        };

        assert_eq!(info.formatted_size(), "1.00 KB");

        info.total_size_bytes = 1_048_576;
        assert_eq!(info.formatted_size(), "1.00 MB");

        info.total_size_bytes = 1_073_741_824;
        assert_eq!(info.formatted_size(), "1.00 GB");
    }
}