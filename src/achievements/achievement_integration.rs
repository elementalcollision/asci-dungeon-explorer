use crossterm::event::KeyCode;
use std::path::PathBuf;
use crate::{
    game_state::GameState,
    achievements::{
        AchievementSystem, AchievementUI, AchievementNotificationSystem, AchievementStorage,
        AchievementStorageConfig, NotificationConfig, GameEvent, AchievementSoundSystem,
        AchievementNotification, AchievementNotificationPopup,
    },
    ui::UIComponent,
};

/// Complete achievement system integration
pub struct AchievementIntegration {
    achievement_system: AchievementSystem,
    achievement_ui: AchievementUI,
    notification_system: AchievementNotificationSystem,
    storage: AchievementStorage,
    sound_system: AchievementSoundSystem,
    notification_popups: Vec<AchievementNotificationPopup>,
    player_id: String,
    enabled: bool,
}

impl AchievementIntegration {
    /// Create new achievement integration
    pub fn new(
        player_id: String,
        storage_config: AchievementStorageConfig,
        notification_config: NotificationConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut achievement_system = AchievementSystem::new();
        let achievement_ui = AchievementUI::new();
        let notification_system = AchievementNotificationSystem::new(notification_config);
        let storage = AchievementStorage::new(storage_config)?;
        let sound_system = AchievementSoundSystem::new();

        let mut integration = AchievementIntegration {
            achievement_system,
            achievement_ui,
            notification_system,
            storage,
            sound_system,
            notification_popups: Vec::new(),
            player_id,
            enabled: true,
        };

        // Load existing achievements
        integration.load_achievements()?;

        Ok(integration)
    }

    /// Update the achievement system
    pub fn update(&mut self, game_state: &GameState) -> Result<(), Box<dyn std::error::Error>> {
        if !self.enabled {
            return Ok(());
        }

        // Update notification system
        self.notification_system.update();

        // Process pending notifications from achievement system
        let pending_notifications = self.achievement_system.get_pending_notifications();
        for notification in pending_notifications {
            // Play sound
            self.sound_system.play_unlock_sound(&notification.rarity);
            
            // Add to notification system
            self.notification_system.add_notification(notification.clone());
            
            // Add to UI
            self.achievement_ui.add_notification(notification.clone());
            
            // Create popup
            let popup = AchievementNotificationPopup::new(notification, 3.0);
            self.notification_popups.push(popup);
        }

        // Update notification popups
        self.notification_popups.retain_mut(|popup| {
            popup.update(0.016) // Assuming 60 FPS
        });

        // Update UI notifications
        self.achievement_ui.update_notifications();

        // Auto-save if needed
        self.storage.update_auto_save(&self.achievement_system, &self.player_id)?;

        Ok(())
    }

    /// Handle input for achievement UI
    pub fn handle_input(&mut self, key: KeyCode) -> bool {
        if !self.enabled {
            return false;
        }

        // Handle achievement UI input
        if self.achievement_ui.is_open() {
            return self.achievement_ui.handle_input(key, &self.achievement_system);
        }

        // Handle global shortcuts
        match key {
            KeyCode::F1 => {
                self.achievement_ui.open();
                true
            },
            _ => false,
        }
    }

    /// Process game events for achievements
    pub fn process_game_event(&mut self, event: &GameEvent) {
        if !self.enabled {
            return;
        }

        self.achievement_system.process_game_event(event);
    }

    /// Open achievement UI
    pub fn open_achievement_ui(&mut self) {
        self.achievement_ui.open();
    }

    /// Close achievement UI
    pub fn close_achievement_ui(&mut self) {
        self.achievement_ui.close();
    }

    /// Check if achievement UI is open
    pub fn is_achievement_ui_open(&self) -> bool {
        self.achievement_ui.is_open()
    }

    /// Save achievements to storage
    pub fn save_achievements(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.storage.save_achievements(&self.achievement_system, &self.player_id)?;
        Ok(())
    }

    /// Load achievements from storage
    pub fn load_achievements(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.storage.load_achievements(&mut self.achievement_system, &self.player_id)?;
        Ok(())
    }

    /// Export achievements to file
    pub fn export_achievements(&self, export_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.storage.export_achievements(&self.achievement_system, &self.player_id, export_path)?;
        Ok(())
    }

    /// Import achievements from file
    pub fn import_achievements(&mut self, import_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.storage.import_achievements(&mut self.achievement_system, import_path)?;
        Ok(())
    }

    /// Get achievement statistics
    pub fn get_statistics(&self) -> &crate::achievements::AchievementStatistics {
        self.achievement_system.get_statistics()
    }

    /// Get storage statistics
    pub fn get_storage_statistics(&self) -> Result<crate::achievements::StorageStatistics, Box<dyn std::error::Error>> {
        Ok(self.storage.get_storage_statistics(&self.player_id)?)
    }

    /// Enable or disable achievement system
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if achievement system is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Configure notification settings
    pub fn configure_notifications(&mut self, config: NotificationConfig) {
        self.notification_system.update_config(config);
    }

