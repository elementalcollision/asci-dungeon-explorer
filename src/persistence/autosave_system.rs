use std::time::{Duration, Instant};
use specs::World;
use crate::persistence::{
    save_load_system::SaveLoadSystem,
    save_system::{SaveResult, SaveError},
    crash_recovery::{CrashRecoveryManager, CrashRecoveryReason},
    save_rotation::{SaveRotationSystem, SaveRotationConfig},
    save_cleanup::{SaveCleanupSystem, SaveCleanupConfig},
};
use crate::game_state::GameState;
use crate::resources::GameLog;

/// Autosave configuration
#[derive(Debug, Clone)]
pub struct AutosaveConfig {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub max_autosaves: usize,
    pub autosave_on_level_change: bool,
    pub autosave_on_significant_events: bool,
    pub backup_before_autosave: bool,
}

impl Default for AutosaveConfig {
    fn default() -> Self {
        AutosaveConfig {
            enabled: true,
            interval_seconds: 300, // 5 minutes
            max_autosaves: 3,
            autosave_on_level_change: true,
            autosave_on_significant_events: true,
            backup_before_autosave: true,
        }
    }
}

/// Autosave trigger events
#[derive(Debug, Clone, PartialEq)]
pub enum AutosaveTrigger {
    Timer,
    LevelChange,
    SignificantEvent(String),
    Manual,
}

/// Autosave system for managing automatic game saves
pub struct AutosaveSystem {
    config: AutosaveConfig,
    last_autosave: Instant,
    last_level: i32,
    autosave_slots: Vec<u32>,
    current_autosave_index: usize,
    pending_triggers: Vec<AutosaveTrigger>,
}

impl AutosaveSystem {
    pub fn new(config: AutosaveConfig) -> Self {
        // Reserve slots for autosaves (e.g., slots 90-99)
        let autosave_slots: Vec<u32> = (90..90 + config.max_autosaves as u32).collect();
        
        AutosaveSystem {
            config,
            last_autosave: Instant::now(),
            last_level: 1,
            autosave_slots,
            current_autosave_index: 0,
            pending_triggers: Vec::new(),
        }
    }

    /// Update autosave system
    pub fn update(&mut self, game_state: &GameState, save_load_system: &mut SaveLoadSystem) -> SaveResult<bool> {
        if !self.config.enabled {
            return Ok(false);
        }

        // Check for timer-based autosave
        if self.should_autosave_by_timer() {
            self.pending_triggers.push(AutosaveTrigger::Timer);
        }

        // Check for level change autosave
        if self.config.autosave_on_level_change {
            let current_level = self.get_current_level(game_state);
            if current_level != self.last_level {
                self.pending_triggers.push(AutosaveTrigger::LevelChange);
                self.last_level = current_level;
            }
        }

        // Process pending triggers
        if !self.pending_triggers.is_empty() {
            let trigger = self.pending_triggers.remove(0);
            return self.perform_autosave(game_state, save_load_system, trigger);
        }

        Ok(false)
    }

    /// Check if autosave should trigger based on timer
    fn should_autosave_by_timer(&self) -> bool {
        self.last_autosave.elapsed() >= Duration::from_secs(self.config.interval_seconds)
    }

    /// Get current level from game state
    fn get_current_level(&self, game_state: &GameState) -> i32 {
        // In a real implementation, extract level from player position or game state
        // For now, return a placeholder
        1
    }

    /// Perform autosave
    fn perform_autosave(
        &mut self, 
        game_state: &GameState, 
        save_load_system: &mut SaveLoadSystem,
        trigger: AutosaveTrigger
    ) -> SaveResult<bool> {
        // Get next autosave slot
        let slot = self.get_next_autosave_slot();
        
        // Create backup if configured
        let backup_data = if self.config.backup_before_autosave {
            Some(save_load_system.world_serializer.create_snapshot(&game_state.world)
                .map_err(|e| SaveError::SerializationError(e))?)
        } else {
            None
        };

        // Perform the autosave
        match save_load_system.save_game(&game_state.world, slot, true) {
            Ok(()) => {
                self.last_autosave = Instant::now();
                self.log_autosave_success(game_state, slot, &trigger);
                Ok(true)
            },
            Err(e) => {
                // Restore backup if save failed and we have one
                if let Some(backup) = backup_data {
                    let _ = save_load_system.world_serializer.restore_from_snapshot(
                        &mut game_state.world.clone(), &backup
                    );
                }
                self.log_autosave_failure(game_state, &trigger, &e);
                Err(e)
            }
        }
    }

