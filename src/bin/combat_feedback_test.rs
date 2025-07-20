use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::systems::{
    CombatFeedbackSystem, SoundEffectSystem, ScreenShakeSystem, 
    VisualEffectsSystem, ParticleEffectSystem, ScreenShakeState
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
    world.insert(ScreenShakeState::new());
    
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
        .with(Name { name: "Hero".to_string() })
        .with(CombatStats {
            max_hp: 100,
            hp: 100,
            defense: 5,
            power: 12,
        })
        .with(StatusEffects::new())
        .build();
    
    // Create an enemy
    let enemy = world.create_entity()
        .with(Position { x: 42, y: 25 })
        .with(Renderable {
            glyph: 'D',
            fg: Color::Red,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name { name: "Dragon".to_string() })
        .with(CombatStats {
            max_hp: 150,
            hp: 150,
            defense: 3,
            power: 18,
        })
        .with(StatusEffects::new())
        .with(BlocksTile {})
        .build();
    
    // Create systems
    let mut combat_feedback_system = CombatFeedbackSystem {};
    let mut sound_effect_system = SoundEffectSystem {};
    let mut screen_shake_system = ScreenShakeSystem {};
    let mut visual_effects_system = VisualEffectsSystem {};
    let mut particle_effect_system = ParticleEffectSystem {};
    
    // Main loop
    let mut running = true;
    let mut damage_type_cycle = 0;
    let mut test_mode = 0; // 0 = normal, 1 = critical, 2 = healing
    
    while running {
        // Display game state
        let _ = with_terminal(|terminal| {
            // Clear the screen
            terminal.clear()?;
            
            // Get screen shake offset
            let screen_shake = world.read_resource::<ScreenShakeState>();
            let shake_x = screen_shake.offset_x as i16;
            let shake_y = screen_shake.offset_y as i16;
            
            // Get terminal size
            let (width, height) = terminal.size();
            
            // Calculate center position with shake
            let center_x = ((width / 2) as i16 + shake_x).max(0) as u16;
            let center_y = ((height / 2) as i16 + shake_y).max(0) as u16;
            
            // Draw title
            terminal.draw_text_centered(center_y - 18, "COMBAT FEEDBACK SYSTEM TEST", Color::Yellow, Color::Black)?;
            
            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            terminal.draw_text_centered(
                center_y - 14,
                &format!("HERO: HP {}/{}", player_stats.hp, player_stats.max_hp),
                Color::Green,
                Color::Black
            )?;
            
            // Draw enemy stats
            let enemy_stats = world.read_storage::<CombatStats>().get(enemy).unwrap();
            terminal.draw_text_centered(
                center_y - 12,
                &format!("DRAGON: HP {}/{}", enemy_stats.hp, enemy_stats.max_hp),
                Color::Red,
                Color::Black
            )?;
            
            // Draw current test mode
            let mode_text = match test_mode {
                0 => "Normal Damage",
                1 => "Critical Damage",
                2 => "Healing",
                _ => "Unknown",
            };
            terminal.draw_text_centered(
                center_y - 10,
                &format!("Test Mode: {}", mode_text),
                Color::Cyan,
                Color::Black
            )?;
            
            // Draw current damage type
            let damage_types = [
                DamageType::Physical, DamageType::Fire, DamageType::Ice, 
                DamageType::Lightning, DamageType::Poison, DamageType::Holy, DamageType::Dark
            ];
            let current_damage_type = damage_types[damage_type_cycle % damage_types.len()];
            terminal.draw_text_centered(
                center_y - 8,
                &format!("Damage Type: {}", current_damage_type.name()),
                Color::Yellow,
                Color::Black
            )?;
            
            // Draw screen shake status
            if screen_shake.is_shaking() {
                terminal.draw_text_centered(
                    center_y - 6,
                    &format!("SCREEN SHAKE: {:.1} intensity", screen_shake.current_intensity),
                    Color::Red,
                    Color::Black
                )?;
            }
            
            // Draw instructions
            terminal.draw_text_centered(
                center_y - 4,
                "Press 'a' to attack enemy, 'h' to heal, 't' to change damage type",
                Color::Grey,
                Color::Black
            )?;
            terminal.draw_text_centered(
                center_y - 3,
                "Press 'm' to change test mode, 's' for screen shake test, 'q' to quit",
                Color::Grey,
                Color::Black
            )?;
            
            // Draw entities with potential flash effects
            let player_pos = world.read_storage::<Position>().get(player).unwrap();
            let player_render = world.read_storage::<Renderable>().get(player).unwrap();
            terminal.draw_text(
                (player_pos.x as i16 + shake_x).max(0) as u16, 
                (player_pos.y as i16 + shake_y).max(0) as u16, 
                &player_render.glyph.to_string(), 
                player_render.fg, 
                player_render.bg
            )?;
            
            let enemy_pos = world.read_storage::<Position>().get(enemy).unwrap();
            let enemy_render = world.read_storage::<Renderable>().get(enemy).unwrap();
            terminal.draw_text(
                (enemy_pos.x as i16 + shake_x).max(0) as u16, 
                (enemy_pos.y as i16 + shake_y).max(0) as u16, 
                &enemy_render.glyph.to_string(), 
                enemy_render.fg, 
                enemy_render.bg
            )?;
            
            // Draw combat log
            let game_log = world.read_resource::<GameLog>();
            for (i, entry) in game_log.entries.iter().rev().take(15).enumerate() {
                let log_y = (center_y as i16 + 2 + i as i16 + shake_y).max(0) as u16;
                terminal.draw_text(5, log_y, entry, Color::White, Color::Black)?;
            }
            
            // Flush the output
            terminal.flush()
        });
        
        // Wait for key press
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('a') => {
                        // Create damage effect
                        let damage_types = [
                            DamageType::Physical, DamageType::Fire, DamageType::Ice, 
                            DamageType::Lightning, DamageType::Poison, DamageType::Holy, DamageType::Dark
                        ];
                        let current_damage_type = damage_types[damage_type_cycle % damage_types.len()];
                        
                        let damage_amount = match test_mode {
                            1 => 25, // Critical damage
                            _ => 15, // Normal damage
                        };
                        
                        let is_critical = test_mode == 1;
                        
                        // Create damage info
                        let mut damage_info = world.write_storage::<DamageInfo>();
                        damage_info.insert(enemy, DamageInfo {
                            base_damage: damage_amount,
                            damage_type: current_damage_type,
                            source: player,
                            is_critical,
                            penetration: 0,
                        }).expect("Failed to insert damage info");
                        
                        // Apply damage to enemy stats
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(enemy) {
                            stats.hp = (stats.hp - damage_amount).max(0);
                        }
                        
                        // Run feedback systems
                        combat_feedback_system.run_now(&world);
                        sound_effect_system.run_now(&world);
                        screen_shake_system.run_now(&world);
                        visual_effects_system.run_now(&world);
                        particle_effect_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('h') => {
                        // Create healing effect
                        let healing_amount = 20;
                        
                        // Apply healing to player stats
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(player) {
                            stats.hp = (stats.hp + healing_amount).min(stats.max_hp);
                        }
                        
                        // Create healing feedback
                        let mut combat_feedback = world.write_storage::<CombatFeedback>();
                        let player_pos = world.read_storage::<Position>().get(player).unwrap();
                        
                        let healing_feedback = CombatFeedback {
                            feedback_type: CombatFeedbackType::HealingText { healing: healing_amount },
                            position: FloatingPosition {
                                x: player_pos.x as f32,
                                y: player_pos.y as f32,
                                offset_x: 0.0,
                                offset_y: -0.5,
                            },
                            duration: 1.5,
                            max_duration: 1.5,
                            color: Color::Green,
                            animation_type: AnimationType::FloatUp,
                        };
                        
                        combat_feedback.insert(player, healing_feedback)
                            .expect("Failed to insert healing feedback");
                        
                        // Add to game log
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry(format!("Hero heals for {} HP! ♪ *HEALING CHIME*", healing_amount));
                        
                        // Run feedback systems
                        visual_effects_system.run_now(&world);
                        world.maintain();
                    },
                    KeyCode::Char('t') => {
                        // Cycle through damage types
                        damage_type_cycle += 1;
                        let damage_types = [
                            DamageType::Physical, DamageType::Fire, DamageType::Ice, 
                            DamageType::Lightning, DamageType::Poison, DamageType::Holy, DamageType::Dark
                        ];
                        let current_type = damage_types[damage_type_cycle % damage_types.len()];
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry(format!("Switched to {} damage type!", current_type.name()));
                    },
                    KeyCode::Char('m') => {
                        // Cycle through test modes
                        test_mode = (test_mode + 1) % 3;
                        let mode_name = match test_mode {
                            0 => "Normal Damage",
                            1 => "Critical Damage",
                            2 => "Healing",
                            _ => "Unknown",
                        };
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry(format!("Switched to {} mode!", mode_name));
                    },
                    KeyCode::Char('s') => {
                        // Test screen shake
                        let mut screen_shake = world.write_resource::<ScreenShakeState>();
                        screen_shake.add_shake(ShakeIntensity::Heavy, 1.0);
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("*** SCREEN SHAKE TEST! ***".to_string());
                    },
                    KeyCode::Char('r') => {
                        // Reset HP
                        let mut combat_stats = world.write_storage::<CombatStats>();
                        if let Some(stats) = combat_stats.get_mut(player) {
                            stats.hp = stats.max_hp;
                        }
                        if let Some(stats) = combat_stats.get_mut(enemy) {
                            stats.hp = stats.max_hp;
                        }
                        
                        let mut game_log = world.write_resource::<GameLog>();
                        game_log.add_entry("HP restored for both characters!".to_string());
                    },
                    KeyCode::Char('q') => {
                        running = false;
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        
        // Update screen shake
        {
            let mut screen_shake = world.write_resource::<ScreenShakeState>();
            screen_shake.update();
        }
    }
}