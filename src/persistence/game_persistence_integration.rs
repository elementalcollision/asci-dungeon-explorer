use crossterm::event::KeyCode;
use std::path::PathBuf;
use crate::{
    game_state::{GameState, State},
    persistence::{
        save_load_system::{SaveLoadIntegration, SaveLoadSystem},
        autosave_system::{AutosaveManager, AutosaveConfig},
        save_system::{SaveResult, SaveError, SaveSlot},
    },
    ui::{
        save_load_ui::{SaveLoadUI, SaveLoadUIState, SaveLoadAction},
        UIComponent,
    },
    resources::GameLog,
};

/// Game persistence integration manager
pub struct GamePersistenceIntegration {
    save_load_integration: SaveLoadIntegration,
    autosave_manager: AutosaveManager,
    save_load_ui: SaveLoadUI,
    save_directory: PathBuf,
}

impl GamePersistenceIntegration {
    pub fn new(save_directory: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Create save directory if it doesn't exist
        std::fs::create_dir_all(&save_directory)?;

        // Initialize save/load system
        let save_load_integration = SaveLoadIntegration::new(&save_directory)?;
        
        // Initialize autosave system
        let autosave_config = AutosaveConfig::default();
        let save_load_system = SaveLoadSystem::new(&save_directory)?;
        let autosave_manager = AutosaveManager::new(autosave_config, save_load_system);
        
        // Initialize UI
        let save_load_ui = SaveLoadUI::new();

        Ok(GamePersistenceIntegration {
            save_load_integration,
            autosave_manager,
            save_load_ui,
            save_directory,
        })
    }

    /// Update persistence systems
    pub fn update(&mut self, game_state: &mut GameState) -> SaveResult<()> {
        // Update save/load integration
        self.save_load_integration.update(game_state)?;
        
        // Update autosave manager
        if let Ok(autosaved) = self.autosave_manager.update(game_state) {
            if autosaved {
                // Log autosave in game
                if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
                    game_log.entries.push("Game autosaved".to_string());
                }
            }
        }

