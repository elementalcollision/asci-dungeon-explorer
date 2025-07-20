use ascii_dungeon_explorer::combat::{CombatSystem, DamageSystem, DeathSystem};
use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use crossterm::event::{read, Event, KeyCode};
use crossterm::style::Color;
use specs::{Builder, RunNow, World, WorldExt};

fn main() {
    // Create a world
    let mut world = World::new();

    // Register components
    register_components(&mut world);

    // Add resources
    world.insert(GameLog::new());
    world.insert(RandomNumberGenerator::new_with_random_seed());

    // Create a player
    let player = world
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::White,
            bg: Color::Black,
            render_order: 0,
        })
        .with(Player {})
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build();

    // Create an enemy
    let enemy = world
        .create_entity()
        .with(Position { x: 42, y: 25 })
        .with(Renderable {
            glyph: 'g',
            fg: Color::Red,
            bg: Color::Black,
            render_order: 1,
        })
        .with(Monster {})
        .with(Name {
            name: "Goblin".to_string(),
        })
        .with(CombatStats {
            max_hp: 15,
            hp: 15,
            defense: 1,
            power: 3,
        })
        .with(BlocksTile {})
        .build();

    // Create systems
    let mut combat_system = CombatSystem {};
    let mut damage_system = DamageSystem {};
    let mut death_system = DeathSystem {};

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
            terminal.draw_text_centered(
                center_y - 15,
                "COMBAT TEST",
                Color::Yellow,
                Color::Black,
            )?;

            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            terminal.draw_text_centered(
                center_y - 10,
                &format!(
                    "Player: HP {}/{}, Power {}, Defense {}",
                    player_stats.hp, player_stats.max_hp, player_stats.power, player_stats.defense
                ),
                Color::White,
                Color::Black,
            )?;

            // Draw enemy stats if alive
            if let Some(enemy_stats) = world.read_storage::<CombatStats>().get(enemy) {
                terminal.draw_text_centered(
                    center_y - 8,
                    &format!(
                        "Goblin: HP {}/{}, Power {}, Defense {}",
                        enemy_stats.hp, enemy_stats.max_hp, enemy_stats.power, enemy_stats.defense
                    ),
                    Color::Red,
                    Color::Black,
                )?;
            } else {
                terminal.draw_text_centered(
                    center_y - 8,
                    "Goblin: Dead",
                    Color::Grey,
                    Color::Black,
                )?;
            }

            // Draw instructions
            terminal.draw_text_centered(
                center_y - 5,
                "Press 'a' to attack the goblin, 'q' to quit",
                Color::Grey,
                Color::Black,
            )?;

            // Draw combat log
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
                    KeyCode::Char('a') => {
                        // Create attack intent
                        let mut wants_attack = world.write_storage::<WantsToAttack>();
                        wants_attack
                            .insert(player, WantsToAttack { target: enemy })
                            .expect("Failed to insert attack intent");

                        // Run combat systems
                        combat_system.run_now(&world);
                        damage_system.run_now(&world);
                        death_system.run_now(&world);
                        world.maintain();

                        // Enemy attacks back if alive
                        if world.read_storage::<CombatStats>().get(enemy).is_some() {
                            let mut wants_attack = world.write_storage::<WantsToAttack>();
                            wants_attack
                                .insert(enemy, WantsToAttack { target: player })
                                .expect("Failed to insert attack intent");

                            // Run combat systems again
                            combat_system.run_now(&world);
                            damage_system.run_now(&world);
                            death_system.run_now(&world);
                            world.maintain();
                        }
                    }
                    KeyCode::Char('q') => {
                        running = false;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
