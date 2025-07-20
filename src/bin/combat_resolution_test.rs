use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::systems::{
    CombatResolutionSystem, CriticalHitSystem, CriticalChanceSystem, 
    DamageTypeSystem, ResistanceManagementSystem
};
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
    
    // Create a player with full combat resolution components
    let player = world.create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::White,
            bg: Color::Black,
            render_order: 0,
        })
        .with(Player {})
        .with(Name { name: "Paladin".to_string() })
        .with(CombatStats {
            max_hp: 60,
            hp: 60,
            defense: 4,
            power: 10,
        })
        .with(Attacker {
            attack_bonus: 3,
            critical_chance: 0.2, // 20% crit chance
            critical_multiplier: 2.5,
            attack_speed: 100,
            last_attack_turn: 0,
        })
        .with(Defender {
            armor_class: 14,
            damage_reduction: 2,
            evasion_chance: 0.08,
            block_chance: 0.12,
            parry_chance: 0.1,
        })
        .with(Attributes {
            strength: 16,
            dexterity: 14,
            constitution: 15,
            intelligence: 12,
            wisdom: 13,
            charisma: 16,
            unspent_points: 0,
        })
        .with(Skills::new())
        .with(DamageResistances::new())
        .with(StatusEffects::new())
        .with(Initiative::new(18))
        .with(Inventory::new(10))
        .build();
    
    // Create an enemy with different damage types
    let fire_elemental = world.create_entity()
        .with(Position { x: 42, y: 25 })
        .with(Renderable {
            glyph: 'F',
            fg: Color::Red,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Fire Elemental".to_string() })
        .with(CombatStats {
            max_hp: 45,
            hp: 45,
            defense: 1,
            power: 8,
        })
        .with(Attacker {
            attack_bonus: 2,
            critical_chance: 0.15,
            critical_multiplier: 2.0,
            attack_speed: 90,
            last_attack_turn: 0,
        })
        .with(Defender {
            armor_class: 12,
            damage_reduction: 0,
            evasion_chance: 0.15, // High evasion
            block_chance: 0.0,
            parry_chance: 0.0,
        })
        .with(Attributes {
            strength: 12,
            dexterity: 18,
            constitution: 14,
            intelligence: 8,
            wisdom: 10,
            charisma: 6,
            unspent_points: 0,
        })
        .with(Skills::new())
        .with(DamageResistances::new())
        .with(StatusEffects::new())
        .with(Initiative::new(16))
        .with(BlocksTile {})
        .build();
    
    // Set up player resistances and skills
    {
        let mut resistances = world.write_storage::<DamageResistances>();
        if let Some(player_resist) = resistances.get_mut(player) {
            player_resist.add_resistance(DamageType::Fire, 0.3); // 30% fire resistance
            player_resist.add_resistance(DamageType::Dark, 0.4); // 40% dark resistance
            player_resist.add_resistance(DamageType::Holy, -0.2); // Vulnerable to holy (healing)
        }
        
        if let Some(elemental_resist) = resistances.get_mut(fire_elemental) {
            elemental_resist.add_resistance(DamageType::Fire, 0.8); // 80% fire resistance
            elemental_resist.add_resistance(DamageType::Ice, -0.5); // 50% vulnerable to ice
            elemental_resist.add_resistance(DamageType::Physical, 0.2); // 20% physical resistance
        }
        
        let mut skills = world.write_storage::<Skills>();
        if let Some(player_skills) = skills.get_mut(player) {
            player_skills.skills.insert(SkillType::MeleeWeapons, 4);
            player_skills.skills.insert(SkillType::Defense, 3);
        }
    }
    
    // Create systems
    let mut combat_resolution_system = CombatResolutionSystem {};
    let mut critical_hit_system = CriticalHitSystem {};
    let mut critical_chance_system = CriticalChanceSystem {};
    let mut damage_type_system = DamageTypeSystem {};
    let mut resistance_management_system = ResistanceManagementSystem {};
    
    // Main loop
    let mut running = true;
    let mut combat_round = 1;
    let mut damage_type_cycle = 0;
    
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
            terminal.draw_text_centered(center_y - 18, "COMBAT RESOLUTION SYSTEM TEST", Color::Yellow, Color::Black)?;
            terminal.draw_text_centered(center_y - 16, &format!("Combat Round: {}", combat_round), Color::Cyan, Color::Black)?;
            
            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            let player_attacker = world.read_storage::<Attacker>().get(player).unwrap();
            let player_defender = world.read_storage::<Defender>().get(player).unwrap();
            let player_attrs = world.read_storage::<Attributes>().get(player).unwrap();
            
            terminal.draw_text_centered(
                center_y - 12,
                &format!("PALADIN: HP {}/{} | AC {} | STR {} | DEX {}", 
                    player_stats.hp, player_stats.max_hp, 
                    player_defender.armor_class,
                    player_attrs.strength,
                    player_attrs.dexterity),
                Color::Green,
                Color::Black
            )?;
            
            terminal.draw_text_centered(
                center_y - 11,
                &format!("Crit: {}% | Evasion: {}% | Block: {}% | Parry: {}%", 
                    (player_attacker.critical_chance * 100.0) as i32,
                    (player_defender.evasion_chance * 100.0) as i32,
                    (player_defender.block_chance * 100.0) as i32,
                    (player_defender.parry_chance * 100.0) as i32),
                Color::Green,
                Color::Black
            )?;
            
            // Draw enemy stats
            let enemy_stats = world.read_storage::<CombatStats>().get(fire_elemental).unwrap();
            let enemy_attacker = world.read_storage::<Attacker>().get(fire_elemental).unwrap();
            let enemy_defender = world.read_storage::<Defender>().get(fire_elemental).unwrap();
            
            terminal.draw_text_centered(
                center_y - 9,
                &format!("FIRE ELEMENTAL: HP {}/{} | AC {} | Evasion {}%", 
                    enemy_stats.hp, enemy_stats.max_hp,
                    enemy_defender.armor_class,
                    (enemy_defender.evasion_chance * 100.0) as i32),
                Color::Red,
                Color::Black
            )?;
            
            // Draw resistances
            let player_resist = world.read_storage::<DamageResistances>().get(player).unwrap();
            let enemy_resist = world.read_storage::<DamageResistances>().get(fire_elemental).unwrap();
            
            terminal.draw_text_centered(
                center_y - 7,
                &format!("Player Fire Resist: {}% | Enemy Ice Vuln: {}%", 
                    (player_resist.get_resistance(DamageType::Fire) * 100.0) as i32,
                    (enemy_resist.get_resistance(DamageType::Ice) * -100.0) as i32),
                Color::Cyan,
                Color::Black
            )?;
            
            // Draw current damage type
            let damage_types = [
                DamageType::Physical, DamageType::Fire, DamageType::Ice, 
                DamageType::Lightning, DamageType::Holy, DamageType::Dark
            ];
            let current_damage_type = damage_types[damage_type_cycle % damage_types.len()];
            
            terminal.draw_text_centered(
                center_y - 5,
                &format!("Current Attack Type: {}", current_damage_type.name()),
                Color::Yellow,
                Color::Black
            )?;
            
            // Draw instructions
            terminal.draw_text_centered(
                center_y - 3,
                "Press 'a' to attack, 't' to change damage type, 'r' for new round, 'q' to quit",
                Color::Grey,
                Color::Black
            )?;
            
            // Draw status effects
            let player_effects = world.read_storage::<StatusEffects>().get(player).unwrap();
            let enemy_effects = world.read_storage::<StatusEffects>().get(fire_elemental).unwrap();
            
            if !player_effects.effects.is_empty() {
                terminal.draw_text_centered(
                    center_y - 1,
                    &format!("Player Effects: {}", player_effects.effects.len()),
                    Color::Magenta,
                    Color::Black
                )?;
            }
            
            if !enemy_effects.effects.is_empty() {
                terminal.draw_text_centered(
                    center_y,
                    &format!("Enemy Effects: {}", enemy_effects.effects.len()),
                    Color::Magenta,
                    Color::Black
                )?;
            }
            
            // Draw combat log
            let game_log = world.read_resource::<GameLog>();
            for (i, entry) in game_log.entries.iter().rev().take(12).enumerate() {
                terminal.draw_text(5, center_y + 2 + i as u16, entry, Color::White, Color::Black)?;
            }
            
            // Flush the output
            terminal.flush()
        });
        
        // Wait for key press
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('a') => {
                        // Player attacks with current damage type
                        let mut wants_attack = world.write_storage::<WantsToAttack>();
                        wants_attack.insert(player, WantsToAttack { target: fire_elemental })
                            .expect("Failed to insert attack intent");
                        
                        // Run combat resolution systems
                        critical_chance_system.run_now(&world);
                        resistance_management_system.run_now(&world);
                        combat_resolution_system.run_now(&world);
                        critical_hit_system.run_now(&world);
                        damage_type_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('e') => {
                        // Enemy attacks (always fire damage)
                        let mut wants_attack = world.write_storage::<WantsToAttack>();
                        wants_attack.insert(fire_elemental, WantsToAttack { target: player })
                            .expect("Failed to insert attack intent");
                        
                        // Manually set damage type to fire for elemental
                        // In a real system, this would be determined by the attacker's weapon/spell
                        
                        // Run combat resolution systems
                        critical_chance_system.run_now(&world);
                        resistance_management_system.run_now(&world);
                        combat_resolution_system.run_now(&world);
                        critical_hit_system.run_now(&world);
                        damage_type_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('t') => {
                        // Cycle through damage types
                        damage_type_cycle += 1;
                        let damage_types = [
                            DamageType::Physical, DamageType::Fire, DamageType::Ice, 
                            DamageType::Lightning, DamageType::Holy, DamageType::Dark
                        ];
                        let current_type = damage_types[damage_type_cycle % damage_types.len()];
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry(format!("Switched to {} damage type!", current_type.name()));
                    },
                    KeyCode::Char('r') => {
                        // Start new round
                        combat_round += 1;
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry(format!("=== Combat Round {} ===", combat_round));
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