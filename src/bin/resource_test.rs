use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::ui::{show_resource_management_screen, ResourceAction, draw_resource_bars};
use ascii_dungeon_explorer::systems::{ResourceRegenerationSystem, StatusEffectSystem, AbilityUsageSystem};
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
    
    // Create a player with resource management
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
            max_hp: 100,
            hp: 75,
            defense: 5,
            power: 10,
        })
        .with(PlayerResources::new(50, 30)) // 50 max mana, 30 max stamina
        .with(StatusEffects::new())
        .with(Abilities::new())
        .build();
    
    // Create systems
    let mut resource_regen_system = ResourceRegenerationSystem {};
    let mut status_effect_system = StatusEffectSystem {};
    let mut ability_usage_system = AbilityUsageSystem {};
    
    // Main loop
    let mut running = true;
    let mut turn_counter = 0;
    
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
            terminal.draw_text_centered(center_y - 15, "RESOURCE MANAGEMENT TEST", Color::Yellow, Color::Black)?;
            
            // Draw turn counter
            terminal.draw_text_centered(center_y - 13, &format!("Turn: {}", turn_counter), Color::White, Color::Black)?;
            
            // Draw resource bars
            draw_resource_bars(&world, player, center_x - 15, center_y - 10)?;
            
            // Draw status effects
            let status_effects = world.read_storage::<StatusEffects>();
            if let Some(effects) = status_effects.get(player) {
                terminal.draw_text_centered(
                    center_y - 6,
                    &format!("Active Status Effects: {}", effects.effects.len()),
                    Color::Cyan,
                    Color::Black
                )?;
                
                for (i, effect) in effects.effects.iter().enumerate() {
                    let y_pos = center_y - 4 + i as u16;
                    let color = if effect.effect_type.is_beneficial() { Color::Green } else { Color::Red };
                    terminal.draw_text_centered(
                        y_pos,
                        &format!("{} ({})", effect.effect_type.name(), effect.duration),
                        color,
                        Color::Black
                    )?;
                }
            }
            
            // Draw instructions
            terminal.draw_text_centered(
                center_y + 5,
                "Press 'r' to rest, 'm' to meditate, 's' for stamina boost, 'p' for poison, 'n' for next turn, 'q' to quit",
                Color::Grey,
                Color::Black
            )?;
            
            // Draw game log
            let game_log = world.read_resource::<GameLog>();
            for (i, entry) in game_log.entries.iter().rev().take(5).enumerate() {
                terminal.draw_text(5, center_y + 8 + i as u16, entry, Color::White, Color::Black)?;
            }
            
            // Flush the output
            terminal.flush()
        });
        
        // Wait for key press
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('r') => {
                        // Rest - restore 25% of all resources
                        let mut resources = world.write_storage::<PlayerResources>();
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        let mut game_log = world.write_resource::<GameLog>();
                        
                        if let (Some(resource), Some(stats)) = (resources.get_mut(player), combat_stats.get_mut(player)) {
                            let hp_restore = stats.max_hp / 4;
                            let mana_restore = resource.max_mana / 4;
                            let stamina_restore = resource.max_stamina / 4;
                            
                            stats.hp = i32::min(stats.hp + hp_restore, stats.max_hp);
                            resource.restore_mana(mana_restore);
                            resource.restore_stamina(stamina_restore);
                            
                            game_log.add_entry(format!("You rest and recover {} HP, {} mana, {} stamina.", 
                                hp_restore, mana_restore, stamina_restore));
                        }
                        
                        turn_counter += 1;
                    },
                    KeyCode::Char('m') => {
                        // Meditate - restore 50% mana, consume 25% stamina
                        let mut resources = world.write_storage::<PlayerResources>();
                        let mut game_log = world.write_resource::<GameLog>();
                        
                        if let Some(resource) = resources.get_mut(player) {
                            let stamina_cost = resource.max_stamina / 4;
                            if resource.consume_stamina(stamina_cost) {
                                let mana_restore = resource.max_mana / 2;
                                resource.restore_mana(mana_restore);
                                game_log.add_entry(format!("You meditate and restore {} mana for {} stamina.", 
                                    mana_restore, stamina_cost));
                            } else {
                                game_log.add_entry("Not enough stamina to meditate!".to_string());
                            }
                        }
                        
                        turn_counter += 1;
                    },
                    KeyCode::Char('s') => {
                        // Add stamina regeneration boost
                        let mut status_effects = world.write_storage::<StatusEffects>();
                        let mut game_log = world.write_resource::<GameLog>();
                        
                        if let Some(effects) = status_effects.get_mut(player) {
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::StaminaRegenBoost,
                                duration: 10,
                                magnitude: 3,
                            });
                            game_log.add_entry("You feel energized! Stamina regeneration boosted.".to_string());
                        }
                        
                        turn_counter += 1;
                    },
                    KeyCode::Char('p') => {
                        // Add poison effect
                        let mut status_effects = world.write_storage::<StatusEffects>();
                        let mut game_log = world.write_resource::<GameLog>();
                        
                        if let Some(effects) = status_effects.get_mut(player) {
                            effects.add_effect(StatusEffect {
                                effect_type: StatusEffectType::Poisoned,
                                duration: 5,
                                magnitude: 2,
                            });
                            game_log.add_entry("You are poisoned!".to_string());
                        }
                        
                        turn_counter += 1;
                    },
                    KeyCode::Char('n') => {
                        // Next turn - run systems
                        resource_regen_system.run_now(&world);
                        status_effect_system.run_now(&world);
                        ability_usage_system.run_now(&world);
                        world.maintain();
                        
                        turn_counter += 1;
                    },
                    KeyCode::Char('u') => {
                        // Use ability (test)
                        let mut wants_use_ability = world.write_storage::<WantsToUseAbility>();
                        wants_use_ability.insert(player, WantsToUseAbility {
                            ability: AbilityType::Fireball,
                            target: None,
                            mana_cost: 10,
                            stamina_cost: 5,
                        }).expect("Failed to insert ability use");
                        
                        ability_usage_system.run_now(&world);
                        world.maintain();
                        
                        turn_counter += 1;
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