    /// Get next autosave slot using round-robin
    fn get_next_autosave_slot(&mut self) -> u32 {
        let slot = self.autosave_slots[self.current_autosave_index];
        self.current_autosave_index = (self.current_autosave_index + 1) % self.autosave_slots.len();
        slot
    }

    /// Log successful autosave
    fn log_autosave_success(&self, game_state: &GameState, slot: u32, trigger: &AutosaveTrigger) {
        let trigger_msg = match trigger {
            AutosaveTrigger::Timer => "timer",
            AutosaveTrigger::LevelChange => "level change",
            AutosaveTrigger::SignificantEvent(event) => event,
            AutosaveTrigger::Manual => "manual",
        };

        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
            game_log.entries.push(format!("Autosaved to slot {} ({})", slot, trigger_msg));
        }
    }

    /// Log autosave failure
    fn log_autosave_failure(&self, game_state: &GameState, trigger: &AutosaveTrigger, error: &SaveError) {
        let trigger_msg = match trigger {
            AutosaveTrigger::Timer => "timer",
            AutosaveTrigger::LevelChange => "level change",
            AutosaveTrigger::SignificantEvent(event) => event,
            AutosaveTrigger::Manual => "manual",
        };

        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
            game_log.entries.push(format!("Autosave failed ({}): {}", trigger_msg, error));
        }
    }

    /// Trigger autosave on significant event
    pub fn trigger_on_event(&mut self, event_description: String) {
        if self.config.enabled && self.config.autosave_on_significant_events {
            self.pending_triggers.push(AutosaveTrigger::SignificantEvent(event_description));
        }
    }

    /// Manually trigger autosave
    pub fn trigger_manual(&mut self) {
        if self.config.enabled {
            self.pending_triggers.push(AutosaveTrigger::Manual);
        }
    }

    /// Update configuration
    pub fn update_config(&mut self, config: AutosaveConfig) {
        // If max autosaves changed, update slots
        if config.max_autosaves != self.config.max_autosaves {
            self.autosave_slots = (90..90 + config.max_autosaves as u32).collect();
            self.current_autosave_index = 0;
        }

        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AutosaveConfig {
        &self.config
    }

    /// Get autosave slots
    pub fn get_autosave_slots(&self) -> &[u32] {
        &self.autosave_slots
    }

    /// Check if a slot is an autosave slot
    pub fn is_autosave_slot(&self, slot: u32) -> bool {
        self.autosave_slots.contains(&slot)
    }

    /// Get time until next autosave
    pub fn time_until_next_autosave(&self) -> Duration {
        let elapsed = self.last_autosave.elapsed();
        let interval = Duration::from_secs(self.config.interval_seconds);
        
        if elapsed >= interval {
            Duration::from_secs(0)
        } else {
            interval - elapsed
        }
    }

    /// Get autosave status
    pub fn get_status(&self) -> AutosaveStatus {
        AutosaveStatus {
            enabled: self.config.enabled,
            time_until_next: self.time_until_next_autosave(),
            last_autosave_elapsed: self.last_autosave.elapsed(),
            pending_triggers: self.pending_triggers.len(),
            current_slot: self.autosave_slots[self.current_autosave_index],
        }
    }

    /// Reset autosave timer
    pub fn reset_timer(&mut self) {
        self.last_autosave = Instant::now();
    }

    /// Clear pending triggers
    pub fn clear_pending_triggers(&mut self) {
        self.pending_triggers.clear();
    }
}

/// Autosave status information
#[derive(Debug, Clone)]
pub struct AutosaveStatus {
    pub enabled: bool,
    pub time_until_next: Duration,
    pub last_autosave_elapsed: Duration,
    pub pending_triggers: usize,
    pub current_slot: u32,
}