    /// Configure storage settings
    pub fn configure_storage(&mut self, config: AchievementStorageConfig) -> Result<(), Box<dyn std::error::Error>> {
        self.storage.update_config(config)?;
        Ok(())
    }

    /// Enable or disable sounds
    pub fn set_sound_enabled(&mut self, enabled: bool) {
        self.sound_system.set_enabled(enabled);
    }

    /// Clear all notifications
    pub fn clear_notifications(&mut self) {
        self.notification_system.clear_all();
        self.notification_popups.clear();
    }

    /// Get notification count
    pub fn get_notification_count(&self) -> usize {
        self.notification_system.get_notification_count()
    }

    /// Repair corrupted achievement data
    pub fn repair_achievements(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        Ok(self.storage.repair_achievements(&mut self.achievement_system, &self.player_id)?)
    }

    /// Get achievement UI for rendering
    pub fn get_achievement_ui(&self) -> &AchievementUI {
        &self.achievement_ui
    }

    /// Get notification system for rendering
    pub fn get_notification_system(&self) -> &AchievementNotificationSystem {
        &self.notification_system
    }

    /// Get notification popups for rendering
    pub fn get_notification_popups(&self) -> &[AchievementNotificationPopup] {
        &self.notification_popups
    }

    /// Check if an achievement is unlocked
    pub fn is_achievement_unlocked(&self, achievement_id: &str) -> bool {
        self.achievement_system.is_unlocked(achievement_id)
    }

    /// Get achievement progress
    pub fn get_achievement_progress(&self, achievement_id: &str) -> Option<&crate::achievements::AchievementProgress> {
        self.achievement_system.get_progress(achievement_id)
    }

    /// Manually unlock an achievement (for testing/debugging)
    pub fn debug_unlock_achievement(&mut self, achievement_id: &str) {
        self.achievement_system.increment_progress(achievement_id, 1000); // Large number to ensure unlock
    }

    /// Get comprehensive system status
    pub fn get_system_status(&self) -> AchievementSystemStatus {
        let stats = self.get_statistics();
        let storage_stats = self.storage.get_storage_statistics(&self.player_id).ok();
        
        AchievementSystemStatus {
            enabled: self.enabled,
            total_achievements: stats.total_achievements,
            unlocked_achievements: stats.unlocked_achievements,
            completion_percentage: stats.completion_percentage,
            total_points: stats.total_points,
            earned_points: stats.earned_points,
            notification_count: self.get_notification_count(),
            ui_open: self.is_achievement_ui_open(),
            sound_enabled: self.sound_system.is_enabled(),
            storage_file_size: storage_stats.as_ref().map(|s| s.main_file_size).unwrap_or(0),
            backup_count: storage_stats.as_ref().map(|s| s.backup_count).unwrap_or(0),
            auto_save_enabled: storage_stats.as_ref().map(|s| s.auto_save_enabled).unwrap_or(false),
        }
    }
}

/// Achievement system status information
#[derive(Debug, Clone)]
pub struct AchievementSystemStatus {
    pub enabled: bool,
    pub total_achievements: usize,
    pub unlocked_achievements: usize,
    pub completion_percentage: f32,
    pub total_points: u32,
    pub earned_points: u32,
    pub notification_count: usize,
    pub ui_open: bool,
    pub sound_enabled: bool,
    pub storage_file_size: u64,
    pub backup_count: usize,
    pub auto_save_enabled: bool,
}

/// Convenience functions for common game events
impl AchievementIntegration {
    /// Player killed an enemy
    pub fn on_enemy_killed(&mut self) {
        self.process_game_event(&GameEvent::EnemyKilled);
    }

    /// Player defeated a boss
    pub fn on_boss_defeated(&mut self) {
        self.process_game_event(&GameEvent::BossDefeated);
    }

    /// Player moved
    pub fn on_player_moved(&mut self) {
        self.process_game_event(&GameEvent::PlayerMoved);
    }

    /// Player visited a new room
    pub fn on_room_visited(&mut self) {
        self.process_game_event(&GameEvent::RoomVisited);
    }

    /// Player level changed
    pub fn on_level_changed(&mut self, level: i32) {
        self.process_game_event(&GameEvent::LevelChanged(level));
    }

    /// Player collected gold
    pub fn on_gold_collected(&mut self, amount: u32) {
        self.process_game_event(&GameEvent::GoldCollected(amount));
    }

    /// Player collected an item
    pub fn on_item_collected(&mut self) {
        self.process_game_event(&GameEvent::ItemCollected);
    }

    /// Update playtime
    pub fn on_playtime_update(&mut self, seconds: u32) {
        self.process_game_event(&GameEvent::PlaytimeUpdate(seconds));
    }

    /// Player found a secret room
    pub fn on_secret_room_found(&mut self) {
        self.process_game_event(&GameEvent::SecretRoomFound);
    }

    /// Player found an easter egg
    pub fn on_easter_egg_found(&mut self) {
        self.process_game_event(&GameEvent::EasterEggFound);
    }

