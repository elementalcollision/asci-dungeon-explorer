use ascii_dungeon_explorer::components::*;
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::resources::{GameLog, RandomNumberGenerator};
use ascii_dungeon_explorer::systems::{EquipmentBonusSystem, EquipmentSystem};
use ascii_dungeon_explorer::ui::{show_equipment_screen, EquipmentAction};
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
        .with(Inventory::new(10))
        .build();

    // Create some equipment items
    let sword = world
        .create_entity()
        .with(Item {})
        .with(Name {
            name: "Steel Sword".to_string(),
        })
        .with(Renderable {
            glyph: '/',
            fg: Color::Cyan,
            bg: Color::Black,
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 4 })
        .build();

    let shield = world
        .create_entity()
        .with(Item {})
        .with(Name {
            name: "Iron Shield".to_string(),
        })
        .with(Renderable {
            glyph: '(',
            fg: Color::Cyan,
            bg: Color::Black,
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .build();

    let armor = world
        .create_entity()
        .with(Item {})
        .with(Name {
            name: "Chain Mail".to_string(),
        })
        .with(Renderable {
            glyph: '[',
            fg: Color::Cyan,
            bg: Color::Black,
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Armor,
        })
        .with(DefenseBonus { defense: 2 })
        .build();

    let helmet = world
        .create_entity()
        .with(Item {})
        .with(Name {
            name: "Iron Helmet".to_string(),
        })
        .with(Renderable {
            glyph: '^',
            fg: Color::Cyan,
            bg: Color::Black,
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Helmet,
        })
        .with(DefenseBonus { defense: 1 })
        .build();

    // Add items to player's inventory
    {
        let mut inventories = world.write_storage::<Inventory>();
        let player_inventory = inventories.get_mut(player).unwrap();
        player_inventory.items.push(sword);
        player_inventory.items.push(shield);
        player_inventory.items.push(armor);
        player_inventory.items.push(helmet);
    }

    // Create systems
    let mut equipment_system = EquipmentSystem {};
    let mut equipment_bonus_system = EquipmentBonusSystem {};

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
                "EQUIPMENT TEST",
                Color::Yellow,
                Color::Black,
            )?;

            // Draw player stats
            let player_stats = world.read_storage::<CombatStats>().get(player).unwrap();
            terminal.draw_text_centered(
                center_y - 10,
                &format!(
                    "Player Stats: HP {}/{}, Power {}, Defense {}",
                    player_stats.hp, player_stats.max_hp, player_stats.power, player_stats.defense
                ),
                Color::White,
                Color::Black,
            )?;

            // Show equipped items count
            let equipped_count = world
                .read_storage::<Equipped>()
                .join()
                .filter(|equipped| equipped.owner == player)
                .count();
            terminal.draw_text_centered(
                center_y - 8,
                &format!("Equipped Items: {}", equipped_count),
                Color::Green,
                Color::Black,
            )?;

            // Draw instructions
            terminal.draw_text_centered(
                center_y - 5,
                "Press 'e' to open equipment screen, 'q' to quit",
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
                    KeyCode::Char('e') => {
                        // Open equipment screen
                        if let Some(action) = show_equipment_screen(&world, player) {
                            match action {
                                EquipmentAction::UnequipItem(slot) => {
                                    // Find and unequip item in this slot
                                    let mut equipped = world.write_storage::<Equipped>();
                                    let inventories = world.read_storage::<Inventory>();
                                    let names = world.read_storage::<Name>();

                                    if let Some(inventory) = inventories.get(player) {
                                        for &item_entity in &inventory.items {
                                            if let Some(item_equipped) = equipped.get(item_entity) {
                                                if item_equipped.owner == player
                                                    && item_equipped.slot == slot
                                                {
                                                    equipped.remove(item_entity);

                                                    if let Some(name) = names.get(item_entity) {
                                                        let mut game_log =
                                                            world.write_resource::<GameLog>();
                                                        game_log.add_entry(format!(
                                                            "You unequip the {}.",
                                                            name.name
                                                        ));
                                                    }
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                EquipmentAction::ShowEquippableItems(_slot) => {
                                    // This would show items that can be equipped in this slot
                                }
                                EquipmentAction::Exit => {}
                            }
                        }

                        // Run equipment systems to update bonuses
                        equipment_system.run_now(&world);
                        equipment_bonus_system.run_now(&world);
                        world.maintain();
                    }
                    KeyCode::Char('1') => {
                        // Quick equip sword
                        let mut wants_use = world.write_storage::<WantsToUseItem>();
                        wants_use
                            .insert(
                                player,
                                WantsToUseItem {
                                    item: sword,
                                    target: None,
                                },
                            )
                            .expect("Failed to insert use item intent");

                        equipment_system.run_now(&world);
                        equipment_bonus_system.run_now(&world);
                        world.maintain();
                    }
                    KeyCode::Char('2') => {
                        // Quick equip shield
                        let mut wants_use = world.write_storage::<WantsToUseItem>();
                        wants_use
                            .insert(
                                player,
                                WantsToUseItem {
                                    item: shield,
                                    target: None,
                                },
                            )
                            .expect("Failed to insert use item intent");

                        equipment_system.run_now(&world);
                        equipment_bonus_system.run_now(&world);
                        world.maintain();
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