/// Autosave manager for integration with game systems
pub struct AutosaveManager {
    autosave_system: AutosaveSystem,
    save_load_system: SaveLoadSystem,
    crash_recovery_manager: Option<CrashRecoveryManager>,
    save_rotation_system: Option<SaveRotationSystem>,
    save_cleanup_system: Option<SaveCleanupSystem>,
}

impl AutosaveManager {
    pub fn new(
        autosave_config: AutosaveConfig,
        save_load_system: SaveLoadSystem,
    ) -> Self {
        AutosaveManager {
            autosave_system: AutosaveSystem::new(autosave_config),
            save_load_system,
            crash_recovery_manager: None,
            save_rotation_system: None,
            save_cleanup_system: None,
        }
    }

    /// Create autosave manager with full functionality
    pub fn new_with_full_features(
        autosave_config: AutosaveConfig,
        save_load_system: SaveLoadSystem,
        crash_recovery_manager: CrashRecoveryManager,
        save_rotation_system: SaveRotationSystem,
        save_cleanup_system: SaveCleanupSystem,
    ) -> Self {
        AutosaveManager {
            autosave_system: AutosaveSystem::new(autosave_config),
            save_load_system,
            crash_recovery_manager: Some(crash_recovery_manager),
            save_rotation_system: Some(save_rotation_system),
            save_cleanup_system: Some(save_cleanup_system),
        }
    }

    /// Update autosave manager
    pub fn update(&mut self, game_state: &mut GameState) -> SaveResult<bool> {
        let mut autosaved = false;

        // Update main autosave system
        if let Ok(saved) = self.autosave_system.update(game_state, &mut self.save_load_system) {
            autosaved = saved;
        }

        // Update crash recovery
        if let Some(ref mut crash_recovery) = self.crash_recovery_manager {
            let _ = crash_recovery.update(game_state);
        }

        // Perform save rotation if needed
        if let Some(ref save_rotation) = self.save_rotation_system {
            // Run rotation periodically (e.g., after every 10th autosave)
            static mut AUTOSAVE_COUNT: u32 = 0;
            unsafe {
                if autosaved {
                    AUTOSAVE_COUNT += 1;
                    if AUTOSAVE_COUNT % 10 == 0 {
                        let _ = save_rotation.rotate_saves();
                    }
                }
            }
        }

        // Perform cleanup if needed
        if let Some(ref mut cleanup_system) = self.save_cleanup_system {
            if cleanup_system.should_run_cleanup() {
                let _ = cleanup_system.run_cleanup();
            }
        }

        Ok(autosaved)
    }

    /// Configure autosave settings
    pub fn configure(&mut self, config: AutosaveConfig) {
        self.autosave_system.update_config(config);
    }

    /// Trigger autosave on event
    pub fn trigger_on_event(&mut self, event: String) {
        self.autosave_system.trigger_on_event(event);
    }

    /// Manual autosave trigger
    pub fn trigger_manual(&mut self) {
        self.autosave_system.trigger_manual();
    }

    /// Get autosave status
    pub fn get_status(&self) -> AutosaveStatus {
        self.autosave_system.get_status()
    }

    /// Get save/load system reference
    pub fn get_save_load_system(&mut self) -> &mut SaveLoadSystem {
        &mut self.save_load_system
    }

    /// Check if slot is autosave slot
    pub fn is_autosave_slot(&self, slot: u32) -> bool {
        self.autosave_system.is_autosave_slot(slot)
    }

    /// Create emergency save before shutdown
    pub fn create_emergency_save(&mut self, game_state: &GameState) -> SaveResult<()> {
        if let Some(ref mut crash_recovery) = self.crash_recovery_manager {
            crash_recovery.create_emergency_save(game_state)?;
        }
        Ok(())
    }

    /// Check for crash recovery on startup
    pub fn check_crash_recovery(&self) -> SaveResult<Vec<crate::persistence::crash_recovery::CrashRecoverySave>> {
        if let Some(ref crash_recovery) = self.crash_recovery_manager {
            crash_recovery.check_startup_recovery()
        } else {
            Ok(Vec::new())
        }
    }

