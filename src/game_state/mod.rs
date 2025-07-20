mod state_machine;
mod state_stack;
mod run_state;

pub use run_state::RunState;

use crossterm::event::{KeyCode, KeyEvent};
use specs::{World, WorldExt, Entity};
use crate::components::*;
use crate::resources::{GameLog, RandomNumberGenerator, GameStateResource};
use crate::map::Map;
use crate::entity_factory::EntityFactory;
use crate::systems::SystemRunner;
use crate::character_creation::{CharacterCreationState, handle_character_creation_input, render_character_creation};

pub use state_machine::StateType;
use state_stack::StateStack;

pub struct GameState {
    pub running: bool,
    pub state_stack: StateStack,
    pub world: World,
    pub player: Option<Entity>,
    pub current_depth: i32,
    pub turn_count: u32,
    pub system_runner: SystemRunner,
    pub run_state: RunState,
    pub character_creation: CharacterCreationState,
}

impl GameState {
    pub fn new() -> Self {
        let mut world = World::new();
        
        // Register components
        crate::components::register_components(&mut world);
        
        // Create resources
        world.insert(GameLog::new(100));
        world.insert(RandomNumberGenerator::new_with_random_seed());
        world.insert(GameStateResource::default());
        
        // Create a default map (will be replaced when a game starts)
        let map = Map::new(80, 50, 1);
        world.insert(map);
        
        GameState {
            running: true,
            state_stack: StateStack::new(),
            world,
            player: None,
            current_depth: 1,
            turn_count: 0,
            system_runner: SystemRunner::new(),
            run_state: RunState::MainMenu,
            character_creation: CharacterCreationState::new(),
        }
    }
    
    // Initialize a new game
    fn initialize_new_game(&mut self) {
        // Clear existing entities
        self.world.delete_all();
        
        // Create a new map
        let mut map = Map::new(80, 50, 1);
        
        // For now, just create a simple room in the center of the map
        for y in 20..30 {
            for x in 30..50 {
                let idx = map.xy_idx(x, y);
                map.tiles[idx] = crate::map::TileType::Floor;
                map.blocked[idx] = false;
            }
        }
        
        // Place the player in the center of the room
        let player_x = 40;
        let player_y = 25;
        
        // Update the map resource
        self.world.insert(map);
        
        // Create the player entity
        let player = EntityFactory::create_player(&mut self.world, player_x, player_y);
        self.player = Some(player);
        
        // Generate a new seed for the RNG
        {
            let mut rng = self.world.write_resource::<RandomNumberGenerator>();
            *rng = RandomNumberGenerator::new_with_random_seed();
        }
        
        // Add monsters
        {
            let mut rng = self.world.write_resource::<RandomNumberGenerator>();
            let monster_type1 = rng.range(0, 3);
            let monster_type2 = rng.range(0, 3);
            
            // Release the RNG borrow
            drop(rng);
            
            // Now create the monsters
            EntityFactory::create_monster(&mut self.world, 42, 23, monster_type1);
            EntityFactory::create_monster(&mut self.world, 45, 27, monster_type2);
        }
        
        // Add a health potion
        EntityFactory::create_health_potion(&mut self.world, 38, 22);
        
        // Add stairs down
        EntityFactory::create_stairs_down(&mut self.world, 48, 28);
        
        // Reset game state
        {
            let mut game_state = self.world.write_resource::<GameStateResource>();
            game_state.turn_count = 0;
            game_state.depth = 1;
            game_state.game_over = false;
        }
        
        // Add a welcome message
        {
            let mut log = self.world.write_resource::<GameLog>();
            log.clear();
            log.add_entry("Welcome to ASCII Dungeon Explorer!".to_string());
            log.add_entry("Use arrow keys or HJKL to move.".to_string());
        }
        
        // Set the current state to playing
        self.state_stack.replace(StateType::Playing);
    }
    
