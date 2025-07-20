/// Comprehensive example of autosave functionality integration
/// This demonstrates how to set up and use the complete autosave system with
/// crash recovery, save rotation, and cleanup features.

use std::path::PathBuf;
use crate::{
    game_state::GameState,
    persistence::{
        autosave_system::{AutosaveManager, AutosaveConfig, ComprehensiveAutosaveStatistics},
        save_load_system::SaveLoadSystem,
        crash_recovery::{CrashRecoveryManager, CrashRecoveryReason},
        save_rotation::{SaveRotationSystem, SaveRotationConfig, RotationStrategy},
        save_cleanup::{SaveCleanupSystem, SaveCleanupConfig},
        serialization,
    },
};

/// Complete autosave system setup
pub struct CompleteAutosaveSystem {
    autosave_manager: AutosaveManager,
    save_directory: PathBuf,
}

impl CompleteAutosaveSystem {
    /// Create a new complete autosave system
    pub fn new(save_directory: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Create save directory structure
        std::fs::create_dir_all(&save_directory)?;
        std::fs::create_dir_all(save_directory.join("recovery"))?;
        std::fs::create_dir_all(save_directory.join("backups"))?;

        // Configure autosave
        let autosave_config = AutosaveConfig {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            max_autosaves: 5,
            autosave_on_level_change: true,
            autosave_on_significant_events: true,
            backup_before_autosave: true,
        };

        // Configure save rotation
        let rotation_config = SaveRotationConfig {
            max_saves_per_slot: 10,
            max_total_saves: 100,
            max_age_days: 90,
            compress_old_saves: true,
            backup_before_rotation: true,
            rotation_strategy: RotationStrategy::TimeBasedWithCount,
        };

        // Configure cleanup
        let cleanup_config = SaveCleanupConfig {
            enabled: true,
            cleanup_interval_hours: 24, // Daily
            max_total_save_size_mb: 1000, // 1GB
            max_save_age_days: 180, // 6 months
            keep_important_saves: true,
            cleanup_empty_directories: true,
            cleanup_temp_files: true,
            cleanup_crash_recovery: true,
            max_crash_recovery_age_days: 14, // 2 weeks
            compress_old_saves: true,
            compression_age_days: 60, // 2 months
        };

        // Create systems
        let save_load_system = SaveLoadSystem::new(&save_directory)?;
        
        let serialization_system = serialization::create_serialization_system();
        let crash_recovery_manager = CrashRecoveryManager::new(
            save_directory.join("recovery"),
            serialization_system,
        )?;
        
        let save_rotation_system = SaveRotationSystem::new(&save_directory, rotation_config)?;
        let save_cleanup_system = SaveCleanupSystem::new(&save_directory, cleanup_config)?;

        // Create comprehensive autosave manager
        let autosave_manager = AutosaveManager::new_with_full_features(
            autosave_config,
            save_load_system,
            crash_recovery_manager,
            save_rotation_system,
            save_cleanup_system,
        );

        Ok(CompleteAutosaveSystem {
            autosave_manager,
            save_directory,
        })
    }

    /// Initialize system and check for crash recovery
    pub fn initialize(&mut self, game_state: &mut GameState) -> Result<InitializationResult, Box<dyn std::error::Error>> {
        let mut result = InitializationResult::new();

        // Check for crash recovery saves
        match self.autosave_manager.check_crash_recovery() {
            Ok(recovery_saves) => {
                if !recovery_saves.is_empty() {
                    result.crash_recovery_available = true;
                    result.recovery_saves_count = recovery_saves.len();
                    result.recovery_saves = recovery_saves;
                    
                    println!("Found {} crash recovery saves from previous sessions", recovery_saves.len());
                    for (i, save) in result.recovery_saves.iter().enumerate() {
                        println!("  {}: {} - {} ({:?})", 
                            i + 1,
                            save.metadata.player_name,
                            save.metadata.save_name,
                            save.recovery_reason
                        );
                    }
                }
            },
            Err(e) => {
                result.errors.push(format!("Failed to check crash recovery: {}", e));
            }
        }

        // Get initial statistics
        match self.autosave_manager.get_comprehensive_statistics() {
            Ok(stats) => {
                result.initial_statistics = Some(stats);
            },
            Err(e) => {
                result.errors.push(format!("Failed to get statistics: {}", e));
            }
        }

        Ok(result)
    }

