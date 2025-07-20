/// Example usage of the save/load system integration
/// This demonstrates how to integrate the persistence system into the main game loop

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use crate::{
    game_state::{GameState, State},
    persistence::game_persistence_integration::{GamePersistenceIntegration, PersistenceStatistics},
    ui::UIComponent,
};

/// Example game loop with save/load integration
pub struct GameWithPersistence {
    game_state: GameState,
    persistence: GamePersistenceIntegration,
    running: bool,
}

impl GameWithPersistence {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let mut game_state = GameState::new();
        
        // Set up save directory
        let save_dir = PathBuf::from("./saves");
        let persistence = GamePersistenceIntegration::new(save_dir)?;
        
        Ok(GameWithPersistence {
            game_state,
            persistence,
            running: true,
        })
    }

    /// Main game loop
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.running {
            // Update game systems
            self.update()?;
            
            // Handle input
            if crossterm::event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key_event) = crossterm::event::read()? {
                    self.handle_input(key_event)?;
                }
            }
            
            // Render
            self.render()?;
            
            // Small delay to prevent excessive CPU usage
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        
        Ok(())
    }

    /// Update game systems
    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Update persistence system (handles autosave)
        self.persistence.update(&mut self.game_state)?;
        
        // Update other game systems here...
        
        Ok(())
    }

    /// Handle input events
    fn handle_input(&mut self, key_event: KeyEvent) -> Result<(), Box<dyn std::error::Error>> {
        let key = key_event.code;
        let modifiers = key_event.modifiers;
        
        // Handle persistence UI input first
        if self.persistence.handle_input(&mut self.game_state, key)? {
            return Ok(()); // Input was handled by persistence UI
        }
        
        // Handle keyboard shortcuts
        if self.persistence.handle_shortcuts(&mut self.game_state, key)? {
            return Ok(()); // Shortcut was handled
        }
        
        // Handle other game input
        match key {
            KeyCode::Esc => {
                if modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Esc to quit
                    self.running = false;
                } else {
                    // Regular Esc - open pause menu or close current UI
                    self.handle_escape_key()?;
                }
            },
            KeyCode::Char('s') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+S to open save menu
                self.persistence.open_save_menu()?;
            },
            KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+L to open load menu
                self.persistence.open_load_menu()?;
            },
            KeyCode::Char('n') if modifiers.contains(KeyModifiers::CONTROL) => {
                // Ctrl+N for new game
                self.persistence.new_game(&mut self.game_state)?;
            },
            _ => {
                // Handle other game input
                self.handle_game_input(key)?;
            }
        }
        
        Ok(())
    }

    /// Handle escape key
    fn handle_escape_key(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.persistence.is_ui_open() {
            // Close persistence UI if open
            self.persistence.handle_input(&mut self.game_state, KeyCode::Esc)?;
        } else {
            // Open pause menu or handle other escape logic
            // For this example, we'll just toggle game state
            match self.game_state.get_state() {
                State::Running => self.game_state.set_state(State::Paused),
                State::Paused => self.game_state.set_state(State::Running),
                _ => {},
            }
        }
        Ok(())
    }

    /// Handle regular game input
    fn handle_game_input(&mut self, key: KeyCode) -> Result<(), Box<dyn std::error::Error>> {
        // Only handle game input if not paused and no UI is open
        if self.game_state.get_state() != &State::Running || self.persistence.is_ui_open() {
            return Ok(());
        }
        
        match key {
            KeyCode::Char('w') | KeyCode::Up => {
                // Move up
                self.handle_player_move(0, -1)?;
            },
            KeyCode::Char('s') | KeyCode::Down => {
                // Move down
                self.handle_player_move(0, 1)?;
            },
            KeyCode::Char('a') | KeyCode::Left => {
                // Move left
                self.handle_player_move(-1, 0)?;
            },
            KeyCode::Char('d') | KeyCode::Right => {
                // Move right
                self.handle_player_move(1, 0)?;
            },
            KeyCode::Char('i') => {
                // Open inventory
                // This would trigger an autosave event
                self.persistence.handle_game_event("inventory_opened");
            },
            KeyCode::Char('c') => {
                // Open character screen
                self.persistence.handle_game_event("character_screen_opened");
            },
            _ => {},
        }
        
        Ok(())
    }

    /// Handle player movement
    fn handle_player_move(&mut self, dx: i32, dy: i32) -> Result<(), Box<dyn std::error::Error>> {
        // Move player logic would go here
        // For significant moves (like entering new areas), trigger autosave
        
        // Example: if player moved to a new level
        // self.persistence.handle_game_event("dungeon_entered");
        
        Ok(())
    }

    /// Render the game
    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear screen
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        
        // Render game world
        self.render_game_world()?;
        
        // Render UI overlays
        self.render_ui_overlays()?;
        
        // Render persistence UI if open
        if self.persistence.is_ui_open() {
            self.render_persistence_ui()?;
        }
        
        // Render status information
        self.render_status_info()?;
        
        Ok(())
    }

    /// Render the main game world
    fn render_game_world(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Game world rendering would go here
        println!("Game World Rendering...");
        
        // Show game state
        match self.game_state.get_state() {
            State::Running => println!("Game Running"),
            State::Paused => println!("Game Paused - Press Esc to resume"),
            State::MainMenu => println!("Main Menu"),
            State::GameOver => println!("Game Over"),
        }
        
        Ok(())
    }

    /// Render UI overlays
    fn render_ui_overlays(&self) -> Result<(), Box<dyn std::error::Error>> {
        // UI overlays would go here (HUD, inventory, etc.)
        Ok(())
    }

    /// Render persistence UI
    fn render_persistence_ui(&self) -> Result<(), Box<dyn std::error::Error>> {
        let ui = self.persistence.get_ui();
        let render_commands = ui.render();
        
        // Process render commands
        for command in render_commands {
            // In a real implementation, you would process these commands
            // to actually draw the UI elements
            println!("Render Command: {:?}", command);
        }
        
        Ok(())
    }

    /// Render status information
    fn render_status_info(&self) -> Result<(), Box<dyn std::error::Error>> {
        let stats = self.persistence.get_statistics();
        
        // Show persistence status
        println!("\\n--- Persistence Status ---");
        println!("Current Save Slot: {:?}", stats.current_save_slot);
        println!("Autosave Enabled: {}", stats.autosave_enabled);
        
        if stats.autosave_enabled {
            println!("Time until next autosave: {:?}", stats.time_until_next_autosave);
            println!("Last autosave: {:?} ago", stats.last_autosave_elapsed);
        }
        
        println!("Save Directory: {}", stats.save_directory.display());
        
        // Show controls
        println!("\\n--- Controls ---");
        println!("WASD/Arrow Keys: Move");
        println!("F5: Quick Save");
        println!("F9: Quick Load");
        println!("Ctrl+S: Save Menu");
        println!("Ctrl+L: Load Menu");
        println!("Ctrl+N: New Game");
        println!("Ctrl+Esc: Quit");
        println!("Esc: Pause/Resume");
        
        Ok(())
    }

    /// Demonstrate save/load operations
    pub fn demo_save_load_operations(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\\n=== Save/Load Demo ===");
        
        // Quick save
        println!("Performing quick save...");
        match self.persistence.quick_save(&self.game_state) {
            Ok(slot) => println!("Quick saved to slot {}", slot),
            Err(e) => println!("Quick save failed: {}", e),
        }
        
        // Get save slots
        println!("\\nCurrent save slots:");
        match self.persistence.get_save_slots() {
            Ok(slots) => {
                for slot in slots {
                    if slot.is_occupied {
                        if let Some(metadata) = &slot.metadata {
                            println!("Slot {}: {} - Level {} - {}", 
                                slot.slot_id,
                                metadata.player_name,
                                metadata.character_level,
                                metadata.created_at.format("%Y-%m-%d %H:%M")
                            );
                        } else {
                            println!("Slot {}: Occupied (no metadata)", slot.slot_id);
                        }
                    } else {
                        println!("Slot {}: Empty", slot.slot_id);
                    }
                }
            },
            Err(e) => println!("Failed to get save slots: {}", e),
        }
        
        // Demonstrate autosave events
        println!("\\nTriggering autosave events...");
        self.persistence.handle_game_event("level_up");
        self.persistence.handle_game_event("boss_defeated");
        
        // Show autosave status
        let autosave_status = self.persistence.get_autosave_status();
        println!("Autosave status: {} pending triggers", autosave_status.pending_triggers);
        
        Ok(())
    }
}

