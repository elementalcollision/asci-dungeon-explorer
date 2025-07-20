use std::path::{Path, PathBuf};
use std::fs;
use std::time::{SystemTime, Duration};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::persistence::{
    save_system::{SaveResult, SaveError},
    save_rotation::{SaveRotationSystem, RotationResult, SaveFileInfo},
    crash_recovery::{CrashRecoverySystem, CrashRecoverySave},
};

/// Save cleanup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCleanupConfig {
    pub enabled: bool,
    pub cleanup_interval_hours: u64,
    pub max_total_save_size_mb: u64,
    pub max_save_age_days: u64,
    pub keep_important_saves: bool,
    pub cleanup_empty_directories: bool,
    pub cleanup_temp_files: bool,
    pub cleanup_crash_recovery: bool,
    pub max_crash_recovery_age_days: u64,
    pub compress_old_saves: bool,
    pub compression_age_days: u64,
}

impl Default for SaveCleanupConfig {
    fn default() -> Self {
        SaveCleanupConfig {
            enabled: true,
            cleanup_interval_hours: 24, // Daily cleanup
            max_total_save_size_mb: 500, // 500MB total
            max_save_age_days: 90, // 3 months
            keep_important_saves: true,
            cleanup_empty_directories: true,
            cleanup_temp_files: true,
            cleanup_crash_recovery: true,
            max_crash_recovery_age_days: 7, // 1 week
            compress_old_saves: true,
            compression_age_days: 30, // Compress saves older than 30 days
        }
    }
}

/// Types of files that can be cleaned up
#[derive(Debug, Clone, PartialEq)]
pub enum CleanupFileType {
    SaveFile,
    BackupFile,
    TempFile,
    CrashRecovery,
    CompressedSave,
    EmptyDirectory,
}

/// Information about a file to be cleaned up
#[derive(Debug, Clone)]
pub struct CleanupFileInfo {
    pub path: PathBuf,
    pub file_type: CleanupFileType,
    pub size_bytes: u64,
    pub age_days: u64,
    pub is_important: bool,
    pub reason: String,
}

/// Result of cleanup operation
#[derive(Debug, Clone)]
pub struct CleanupResult {
    pub files_deleted: Vec<CleanupFileInfo>,
    pub files_compressed: Vec<PathBuf>,
    pub directories_removed: Vec<PathBuf>,
    pub space_freed_bytes: u64,
    pub space_saved_by_compression: u64,
    pub errors: Vec<String>,
}

impl CleanupResult {
    pub fn new() -> Self {
        CleanupResult {
            files_deleted: Vec::new(),
            files_compressed: Vec::new(),
            directories_removed: Vec::new(),
            space_freed_bytes: 0,
            space_saved_by_compression: 0,
            errors: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: CleanupResult) {
        self.files_deleted.extend(other.files_deleted);
        self.files_compressed.extend(other.files_compressed);
        self.directories_removed.extend(other.directories_removed);
        self.space_freed_bytes += other.space_freed_bytes;
        self.space_saved_by_compression += other.space_saved_by_compression;
        self.errors.extend(other.errors);
    }
}

/// Comprehensive save cleanup system
pub struct SaveCleanupSystem {
    config: SaveCleanupConfig,
    save_directory: PathBuf,
    last_cleanup: SystemTime,
}

impl SaveCleanupSystem {
    pub fn new<P: AsRef<Path>>(
        save_directory: P,
        config: SaveCleanupConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let save_dir = save_directory.as_ref().to_path_buf();
        
        Ok(SaveCleanupSystem {
            config,
            save_directory: save_dir,
            last_cleanup: SystemTime::now(),
        })
    }

    /// Check if cleanup should run
    pub fn should_run_cleanup(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        let elapsed = self.last_cleanup.elapsed().unwrap_or_default();
        let cleanup_interval = Duration::from_secs(self.config.cleanup_interval_hours * 3600);
        
        elapsed >= cleanup_interval
    }