    /// Restore from crash recovery save
    pub fn restore_from_crash_recovery(
        &self,
        game_state: &mut GameState,
        recovery_save: &crate::persistence::crash_recovery::CrashRecoverySave,
    ) -> SaveResult<()> {
        if let Some(ref crash_recovery) = self.crash_recovery_manager {
            crash_recovery.restore_from_recovery(game_state, recovery_save)
        } else {
            Err(SaveError::InvalidOperation("Crash recovery not available".to_string()))
        }
    }

    /// Manually trigger save rotation
    pub fn rotate_saves(&self) -> SaveResult<crate::persistence::save_rotation::RotationResult> {
        if let Some(ref rotation_system) = self.save_rotation_system {
            rotation_system.rotate_saves()
        } else {
            Ok(crate::persistence::save_rotation::RotationResult::new())
        }
    }

    /// Manually trigger cleanup
    pub fn cleanup_saves(&mut self) -> SaveResult<crate::persistence::save_cleanup::CleanupResult> {
        if let Some(ref mut cleanup_system) = self.save_cleanup_system {
            cleanup_system.run_cleanup()
        } else {
            Ok(crate::persistence::save_cleanup::CleanupResult::new())
        }
    }

    /// Get comprehensive autosave statistics
    pub fn get_comprehensive_statistics(&self) -> SaveResult<ComprehensiveAutosaveStatistics> {
        let autosave_status = self.get_status();
        
        let crash_recovery_stats = if let Some(ref crash_recovery) = self.crash_recovery_manager {
            Some(crash_recovery.get_statistics()?)
        } else {
            None
        };

        let rotation_stats = if let Some(ref rotation_system) = self.save_rotation_system {
            Some(rotation_system.get_statistics()?)
        } else {
            None
        };

        let cleanup_stats = if let Some(ref cleanup_system) = self.save_cleanup_system {
            Some(cleanup_system.get_statistics()?)
        } else {
            None
        };

        Ok(ComprehensiveAutosaveStatistics {
            autosave_status,
            crash_recovery_stats,
            rotation_stats,
            cleanup_stats,
        })
    }

    /// Configure all systems
    pub fn configure_all_systems(
        &mut self,
        autosave_config: AutosaveConfig,
        rotation_config: Option<SaveRotationConfig>,
        cleanup_config: Option<SaveCleanupConfig>,
    ) {
        self.configure(autosave_config);

        if let (Some(config), Some(ref mut rotation_system)) = (rotation_config, &mut self.save_rotation_system) {
            rotation_system.update_config(config);
        }

        if let (Some(config), Some(ref mut cleanup_system)) = (cleanup_config, &mut self.save_cleanup_system) {
            cleanup_system.update_config(config);
        }
    }
}

/// Comprehensive autosave statistics
#[derive(Debug, Clone)]
pub struct ComprehensiveAutosaveStatistics {
    pub autosave_status: AutosaveStatus,
    pub crash_recovery_stats: Option<crate::persistence::crash_recovery::CrashRecoveryStatistics>,
    pub rotation_stats: Option<crate::persistence::save_rotation::RotationStatistics>,
    pub cleanup_stats: Option<crate::persistence::save_cleanup::CleanupStatistics>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use crate::resources::GameLog;
    use tempfile::TempDir;
    use std::thread;

    fn create_test_game_state() -> GameState {
        let mut game_state = GameState::new();
        game_state.world.insert(GameLog::new());
        game_state
    }

    fn create_test_autosave_system() -> AutosaveSystem {
        let config = AutosaveConfig {
            enabled: true,
            interval_seconds: 1, // 1 second for testing
            max_autosaves: 3,
            autosave_on_level_change: true,
            autosave_on_significant_events: true,
            backup_before_autosave: true,
        };
        AutosaveSystem::new(config)
    }

    #[test]
    fn test_autosave_system_creation() {
        let system = create_test_autosave_system();
        
        assert!(system.config.enabled);
        assert_eq!(system.config.interval_seconds, 1);
        assert_eq!(system.config.max_autosaves, 3);
        assert_eq!(system.autosave_slots.len(), 3);
        assert_eq!(system.autosave_slots, vec![90, 91, 92]);
    }

