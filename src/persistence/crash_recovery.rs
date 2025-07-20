use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use serde::{Serialize, Deserialize};
use specs::World;
use crate::persistence::{
    save_system::{SaveResult, SaveError, SaveData, SaveMetadata},
    world_serializer::WorldSerializer,
    serialization::SerializationSystem,
};
use crate::game_state::GameState;

/// Crash recovery save data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecoverySave {
    pub timestamp: u64,
    pub session_id: String,
    pub save_data: SaveData,
    pub metadata: SaveMetadata,
    pub recovery_reason: CrashRecoveryReason,
}

/// Reasons for crash recovery saves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrashRecoveryReason {
    PeriodicBackup,
    UnexpectedShutdown,
    GameCrash,
    SystemCrash,
    UserRequested,
}

/// Crash recovery system for automatic game state preservation
pub struct CrashRecoverySystem {
    recovery_directory: PathBuf,
    world_serializer: WorldSerializer,
    session_id: String,
    max_recovery_saves: usize,
    recovery_interval_seconds: u64,
    last_recovery_save: SystemTime,
    enabled: bool,
}

impl CrashRecoverySystem {
    pub fn new<P: AsRef<Path>>(
        recovery_directory: P,
        serialization_system: SerializationSystem,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let recovery_dir = recovery_directory.as_ref().to_path_buf();
        
        // Create recovery directory if it doesn't exist
        fs::create_dir_all(&recovery_dir)?;
        
        // Generate unique session ID
        let session_id = format!("session_{}", 
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        );

        Ok(CrashRecoverySystem {
            recovery_directory: recovery_dir,
            world_serializer: WorldSerializer::new(serialization_system),
            session_id,
            max_recovery_saves: 10,
            recovery_interval_seconds: 60, // 1 minute
            last_recovery_save: SystemTime::now(),
            enabled: true,
        })
    }

    /// Enable or disable crash recovery
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Set recovery interval
    pub fn set_recovery_interval(&mut self, seconds: u64) {
        self.recovery_interval_seconds = seconds;
    }

    /// Set maximum number of recovery saves to keep
    pub fn set_max_recovery_saves(&mut self, max: usize) {
        self.max_recovery_saves = max;
    }

    /// Check if recovery save should be created
    pub fn should_create_recovery_save(&self) -> bool {
        if !self.enabled {
            return false;
        }

        let elapsed = self.last_recovery_save
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        elapsed >= self.recovery_interval_seconds
    }

