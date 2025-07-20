use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::systems::{CombatRewardsSystem, TreasureSystem, TreasureGenerationSystem};
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
    
    // Create a player
    let player = world.create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::White,
            bg: Color::Black,
            render_order: 0,
        })
        .with(Player {})
        .with(Name { name: "Adventurer".to_string() })
        .with(CombatStats {
            max_hp: 100,
            hp: 100,
            defense: 5,
            power: 12,
        })
        .with(Experience::new())
        .build();
    
    // Create a regular monster with loot table
    let mut regular_loot_table = LootTable::new();
    regular_loot_table.add_entry(
        LootDrop::Consumable {
            name: "Health Potion".to_string(),
            healing: 15,
        },
        50 // 50% chance
    );
    regular_loot_table.add_entry(
        LootDrop::Equipment {
            name: "Iron Sword".to_string(),
            slot: EquipmentSlot::Melee,
            power_bonus: 3,
            defense_bonus: 0,
        },
        25 // 25% chance
    );
    regular_loot_table.add_entry(
        LootDrop::Currency { amount: 10 },
        75 // 75% chance
    );
    
    let regular_monster = world.create_entity()
        .with(Position { x: 42, y: 25 })
        .with(Renderable {
            glyph: 'o',
            fg: Color::Red,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Orc Warrior".to_string() })
        .with(CombatStats {
            max_hp: 30,
            hp: 1, // Almost dead for testing
            defense: 2,
            power: 8,
        })
        .with(regular_loot_table)
        .with(BlocksTile {})
        .build();
    
    // Create a unique/boss enemy
    let mut boss_loot_table = LootTable::new();
    boss_loot_table.add_entry(
        LootDrop::Equipment {
            name: "Dragon Scale Armor".to_string(),
            slot: EquipmentSlot::Armor,
            power_bonus: 0,
            defense_bonus: 8,
        },
        100 // Guaranteed drop
    );
    boss_loot_table.add_entry(
        LootDrop::Consumable {
            name: "Greater Health Potion".to_string(),
            healing: 50,
        },
        100 // Guaranteed drop
    );
    boss_loot_table.add_entry(
        LootDrop::Currency { amount: 100 },
        100 // Guaranteed drop
    );
    
    let boss_monster = world.create_entity()
        .with(Position { x: 38, y: 23 })
        .with(Renderable {
            glyph: 'D',
            fg: Color::DarkRed,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Ancient Dragon".to_string() })
        .with(CombatStats {
            max_hp: 150,
            hp: 1, // Almost dead for testing
            defense: 8,
            power: 20,
        })
        .with(boss_loot_table)
        .with(UniqueEnemy {})
        .with(BossEnemy {
            boss_type: BossType::AreaBoss,
            difficulty_multiplier: 2.0,
            guaranteed_drops: vec![
                LootDrop::Equipment {
                    name: "Dragon's Claw".to_string(),
                    slot: EquipmentSlot::Melee,
                    power_bonus: 12,
                    defense_bonus: 0,
                }
            ],
        })
        .with(BlocksTile {})
        .build();
    
    // Create treasure chests
    let treasure_loot_table = TreasureGenerationSystem::create_standard_loot_table(5, &mut world.write_resource::<RandomNumberGenerator>());
    
    let treasure_chest = TreasureGenerationSystem::create_treasure_chest(
        &world.entities(),
        Position { x: 35, y: 27 },
        TreasureType::Chest,
        treasure_loot_table,
        false // No key required
    );
    
    // Create a locked treasure
    let locked_treasure_loot = {
        let mut loot_table = LootTable::new();
        loot_table.add_entry(
            LootDrop::Equipment {
                name: "Legendary Sword".to_string(),
                slot: EquipmentSlot::Melee,
                power_bonus: 15,
                defense_bonus: 0,
            },
            100 // Guaranteed
        );
        loot_table
    };
    
    let locked_chest = TreasureGenerationSystem::create_treasure_chest(
        &world.entities(),
        Position { x: 45, y: 27 },
        TreasureType::SecretCache,
        locked_treasure_loot,
        true // Requires key
    );
    
    // Create systems
    let mut combat_rewards_system = CombatRewardsSystem {};
    let mut treasure_system = TreasureSystem {};
    
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
            terminal.draw_text_centered(center_y - 18, "COMBAT REWARDS SYSTEM TEST", Color::Yellow, Color::Black)?;
            
            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            let player_exp = world.read_storage::<Experience>().get(player).unwrap();
            
            terminal.draw_text_centered(
                center_y - 14,
                &format!("ADVENTURER: HP {}/{} | Level {} | XP {}/{}", 
                    player_stats.hp, player_stats.max_hp,
                    player_exp.level, player_exp.current, player_exp.level_up_target),
                Color::Green,
                Color::Black
            )?;
            
            // Draw monster stats
            let regular_stats = world.read_storage::<CombatStats>().get(regular_monster);
            let boss_stats = world.read_storage::<CombatStats>().get(boss_monster);
            
            if let Some(stats) = regular_stats {
                terminal.draw_text_centered(
                    center_y - 12,
                    &format!("ORC WARRIOR: HP {}/{} (Regular Enemy)", stats.hp, stats.max_hp),
                    if stats.hp > 0 { Color::Red } else { Color::DarkRed },
                    Color::Black
                )?;
            }
            
            if let Some(stats) = boss_stats {
                terminal.draw_text_centered(
                    center_y - 10,
                    &format!("ANCIENT DRAGON: HP {}/{} (Unique Boss)", stats.hp, stats.max_hp),
                    if stats.hp > 0 { Color::Red } else { Color::DarkRed },
                    Color::Black
                )?;
            }
            
            // Draw treasure status
            let treasure_chest_treasure = world.read_storage::<Treasure>().get(treasure_chest);
            let locked_chest_treasure = world.read_storage::<Treasure>().get(locked_chest);
            
            if let Some(treasure) = treasure_chest_treasure {
                terminal.draw_text_centered(
                    center_y - 8,
                    &format!("TREASURE CHEST: {}", if treasure.is_opened { "OPENED" } else { "CLOSED" }),
                    if treasure.is_opened { Color::Grey } else { Color::Yellow },
                    Color::Black
                )?;
            }
            
            if let Some(treasure) = locked_chest_treasure {
                terminal.draw_text_centered(
                    center_y - 6,
                    &format!("SECRET CACHE: {} {}", 
                        if treasure.is_opened { "OPENED" } else { "LOCKED" },
                        if treasure.requires_key { "(Requires Key)" } else { "" }),
                    if treasure.is_opened { Color::Grey } else { Color::Magenta },
                    Color::Black
                )?;
            }
            
            // Draw instructions
            terminal.draw_text_centered(
                center_y - 4,
                "Press '1' to kill orc, '2' to kill dragon, '3' to open chest, '4' to open cache",
                Color::Grey,
                Color::Black
            )?;
            terminal.draw_text_centered(
                center_y - 3,
                "Press 'r' to reset monsters, 'q' to quit",
                Color::Grey,
                Color::Black
            )?;
            
            // Draw entities on map
            let player_pos = world.read_storage::<Position>().get(player).unwrap();
            let player_render = world.read_storage::<Renderable>().get(player).unwrap();
            terminal.draw_text(player_pos.x as u16, player_pos.y as u16, &player_render.glyph.to_string(), player_render.fg, player_render.bg)?;
            
            // Draw living monsters
            if let Some(stats) = world.read_storage::<CombatStats>().get(regular_monster) {
                if stats.hp > 0 {
                    let pos = world.read_storage::<Position>().get(regular_monster).unwrap();
                    let render = world.read_storage::<Renderable>().get(regular_monster).unwrap();
                    terminal.draw_text(pos.x as u16, pos.y as u16, &render.glyph.to_string(), render.fg, render.bg)?;
                }
            }
            
            if let Some(stats) = world.read_storage::<CombatStats>().get(boss_monster) {
                if stats.hp > 0 {
                    let pos = world.read_storage::<Position>().get(boss_monster).unwrap();
                    let render = world.read_storage::<Renderable>().get(boss_monster).unwrap();
                    terminal.draw_text(pos.x as u16, pos.y as u16, &render.glyph.to_string(), render.fg, render.bg)?;
                }
            }
            
            // Draw treasures
            let chest_pos = world.read_storage::<Position>().get(treasure_chest).unwrap();
            let chest_render = world.read_storage::<Renderable>().get(treasure_chest).unwrap();
            terminal.draw_text(chest_pos.x as u16, chest_pos.y as u16, &chest_render.glyph.to_string(), chest_render.fg, chest_render.bg)?;
            
            let locked_pos = world.read_storage::<Position>().get(locked_chest).unwrap();
            let locked_render = world.read_storage::<Renderable>().get(locked_chest).unwrap();
            terminal.draw_text(locked_pos.x as u16, locked_pos.y as u16, &locked_render.glyph.to_string(), locked_render.fg, locked_render.bg)?;
            
            // Draw items on ground
            let items = world.read_storage::<Item>();
            let positions = world.read_storage::<Position>();
            let renderables = world.read_storage::<Renderable>();
            
            for (_, pos, render) in (&items, &positions, &renderables).join() {
                terminal.draw_text(pos.x as u16, pos.y as u16, &render.glyph.to_string(), render.fg, render.bg)?;
            }
            
            // Draw combat log
            let game_log = world.read_resource::<GameLog>();
            for (i, entry) in game_log.entries.iter().rev().take(15).enumerate() {
                terminal.draw_text(5, center_y + 2 + i as u16, entry, Color::White, Color::Black)?;
            }
            
            // Flush the output
            terminal.flush()
        });
        
        // Wait for key press
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('1') => {
                        // Kill the orc
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(regular_monster) {
                            if stats.hp > 0 {
                                stats.hp = 0;
                                
                                // Run combat rewards system
                                combat_rewards_system.run_now(&world);
                                world.maintain();
                            }
                        }
                    },
                    KeyCode::Char('2') => {
                        // Kill the dragon
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(boss_monster) {
                            if stats.hp > 0 {
                                stats.hp = 0;
                                
                                // Run combat rewards system
                                combat_rewards_system.run_now(&world);
                                world.maintain();
                            }
                        }
                    },
                    KeyCode::Char('3') => {
                        // Open treasure chest
                        let mut wants_interact = world.write_storage::<WantsToInteract>();
                        wants_interact.insert(player, WantsToInteract { target: treasure_chest })
                            .expect("Failed to insert interaction");
                        
                        treasure_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('4') => {
                        // Try to open locked cache
                        let mut wants_interact = world.write_storage::<WantsToInteract>();
                        wants_interact.insert(player, WantsToInteract { target: locked_chest })
                            .expect("Failed to insert interaction");
                        
                        treasure_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('r') => {
                        // Reset monsters
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        
                        if let Some(stats) = combat_stats.get_mut(regular_monster) {
                            stats.hp = stats.max_hp;
                        }
                        
                        if let Some(stats) = combat_stats.get_mut(boss_monster) {
                            stats.hp = stats.max_hp;
                        }
                        
                        // Clear existing items
                        let items: Vec<Entity> = {
                            let items = world.read_storage::<Item>();
                            let entities = world.entities();
                            (&entities, &items).join().map(|(e, _)| e).collect()
                        };
                        
                        for item in items {
                            world.entities().delete(item).expect("Failed to delete item");
                        }
                        
                        // Reset treasures
                        let mut treasures = world.write_storage::<Treasure>();
                        if let Some(treasure) = treasures.get_mut(treasure_chest) {
                            treasure.is_opened = false;
                        }
                        if let Some(treasure) = treasures.get_mut(locked_chest) {
                            treasure.is_opened = false;
                        }
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("=== RESET ===".to_string());
                        
                        world.maintain();
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