    #[test]
    fn test_autosave_slot_rotation() {
        let mut system = create_test_autosave_system();
        
        assert_eq!(system.get_next_autosave_slot(), 90);
        assert_eq!(system.get_next_autosave_slot(), 91);
        assert_eq!(system.get_next_autosave_slot(), 92);
        assert_eq!(system.get_next_autosave_slot(), 90); // Should wrap around
    }

    #[test]
    fn test_timer_based_autosave() {
        let system = create_test_autosave_system();
        
        // Should not trigger immediately
        assert!(!system.should_autosave_by_timer());
        
        // Wait for timer (in real test, would mock time)
        thread::sleep(Duration::from_millis(1100));
        assert!(system.should_autosave_by_timer());
    }

    #[test]
    fn test_manual_trigger() {
        let mut system = create_test_autosave_system();
        
        assert_eq!(system.pending_triggers.len(), 0);
        
        system.trigger_manual();
        assert_eq!(system.pending_triggers.len(), 1);
        assert_eq!(system.pending_triggers[0], AutosaveTrigger::Manual);
    }

    #[test]
    fn test_event_trigger() {
        let mut system = create_test_autosave_system();
        
        system.trigger_on_event("Boss defeated".to_string());
        assert_eq!(system.pending_triggers.len(), 1);
        
        if let AutosaveTrigger::SignificantEvent(event) = &system.pending_triggers[0] {
            assert_eq!(event, "Boss defeated");
        } else {
            panic!("Expected SignificantEvent trigger");
        }
    }

    #[test]
    fn test_disabled_autosave() {
        let mut config = AutosaveConfig::default();
        config.enabled = false;
        
        let mut system = AutosaveSystem::new(config);
        
        // Should not trigger when disabled
        system.trigger_manual();
        assert_eq!(system.pending_triggers.len(), 0);
        
        system.trigger_on_event("Test event".to_string());
        assert_eq!(system.pending_triggers.len(), 0);
    }

    #[test]
    fn test_autosave_status() {
        let system = create_test_autosave_system();
        let status = system.get_status();
        
        assert!(status.enabled);
        assert_eq!(status.pending_triggers, 0);
        assert_eq!(status.current_slot, 90);
    }

    #[test]
    fn test_config_update() {
        let mut system = create_test_autosave_system();
        
        let mut new_config = AutosaveConfig::default();
        new_config.max_autosaves = 5;
        new_config.interval_seconds = 600;
        
        system.update_config(new_config);
        
        assert_eq!(system.config.max_autosaves, 5);
        assert_eq!(system.config.interval_seconds, 600);
        assert_eq!(system.autosave_slots.len(), 5);
        assert_eq!(system.autosave_slots, vec![90, 91, 92, 93, 94]);
    }

    #[test]
    fn test_is_autosave_slot() {
        let system = create_test_autosave_system();
        
        assert!(system.is_autosave_slot(90));
        assert!(system.is_autosave_slot(91));
        assert!(system.is_autosave_slot(92));
        assert!(!system.is_autosave_slot(0));
        assert!(!system.is_autosave_slot(50));
    }

    #[test]
    fn test_clear_pending_triggers() {
        let mut system = create_test_autosave_system();
        
        system.trigger_manual();
        system.trigger_on_event("Test".to_string());
        assert_eq!(system.pending_triggers.len(), 2);
        
        system.clear_pending_triggers();
        assert_eq!(system.pending_triggers.len(), 0);
    }

    #[test]
    fn test_autosave_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let save_load_system = SaveLoadSystem::new(temp_dir.path()).unwrap();
        let config = AutosaveConfig::default();
        
        let manager = AutosaveManager::new(config, save_load_system);
        
        let status = manager.get_status();
        assert!(status.enabled);
    }

    #[test]
    fn test_autosave_manager_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let save_load_system = SaveLoadSystem::new(temp_dir.path()).unwrap();
        let config = AutosaveConfig::default();
        
        let mut manager = AutosaveManager::new(config, save_load_system);
        
        let mut new_config = AutosaveConfig::default();
        new_config.enabled = false;
        
        manager.configure(new_config);
        
        let status = manager.get_status();
        assert!(!status.enabled);
    }
}