        Ok(())
    }

    /// Handle input for save/load UI
    pub fn handle_input(&mut self, game_state: &mut GameState, key: KeyCode) -> SaveResult<bool> {
        if !self.save_load_ui.is_open() {
            return Ok(false);
        }

        if let Some(action) = self.save_load_ui.handle_input(key) {
            match action {
                SaveLoadAction::Save(slot) => {
                    match self.save_load_integration.save_game(game_state, slot) {
                        Ok(()) => {
                            self.save_load_ui.show_success(&format!("Game saved to slot {}", slot));
                            self.save_load_integration.set_current_save_slot(Some(slot));
                        },
                        Err(e) => {
                            self.save_load_ui.show_error(&format!("Save failed: {}", e));
                        }
                    }
                },
                SaveLoadAction::Load(slot) => {
                    match self.save_load_integration.load_game(game_state, slot) {
                        Ok(()) => {
                            self.save_load_ui.show_success(&format!("Game loaded from slot {}", slot));
                            self.save_load_integration.set_current_save_slot(Some(slot));
                        },
                        Err(e) => {
                            self.save_load_ui.show_error(&format!("Load failed: {}", e));
                        }
                    }
                },
                SaveLoadAction::Delete(slot) => {
                    match self.save_load_integration.delete_save(slot) {
                        Ok(()) => {
                            self.save_load_ui.show_success(&format!("Save slot {} deleted", slot));
                            // Refresh save slots
                            if let Ok(slots) = self.save_load_integration.get_save_slots() {
                                self.save_load_ui.update_save_slots(slots);
                            }
                        },
                        Err(e) => {
                            self.save_load_ui.show_error(&format!("Delete failed: {}", e));
                        }
                    }
                },
                SaveLoadAction::Export(path) => {
                    match self.save_load_integration.export_save(game_state, &path) {
                        Ok(()) => {
                            self.save_load_ui.show_success(&format!("Game exported to {}", path.display()));
                        },
                        Err(e) => {
                            self.save_load_ui.show_error(&format!("Export failed: {}", e));
                        }
                    }
                },
                SaveLoadAction::Import(path) => {
                    match self.save_load_integration.import_save(game_state, &path) {
                        Ok(()) => {
                            self.save_load_ui.show_success(&format!("Game imported from {}", path.display()));
                        },
                        Err(e) => {
                            self.save_load_ui.show_error(&format!("Import failed: {}", e));
                        }
                    }
                },
                SaveLoadAction::Cancel => {
                    self.save_load_ui.close();
                }
            }
            return Ok(true);
        }

        Ok(false)
    }

    /// Open save menu
    pub fn open_save_menu(&mut self) -> SaveResult<()> {
        let save_slots = self.save_load_integration.get_save_slots()?;
        self.save_load_ui.open_save_menu(save_slots);
        Ok(())
    }

    /// Open load menu
    pub fn open_load_menu(&mut self) -> SaveResult<()> {
        let save_slots = self.save_load_integration.get_save_slots()?;
        self.save_load_ui.open_load_menu(save_slots);
        Ok(())
    }

    /// Quick save
    pub fn quick_save(&mut self, game_state: &GameState) -> SaveResult<u32> {
        let slot = self.save_load_integration.quick_save(game_state)?;
        
        // Log quick save
        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
            game_log.entries.push(format!("Quick saved to slot {}", slot));
        }
        
        Ok(slot)
    }

    /// Quick load
    pub fn quick_load(&mut self, game_state: &mut GameState) -> SaveResult<()> {
        self.save_load_integration.quick_load(game_state)?;
        
        // Log quick load
        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
            game_log.entries.push("Quick loaded game".to_string());
        }
        
        Ok(())
    }

    /// New game
    pub fn new_game(&mut self, game_state: &mut GameState) -> SaveResult<()> {
        self.save_load_integration.new_game(game_state)?;
        
        // Reset autosave
        self.autosave_manager.get_save_load_system().set_current_save_slot(None);
        
        Ok(())
    }

    /// Configure autosave
    pub fn configure_autosave(&mut self, config: AutosaveConfig) {
        self.autosave_manager.configure(config);
    }

    /// Trigger autosave on event
    pub fn trigger_autosave_on_event(&mut self, event: String) {
        self.autosave_manager.trigger_on_event(event);
    }

    /// Manual autosave trigger
    pub fn trigger_manual_autosave(&mut self) {
        self.autosave_manager.trigger_manual();
    }

    /// Get autosave status
    pub fn get_autosave_status(&self) -> crate::persistence::autosave_system::AutosaveStatus {
        self.autosave_manager.get_status()
    }

    /// Check if save/load UI is open
    pub fn is_ui_open(&self) -> bool {
        self.save_load_ui.is_open()
    }

    /// Get save/load UI for rendering
    pub fn get_ui(&self) -> &SaveLoadUI {
        &self.save_load_ui
    }

    /// Get current save slot
    pub fn get_current_save_slot(&self) -> Option<u32> {
        self.save_load_integration.get_current_save_slot()
    }

    /// Get save directory
    pub fn get_save_directory(&self) -> &PathBuf {
        &self.save_directory
    }

    /// Get save slots
    pub fn get_save_slots(&self) -> SaveResult<Vec<SaveSlot>> {
        self.save_load_integration.get_save_slots()
    }

    /// Check if slot is autosave slot
    pub fn is_autosave_slot(&self, slot: u32) -> bool {
        self.autosave_manager.is_autosave_slot(slot)
    }

    /// Create save file name
    pub fn create_save_file_name(&self, game_state: &GameState) -> String {
        self.save_load_integration.create_save_file_name(game_state)
    }

    /// Handle game events that might trigger autosave
    pub fn handle_game_event(&mut self, event: &str) {
        match event {
            "level_up" => self.trigger_autosave_on_event("Player leveled up".to_string()),
            "boss_defeated" => self.trigger_autosave_on_event("Boss defeated".to_string()),
            "dungeon_entered" => self.trigger_autosave_on_event("Entered new dungeon level".to_string()),
            "significant_item_found" => self.trigger_autosave_on_event("Found significant item".to_string()),
            "quest_completed" => self.trigger_autosave_on_event("Quest completed".to_string()),
            _ => {}, // Ignore other events
        }
    }

    /// Get persistence statistics
    pub fn get_statistics(&self) -> PersistenceStatistics {
        let autosave_status = self.get_autosave_status();
        let current_slot = self.get_current_save_slot();
        
        PersistenceStatistics {
            current_save_slot: current_slot,
            autosave_enabled: autosave_status.enabled,
            time_until_next_autosave: autosave_status.time_until_next,
            last_autosave_elapsed: autosave_status.last_autosave_elapsed,
            save_directory: self.save_directory.clone(),
        }
    }
}

/// Persistence statistics
#[derive(Debug, Clone)]
pub struct PersistenceStatistics {
    pub current_save_slot: Option<u32>,
    pub autosave_enabled: bool,
    pub time_until_next_autosave: std::time::Duration,
    pub last_autosave_elapsed: std::time::Duration,
    pub save_directory: PathBuf,
}

