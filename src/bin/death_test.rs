use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::ui::{show_death_screen, DeathAction};
use ascii_dungeon_explorer::systems::{PlayerDeathSystem, DeathPenaltySystem, RevivalSystem, GameOverSystem};
use crossterm::style::Color;
use crossterm::event::{read, Event, KeyCode};
use specs::{World, WorldExt, Builder, RunNow};

fn main() {
    // Create a world
    let mut world = World::new();
    
    // Register components
    register_components(&mut world);
    
    // Add resources
    world.insert(GameLog::new());
    world.insert(RandomNumberGenerator::new_with_random_seed());
    
    // Create a player with death system components
    let player = world.create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::White,
            bg: Color::Black,
            render_order: 0,
        })
        .with(Player {})
        .with(Name { name: "Test Hero".to_string() })
        .with(CombatStats {
            max_hp: 50,
            hp: 50,
            defense: 5,
            power: 10,
        })
        .with(PlayerResources::new(30, 20))
        .with(Experience::new())
        .with(DeathState::new())
        .with(GameSettings::new(GameMode::Normal))
        .with(Inventory::new(10))
        .build();
    
    // Create a revival potion
    let revival_potion = world.create_entity()
        .with(Item {})
        .with(Name { name: "Phoenix Feather".to_string() })
        .with(Renderable {
            glyph: '!',
            fg: Color::Yellow,
            bg: Color::Black,
            render_order: 2,
        })
        .with(RevivalItem {
            revival_power: 25,
            auto_use: false,
            consumed_on_use: true,
        })
        .build();
    
    // Add revival item to player's inventory
    {
        let mut inventories = world.write_storage::<Inventory>();
        let player_inventory = inventories.get_mut(player).unwrap();
        player_inventory.items.push(revival_potion);
    }
    
    // Create systems
    let mut player_death_system = PlayerDeathSystem {};
    let mut death_penalty_system = DeathPenaltySystem {};
    let mut revival_system = RevivalSystem {};
    let mut game_over_system = GameOverSystem {};
    
    // Main loop
    let mut running = true;
    
    while running {
        // Display game state
        let _ = with_terminal(|terminal| {
            // Clear the screen
            terminal.clear()?;
            
            // Get terminal size
            let (width, height) = terminal.size();
            
            // Calculate center position
            let center_x = width / 2;
            let center_y = height / 2;
            
            // Draw title
            terminal.draw_text_centered(center_y - 15, "DEATH AND REVIVAL TEST", Color::Yellow, Color::Black)?;
            
            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            let death_state = world.read_storage::<DeathState>().get(player).unwrap();
            let game_settings = world.read_storage::<GameSettings>().get(player).unwrap();
            
            terminal.draw_text_centered(
                center_y - 10,
                &format!("HP: {}/{} | Game Mode: {}", 
                    player_stats.hp, player_stats.max_hp, game_settings.game_mode.name()),
                Color::White,
                Color::Black
            )?;
            
            terminal.draw_text_centered(
                center_y - 8,
                &format!("Death State: {} | Revivals: {}/{}", 
                    if death_state.is_dead { "DEAD" } else { "ALIVE" },
                    death_state.revival_attempts,
                    death_state.max_revival_attempts),
                if death_state.is_dead { Color::Red } else { Color::Green },
                Color::Black
            )?;
            
            if death_state.is_dead {
                terminal.draw_text_centered(
                    center_y - 6,
                    &format!("Cause of Death: {}", death_state.death_cause),
                    Color::Red,
                    Color::Black
                )?;
            }
            
            // Draw instructions
            if death_state.is_dead {
                terminal.draw_text_centered(
                    center_y - 3,
                    "Press 'd' to show death screen, 'r' to revive manually",
                    Color::Grey,
                    Color::Black
                )?;
            } else {
                terminal.draw_text_centered(
                    center_y - 3,
                    "Press 'k' to kill player, 'm' to change game mode, 'q' to quit",
                    Color::Grey,
                    Color::Black
                )?;
            }
            
            // Draw game log
            let game_log = world.read_resource::<GameLog>();
            for (i, entry) in game_log.entries.iter().rev().take(10).enumerate() {
                terminal.draw_text(5, center_y + i as u16, entry, Color::White, Color::Black)?;
            }
            
            // Flush the output
            terminal.flush()
        });
        
        // Wait for key press
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('k') => {
                        // Kill the player
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(player) {
                            stats.hp = 0;
                        }
                        
                        // Run death systems
                        player_death_system.run_now(&world);
                        death_penalty_system.run_now(&world);
                        revival_system.run_now(&world);
                        game_over_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('r') => {
                        // Manual revival
                        let mut death_states = world.write_storage::<DeathState>();
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        let mut game_log = world.write_resource::<GameLog>();
                        
                        if let (Some(death_state), Some(stats)) = (death_states.get_mut(player), combat_stats.get_mut(player)) {
                            if death_state.is_dead && death_state.can_revive() {
                                if death_state.revive() {
                                    stats.hp = stats.max_hp / 4; // Revive with 25% HP
                                    game_log.add_entry("You have been revived with penalties!".to_string());
                                }
                            }
                        }
                    },
                    KeyCode::Char('m') => {
                        // Cycle through game modes
                        let mut game_settings = world.write_storage::<GameSettings>();
                        let mut death_states = world.write_storage::<DeathState>();
                        
                        if let (Some(settings), Some(death_state)) = (game_settings.get_mut(player), death_states.get_mut(player)) {
                            settings.game_mode = match settings.game_mode {
                                GameMode::Normal => GameMode::Hardcore,
                                GameMode::Hardcore => GameMode::Permadeath,
                                GameMode::Permadeath => GameMode::Casual,
                                GameMode::Casual => GameMode::Normal,
                            };
                            
                            // Update max revivals based on new mode
                            death_state.max_revival_attempts = settings.game_mode.max_revivals();
                            settings.permadeath_enabled = settings.game_mode == GameMode::Permadeath;
                            
                            let mut game_log = world.write_resource::<GameLog>();
                            game_log.add_entry(format!("Game mode changed to: {}", settings.game_mode.name()));
                        }
                    },
                    KeyCode::Char('d') => {
                        // Show death screen
                        let death_states = world.read_storage::<DeathState>();
                        if let Some(death_state) = death_states.get(player) {
                            if death_state.is_dead {
                                if let Some(action) = show_death_screen(&world, player) {
                                    match action {
                                        DeathAction::ReviveWithPenalty => {
                                            let mut death_states = world.write_storage::<DeathState>();
                                            let mut combat_stats = world.write_storage::<CombatStats>();
                                            let mut game_log = world.write_resource::<GameLog>();
                                            
                                            if let (Some(death_state), Some(stats)) = (death_states.get_mut(player), combat_stats.get_mut(player)) {
                                                if death_state.revive() {
                                                    stats.hp = stats.max_hp / 4;
                                                    game_log.add_entry("Revived with penalty!".to_string());
                                                }
                                            }
                                        },
                                        DeathAction::UseRevivalItem(item_entity) => {
                                            // Use revival item logic would go here
                                            let mut game_log = world.write_resource::<GameLog>();
                                            game_log.add_entry("Used revival item!".to_string());
                                        },
                                        DeathAction::GiveUp => {
                                            let mut game_log = world.write_resource::<GameLog>();
                                            game_log.add_entry("Game Over - Player gave up.".to_string());
                                        },
                                        DeathAction::GameOver => {
                                            running = false;
                                        }
                                    }
                                }
                            }
                        }
                    },
                    KeyCode::Char('q') => {
                        running = false;
                    },
                    _ => {}
                }
            },
            _ => {}
        }
    }
}