    pub fn handle_input(&mut self, key_event: KeyEvent) {
        // Handle character creation input if in character creation state
        if matches!(self.run_state, 
            RunState::CharacterName | 
            RunState::CharacterClass | 
            RunState::CharacterBackground | 
            RunState::CharacterAttributes | 
            RunState::CharacterEquipment | 
            RunState::CharacterConfirm) {
            
            if handle_character_creation_input(key_event, self, &mut self.character_creation) {
                return;
            }
        }
        
        match self.state_stack.current() {
            StateType::MainMenu => self.handle_main_menu_input(key_event),
            StateType::Playing => self.handle_playing_input(key_event),
            StateType::Inventory => self.handle_inventory_input(key_event),
            StateType::CharacterSheet => self.handle_character_sheet_input(key_event),
            StateType::GameOver => self.handle_game_over_input(key_event),
            StateType::LevelUp => self.handle_level_up_input(key_event),
            StateType::Targeting => self.handle_targeting_input(key_event),
            StateType::SaveGame => self.handle_save_game_input(key_event),
            StateType::LoadGame => self.handle_load_game_input(key_event),
            StateType::Options => self.handle_options_input(key_event),
            StateType::Help => self.handle_help_input(key_event),
            StateType::Pause => self.handle_pause_input(key_event),
            StateType::GuildManagement => self.handle_guild_management_input(key_event),
            StateType::MissionAssignment => self.handle_mission_assignment_input(key_event),
            StateType::AgentConfiguration => self.handle_agent_configuration_input(key_event),
        }
    }
    
