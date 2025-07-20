use ascii_dungeon_explorer::map::{
    DungeonFeatureGenerator, MapGenerator, RoomBasedDungeonGenerator, TileType,
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
    let mut feature_gen = DungeonFeatureGenerator::new(rng);
    feature_gen.add_features(&mut map);

    // Display the map
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

        // Draw a legend
        terminal.draw_text(
            0,
            map.height as u16 + 1,
            "Dungeon with Features Test",
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            0,
            map.height as u16 + 2,
            "Legend:",
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
        terminal.draw_text(
            0,
            map.height as u16 + 7,
            "+ - Closed Door",
            Color::Yellow,
            Color::Black,
        )?;

        terminal.draw_text(
            25,
            map.height as u16 + 3,
            "~ - Water",
            Color::Blue,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 4,
            "≈ - Lava",
            Color::Red,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 5,
            "^ - Trap",
            Color::Magenta,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 6,
            "= - Bridge",
            Color::DarkYellow,
            Color::Black,
        )?;
        terminal.draw_text(
            25,
            map.height as u16 + 7,
            "○ - Rock",
            Color::DarkGrey,
            Color::Black,
        )?;

        terminal.draw_text(
            45,
            map.height as u16 + 3,
            "\" - Grass",
            Color::Green,
            Color::Black,
        )?;
        terminal.draw_text(
            45,
            map.height as u16 + 4,
            "♠ - Tree",
            Color::DarkGreen,
            Color::Black,
        )?;
        terminal.draw_text(
            45,
            map.height as u16 + 5,
            "· - Sand",
            Color::Yellow,
            Color::Black,
        )?;
        terminal.draw_text(
            45,
            map.height as u16 + 6,
            "* - Ice",
            Color::Cyan,
            Color::Black,
        )?;
        terminal.draw_text(
            45,
            map.height as u16 + 7,
            "  - Void",
            Color::Black,
            Color::Black,
        )?;

        // Draw generation info
        terminal.draw_text(
            60,
            map.height as u16 + 3,
            &format!("Rooms: {}", map.rooms.len()),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            60,
            map.height as u16 + 4,
            &format!("Corridors: {}", map.corridors.len()),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            60,
            map.height as u16 + 5,
            &format!("Theme: {:?}", map.theme),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            60,
            map.height as u16 + 6,
            &format!("Depth: {}", map.depth),
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
