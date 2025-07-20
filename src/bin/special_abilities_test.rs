use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::systems::{
    SpecialAbilitiesSystem, AbilityTargetingSystem, AbilityCooldownSystem
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
    
    // Create a player with special abilities
    let player = world.create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::White,
            bg: Color::Black,
            render_order: 0,
        })
        .with(Player {})
        .with(Name { name: "Battlemage".to_string() })
        .with(CombatStats {
            max_hp: 80,
            hp: 80,
            defense: 4,
            power: 12,
        })
        .with(PlayerResources::new(50, 40)) // 50 mana, 40 stamina
        .with(StatusEffects::new())
        .with(Abilities::new())
        .build();
    
    // Create multiple enemies for testing area abilities
    let orc1 = world.create_entity()
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
            hp: 30,
            defense: 2,
            power: 8,
        })
        .with(StatusEffects::new())
        .with(BlocksTile {})
        .build();
    
    let orc2 = world.create_entity()
        .with(Position { x: 41, y: 26 })
        .with(Renderable {
            glyph: 'o',
            fg: Color::Red,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Orc Grunt".to_string() })
        .with(CombatStats {
            max_hp: 25,
            hp: 25,
            defense: 1,
            power: 6,
        })
        .with(StatusEffects::new())
        .with(BlocksTile {})
        .build();
    
    let goblin = world.create_entity()
        .with(Position { x: 38, y: 24 })
        .with(Renderable {
            glyph: 'g',
            fg: Color::Green,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Goblin Scout".to_string() })
        .with(CombatStats {
            max_hp: 15,
            hp: 10, // Wounded for healing test
            defense: 3,
            power: 4,
        })
        .with(StatusEffects::new())
        .with(BlocksTile {})
        .build();
    
    // Give the player various abilities
    {
        let mut abilities = world.write_storage::<Abilities>();
        if let Some(player_abilities) = abilities.get_mut(player) {
            // Fighter abilities
            player_abilities.add_ability(AbilityType::PowerAttack);
            player_abilities.add_ability(AbilityType::Cleave);
            player_abilities.add_ability(AbilityType::ShieldBash);
            player_abilities.add_ability(AbilityType::SecondWind);
            
            // Mage abilities
            player_abilities.add_ability(AbilityType::Fireball);
            player_abilities.add_ability(AbilityType::IceSpike);
            player_abilities.add_ability(AbilityType::MagicMissile);
            player_abilities.add_ability(AbilityType::Teleport);
            
            // Cleric abilities
            player_abilities.add_ability(AbilityType::Heal);
            player_abilities.add_ability(AbilityType::DivineProtection);
            
            // Rogue abilities
            player_abilities.add_ability(AbilityType::Backstab);
            player_abilities.add_ability(AbilityType::ShadowStep);
        }
    }
    
    // Create systems
    let mut special_abilities_system = SpecialAbilitiesSystem {};
    let mut ability_targeting_system = AbilityTargetingSystem {};
    let mut ability_cooldown_system = AbilityCooldownSystem {};
    
    // Main loop
    let mut running = true;
    let mut selected_ability = 0;
    let available_abilities = [
        AbilityType::PowerAttack,
        AbilityType::Cleave,
        AbilityType::ShieldBash,
        AbilityType::SecondWind,
        AbilityType::Fireball,
        AbilityType::IceSpike,
        AbilityType::MagicMissile,
        AbilityType::Heal,
        AbilityType::DivineProtection,
        AbilityType::Backstab,
        AbilityType::ShadowStep,
    ];
    
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
            terminal.draw_text_centered(center_y - 18, "SPECIAL ABILITIES SYSTEM TEST", Color::Yellow, Color::Black)?;
            
            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            let player_resources = world.read_storage::<PlayerResources>().get(player).unwrap();
            let player_abilities = world.read_storage::<Abilities>().get(player).unwrap();
            
            terminal.draw_text_centered(
                center_y - 14,
                &format!("BATTLEMAGE: HP {}/{} | Mana {}/{} | Stamina {}/{}", 
                    player_stats.hp, player_stats.max_hp,
                    player_resources.mana, player_resources.max_mana,
                    player_resources.stamina, player_resources.max_stamina),
                Color::Green,
                Color::Black
            )?;
            
            // Draw enemy stats
            let orc1_stats = world.read_storage::<CombatStats>().get(orc1).unwrap();
            let orc2_stats = world.read_storage::<CombatStats>().get(orc2).unwrap();
            let goblin_stats = world.read_storage::<CombatStats>().get(goblin).unwrap();
            
            terminal.draw_text_centered(
                center_y - 12,
                &format!("ENEMIES: Orc1 {}/{} HP | Orc2 {}/{} HP | Goblin {}/{} HP", 
                    orc1_stats.hp, orc1_stats.max_hp,
                    orc2_stats.hp, orc2_stats.max_hp,
                    goblin_stats.hp, goblin_stats.max_hp),
                Color::Red,
                Color::Black
            )?;
            
            // Draw selected ability
            let current_ability = available_abilities[selected_ability % available_abilities.len()];
            let mana_cost = current_ability.get_mana_cost();
            let stamina_cost = current_ability.get_stamina_cost();
            let cooldown = player_abilities.get_cooldown(current_ability);
            
            let ability_color = if cooldown > 0 {
                Color::Red
            } else if player_resources.mana >= mana_cost && player_resources.stamina >= stamina_cost {
                Color::Green
            } else {
                Color::Yellow
            };
            
            terminal.draw_text_centered(
                center_y - 10,
                &format!("Selected: {} (Mana: {}, Stamina: {}, Cooldown: {})", 
                    current_ability.name(), mana_cost, stamina_cost, cooldown),
                ability_color,
                Color::Black
            )?;
            
            // Draw ability description
            terminal.draw_text_centered(
                center_y - 8,
                current_ability.description(),
                Color::Cyan,
                Color::Black
            )?;
            
            // Draw status effects
            let player_effects = world.read_storage::<StatusEffects>().get(player).unwrap();
            if !player_effects.effects.is_empty() {
                let effects_text = player_effects.effects.iter()
                    .map(|e| format!("{} ({})", e.effect_type.name(), e.duration))
                    .collect::<Vec<_>>()
                    .join(", ");
                terminal.draw_text_centered(
                    center_y - 6,
                    &format!("Status Effects: {}", effects_text),
                    Color::Magenta,
                    Color::Black
                )?;
            }
            
            // Draw instructions
            terminal.draw_text_centered(
                center_y - 4,
                "Left/Right: Select ability | Space: Use ability | R: Restore resources",
                Color::Grey,
                Color::Black
            )?;
            terminal.draw_text_centered(
                center_y - 3,
                "H: Heal self | T: Next turn | Q: Quit",
                Color::Grey,
                Color::Black
            )?;
            
            // Draw entities on map
            let player_pos = world.read_storage::<Position>().get(player).unwrap();
            let player_render = world.read_storage::<Renderable>().get(player).unwrap();
            terminal.draw_text(player_pos.x as u16, player_pos.y as u16, &player_render.glyph.to_string(), player_render.fg, player_render.bg)?;
            
            let orc1_pos = world.read_storage::<Position>().get(orc1).unwrap();
            let orc1_render = world.read_storage::<Renderable>().get(orc1).unwrap();
            terminal.draw_text(orc1_pos.x as u16, orc1_pos.y as u16, &orc1_render.glyph.to_string(), orc1_render.fg, orc1_render.bg)?;
            
            let orc2_pos = world.read_storage::<Position>().get(orc2).unwrap();
            let orc2_render = world.read_storage::<Renderable>().get(orc2).unwrap();
            terminal.draw_text(orc2_pos.x as u16, orc2_pos.y as u16, &orc2_render.glyph.to_string(), orc2_render.fg, orc2_render.bg)?;
            
            let goblin_pos = world.read_storage::<Position>().get(goblin).unwrap();
            let goblin_render = world.read_storage::<Renderable>().get(goblin).unwrap();
            terminal.draw_text(goblin_pos.x as u16, goblin_pos.y as u16, &goblin_render.glyph.to_string(), goblin_render.fg, goblin_render.bg)?;
            
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
                    KeyCode::Left => {
                        selected_ability = (selected_ability + available_abilities.len() - 1) % available_abilities.len();
                    },
                    KeyCode::Right => {
                        selected_ability = (selected_ability + 1) % available_abilities.len();
                    },
                    KeyCode::Char(' ') => {
                        // Use selected ability
                        let current_ability = available_abilities[selected_ability % available_abilities.len()];
                        let mana_cost = current_ability.get_mana_cost();
                        let stamina_cost = current_ability.get_stamina_cost();
                        
                        // Create ability use request
                        let mut wants_use_ability = world.write_storage::<WantsToUseAbility>();
                        wants_use_ability.insert(player, WantsToUseAbility {
                            ability: current_ability,
                            target: None, // Will be auto-targeted if needed
                            mana_cost,
                            stamina_cost,
                        }).expect("Failed to insert ability use");
                        
                        // Run ability systems
                        ability_targeting_system.run_now(&world);
                        special_abilities_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('h') => {
                        // Quick heal
                        let mut wants_use_ability = world.write_storage::<WantsToUseAbility>();
                        wants_use_ability.insert(player, WantsToUseAbility {
                            ability: AbilityType::Heal,
                            target: Some(player), // Self-target
                            mana_cost: AbilityType::Heal.get_mana_cost(),
                            stamina_cost: AbilityType::Heal.get_stamina_cost(),
                        }).expect("Failed to insert heal ability");
                        
                        special_abilities_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('r') => {
                        // Restore resources
                        let mut resources = world.write_storage::<PlayerResources>();
                        if let Some(resource) = resources.get_mut(player) {
                            resource.mana = resource.max_mana;
                            resource.stamina = resource.max_stamina;
                        }
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("Resources fully restored!".to_string());
                    },
                    KeyCode::Char('t') => {
                        // Next turn - update cooldowns
                        ability_cooldown_system.run_now(&world);
                        
                        // Regenerate some resources
                        let mut resources = world.write_storage::<PlayerResources>();
                        if let Some(resource) = resources.get_mut(player) {
                            resource.restore_mana(2);
                            resource.restore_stamina(3);
                        }
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("--- Next Turn ---".to_string());
                        
                        world.maintain();
                    },
                    KeyCode::Char('d') => {
                        // Damage player for testing healing
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(player) {
                            stats.hp = (stats.hp - 15).max(1);
                        }
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("You take 15 damage for testing!".to_string());
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