    /// Player completed a level without taking damage
    pub fn on_perfect_level(&mut self) {
        self.process_game_event(&GameEvent::PerfectLevel);
    }
}

/// Achievement integration builder for easy setup
pub struct AchievementIntegrationBuilder {
    player_id: String,
    storage_config: AchievementStorageConfig,
    notification_config: NotificationConfig,
}

impl AchievementIntegrationBuilder {
    pub fn new(player_id: String) -> Self {
        AchievementIntegrationBuilder {
            player_id,
            storage_config: AchievementStorageConfig::default(),
            notification_config: NotificationConfig::default(),
        }
    }

    pub fn with_storage_config(mut self, config: AchievementStorageConfig) -> Self {
        self.storage_config = config;
        self
    }

    pub fn with_notification_config(mut self, config: NotificationConfig) -> Self {
        self.notification_config = config;
        self
    }

    pub fn with_storage_directory(mut self, directory: PathBuf) -> Self {
        self.storage_config.storage_directory = directory;
        self
    }

    pub fn with_auto_save(mut self, enabled: bool, interval_seconds: u64) -> Self {
        self.storage_config.auto_save = enabled;
        self.storage_config.auto_save_interval_seconds = interval_seconds;
        self
    }

    pub fn with_backups(mut self, enabled: bool, count: usize) -> Self {
        self.storage_config.backup_enabled = enabled;
        self.storage_config.backup_count = count;
        self
    }

    pub fn with_notifications(mut self, style: crate::achievements::NotificationStyle, position: crate::achievements::NotificationPosition) -> Self {
        self.notification_config.style = style;
        self.notification_config.position = position;
        self
    }

    pub fn build(self) -> Result<AchievementIntegration, Box<dyn std::error::Error>> {
        AchievementIntegration::new(self.player_id, self.storage_config, self.notification_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_integration() -> AchievementIntegration {
        let temp_dir = TempDir::new().unwrap();
        let storage_config = AchievementStorageConfig {
            storage_directory: temp_dir.path().to_path_buf(),
            ..AchievementStorageConfig::default()
        };
        
        AchievementIntegration::new(
            "test_player".to_string(),
            storage_config,
            NotificationConfig::default(),
        ).unwrap()
    }

    #[test]
    fn test_integration_creation() {
        let integration = create_test_integration();
        assert!(integration.is_enabled());
        assert!(!integration.is_achievement_ui_open());
    }

    #[test]
    fn test_game_event_processing() {
        let mut integration = create_test_integration();
        
        // Process enemy kill event
        integration.on_enemy_killed();
        
        // Check if achievement was unlocked
        assert!(integration.is_achievement_unlocked("first_kill"));
    }

    #[test]
    fn test_ui_interaction() {
        let mut integration = create_test_integration();
        
        // Open UI
        integration.open_achievement_ui();
        assert!(integration.is_achievement_ui_open());
        
        // Close UI
        integration.close_achievement_ui();
        assert!(!integration.is_achievement_ui_open());
    }

    #[test]
    fn test_save_load() {
        let mut integration = create_test_integration();
        
        // Unlock an achievement
        integration.on_enemy_killed();
        
        // Save achievements
        integration.save_achievements().unwrap();
        
        // Create new integration and load
        let mut new_integration = create_test_integration();
        new_integration.load_achievements().unwrap();
        
        // Verify achievement was loaded
        assert!(new_integration.is_achievement_unlocked("first_kill"));
    }

    #[test]
    fn test_system_status() {
        let integration = create_test_integration();
        let status = integration.get_system_status();
        
        assert!(status.enabled);
        assert!(status.total_achievements > 0);
        assert_eq!(status.unlocked_achievements, 0);
        assert_eq!(status.completion_percentage, 0.0);
    }

    #[test]
    fn test_builder_pattern() {
        let temp_dir = TempDir::new().unwrap();
        
        let integration = AchievementIntegrationBuilder::new("test_player".to_string())
            .with_storage_directory(temp_dir.path().to_path_buf())
            .with_auto_save(true, 60)
            .with_backups(true, 3)
            .with_notifications(
                crate::achievements::NotificationStyle::Toast,
                crate::achievements::NotificationPosition::TopLeft
            )
            .build()
            .unwrap();
        
        assert!(integration.is_enabled());
    }

    #[test]
    fn test_enable_disable() {
        let mut integration = create_test_integration();
        
        assert!(integration.is_enabled());
        
        integration.set_enabled(false);
        assert!(!integration.is_enabled());
        
        // Events should be ignored when disabled
        integration.on_enemy_killed();
        assert!(!integration.is_achievement_unlocked("first_kill"));
    }

    #[test]
    fn test_notification_management() {
        let mut integration = create_test_integration();
        
        // Unlock achievement to generate notification
        integration.on_enemy_killed();
        
        // Update to process notifications
        let game_state = GameState::new();
        integration.update(&game_state).unwrap();
        
        assert!(integration.get_notification_count() > 0);
        
        // Clear notifications
        integration.clear_notifications();
        assert_eq!(integration.get_notification_count(), 0);
    }
}