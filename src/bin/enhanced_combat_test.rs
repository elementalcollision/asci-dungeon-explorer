use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::systems::{EnhancedCombatSystem, EnhancedDamageSystem, InitiativeSystem, TurnOrderSystem};
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
    
    // Create a player with enhanced combat components
    let player = world.create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::White,
            bg: Color::Black,
            render_order: 0,
        })
        .with(Player {})
        .with(Name { name: "Hero".to_string() })
        .with(CombatStats {
            max_hp: 50,
            hp: 50,
            defense: 3,
            power: 8,
        })
        .with(Attacker {
            attack_bonus: 2,
            critical_chance: 0.15, // 15% crit chance
            critical_multiplier: 2.5,
            attack_speed: 100,
            last_attack_turn: 0,
        })
        .with(Defender {
            armor_class: 12,
            damage_reduction: 1,
            evasion_chance: 0.1, // 10% evasion
            block_chance: 0.05,  // 5% block
            parry_chance: 0.05,  // 5% parry
        })
        .with(DamageResistances::new())
        .with(Initiative::new(15))
        .build();
    
    // Create an enemy with enhanced combat components
    let enemy = world.create_entity()
        .with(Position { x: 42, y: 25 })
        .with(Renderable {
            glyph: 'O',
            fg: Color::Red,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Orc Warrior".to_string() })
        .with(CombatStats {
            max_hp: 35,
            hp: 35,
            defense: 2,
            power: 6,
        })
        .with(Attacker {
            attack_bonus: 1,
            critical_chance: 0.08, // 8% crit chance
            critical_multiplier: 2.0,
            attack_speed: 120,
            last_attack_turn: 0,
        })
        .with(Defender {
            armor_class: 11,
            damage_reduction: 0,
            evasion_chance: 0.05, // 5% evasion
            block_chance: 0.0,
            parry_chance: 0.0,
        })
        .with(DamageResistances::new())
        .with(Initiative::new(12))
        .with(BlocksTile {})
        .build();
    
    // Add some damage resistances to the player
    {
        let mut resistances = world.write_storage::<DamageResistances>();
        if let Some(player_resist) = resistances.get_mut(player) {
            player_resist.add_resistance(DamageType::Fire, 0.25); // 25% fire resistance
            player_resist.add_resistance(DamageType::Ice, 0.1);   // 10% ice resistance
        }
    }
    
    // Create systems
    let mut enhanced_combat_system = EnhancedCombatSystem {};
    let mut enhanced_damage_system = EnhancedDamageSystem {};
    let mut initiative_system = InitiativeSystem {};
    let mut turn_order_system = TurnOrderSystem {};
    
    // Main loop
    let mut running = true;
    let mut combat_round = 1;
    
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
            terminal.draw_text_centered(center_y - 18, "ENHANCED COMBAT SYSTEM TEST", Color::Yellow, Color::Black)?;
            terminal.draw_text_centered(center_y - 16, &format!("Combat Round: {}", combat_round), Color::Cyan, Color::Black)?;
            
            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            let player_attacker = world.read_storage::<Attacker>().get(player).unwrap();
            let player_defender = world.read_storage::<Defender>().get(player).unwrap();
            let player_initiative = world.read_storage::<Initiative>().get(player).unwrap();
            
            terminal.draw_text_centered(
                center_y - 12,
                &format!("HERO: HP {}/{} | AC {} | Crit {}% | Initiative {}", 
                    player_stats.hp, player_stats.max_hp, 
                    player_defender.armor_class,
                    (player_attacker.critical_chance * 100.0) as i32,
                    player_initiative.current_initiative),
                Color::Green,
                Color::Black
            )?;
            
            // Draw enemy stats
            let enemy_stats = world.read_storage::<CombatStats>().get(enemy).unwrap();
            let enemy_attacker = world.read_storage::<Attacker>().get(enemy).unwrap();
            let enemy_defender = world.read_storage::<Defender>().get(enemy).unwrap();
            let enemy_initiative = world.read_storage::<Initiative>().get(enemy).unwrap();
            
            terminal.draw_text_centered(
                center_y - 10,
                &format!("ORC: HP {}/{} | AC {} | Crit {}% | Initiative {}", 
                    enemy_stats.hp, enemy_stats.max_hp,
                    enemy_defender.armor_class,
                    (enemy_attacker.critical_chance * 100.0) as i32,
                    enemy_initiative.current_initiative),
                Color::Red,
                Color::Black
            )?;
            
            // Draw turn indicator
            let current_turn = if player_initiative.has_acted && !enemy_initiative.has_acted {
                "Orc's Turn"
            } else if !player_initiative.has_acted && enemy_initiative.has_acted {
                "Hero's Turn"
            } else if !player_initiative.has_acted && !enemy_initiative.has_acted {
                if player_initiative.current_initiative > enemy_initiative.current_initiative {
                    "Hero's Turn"
                } else {
                    "Orc's Turn"
                }
            } else {
                "Round Complete"
            };
            
            terminal.draw_text_centered(center_y - 8, current_turn, Color::Yellow, Color::Black)?;
            
            // Draw instructions
            terminal.draw_text_centered(
                center_y - 5,
                "Press 'a' for Hero attack, 'e' for Orc attack, 'r' for new round, 'q' to quit",
                Color::Grey,
                Color::Black
            )?;
            
            // Draw combat log
            let game_log = world.read_resource::<GameLog>();
            for (i, entry) in game_log.entries.iter().rev().take(12).enumerate() {
                terminal.draw_text(5, center_y - 2 + i as u16, entry, Color::White, Color::Black)?;
            }
            
            // Flush the output
            terminal.flush()
        });
        
        // Wait for key press
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('a') => {
                        // Hero attacks Orc
                        let mut wants_attack = world.write_storage::<WantsToAttack>();
                        wants_attack.insert(player, WantsToAttack { target: enemy })
                            .expect("Failed to insert attack intent");
                        
                        // Run combat systems
                        enhanced_combat_system.run_now(&world);
                        enhanced_damage_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('e') => {
                        // Orc attacks Hero
                        let mut wants_attack = world.write_storage::<WantsToAttack>();
                        wants_attack.insert(enemy, WantsToAttack { target: player })
                            .expect("Failed to insert attack intent");
                        
                        // Run combat systems
                        enhanced_combat_system.run_now(&world);
                        enhanced_damage_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('r') => {
                        // Start new round
                        initiative_system.run_now(&world);
                        turn_order_system.run_now(&world);
                        world.maintain();
                        
                        combat_round += 1;
                    },
                    KeyCode::Char('i') => {
                        // Roll new initiative
                        let mut initiatives = world.write_storage::<Initiative>();
                        let mut rng = world.write_resource::<RandomNumberGenerator>();
                        
                        if let Some(player_init) = initiatives.get_mut(player) {
                            player_init.roll_initiative(&mut rng);
                        }
                        if let Some(enemy_init) = initiatives.get_mut(enemy) {
                            enemy_init.roll_initiative(&mut rng);
                        }
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("Initiative re-rolled!".to_string());
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