    /// Run comprehensive cleanup
    pub fn run_cleanup(&mut self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();

        // Clean up old save files
        if let Ok(save_cleanup) = self.cleanup_old_saves() {
            result.merge(save_cleanup);
        }

        // Clean up temporary files
        if self.config.cleanup_temp_files {
            if let Ok(temp_cleanup) = self.cleanup_temp_files() {
                result.merge(temp_cleanup);
            }
        }

        // Clean up crash recovery files
        if self.config.cleanup_crash_recovery {
            if let Ok(crash_cleanup) = self.cleanup_crash_recovery_files() {
                result.merge(crash_cleanup);
            }
        }

        // Compress old saves
        if self.config.compress_old_saves {
            if let Ok(compression_result) = self.compress_old_saves() {
                result.merge(compression_result);
            }
        }

        // Clean up empty directories
        if self.config.cleanup_empty_directories {
            if let Ok(dir_cleanup) = self.cleanup_empty_directories() {
                result.merge(dir_cleanup);
            }
        }

        // Enforce total size limit
        if let Ok(size_cleanup) = self.enforce_size_limit() {
            result.merge(size_cleanup);
        }

        self.last_cleanup = SystemTime::now();
        Ok(result)
    }

    /// Clean up old save files
    fn cleanup_old_saves(&self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();
        let files_to_check = self.scan_all_files()?;

        for file_info in files_to_check {
            if self.should_delete_file(&file_info) {
                match self.delete_file(&file_info) {
                    Ok(()) => {
                        result.space_freed_bytes += file_info.size_bytes;
                        result.files_deleted.push(file_info);
                    },
                    Err(e) => {
                        result.errors.push(format!("Failed to delete {}: {}", 
                            file_info.path.display(), e));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Clean up temporary files
    fn cleanup_temp_files(&self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();
        
        let temp_patterns = vec![
            "*.tmp",
            "*.temp",
            "*.bak",
            "*.~*",
            ".#*",
        ];

        for pattern in temp_patterns {
            if let Ok(temp_files) = self.find_files_by_pattern(pattern) {
                for file_path in temp_files {
                    if let Ok(metadata) = fs::metadata(&file_path) {
                        let age = self.calculate_file_age(&file_path).unwrap_or(0);
                        
                        // Delete temp files older than 1 day
                        if age > 1 {
                            let cleanup_info = CleanupFileInfo {
                                path: file_path.clone(),
                                file_type: CleanupFileType::TempFile,
                                size_bytes: metadata.len(),
                                age_days: age,
                                is_important: false,
                                reason: "Temporary file cleanup".to_string(),
                            };

                            match fs::remove_file(&file_path) {
                                Ok(()) => {
                                    result.space_freed_bytes += metadata.len();
                                    result.files_deleted.push(cleanup_info);
                                },
                                Err(e) => {
                                    result.errors.push(format!("Failed to delete temp file {}: {}", 
                                        file_path.display(), e));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Clean up crash recovery files
    fn cleanup_crash_recovery_files(&self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();
        let recovery_dir = self.save_directory.join("recovery");
        
        if !recovery_dir.exists() {
            return Ok(result);
        }

        let entries = fs::read_dir(&recovery_dir)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();
            
            if let Ok(metadata) = fs::metadata(&path) {
                let age = self.calculate_file_age(&path).unwrap_or(0);
                
                if age > self.config.max_crash_recovery_age_days {
                    let cleanup_info = CleanupFileInfo {
                        path: path.clone(),
                        file_type: CleanupFileType::CrashRecovery,
                        size_bytes: metadata.len(),
                        age_days: age,
                        is_important: false,
                        reason: "Old crash recovery file".to_string(),
                    };

                    match fs::remove_file(&path) {
                        Ok(()) => {
                            result.space_freed_bytes += metadata.len();
                            result.files_deleted.push(cleanup_info);
                        },
                        Err(e) => {
                            result.errors.push(format!("Failed to delete recovery file {}: {}", 
                                path.display(), e));
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Compress old save files
    fn compress_old_saves(&self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();
        let files_to_check = self.scan_all_files()?;

        for file_info in files_to_check {
            if file_info.age_days > self.config.compression_age_days && 
               !file_info.path.extension().map_or(false, |ext| ext == "gz") {
                
                match self.compress_file(&file_info.path) {
                    Ok(space_saved) => {
                        result.files_compressed.push(file_info.path);
                        result.space_saved_by_compression += space_saved;
                    },
                    Err(e) => {
                        result.errors.push(format!("Failed to compress {}: {}", 
                            file_info.path.display(), e));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Clean up empty directories
    fn cleanup_empty_directories(&self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();
        
        if let Ok(empty_dirs) = self.find_empty_directories(&self.save_directory) {
            for dir_path in empty_dirs {
                match fs::remove_dir(&dir_path) {
                    Ok(()) => {
                        result.directories_removed.push(dir_path);
                    },
                    Err(e) => {
                        result.errors.push(format!("Failed to remove empty directory {}: {}", 
                            dir_path.display(), e));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Enforce total size limit
    fn enforce_size_limit(&self) -> SaveResult<CleanupResult> {
        let mut result = CleanupResult::new();
        let max_size_bytes = self.config.max_total_save_size_mb * 1024 * 1024;
        
        let mut all_files = self.scan_all_files()?;
        let total_size: u64 = all_files.iter().map(|f| f.size_bytes).sum();
        
        if total_size <= max_size_bytes {
            return Ok(result);
        }

        // Sort by importance (least important first)
        all_files.sort_by(|a, b| {
            // Important files go to the end (kept)
            match (a.is_important, b.is_important) {
                (true, false) => std::cmp::Ordering::Greater,
                (false, true) => std::cmp::Ordering::Less,
                _ => b.age_days.cmp(&a.age_days), // Older files first
            }
        });

        let mut current_size = total_size;
        for file_info in all_files {
            if current_size <= max_size_bytes {
                break;
            }

            if !file_info.is_important {
                match self.delete_file(&file_info) {
                    Ok(()) => {
                        current_size -= file_info.size_bytes;
                        result.space_freed_bytes += file_info.size_bytes;
                        result.files_deleted.push(file_info);
                    },
                    Err(e) => {
                        result.errors.push(format!("Failed to delete {}: {}", 
                            file_info.path.display(), e));
                    }
                }
            }
        }

        Ok(result)
    }

    /// Scan all files in save directory
    fn scan_all_files(&self) -> SaveResult<Vec<CleanupFileInfo>> {
        let mut files = Vec::new();
        self.scan_directory_recursive(&self.save_directory, &mut files)?;
        Ok(files)
    }

    /// Recursively scan directory for files
    fn scan_directory_recursive(
        &self,
        dir: &Path,
        files: &mut Vec<CleanupFileInfo>,
    ) -> SaveResult<()> {
        let entries = fs::read_dir(dir)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                if let Ok(file_info) = self.analyze_file(&path) {
                    files.push(file_info);
                }
            } else if path.is_dir() {
                self.scan_directory_recursive(&path, files)?;
            }
        }

        Ok(())
    }

    /// Analyze a file for cleanup purposes
    fn analyze_file(&self, path: &Path) -> SaveResult<CleanupFileInfo> {
        let metadata = fs::metadata(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        let size_bytes = metadata.len();
        let age_days = self.calculate_file_age(path).unwrap_or(0);
        let file_type = self.determine_file_type(path);
        let is_important = self.is_important_file(path, &file_type, age_days);
        let reason = self.get_cleanup_reason(path, &file_type, age_days, is_important);

        Ok(CleanupFileInfo {
            path: path.to_path_buf(),
            file_type,
            size_bytes,
            age_days,
            is_important,
            reason,
        })
    }

    /// Determine file type based on path and extension
    fn determine_file_type(&self, path: &Path) -> CleanupFileType {
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        if filename.contains("backup") {
            CleanupFileType::BackupFile
        } else if filename.contains("recovery") {
            CleanupFileType::CrashRecovery
        } else if filename.ends_with(".tmp") || filename.ends_with(".temp") {
            CleanupFileType::TempFile
        } else if path.extension().map_or(false, |ext| ext == "gz") {
            CleanupFileType::CompressedSave
        } else {
            CleanupFileType::SaveFile
        }
    }

    /// Check if a file is important and should be preserved
    fn is_important_file(&self, path: &Path, file_type: &CleanupFileType, age_days: u64) -> bool {
        if !self.config.keep_important_saves {
            return false;
        }

        match file_type {
            CleanupFileType::SaveFile => {
                // Manual saves are important
                let filename = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                
                !filename.contains("autosave") && age_days < 30
            },
            CleanupFileType::BackupFile => age_days < 7,
            CleanupFileType::TempFile => false,
            CleanupFileType::CrashRecovery => age_days < 1,
            CleanupFileType::CompressedSave => true, // Already compressed, keep
            CleanupFileType::EmptyDirectory => false,
        }
    }

    /// Get reason for cleanup
    fn get_cleanup_reason(
        &self,
        _path: &Path,
        file_type: &CleanupFileType,
        age_days: u64,
        is_important: bool,
    ) -> String {
        if is_important {
            return "Important file - preserved".to_string();
        }

        match file_type {
            CleanupFileType::SaveFile => {
                if age_days > self.config.max_save_age_days {
                    format!("Save file older than {} days", self.config.max_save_age_days)
                } else {
                    "Size limit enforcement".to_string()
                }
            },
            CleanupFileType::BackupFile => "Old backup file".to_string(),
            CleanupFileType::TempFile => "Temporary file cleanup".to_string(),
            CleanupFileType::CrashRecovery => "Old crash recovery file".to_string(),
            CleanupFileType::CompressedSave => "Compressed save maintenance".to_string(),
            CleanupFileType::EmptyDirectory => "Empty directory cleanup".to_string(),
        }
    }

    /// Check if a file should be deleted
    fn should_delete_file(&self, file_info: &CleanupFileInfo) -> bool {
        if file_info.is_important {
            return false;
        }

        match file_info.file_type {
            CleanupFileType::SaveFile => {
                file_info.age_days > self.config.max_save_age_days
            },
            CleanupFileType::BackupFile => {
                file_info.age_days > 14 // Keep backups for 2 weeks
            },
            CleanupFileType::TempFile => {
                file_info.age_days > 1 // Delete temp files after 1 day
            },
            CleanupFileType::CrashRecovery => {
                file_info.age_days > self.config.max_crash_recovery_age_days
            },
            CleanupFileType::CompressedSave => false, // Don't delete compressed saves
            CleanupFileType::EmptyDirectory => true,
        }
    }

    /// Delete a file
    fn delete_file(&self, file_info: &CleanupFileInfo) -> SaveResult<()> {
        fs::remove_file(&file_info.path)
            .map_err(|e| SaveError::IoError(e.to_string()))
    }

    /// Calculate file age in days
    fn calculate_file_age(&self, path: &Path) -> SaveResult<u64> {
        let metadata = fs::metadata(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        let modified = metadata.modified()
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        let age = SystemTime::now()
            .duration_since(modified)
            .unwrap_or_default()
            .as_secs() / (24 * 60 * 60);
        
        Ok(age)
    }

    /// Find files matching a pattern
    fn find_files_by_pattern(&self, _pattern: &str) -> SaveResult<Vec<PathBuf>> {
        // Simplified implementation - in real code would use glob patterns
        Ok(Vec::new())
    }

    /// Find empty directories
    fn find_empty_directories(&self, dir: &Path) -> SaveResult<Vec<PathBuf>> {
        let mut empty_dirs = Vec::new();
        
        let entries = fs::read_dir(dir)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                // Check if directory is empty
                let dir_entries = fs::read_dir(&path)
                    .map_err(|e| SaveError::IoError(e.to_string()))?;
                
                if dir_entries.count() == 0 {
                    empty_dirs.push(path);
                } else {
                    // Recursively check subdirectories
                    let mut sub_empty = self.find_empty_directories(&path)?;
                    empty_dirs.append(&mut sub_empty);
                }
            }
        }

        Ok(empty_dirs)
    }

    /// Compress a file (simplified implementation)
    fn compress_file(&self, _path: &Path) -> SaveResult<u64> {
        // In a real implementation, this would compress the file using gzip or similar
        // For now, return 0 space saved
        Ok(0)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: SaveCleanupConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &SaveCleanupConfig {
        &self.config
    }

    /// Get cleanup statistics
    pub fn get_statistics(&self) -> SaveResult<CleanupStatistics> {
        let all_files = self.scan_all_files()?;
        
        let total_files = all_files.len();
        let total_size: u64 = all_files.iter().map(|f| f.size_bytes).sum();
        
        let mut file_type_counts = HashMap::new();
        let mut file_type_sizes = HashMap::new();
        
        for file_info in &all_files {
            *file_type_counts.entry(file_info.file_type.clone()).or_insert(0) += 1;
            *file_type_sizes.entry(file_info.file_type.clone()).or_insert(0u64) += file_info.size_bytes;
        }

        let old_files_count = all_files.iter()
            .filter(|f| f.age_days > self.config.max_save_age_days)
            .count();
        
        let important_files_count = all_files.iter()
            .filter(|f| f.is_important)
            .count();

        Ok(CleanupStatistics {
            total_files,
            total_size_bytes: total_size,
            file_type_counts,
            file_type_sizes,
            old_files_count,
            important_files_count,
            last_cleanup: self.last_cleanup,
            config: self.config.clone(),
        })
    }
}

/// Cleanup statistics
#[derive(Debug, Clone)]
pub struct CleanupStatistics {
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub file_type_counts: HashMap<CleanupFileType, usize>,
    pub file_type_sizes: HashMap<CleanupFileType, u64>,
    pub old_files_count: usize,
    pub important_files_count: usize,
    pub last_cleanup: SystemTime,
    pub config: SaveCleanupConfig,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    fn create_test_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let path = dir.join(filename);
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_save_cleanup_system_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveCleanupConfig::default();
        
        let system = SaveCleanupSystem::new(temp_dir.path(), config);
        assert!(system.is_ok());
    }

    #[test]
    fn test_file_type_determination() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveCleanupConfig::default();
        let system = SaveCleanupSystem::new(temp_dir.path(), config).unwrap();
        
        let save_path = create_test_file(temp_dir.path(), "save.sav", "save data");
        let backup_path = create_test_file(temp_dir.path(), "backup.bak", "backup data");
        let temp_path = create_test_file(temp_dir.path(), "temp.tmp", "temp data");
        
        assert_eq!(system.determine_file_type(&save_path), CleanupFileType::SaveFile);
        assert_eq!(system.determine_file_type(&backup_path), CleanupFileType::BackupFile);
        assert_eq!(system.determine_file_type(&temp_path), CleanupFileType::TempFile);
    }

    #[test]
    fn test_important_file_detection() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveCleanupConfig::default();
        let system = SaveCleanupSystem::new(temp_dir.path(), config).unwrap();
        
        let manual_save = create_test_file(temp_dir.path(), "manual_save.sav", "manual save");
        let autosave = create_test_file(temp_dir.path(), "autosave.sav", "autosave");
        
        assert!(system.is_important_file(&manual_save, &CleanupFileType::SaveFile, 1));
        assert!(!system.is_important_file(&autosave, &CleanupFileType::SaveFile, 1));
    }

    #[test]
    fn test_cleanup_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let config = SaveCleanupConfig::default();
        let system = SaveCleanupSystem::new(temp_dir.path(), config).unwrap();
        
        // Create test files
        create_test_file(temp_dir.path(), "save1.sav", "save data 1");
        create_test_file(temp_dir.path(), "save2.sav", "save data 2");
        create_test_file(temp_dir.path(), "backup.bak", "backup data");
        
        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_files, 3);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_should_run_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SaveCleanupConfig::default();
        config.cleanup_interval_hours = 0; // Should always run
        
        let system = SaveCleanupSystem::new(temp_dir.path(), config).unwrap();
        assert!(system.should_run_cleanup());
    }

    #[test]
    fn test_disabled_cleanup() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = SaveCleanupConfig::default();
        config.enabled = false;
        
        let system = SaveCleanupSystem::new(temp_dir.path(), config).unwrap();
        assert!(!system.should_run_cleanup());
    }
}