    /// Create a crash recovery save
    pub fn create_recovery_save(
        &mut self,
        world: &World,
        reason: CrashRecoveryReason,
    ) -> SaveResult<PathBuf> {
        if !self.enabled {
            return Err(SaveError::InvalidOperation("Crash recovery is disabled".to_string()));
        }

        // Serialize world state
        let world_state = self.world_serializer.serialize_world(world)
            .map_err(|e| SaveError::SerializationError(e))?;

        // Create metadata
        let metadata = self.create_recovery_metadata(world, &reason)?;

        // Create save data
        let save_data = SaveData::new(
            format!("Recovery Save - {}", metadata.player_name),
            metadata.player_name.clone(),
        )
        .with_components(world_state.components)
        .with_resources(world_state.resources);

        // Create recovery save
        let recovery_save = CrashRecoverySave {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            session_id: self.session_id.clone(),
            save_data,
            metadata,
            recovery_reason: reason,
        };

        // Generate filename
        let filename = format!(
            "recovery_{}_{}.dat",
            self.session_id,
            recovery_save.timestamp
        );
        let file_path = self.recovery_directory.join(filename);

        // Save to file
        let file = fs::File::create(&file_path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        bincode::serialize_into(file, &recovery_save)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        // Update last recovery save time
        self.last_recovery_save = SystemTime::now();

        // Clean up old recovery saves
        self.cleanup_old_recovery_saves()?;

        Ok(file_path)
    }

    /// Create recovery save metadata
    fn create_recovery_metadata(
        &self,
        world: &World,
        reason: &CrashRecoveryReason,
    ) -> SaveResult<SaveMetadata> {
        // Extract basic info from world (simplified for this example)
        let player_name = "Player".to_string(); // In real implementation, extract from world
        let save_name = format!("Recovery Save - {:?}", reason);

        let mut metadata = SaveMetadata::new(save_name, player_name);
        metadata.character_level = 1; // Extract from world
        metadata.current_depth = 1; // Extract from world
        
        Ok(metadata)
    }

    /// Get all recovery saves for current session
    pub fn get_session_recovery_saves(&self) -> SaveResult<Vec<CrashRecoverySave>> {
        let mut recovery_saves = Vec::new();

        let entries = fs::read_dir(&self.recovery_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with(&format!("recovery_{}_", self.session_id)) {
                    if let Ok(recovery_save) = self.load_recovery_save(&path) {
                        recovery_saves.push(recovery_save);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        recovery_saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(recovery_saves)
    }

    /// Get all recovery saves (all sessions)
    pub fn get_all_recovery_saves(&self) -> SaveResult<Vec<CrashRecoverySave>> {
        let mut recovery_saves = Vec::new();

        let entries = fs::read_dir(&self.recovery_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("recovery_") && filename.ends_with(".dat") {
                    if let Ok(recovery_save) = self.load_recovery_save(&path) {
                        recovery_saves.push(recovery_save);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        recovery_saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(recovery_saves)
    }

    /// Load a recovery save from file
    fn load_recovery_save(&self, path: &Path) -> SaveResult<CrashRecoverySave> {
        let file = fs::File::open(path)
            .map_err(|e| SaveError::IoError(e.to_string()))?;
        
        bincode::deserialize_from(file)
            .map_err(|e| SaveError::InvalidSaveFile(e.to_string()))
    }

    /// Restore game state from recovery save
    pub fn restore_from_recovery_save(
        &self,
        world: &mut World,
        recovery_save: &CrashRecoverySave,
    ) -> SaveResult<()> {
        // Clear current world state
        world.delete_all();

        // Deserialize components
        self.world_serializer
            .serialization_system
            .deserialize_world(world, &recovery_save.save_data.components)
            .map_err(|e| SaveError::SerializationError(e))?;

        Ok(())
    }

    /// Clean up old recovery saves
    fn cleanup_old_recovery_saves(&self) -> SaveResult<()> {
        let mut all_saves = self.get_all_recovery_saves()?;
        
        if all_saves.len() <= self.max_recovery_saves {
            return Ok(());
        }

        // Sort by timestamp (oldest first for deletion)
        all_saves.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        // Delete oldest saves beyond the limit
        let to_delete = all_saves.len() - self.max_recovery_saves;
        for recovery_save in all_saves.iter().take(to_delete) {
            let filename = format!(
                "recovery_{}_{}.dat",
                recovery_save.session_id,
                recovery_save.timestamp
            );
            let file_path = self.recovery_directory.join(filename);
            
            if file_path.exists() {
                fs::remove_file(file_path)
                    .map_err(|e| SaveError::IoError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Clean up recovery saves older than specified days
    pub fn cleanup_old_recovery_saves_by_age(&self, max_age_days: u64) -> SaveResult<usize> {
        let cutoff_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_sub(max_age_days * 24 * 60 * 60);

        let all_saves = self.get_all_recovery_saves()?;
        let mut deleted_count = 0;

        for recovery_save in all_saves {
            if recovery_save.timestamp < cutoff_timestamp {
                let filename = format!(
                    "recovery_{}_{}.dat",
                    recovery_save.session_id,
                    recovery_save.timestamp
                );
                let file_path = self.recovery_directory.join(filename);
                
                if file_path.exists() {
                    fs::remove_file(file_path)
                        .map_err(|e| SaveError::IoError(e.to_string()))?;
                    deleted_count += 1;
                }
            }
        }

        Ok(deleted_count)
    }

    /// Check for recovery saves from previous sessions
    pub fn check_for_crash_recovery(&self) -> SaveResult<Vec<CrashRecoverySave>> {
        let all_saves = self.get_all_recovery_saves()?;
        
        // Filter out current session saves
        let other_session_saves: Vec<CrashRecoverySave> = all_saves
            .into_iter()
            .filter(|save| save.session_id != self.session_id)
            .collect();

        Ok(other_session_saves)
    }

    /// Update recovery system (call this periodically)
    pub fn update(&mut self, world: &World) -> SaveResult<bool> {
        if self.should_create_recovery_save() {
            self.create_recovery_save(world, CrashRecoveryReason::PeriodicBackup)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Create emergency recovery save (for unexpected shutdowns)
    pub fn create_emergency_save(&mut self, world: &World) -> SaveResult<PathBuf> {
        self.create_recovery_save(world, CrashRecoveryReason::UnexpectedShutdown)
    }

    /// Get recovery directory
    pub fn get_recovery_directory(&self) -> &Path {
        &self.recovery_directory
    }

    /// Get current session ID
    pub fn get_session_id(&self) -> &str {
        &self.session_id
    }

    /// Get recovery statistics
    pub fn get_statistics(&self) -> SaveResult<CrashRecoveryStatistics> {
        let all_saves = self.get_all_recovery_saves()?;
        let session_saves = self.get_session_recovery_saves()?;

        Ok(CrashRecoveryStatistics {
            total_recovery_saves: all_saves.len(),
            current_session_saves: session_saves.len(),
            oldest_save_timestamp: all_saves.last().map(|s| s.timestamp),
            newest_save_timestamp: all_saves.first().map(|s| s.timestamp),
            recovery_directory_size: self.calculate_directory_size()?,
            enabled: self.enabled,
            recovery_interval_seconds: self.recovery_interval_seconds,
            max_recovery_saves: self.max_recovery_saves,
        })
    }

    /// Calculate total size of recovery directory
    fn calculate_directory_size(&self) -> SaveResult<u64> {
        let mut total_size = 0;

        let entries = fs::read_dir(&self.recovery_directory)
            .map_err(|e| SaveError::IoError(e.to_string()))?;

        for entry in entries {
            let entry = entry.map_err(|e| SaveError::IoError(e.to_string()))?;
            let metadata = entry.metadata()
                .map_err(|e| SaveError::IoError(e.to_string()))?;
            total_size += metadata.len();
        }

        Ok(total_size)
    }
}

/// Crash recovery statistics
#[derive(Debug, Clone)]
pub struct CrashRecoveryStatistics {
    pub total_recovery_saves: usize,
    pub current_session_saves: usize,
    pub oldest_save_timestamp: Option<u64>,
    pub newest_save_timestamp: Option<u64>,
    pub recovery_directory_size: u64,
    pub enabled: bool,
    pub recovery_interval_seconds: u64,
    pub max_recovery_saves: usize,
}

/// Crash recovery manager for integration with game systems
pub struct CrashRecoveryManager {
    recovery_system: CrashRecoverySystem,
}

impl CrashRecoveryManager {
    pub fn new(
        recovery_directory: PathBuf,
        serialization_system: SerializationSystem,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(CrashRecoveryManager {
            recovery_system: CrashRecoverySystem::new(recovery_directory, serialization_system)?,
        })
    }

    /// Update crash recovery system
    pub fn update(&mut self, game_state: &GameState) -> SaveResult<bool> {
        self.recovery_system.update(&game_state.world)
    }

    /// Check for crash recovery on startup
    pub fn check_startup_recovery(&self) -> SaveResult<Vec<CrashRecoverySave>> {
        self.recovery_system.check_for_crash_recovery()
    }

    /// Create emergency save (call before shutdown)
    pub fn create_emergency_save(&mut self, game_state: &GameState) -> SaveResult<PathBuf> {
        self.recovery_system.create_emergency_save(&game_state.world)
    }

    /// Restore from recovery save
    pub fn restore_from_recovery(
        &self,
        game_state: &mut GameState,
        recovery_save: &CrashRecoverySave,
    ) -> SaveResult<()> {
        self.recovery_system.restore_from_recovery_save(&mut game_state.world, recovery_save)
    }

    /// Configure recovery system
    pub fn configure(&mut self, enabled: bool, interval_seconds: u64, max_saves: usize) {
        self.recovery_system.set_enabled(enabled);
        self.recovery_system.set_recovery_interval(interval_seconds);
        self.recovery_system.set_max_recovery_saves(max_saves);
    }

    /// Get recovery statistics
    pub fn get_statistics(&self) -> SaveResult<CrashRecoveryStatistics> {
        self.recovery_system.get_statistics()
    }

    /// Clean up old recovery saves
    pub fn cleanup_old_saves(&self, max_age_days: u64) -> SaveResult<usize> {
        self.recovery_system.cleanup_old_recovery_saves_by_age(max_age_days)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::persistence::serialization;
    use specs::{World, WorldExt};

    fn create_test_world() -> World {
        let mut world = World::new();
        // Add some test components/resources
        world
    }

    fn create_test_recovery_system() -> (CrashRecoverySystem, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let serialization_system = serialization::create_serialization_system();
        let recovery_system = CrashRecoverySystem::new(temp_dir.path(), serialization_system).unwrap();
        (recovery_system, temp_dir)
    }

    #[test]
    fn test_crash_recovery_system_creation() {
        let (system, _temp_dir) = create_test_recovery_system();
        
        assert!(system.enabled);
        assert_eq!(system.recovery_interval_seconds, 60);
        assert_eq!(system.max_recovery_saves, 10);
        assert!(!system.session_id.is_empty());
    }

    #[test]
    fn test_recovery_save_creation() {
        let (mut system, _temp_dir) = create_test_recovery_system();
        let world = create_test_world();
        
        let result = system.create_recovery_save(&world, CrashRecoveryReason::UserRequested);
        assert!(result.is_ok());
        
        let file_path = result.unwrap();
        assert!(file_path.exists());
    }

    #[test]
    fn test_recovery_save_loading() {
        let (mut system, _temp_dir) = create_test_recovery_system();
        let world = create_test_world();
        
        // Create a recovery save
        system.create_recovery_save(&world, CrashRecoveryReason::PeriodicBackup).unwrap();
        
        // Load recovery saves
        let saves = system.get_session_recovery_saves().unwrap();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].recovery_reason, CrashRecoveryReason::PeriodicBackup);
    }

    #[test]
    fn test_recovery_save_cleanup() {
        let (mut system, _temp_dir) = create_test_recovery_system();
        system.set_max_recovery_saves(2);
        
        let world = create_test_world();
        
        // Create more saves than the limit
        for _ in 0..5 {
            system.create_recovery_save(&world, CrashRecoveryReason::PeriodicBackup).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different timestamps
        }
        
        let saves = system.get_session_recovery_saves().unwrap();
        assert_eq!(saves.len(), 2); // Should be limited to max_recovery_saves
    }

    #[test]
    fn test_recovery_statistics() {
        let (mut system, _temp_dir) = create_test_recovery_system();
        let world = create_test_world();
        
        // Create some recovery saves
        system.create_recovery_save(&world, CrashRecoveryReason::PeriodicBackup).unwrap();
        system.create_recovery_save(&world, CrashRecoveryReason::UserRequested).unwrap();
        
        let stats = system.get_statistics().unwrap();
        assert_eq!(stats.total_recovery_saves, 2);
        assert_eq!(stats.current_session_saves, 2);
        assert!(stats.enabled);
    }

    #[test]
    fn test_crash_recovery_manager() {
        let temp_dir = TempDir::new().unwrap();
        let serialization_system = serialization::create_serialization_system();
        
        let manager = CrashRecoveryManager::new(temp_dir.path().to_path_buf(), serialization_system);
        assert!(manager.is_ok());
        
        let manager = manager.unwrap();
        let stats = manager.get_statistics().unwrap();
        assert!(stats.enabled);
    }

    #[test]
    fn test_emergency_save() {
        let (mut system, _temp_dir) = create_test_recovery_system();
        let world = create_test_world();
        
        let result = system.create_emergency_save(&world);
        assert!(result.is_ok());
        
        let saves = system.get_session_recovery_saves().unwrap();
        assert_eq!(saves.len(), 1);
        assert_eq!(saves[0].recovery_reason, CrashRecoveryReason::UnexpectedShutdown);
    }

    #[test]
    fn test_recovery_interval() {
        let (system, _temp_dir) = create_test_recovery_system();
        
        // Should not trigger immediately
        assert!(!system.should_create_recovery_save());
    }

    #[test]
    fn test_disabled_recovery() {
        let (mut system, _temp_dir) = create_test_recovery_system();
        system.set_enabled(false);
        
        let world = create_test_world();
        let result = system.create_recovery_save(&world, CrashRecoveryReason::PeriodicBackup);
        
        assert!(result.is_err());
        assert!(!system.should_create_recovery_save());
    }
}