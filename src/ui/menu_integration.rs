use crate::ui::{MainMenu, MainMenuState, MainMenuRunner};
use crate::game_state::{GameState, State};
use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use std::io::stdout;

/// Integration between the main menu and game state
pub struct MenuIntegration;

impl MenuIntegration {
    /// Run the main menu and return the selected action
    pub fn run_main_menu() -> Result<MenuAction, Box<dyn std::error::Error>> {
        // Setup terminal
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let result = {
            let mut menu_runner = MainMenuRunner::new()?;
            let menu_state = menu_runner.run()?;
            
            match menu_state {
                MainMenuState::NewGame => Ok(MenuAction::StartNewGame),
                MainMenuState::LoadGame => Ok(MenuAction::LoadGame),
                MainMenuState::Options => Ok(MenuAction::ShowOptions),
                MainMenuState::Credits => Ok(MenuAction::ShowCredits),
                MainMenuState::Quit => Ok(MenuAction::Quit),
                MainMenuState::MainMenu => Ok(MenuAction::Quit), // Shouldn't happen
            }
        };

        // Cleanup terminal
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        result
    }

    /// Show a message dialog and wait for user input
    pub fn show_message_dialog(title: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let result = {
            let mut menu_runner = MainMenuRunner::new()?;
            menu_runner.show_message(message, 3000) // Show for 3 seconds
        };

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        result
    }

    /// Integrate menu with game state transitions
    pub fn handle_menu_transition(current_state: &GameState, menu_action: MenuAction) -> GameState {
        match menu_action {
            MenuAction::StartNewGame => {
                // Create new game state
                GameState::new_game()
            }
            MenuAction::LoadGame => {
                // Try to load saved game, fallback to new game
                GameState::load_game().unwrap_or_else(|_| GameState::new_game())
            }
            MenuAction::ShowOptions => {
                // Show options menu (for now, just return to main menu)
                current_state.clone()
            }
            MenuAction::ShowCredits => {
                // Show credits (for now, just return to main menu)
                current_state.clone()
            }
            MenuAction::Quit => {
                // Set game state to quit
                let mut new_state = current_state.clone();
                new_state.set_state(State::Quit);
                new_state
            }
        }
    }

    /// Show the pause menu during gameplay
    pub fn show_pause_menu() -> Result<PauseMenuAction, Box<dyn std::error::Error>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;

        let result = {
            let mut pause_menu = PauseMenu::new();
            let mut menu_runner = MainMenuRunner::new()?;
            
            // Customize the menu for pause screen
            menu_runner.menu.title = "GAME PAUSED".to_string();
            menu_runner.menu.subtitle = "What would you like to do?".to_string();
            // For now, reuse existing options - in a full implementation,
            // we'd create a separate PauseMenuOption enum
            menu_runner.menu.options = vec![
                MenuOption::NewGame,    // Repurposed as "Resume"
                MenuOption::LoadGame,   // Save/Load functionality
                MenuOption::Options,
                MenuOption::Credits,    // Repurposed as "Main Menu"
                MenuOption::Quit,
            ];

            let menu_state = menu_runner.run()?;
            
            match menu_state {
                MainMenuState::MainMenu => Ok(PauseMenuAction::Resume),
                MainMenuState::NewGame => Ok(PauseMenuAction::SaveGame), // Repurposed
                MainMenuState::LoadGame => Ok(PauseMenuAction::LoadGame),
                MainMenuState::Options => Ok(PauseMenuAction::Options),
                MainMenuState::Credits => Ok(PauseMenuAction::MainMenu), // Repurposed
                MainMenuState::Quit => Ok(PauseMenuAction::Quit),
            }
        };

        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;

        result
    }
}

/// Actions that can be taken from the main menu
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    StartNewGame,
    LoadGame,
    ShowOptions,
    ShowCredits,
    Quit,
}

/// Actions that can be taken from the pause menu
#[derive(Debug, Clone, PartialEq)]
pub enum PauseMenuAction {
    Resume,
    SaveGame,
    LoadGame,
    Options,
    MainMenu,
    Quit,
}

/// Pause menu component (simplified version of MainMenu)
pub struct PauseMenu {
    pub selected_option: usize,
    pub options: Vec<String>,
}

impl PauseMenu {
    pub fn new() -> Self {
        PauseMenu {
            selected_option: 0,
            options: vec![
                "Resume Game".to_string(),
                "Save Game".to_string(),
                "Load Game".to_string(),
                "Options".to_string(),
                "Main Menu".to_string(),
                "Quit Game".to_string(),
            ],
        }
    }
}

/// Example of how to use the menu integration in main game loop
pub fn example_game_loop() -> Result<(), Box<dyn std::error::Error>> {
    // Show main menu
    let menu_action = MenuIntegration::run_main_menu()?;
    
    match menu_action {
        MenuAction::StartNewGame => {
            println!("Starting new game...");
            // Initialize new game state and start game loop
        }
        MenuAction::LoadGame => {
            println!("Loading saved game...");
            // Load game state and start game loop
        }
        MenuAction::ShowOptions => {
            println!("Showing options...");
            // Show options menu
        }
        MenuAction::ShowCredits => {
            println!("Showing credits...");
            // Show credits screen
        }
        MenuAction::Quit => {
            println!("Goodbye!");
            return Ok(());
        }
    }

    // Example game loop
    loop {
        // Game update logic here...
        
        // Check for pause key (ESC)
        if should_pause() {
            let pause_action = MenuIntegration::show_pause_menu()?;
            
            match pause_action {
                PauseMenuAction::Resume => {
                    // Continue game loop
                    continue;
                }
                PauseMenuAction::SaveGame => {
                    println!("Saving game...");
                    // Save game logic
                }
                PauseMenuAction::LoadGame => {
                    println!("Loading game...");
                    // Load game logic
                }
                PauseMenuAction::Options => {
                    println!("Showing options...");
                    // Show options
                }
                PauseMenuAction::MainMenu => {
                    // Return to main menu
                    return example_game_loop();
                }
                PauseMenuAction::Quit => {
                    println!("Goodbye!");
                    return Ok(());
                }
            }
        }
        
        // Break condition for example
        break;
    }

    Ok(())
}

fn should_pause() -> bool {
    // Placeholder - in real implementation, this would check for ESC key
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_action_creation() {
        let action = MenuAction::StartNewGame;
        assert_eq!(action, MenuAction::StartNewGame);
    }

    #[test]
    fn test_pause_menu_creation() {
        let pause_menu = PauseMenu::new();
        assert_eq!(pause_menu.selected_option, 0);
        assert_eq!(pause_menu.options.len(), 6);
        assert_eq!(pause_menu.options[0], "Resume Game");
    }

    #[test]
    fn test_pause_menu_action_creation() {
        let action = PauseMenuAction::Resume;
        assert_eq!(action, PauseMenuAction::Resume);
    }
}