    /// Update the autosave system (call this every frame or regularly)
    pub fn update(&mut self, game_state: &mut GameState) -> Result<UpdateResult, Box<dyn std::error::Error>> {
        let mut result = UpdateResult::new();

        // Update autosave system
        match self.autosave_manager.update(game_state) {
            Ok(autosaved) => {
                result.autosave_performed = autosaved;
                if autosaved {
                    result.messages.push("Autosave completed".to_string());
                }
            },
            Err(e) => {
                result.errors.push(format!("Autosave failed: {}", e));
            }
        }

        Ok(result)
    }

    /// Handle game events that should trigger autosaves
    pub fn handle_game_event(&mut self, event: &GameEvent) -> Result<(), Box<dyn std::error::Error>> {
        match event {
            GameEvent::PlayerLevelUp => {
                self.autosave_manager.trigger_on_event("Player leveled up".to_string());
            },
            GameEvent::BossDefeated(boss_name) => {
                self.autosave_manager.trigger_on_event(format!("Defeated boss: {}", boss_name));
            },
            GameEvent::DungeonEntered(level) => {
                self.autosave_manager.trigger_on_event(format!("Entered dungeon level {}", level));
            },
            GameEvent::SignificantItemFound(item_name) => {
                self.autosave_manager.trigger_on_event(format!("Found significant item: {}", item_name));
            },
            GameEvent::QuestCompleted(quest_name) => {
                self.autosave_manager.trigger_on_event(format!("Completed quest: {}", quest_name));
            },
            GameEvent::GameSaved => {
                // Manual save occurred, no additional action needed
            },
        }

        Ok(())
    }

    /// Prepare for game shutdown
    pub fn prepare_shutdown(&mut self, game_state: &GameState) -> Result<ShutdownResult, Box<dyn std::error::Error>> {
        let mut result = ShutdownResult::new();

        // Create emergency save
        match self.autosave_manager.create_emergency_save(game_state) {
            Ok(()) => {
                result.emergency_save_created = true;
                result.messages.push("Emergency save created".to_string());
            },
            Err(e) => {
                result.errors.push(format!("Failed to create emergency save: {}", e));
            }
        }

        // Perform final cleanup
        match self.autosave_manager.cleanup_saves() {
            Ok(cleanup_result) => {
                result.cleanup_performed = true;
                result.files_cleaned = cleanup_result.files_deleted.len();
                result.space_freed = cleanup_result.space_freed_bytes;
                result.messages.push(format!("Cleanup completed: {} files removed, {} bytes freed", 
                    cleanup_result.files_deleted.len(), cleanup_result.space_freed_bytes));
            },
            Err(e) => {
                result.errors.push(format!("Cleanup failed: {}", e));
            }
        }

        Ok(result)
    }

    /// Restore from crash recovery save
    pub fn restore_from_crash_recovery(
        &self,
        game_state: &mut GameState,
        recovery_index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let recovery_saves = self.autosave_manager.check_crash_recovery()?;
        
        if recovery_index >= recovery_saves.len() {
            return Err("Invalid recovery save index".into());
        }

        let recovery_save = &recovery_saves[recovery_index];
        self.autosave_manager.restore_from_crash_recovery(game_state, recovery_save)?;

        println!("Restored game from crash recovery save: {}", recovery_save.metadata.save_name);
        Ok(())
    }

    /// Get comprehensive system status
    pub fn get_status(&self) -> Result<SystemStatus, Box<dyn std::error::Error>> {
        let stats = self.autosave_manager.get_comprehensive_statistics()?;
        
        Ok(SystemStatus {
            autosave_enabled: stats.autosave_status.enabled,
            time_until_next_autosave: stats.autosave_status.time_until_next,
            last_autosave_elapsed: stats.autosave_status.last_autosave_elapsed,
            pending_triggers: stats.autosave_status.pending_triggers,
            crash_recovery_available: stats.crash_recovery_stats.is_some(),
            total_recovery_saves: stats.crash_recovery_stats
                .as_ref()
                .map(|s| s.total_recovery_saves)
                .unwrap_or(0),
            total_save_files: stats.rotation_stats
                .as_ref()
                .map(|s| s.total_save_files)
                .unwrap_or(0),
            total_save_size: stats.rotation_stats
                .as_ref()
                .map(|s| s.total_size_bytes)
                .unwrap_or(0),
            cleanup_enabled: stats.cleanup_stats
                .as_ref()
                .map(|s| s.config.enabled)
                .unwrap_or(false),
            save_directory: self.save_directory.clone(),
        })
    }

