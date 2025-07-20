use ascii_dungeon_explorer::map::{CellularAutomataCaveGenerator, Map, MapGenerator, TileType};
use ascii_dungeon_explorer::rendering::terminal::{with_terminal, Terminal};
use ascii_dungeon_explorer::resources::RandomNumberGenerator;
use crossterm::style::Color;
use std::{thread, time::Duration};

fn main() {
    // Create a random number generator
    let rng = RandomNumberGenerator::new_with_random_seed();

    // Create a cave generator
    let mut generator = CellularAutomataCaveGenerator::new(rng);

    // Generate a map
    let map = generator.generate_map(80, 50, 1);

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
                    TileType::Trap(_) => Color::Magenta,
                    TileType::Bridge => Color::DarkYellow,
                    _ => Color::White,
                };

                terminal.draw_char_at(x as u16, y as u16, glyph, color, Color::Black)?;
            }
        }

        // Draw a legend
        terminal.draw_text(
            0,
            map.height as u16 + 1,
            "Cave Generator Test",
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

        // Draw generation info
        terminal.draw_text(
            30,
            map.height as u16 + 3,
            &format!("Theme: {:?}", map.theme),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            30,
            map.height as u16 + 4,
            &format!("Depth: {}", map.depth),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            30,
            map.height as u16 + 5,
            &format!("Entrance: {:?}", map.entrance),
            Color::White,
            Color::Black,
        )?;
        terminal.draw_text(
            30,
            map.height as u16 + 6,
            &format!("Exit: {:?}", map.exit),
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
