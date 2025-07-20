use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::systems::{ExperienceGainSystem, ExperienceSystem, LevelUpSystem};
use ascii_dungeon_explorer::ui::{show_character_progression, ProgressionAction};
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

    // Create a player with progression components
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
            name: "Test Hero".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .with(Experience::new())
        .with(Attributes::new())
        .with(Skills::new())
        .with(Abilities::new())
        .with(CharacterClass {
            class_type: ClassType::Fighter,
        })
        .build();

    // Create an enemy for testing experience gain
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
            name: "Test Goblin".to_string(),
        })
        .with(CombatStats {
            max_hp: 15,
            hp: 0, // Already dead for testing
            defense: 1,
            power: 3,
        })
        .build();

    // Create systems
    let mut experience_gain_system = ExperienceGainSystem {};
    let mut experience_system = ExperienceSystem {};
    let mut level_up_system = LevelUpSystem {};

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
                "CHARACTER PROGRESSION TEST",
                Color::Yellow,
                Color::Black,
            )?;

            // Draw player stats
            let player_exp = world.read_storage::<Experience>().get(player).unwrap();
            let player_attrs = world.read_storage::<Attributes>().get(player).unwrap();

            terminal.draw_text_centered(
                center_y - 10,
                &format!(
                    "Level: {} | XP: {}/{} | Unspent Points: {}",
                    player_exp.level,
                    player_exp.current,
                    player_exp.level_up_target,
                    player_exp.unspent_points
                ),
                Color::White,
                Color::Black,
            )?;

            terminal.draw_text_centered(
                center_y - 8,
                &format!(
                    "Attribute Points: {} | Skill Points: {}",
                    player_attrs.unspent_points,
                    world
                        .read_storage::<Skills>()
                        .get(player)
                        .unwrap()
                        .unspent_skill_points
                ),
                Color::White,
                Color::Black,
            )?;

            // Draw instructions
            terminal.draw_text_centered(
                center_y - 5,
                "Press 'x' to gain 50 XP, 'c' to open character progression, 'q' to quit",
                Color::Grey,
                Color::Black,
            )?;

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
                    KeyCode::Char('x') => {
                        // Gain experience
                        let mut experiences = world.write_storage::<Experience>();
                        if let Some(exp) = experiences.get_mut(player) {
                            exp.gain_exp(50);
                        }

                        // Run progression systems
                        experience_system.run_now(&world);
                        level_up_system.run_now(&world);
                        world.maintain();
                    }
                    KeyCode::Char('c') => {
                        // Open character progression screen
                        if let Some(action) = show_character_progression(&world, player) {
                            match action {
                                ProgressionAction::AllocateAttribute(attr_type) => {
                                    // Allocate attribute point
                                    let mut attributes = world.write_storage::<Attributes>();
                                    let mut experiences = world.write_storage::<Experience>();
                                    let mut game_log = world.write_resource::<GameLog>();

                                    if let (Some(attr), Some(exp)) =
                                        (attributes.get_mut(player), experiences.get_mut(player))
                                    {
                                        if exp.unspent_points > 0 {
                                            if attr.increase_attribute(attr_type) {
                                                exp.unspent_points -= 1;
                                                game_log.add_entry(format!(
                                                    "Increased {:?}!",
                                                    attr_type
                                                ));
                                            }
                                        }
                                    }
                                }
                                ProgressionAction::AllocateSkill(skill_type) => {
                                    // Allocate skill point
                                    let mut skills = world.write_storage::<Skills>();
                                    let mut game_log = world.write_resource::<GameLog>();

                                    if let Some(skill) = skills.get_mut(player) {
                                        if skill.increase_skill(skill_type) {
                                            game_log.add_entry(format!(
                                                "Increased {} skill!",
                                                skill_type.name()
                                            ));
                                        }
                                    }
                                }
                                ProgressionAction::UseAbility(ability_type) => {
                                    // Use ability
                                    let mut abilities = world.write_storage::<Abilities>();
                                    let mut game_log = world.write_resource::<GameLog>();

                                    if let Some(ability) = abilities.get_mut(player) {
                                        if ability.has_ability(ability_type)
                                            && !ability.is_on_cooldown(ability_type)
                                        {
                                            ability.set_cooldown(
                                                ability_type,
                                                ability_type.cooldown(),
                                            );
                                            game_log.add_entry(format!(
                                                "Used {}!",
                                                ability_type.name()
                                            ));
                                        }
                                    }
                                }
                                ProgressionAction::Exit => {}
                            }
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