/// Convenience functions for common persistence operations
impl GamePersistenceIntegration {
    /// Handle common keyboard shortcuts
    pub fn handle_shortcuts(&mut self, game_state: &mut GameState, key: KeyCode) -> SaveResult<bool> {
        match key {
            KeyCode::F5 => {
                // Quick save
                match self.quick_save(game_state) {
                    Ok(slot) => {
                        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
                            game_log.entries.push(format!("Quick saved to slot {}", slot));
                        }
                    },
                    Err(e) => {
                        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
                            game_log.entries.push(format!("Quick save failed: {}", e));
                        }
                    }
                }
                Ok(true)
            },
            KeyCode::F9 => {
                // Quick load
                match self.quick_load(game_state) {
                    Ok(()) => {
                        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
                            game_log.entries.push("Quick loaded game".to_string());
                        }
                    },
                    Err(e) => {
                        if let Ok(mut game_log) = game_state.world.try_write_resource::<GameLog>() {
                            game_log.entries.push(format!("Quick load failed: {}", e));
                        }
                    }
                }
                Ok(true)
            },
            _ => Ok(false),
        }
    }

    /// Save game with user-friendly error handling
    pub fn save_game_safe(&mut self, game_state: &GameState, slot: u32) -> (bool, String) {
        match self.save_load_integration.save_game(game_state, slot) {
            Ok(()) => {
                self.save_load_integration.set_current_save_slot(Some(slot));
                (true, format!("Game saved to slot {}", slot))
            },
            Err(e) => (false, format!("Save failed: {}", e)),
        }
    }

    /// Load game with user-friendly error handling
    pub fn load_game_safe(&mut self, game_state: &mut GameState, slot: u32) -> (bool, String) {
        match self.save_load_integration.load_game(game_state, slot) {
            Ok(()) => {
                self.save_load_integration.set_current_save_slot(Some(slot));
                (true, format!("Game loaded from slot {}", slot))
            },
            Err(e) => (false, format!("Load failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::components::{Player, Name, Position};
    use specs::{WorldExt, Builder};

    fn create_test_game_state() -> GameState {
        let mut game_state = GameState::new();
        
        // Register components
        game_state.world.register::<Player>();
        game_state.world.register::<Name>();
        game_state.world.register::<Position>();
        
        // Add test player
        game_state.world.create_entity()
            .with(Player)
            .with(Name { name: "Test Player".to_string() })
            .with(Position { x: 10, y: 10, z: 1 })
            .build();
        
        game_state.world.insert(GameLog::new());
        
        game_state
    }

    #[test]
    fn test_persistence_integration_creation() {
        let temp_dir = TempDir::new().unwrap();
        let integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf());
        
        assert!(integration.is_ok());
        
        let integration = integration.unwrap();
        assert!(!integration.is_ui_open());
        assert_eq!(integration.get_current_save_slot(), None);
    }

    #[test]
    fn test_save_and_load_integration() {
        let temp_dir = TempDir::new().unwrap();
        let mut integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        let mut game_state = create_test_game_state();
        
        // Save game
        let (success, message) = integration.save_game_safe(&game_state, 0);
        assert!(success);
        assert!(message.contains("saved"));
        
        // Clear world
        game_state.world.delete_all();
        
        // Load game
        let (success, message) = integration.load_game_safe(&mut game_state, 0);
        assert!(success);
        assert!(message.contains("loaded"));
        
        // Verify player was restored
        let players = game_state.world.read_storage::<Player>();
        assert_eq!(players.join().count(), 1);
    }

    #[test]
    fn test_quick_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let mut integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        let mut game_state = create_test_game_state();
        
        // Quick save
        let slot = integration.quick_save(&game_state).unwrap();
        assert_eq!(slot, 0); // Should use first available slot
        
        // Clear world
        game_state.world.delete_all();
        
        // Quick load
        integration.quick_load(&mut game_state).unwrap();
        
        // Verify player was restored
        let players = game_state.world.read_storage::<Player>();
        assert_eq!(players.join().count(), 1);
    }

    #[test]
    fn test_autosave_event_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Trigger autosave events
        integration.handle_game_event("level_up");
        integration.handle_game_event("boss_defeated");
        integration.handle_game_event("unknown_event"); // Should be ignored
        
        let status = integration.get_autosave_status();
        assert!(status.enabled);
    }

    #[test]
    fn test_keyboard_shortcuts() {
        let temp_dir = TempDir::new().unwrap();
        let mut integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        let mut game_state = create_test_game_state();
        
        // Test F5 (quick save)
        let handled = integration.handle_shortcuts(&mut game_state, KeyCode::F5).unwrap();
        assert!(handled);
        
        // Clear world
        game_state.world.delete_all();
        
        // Test F9 (quick load)
        let handled = integration.handle_shortcuts(&mut game_state, KeyCode::F9).unwrap();
        assert!(handled);
        
        // Verify player was restored
        let players = game_state.world.read_storage::<Player>();
        assert_eq!(players.join().count(), 1);
    }

    #[test]
    fn test_ui_integration() {
        let temp_dir = TempDir::new().unwrap();
        let mut integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Open save menu
        integration.open_save_menu().unwrap();
        assert!(integration.is_ui_open());
        
        // Close UI
        let mut game_state = create_test_game_state();
        integration.handle_input(&mut game_state, KeyCode::Esc).unwrap();
        assert!(!integration.is_ui_open());
    }

    #[test]
    fn test_persistence_statistics() {
        let temp_dir = TempDir::new().unwrap();
        let integration = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        
        let stats = integration.get_statistics();
        assert_eq!(stats.current_save_slot, None);
        assert!(stats.autosave_enabled);
        assert_eq!(stats.save_directory, temp_dir.path());
    }
}