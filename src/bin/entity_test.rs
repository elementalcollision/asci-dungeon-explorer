use ascii_dungeon_explorer::map::entity_placement::{EntitySpawn, SpawnType, SpecialFeatureType};
use ascii_dungeon_explorer::map::{
    DungeonFeatureGenerator, EnemyType, EntityPlacementSystem, ItemType, MapGenerator,
    RoomBasedDungeonGenerator, TileType,
};
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::resources::RandomNumberGenerator;
use crossterm::style::Color;
use std::{thread, time::Duration};

fn main() {
    // Create a random number generator
    let rng = RandomNumberGenerator::new_with_random_seed();

    // Create a dungeon generator
    let mut generator = RoomBasedDungeonGenerator::new(rng.clone());

    // Generate a base map
    let mut map = generator.generate_map(80, 50, 1);

    // Create a feature generator and add features
    let mut feature_gen = DungeonFeatureGenerator::new(rng.clone());
    feature_gen.add_features(&mut map);

    // Create an entity placement system and populate the map
    let mut entity_placer = EntityPlacementSystem::new(rng);
    let spawns = entity_placer.populate_map(&map, 3); // Difficulty level 3

    // Display the map with entities
    with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;

        // Draw the map
        for y in 0..map.height {
            for x in 0..map.width {
                let idx = map.xy_idx(x, y);
                let glyph = map.get_tile_glyph(idx);

                // Choose color based on tile type
                let color = match map.tiles[idx] {
                    TileType::Floor => Color::Grey,
                    TileType::Wall => Color::White,
                    TileType::DownStairs => Color::Cyan,
                    TileType::UpStairs => Color::Blue,
                    TileType::Door(_) => Color::Yellow,
                    TileType::Water => Color::Blue,
                    TileType::Lava => Color::Red,
                    TileType::Trap(true) => Color::Magenta,
                    TileType::Trap(false) => Color::Grey, // Hidden traps look like floor
                    TileType::Bridge => Color::DarkYellow,
                    TileType::Grass => Color::Green,
                    TileType::Tree => Color::DarkGreen,
                    TileType::Rock => Color::DarkGrey,
                    TileType::Sand => Color::Yellow,
                    TileType::Ice => Color::Cyan,
                    TileType::Void => Color::Black,
                };

                terminal.draw_char_at(x as u16, y as u16, glyph, color, Color::Black)?;
            }
        }

        // Draw entities
        for spawn in &spawns {
            let (glyph, color) = entity_glyph_and_color(&spawn.entity_type);
            terminal.draw_char_at(spawn.x as u16, spawn.y as u16, glyph, color, Color::Black)?;
        }

        // Draw a legend
        terminal.draw_text(
            0,
            map.height as u16 + 1,
            "Entity Placement Test",
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            0,
            map.height as u16 + 2,
            "Map Legend:",
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            0,
            map.height as u16 + 3,
            "# - Wall",
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            0,
            map.height as u16 + 4,
            ". - Floor",
            Color::Grey,
            Color::Black,
        )?;
        terminal.draw_text(
            0,
            map.height as u16 + 5,
            "< - Stairs Up",
            Color::Blue,
            Color::Black,
        )?;
        terminal.draw_text(
            0,
            map.height as u16 + 6,
            "> - Stairs Down",
            Color::Cyan,
            Color::Black,
        )?;

        // Entity legend
        terminal.draw_text(
            25,
            map.height as u16 + 2,
            "Entity Legend:",
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 3,
            "g - Goblin",
            Color::Green,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 4,
            "o - Orc",
            Color::Green,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 5,
            "s - Skeleton",
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 6,
            "! - Potion",
            Color::Magenta,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 7,
            "$ - Gold",
            Color::Yellow,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 8,
            "C - Chest",
            Color::Yellow,
            Color::Black,
        )?;

        // Stats
        terminal.draw_text(
            50,
            map.height as u16 + 3,
            &format!("Total Entities: {}", spawns.len()),
            Color::White,
            Color::Black,
        )?;

        let enemy_count = spawns
            .iter()
            .filter(|s| matches!(s.entity_type, SpawnType::Enemy(_)))
            .count();
        let item_count = spawns
            .iter()
            .filter(|s| matches!(s.entity_type, SpawnType::Item(_)))
            .count();
        let special_count = spawns
            .iter()
            .filter(|s| matches!(s.entity_type, SpawnType::Special(_)))
            .count();

        terminal.draw_text(
            50,
            map.height as u16 + 4,
            &format!("Enemies: {}", enemy_count),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            50,
            map.height as u16 + 5,
            &format!("Items: {}", item_count),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            50,
            map.height as u16 + 6,
            &format!("Special: {}", special_count),
            Color::White,
            Color::Black,
        )?;

        // Flush the output
        terminal.flush()?;

        // Wait for a key press
        loop {
            if let Some(_) = terminal.poll_key(100)? {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }

        Ok(())
    })
    .unwrap();
}

fn entity_glyph_and_color(entity_type: &SpawnType) -> (char, Color) {
    match entity_type {
        SpawnType::Enemy(enemy_type) => match enemy_type {
            EnemyType::Goblin => ('g', Color::Green),
            EnemyType::Orc => ('o', Color::Green),
            EnemyType::Troll => ('T', Color::Green),
            EnemyType::Skeleton => ('s', Color::White),
            EnemyType::Zombie => ('z', Color::Green),
            EnemyType::Ghost => ('G', Color::White),
            EnemyType::Demon => ('d', Color::Red),
            EnemyType::Dragon => ('D', Color::Red),
            EnemyType::Spider => ('S', Color::DarkMagenta),
            EnemyType::Bat => ('b', Color::DarkGrey),
            EnemyType::Rat => ('r', Color::Brown),
            EnemyType::Snake => ('S', Color::Green),
            EnemyType::Slime => ('j', Color::Green),
        },
        SpawnType::Item(item_type) => match item_type {
            ItemType::HealthPotion => ('!', Color::Red),
            ItemType::ManaPotion => ('!', Color::Blue),
            ItemType::Scroll => ('?', Color::Yellow),
            ItemType::Weapon => (')', Color::White),
            ItemType::Armor => ('[', Color::White),
            ItemType::Shield => (')', Color::White),
            ItemType::Ring => ('=', Color::Yellow),
            ItemType::Amulet => ('"', Color::Yellow),
            ItemType::Gold => ('$', Color::Yellow),
            ItemType::Key => ('k', Color::Yellow),
            ItemType::Gem => ('*', Color::Magenta),
        },
        SpawnType::Special(special_type) => match special_type {
            SpecialFeatureType::Chest => ('C', Color::Yellow),
            SpecialFeatureType::Shrine => ('_', Color::Cyan),
            SpecialFeatureType::Altar => ('_', Color::White),
            SpecialFeatureType::Statue => ('&', Color::White),
        },
    }
}