    /// Manually trigger operations
    pub fn manual_operations(&mut self) -> Result<ManualOperationResult, Box<dyn std::error::Error>> {
        let mut result = ManualOperationResult::new();

        // Manual autosave
        self.autosave_manager.trigger_manual();
        result.manual_autosave_triggered = true;

        // Manual rotation
        match self.autosave_manager.rotate_saves() {
            Ok(rotation_result) => {
                result.rotation_performed = true;
                result.files_rotated = rotation_result.deleted_files.len();
                result.space_freed_by_rotation = rotation_result.space_freed;
            },
            Err(e) => {
                result.errors.push(format!("Rotation failed: {}", e));
            }
        }

        // Manual cleanup
        match self.autosave_manager.cleanup_saves() {
            Ok(cleanup_result) => {
                result.cleanup_performed = true;
                result.files_cleaned = cleanup_result.files_deleted.len();
                result.space_freed_by_cleanup = cleanup_result.space_freed_bytes;
            },
            Err(e) => {
                result.errors.push(format!("Cleanup failed: {}", e));
            }
        }

        Ok(result)
    }

    /// Configure system settings
    pub fn configure_settings(
        &mut self,
        autosave_interval_minutes: Option<u64>,
        max_autosaves: Option<usize>,
        max_save_age_days: Option<u64>,
        enable_crash_recovery: Option<bool>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Get current config
        let current_config = self.autosave_manager.get_config();
        
        // Create new config with updates
        let mut new_autosave_config = current_config.clone();
        if let Some(interval) = autosave_interval_minutes {
            new_autosave_config.interval_seconds = interval * 60;
        }
        if let Some(max) = max_autosaves {
            new_autosave_config.max_autosaves = max;
        }

        let mut new_cleanup_config = SaveCleanupConfig::default();
        if let Some(max_age) = max_save_age_days {
            new_cleanup_config.max_save_age_days = max_age;
        }

        // Apply configuration
        self.autosave_manager.configure_all_systems(
            new_autosave_config,
            Some(SaveRotationConfig::default()),
            Some(new_cleanup_config),
        );

        Ok(())
    }
}

/// Game events that can trigger autosaves
#[derive(Debug, Clone)]
pub enum GameEvent {
    PlayerLevelUp,
    BossDefeated(String),
    DungeonEntered(u32),
    SignificantItemFound(String),
    QuestCompleted(String),
    GameSaved,
}

/// Result of system initialization
#[derive(Debug)]
pub struct InitializationResult {
    pub crash_recovery_available: bool,
    pub recovery_saves_count: usize,
    pub recovery_saves: Vec<crate::persistence::crash_recovery::CrashRecoverySave>,
    pub initial_statistics: Option<ComprehensiveAutosaveStatistics>,
    pub errors: Vec<String>,
}

impl InitializationResult {
    pub fn new() -> Self {
        InitializationResult {
            crash_recovery_available: false,
            recovery_saves_count: 0,
            recovery_saves: Vec::new(),
            initial_statistics: None,
            errors: Vec::new(),
        }
    }
}