    fn handle_main_menu_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('n') => {
                // Start character creation
                self.run_state = RunState::CharacterCreation;
                self.character_creation = CharacterCreationState::new();
                self.run_state = RunState::CharacterName;
            },
            KeyCode::Char('l') => {
                // Load a game
                self.state_stack.push(StateType::LoadGame);
            },
            KeyCode::Char('o') => {
                // Options
                self.state_stack.push(StateType::Options);
            },
            KeyCode::Char('h') => {
                // Help
                self.state_stack.push(StateType::Help);
            },
            KeyCode::Char('q') => {
                // Quit the game
                self.running = false;
            },
            _ => {}
        }
    }
    
    fn handle_playing_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('i') => {
                // Open inventory
                self.state_stack.push(StateType::Inventory);
            },
            KeyCode::Char('c') => {
                // Open character sheet
                self.state_stack.push(StateType::CharacterSheet);
            },
            KeyCode::Char('g') => {
                // Open guild management
                self.state_stack.push(StateType::GuildManagement);
            },
            KeyCode::Esc => {
                // Pause game
                self.state_stack.push(StateType::Pause);
            },
            KeyCode::Char('s') => {
                // Save game
                self.state_stack.push(StateType::SaveGame);
            },
            KeyCode::Char('q') => {
                // Return to main menu
                self.state_stack.clear();
            },
            _ => {
                // Handle movement and other actions
                // Will be implemented later
            }
        }
    }
    
    fn handle_inventory_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for inventory input handling
    }
    
    fn handle_character_sheet_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for character sheet input handling
    }
    
    fn handle_game_over_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for game over input handling
    }
    
    fn handle_level_up_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for level up input handling
    }
    
    fn handle_targeting_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for targeting input handling
    }
    
    fn handle_save_game_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for save game input handling
    }
    
    fn handle_load_game_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for load game input handling
    }
    
    fn handle_options_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for options input handling
    }
    
    fn handle_help_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for help input handling
    }
    
    fn handle_pause_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('p') => {
                // Resume game
                self.state_stack.pop();
            },
            KeyCode::Char('s') => {
                // Save game
                self.state_stack.replace(StateType::SaveGame);
            },
            KeyCode::Char('l') => {
                // Load game
                self.state_stack.replace(StateType::LoadGame);
            },
            KeyCode::Char('o') => {
                // Options
                self.state_stack.replace(StateType::Options);
            },
            KeyCode::Char('q') => {
                // Return to main menu
                self.state_stack.clear();
            },
            _ => {}
        }
    }
    
    fn handle_guild_management_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for guild management input handling
    }
    
    fn handle_mission_assignment_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for mission assignment input handling
    }
    
    fn handle_agent_configuration_input(&mut self, _key_event: KeyEvent) {
        // Placeholder for agent configuration input handling
    }
    
    pub fn update(&mut self) {
        match self.state_stack.current() {
            StateType::MainMenu => self.update_main_menu(),
            StateType::Playing => self.update_playing(),
            StateType::Inventory => self.update_inventory(),
            StateType::CharacterSheet => self.update_character_sheet(),
            StateType::GameOver => self.update_game_over(),
            StateType::LevelUp => self.update_level_up(),
            StateType::Targeting => self.update_targeting(),
            StateType::SaveGame => self.update_save_game(),
            StateType::LoadGame => self.update_load_game(),
            StateType::Options => self.update_options(),
            StateType::Help => self.update_help(),
            StateType::Pause => self.update_pause(),
            StateType::GuildManagement => self.update_guild_management(),
            StateType::MissionAssignment => self.update_mission_assignment(),
            StateType::AgentConfiguration => self.update_agent_configuration(),
        }
    }
    
    fn update_main_menu(&mut self) {
        // Placeholder for main menu update logic
    }
    
    fn update_playing(&mut self) {
        // Run the ECS systems
        self.system_runner.run_systems(&mut self.world);
        
        // Update turn count if player has moved (will be implemented later)
        
        // Check for game over conditions (will be implemented later)
    }
    
    fn update_inventory(&mut self) {
        // Placeholder for inventory update logic
    }
    
    fn update_character_sheet(&mut self) {
        // Placeholder for character sheet update logic
    }
    
    fn update_game_over(&mut self) {
        // Placeholder for game over update logic
    }
    
    fn update_level_up(&mut self) {
        // Placeholder for level up update logic
    }
    
    fn update_targeting(&mut self) {
        // Placeholder for targeting update logic
    }
    
    fn update_save_game(&mut self) {
        // Placeholder for save game update logic
    }
    
    fn update_load_game(&mut self) {
        // Placeholder for load game update logic
    }
    
    fn update_options(&mut self) {
        // Placeholder for options update logic
    }
    
    fn update_help(&mut self) {
        // Placeholder for help update logic
    }
    
    fn update_pause(&mut self) {
        // Placeholder for pause update logic
    }
    
    fn update_guild_management(&mut self) {
        // Placeholder for guild management update logic
    }
    
    fn update_mission_assignment(&mut self) {
        // Placeholder for mission assignment update logic
    }
    
    fn update_agent_configuration(&mut self) {
        // Placeholder for agent configuration update logic
    }
    
    pub fn render(&mut self) {
        // Render character creation if in character creation state
        if matches!(self.run_state, 
            RunState::CharacterName | 
            RunState::CharacterClass | 
            RunState::CharacterBackground | 
            RunState::CharacterAttributes | 
            RunState::CharacterEquipment | 
            RunState::CharacterConfirm) {
            
            render_character_creation(self, &self.character_creation);
            return;
        }
        
        match self.state_stack.current() {
            StateType::MainMenu => self.render_main_menu(),
            StateType::Playing => self.render_playing(),
            StateType::Inventory => self.render_inventory(),
            StateType::CharacterSheet => self.render_character_sheet(),
            StateType::GameOver => self.render_game_over(),
            StateType::LevelUp => self.render_level_up(),
            StateType::Targeting => self.render_targeting(),
            StateType::SaveGame => self.render_save_game(),
            StateType::LoadGame => self.render_load_game(),
            StateType::Options => self.render_options(),
            StateType::Help => self.render_help(),
            StateType::Pause => self.render_pause(),
            StateType::GuildManagement => self.render_guild_management(),
            StateType::MissionAssignment => self.render_mission_assignment(),
            StateType::AgentConfiguration => self.render_agent_configuration(),
        }
    }
    
    fn render_main_menu(&mut self) {
        use crate::rendering::with_terminal;
        use crossterm::style::Color;
        
        let _ = with_terminal(|terminal| {
            // Clear the screen
            terminal.clear()?;
            
            // Get terminal size
            let (width, height) = terminal.size();
            
            // Calculate center position
            let center_x = width / 2;
            let center_y = height / 2;
            
            // Draw title
            terminal.draw_text_centered(center_y - 5, "ASCII DUNGEON EXPLORER", Color::Yellow, Color::Black)?;
            
            // Draw menu options
            terminal.draw_text(center_x - 10, center_y, "n - New Game", Color::White, Color::Black)?;
            terminal.draw_text(center_x - 10, center_y + 1, "l - Load Game", Color::White, Color::Black)?;
            terminal.draw_text(center_x - 10, center_y + 2, "o - Options", Color::White, Color::Black)?;
            terminal.draw_text(center_x - 10, center_y + 3, "h - Help", Color::White, Color::Black)?;
            terminal.draw_text(center_x - 10, center_y + 4, "q - Quit", Color::White, Color::Black)?;
            
            // Draw version
            terminal.draw_text(width - 20, height - 1, "Version 0.1.0", Color::DarkGrey, Color::Black)?;
            
            terminal.flush()
        });
    }
    
    fn render_playing(&mut self) {
        // Use the render system to render the game
        self.system_runner.render(&self.world);
    }
    
    fn render_inventory(&mut self) {
        // Placeholder for inventory rendering
    }
    
    fn render_character_sheet(&mut self) {
        if let Some(player) = self.player {
            crate::ui::render_character_sheet(&self.world, player);
        }
    }
    
    fn render_game_over(&mut self) {
        // Placeholder for game over rendering
    }
    
    fn render_level_up(&mut self) {
        if let Some(player) = self.player {
            crate::ui::render_level_up_screen(&self.world, player);
        }
    }
    
    fn render_targeting(&mut self) {
        // Placeholder for targeting rendering
    }
    
    fn render_save_game(&mut self) {
        // Placeholder for save game rendering
    }
    
    fn render_load_game(&mut self) {
        // Placeholder for load game rendering
    }
    
    fn render_options(&mut self) {
        // Placeholder for options rendering
    }
    
    fn render_help(&mut self) {
        // Placeholder for help rendering
    }
    
    fn render_pause(&mut self) {
        use crate::rendering::with_terminal;
        use crossterm::style::Color;
        
        let _ = with_terminal(|terminal| {
            // Get terminal size
            let (width, height) = terminal.size();
            
            // Calculate center position
            let center_x = width / 2;
            let center_y = height / 2;
            
            // Draw background box
            terminal.fill_rect(center_x - 15, center_y - 6, 30, 12, ' ', Color::White, Color::DarkBlue)?;
            terminal.draw_box(center_x - 15, center_y - 6, 30, 12, Color::White, Color::DarkBlue)?;
            
            // Draw title
            terminal.draw_text(center_x - 5, center_y - 4, "GAME PAUSED", Color::Yellow, Color::DarkBlue)?;
            
            // Draw menu options
            terminal.draw_text(center_x - 10, center_y - 1, "ESC - Resume Game", Color::White, Color::DarkBlue)?;
            terminal.draw_text(center_x - 10, center_y, "s - Save Game", Color::White, Color::DarkBlue)?;
            terminal.draw_text(center_x - 10, center_y + 1, "l - Load Game", Color::White, Color::DarkBlue)?;
            terminal.draw_text(center_x - 10, center_y + 2, "o - Options", Color::White, Color::DarkBlue)?;
            terminal.draw_text(center_x - 10, center_y + 3, "q - Return to Main Menu", Color::White, Color::DarkBlue)?;
            
            terminal.flush()
        });
    }
    
    fn render_guild_management(&mut self) {
        // Placeholder for guild management rendering
    }
    
    fn render_mission_assignment(&mut self) {
        // Placeholder for mission assignment rendering
    }
    
    fn render_agent_configuration(&mut self) {
        // Placeholder for agent configuration rendering
    }
}