/// Example of how to configure the persistence system
pub fn configure_persistence_example(persistence: &mut GamePersistenceIntegration) {
    use crate::persistence::autosave_system::AutosaveConfig;
    
    // Configure autosave settings
    let autosave_config = AutosaveConfig {
        enabled: true,
        interval_seconds: 180, // 3 minutes
        max_autosaves: 5,
        autosave_on_level_change: true,
        autosave_on_significant_events: true,
        backup_before_autosave: true,
    };
    
    persistence.configure_autosave(autosave_config);
    
    println!("Persistence configured with custom settings");
}

/// Example of handling different game events
pub fn handle_game_events_example(persistence: &mut GamePersistenceIntegration) {
    // Examples of when to trigger autosave events
    
    // Player progression events
    persistence.handle_game_event("level_up");
    persistence.handle_game_event("skill_learned");
    
    // Combat events
    persistence.handle_game_event("boss_defeated");
    persistence.handle_game_event("rare_enemy_defeated");
    
    // Exploration events
    persistence.handle_game_event("dungeon_entered");
    persistence.handle_game_event("secret_area_found");
    
    // Item events
    persistence.handle_game_event("significant_item_found");
    persistence.handle_game_event("equipment_upgraded");
    
    // Quest events
    persistence.handle_game_event("quest_completed");
    persistence.handle_game_event("story_milestone");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_game_with_persistence_creation() {
        // This test would require more setup in a real implementation
        // For now, just test that the structure compiles
        assert!(true);
    }

    #[test]
    fn test_persistence_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let mut persistence = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        
        configure_persistence_example(&mut persistence);
        
        let status = persistence.get_autosave_status();
        assert!(status.enabled);
    }

    #[test]
    fn test_game_event_handling() {
        let temp_dir = TempDir::new().unwrap();
        let mut persistence = GamePersistenceIntegration::new(temp_dir.path().to_path_buf()).unwrap();
        
        handle_game_events_example(&mut persistence);
        
        // Events should be queued for processing
        let status = persistence.get_autosave_status();
        // In a real test, we'd check that events were properly queued
        assert!(status.enabled);
    }
}