/// Result of system update
#[derive(Debug)]
pub struct UpdateResult {
    pub autosave_performed: bool,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

impl UpdateResult {
    pub fn new() -> Self {
        UpdateResult {
            autosave_performed: false,
            messages: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// Result of shutdown preparation
#[derive(Debug)]
pub struct ShutdownResult {
    pub emergency_save_created: bool,
    pub cleanup_performed: bool,
    pub files_cleaned: usize,
    pub space_freed: u64,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

impl ShutdownResult {
    pub fn new() -> Self {
        ShutdownResult {
            emergency_save_created: false,
            cleanup_performed: false,
            files_cleaned: 0,
            space_freed: 0,
            messages: Vec::new(),
            errors: Vec::new(),
        }
    }
}

/// System status information
#[derive(Debug)]
pub struct SystemStatus {
    pub autosave_enabled: bool,
    pub time_until_next_autosave: std::time::Duration,
    pub last_autosave_elapsed: std::time::Duration,
    pub pending_triggers: usize,
    pub crash_recovery_available: bool,
    pub total_recovery_saves: usize,
    pub total_save_files: usize,
    pub total_save_size: u64,
    pub cleanup_enabled: bool,
    pub save_directory: PathBuf,
}

/// Result of manual operations
#[derive(Debug)]
pub struct ManualOperationResult {
    pub manual_autosave_triggered: bool,
    pub rotation_performed: bool,
    pub files_rotated: usize,
    pub space_freed_by_rotation: u64,
    pub cleanup_performed: bool,
    pub files_cleaned: usize,
    pub space_freed_by_cleanup: u64,
    pub errors: Vec<String>,
}

impl ManualOperationResult {
    pub fn new() -> Self {
        ManualOperationResult {
            manual_autosave_triggered: false,
            rotation_performed: false,
            files_rotated: 0,
            space_freed_by_rotation: 0,
            cleanup_performed: false,
            files_cleaned: 0,
            space_freed_by_cleanup: 0,
            errors: Vec::new(),
        }
    }
}

/// Example usage of the complete autosave system
pub fn example_usage() -> Result<(), Box<dyn std::error::Error>> {
    // Create save directory
    let save_dir = PathBuf::from("./game_saves");
    
    // Initialize complete autosave system
    let mut autosave_system = CompleteAutosaveSystem::new(save_dir)?;
    
    // Create a game state (simplified)
    let mut game_state = GameState::new();
    
    // Initialize system and check for crash recovery
    let init_result = autosave_system.initialize(&mut game_state)?;
    
    if init_result.crash_recovery_available {
        println!("Crash recovery saves found!");
        println!("Would you like to restore from a previous session? (y/n)");
        
        // In a real game, you'd get user input here
        // For this example, we'll just show how to restore
        // autosave_system.restore_from_crash_recovery(&mut game_state, 0)?;
    }
    
    // Simulate game loop
    for frame in 0..1000 {
        // Update autosave system
        let update_result = autosave_system.update(&mut game_state)?;
        
        if update_result.autosave_performed {
            println!("Frame {}: Autosave performed", frame);
        }
        
        // Simulate game events
        if frame % 100 == 0 {
            autosave_system.handle_game_event(&GameEvent::PlayerLevelUp)?;
        }
        
        if frame % 250 == 0 {
            autosave_system.handle_game_event(&GameEvent::BossDefeated("Dragon".to_string()))?;
        }
        
        // Show status periodically
        if frame % 500 == 0 {
            let status = autosave_system.get_status()?;
            println!("Frame {}: Status - {} save files, {} bytes total", 
                frame, status.total_save_files, status.total_save_size);
        }
    }
    
    // Prepare for shutdown
    let shutdown_result = autosave_system.prepare_shutdown(&game_state)?;
    println!("Shutdown preparation: emergency save = {}, cleanup = {}", 
        shutdown_result.emergency_save_created, shutdown_result.cleanup_performed);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_complete_autosave_system_creation() {
        let temp_dir = TempDir::new().unwrap();
        let system = CompleteAutosaveSystem::new(temp_dir.path().to_path_buf());
        assert!(system.is_ok());
    }

    #[test]
    fn test_system_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let mut system = CompleteAutosaveSystem::new(temp_dir.path().to_path_buf()).unwrap();
        let mut game_state = GameState::new();
        
        let result = system.initialize(&mut game_state);
        assert!(result.is_ok());
        
        let init_result = result.unwrap();
        assert!(!init_result.crash_recovery_available); // No previous saves
    }

    #[test]
    fn test_game_event_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut system = CompleteAutosaveSystem::new(temp_dir.path().to_path_buf()).unwrap();
        
        let events = vec![
            GameEvent::PlayerLevelUp,
            GameEvent::BossDefeated("TestBoss".to_string()),
            GameEvent::DungeonEntered(5),
        ];
        
        for event in events {
            let result = system.handle_game_event(&event);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_system_status() {
        let temp_dir = TempDir::new().unwrap();
        let system = CompleteAutosaveSystem::new(temp_dir.path().to_path_buf()).unwrap();
        
        let status = system.get_status();
        assert!(status.is_ok());
        
        let status = status.unwrap();
        assert!(status.autosave_enabled);
    }

    #[test]
    fn test_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let mut system = CompleteAutosaveSystem::new(temp_dir.path().to_path_buf()).unwrap();
        
        let result = system.configure_settings(
            Some(10), // 10 minutes
            Some(3),  // 3 autosaves
            Some(30), // 30 days
            Some(true), // enable crash recovery
        );
        
        assert!(result.is_ok());
    }
}