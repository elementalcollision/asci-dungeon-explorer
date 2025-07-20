use ascii_dungeon_explorer::map::{MapGenerator, RoomBasedDungeonGenerator, DungeonFeatureGenerator, EntityPlacementSystem, TileType};
use ascii_dungeon_explorer::resources::RandomNumberGenerator;
use ascii_dungeon_explorer::rendering::terminal::with_terminal;
use ascii_dungeon_explorer::components::*;
use crossterm::style::Color;
use specs::{World, WorldExt, Join};
use std::{thread, time::Duration};

fn main() {
    // Create a world and register components
    let mut world = World::new();
    register_all_components(&mut world);
    
    // Create a random number generator
    let rng = RandomNumberGenerator::new_with_random_seed();
    
    // Create a dungeon generator
    let mut generator = RoomBasedDungeonGenerator::new(rng.clone());
    
    // Generate a base map
    let mut map = generator.generate_map(80, 50, 3); // Depth 3 for more interesting enemies
    
    // Create a feature generator and add features
    let mut feature_gen = DungeonFeatureGenerator::new(rng.clone());
    feature_gen.add_features(&mut map);
    
    // Create entity placement system and place entities
    let mut entity_placer = EntityPlacementSystem::new(rng, map.depth);
    entity_placer.place_entities(&mut world, &mut map);
    
    // Display the map with entities
    with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get entity positions for rendering
        let positions = world.read_storage::<Position>();
        let renderables = world.read_storage::<Renderable>();
        let names = world.read_storage::<Name>();
        
        // Create a map of positions to entities for rendering
        let mut entity_map = std::collections::HashMap::new();
        for (pos, render, name) in (&positions, &renderables, &names).join() {
            entity_map.insert((pos.x, pos.y), (render.glyph, render.fg, &name.name));
        }
        
        // Draw the map
        for y in 0..map.height {
            for x in 0..map.width {
                let idx = map.xy_idx(x, y);
                
                // Check if there's an entity at this position
                if let Some((glyph, fg_color, _name)) = entity_map.get(&(x, y)) {
                    // Draw the entity
                    terminal.draw_char_at(x as u16, y as u16, *glyph, *fg_color, Color::Black)?;
                } else {
                    // Draw the map tile
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
        }
        
        // Draw a legend
        terminal.draw_text(0, map.height as u16 + 1, "Entity Placement Test", Color::White, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 2, "Map Legend:", Color::White, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 3, "# - Wall  . - Floor  < > - Stairs", Color::White, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 4, "+ - Door  ~ - Water  ≈ - Lava", Color::White, Color::Black)?;
        
        terminal.draw_text(0, map.height as u16 + 6, "Entity Legend:", Color::White, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 7, "r - Rat    g - Goblin   o - Orc", Color::Red, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 8, "s - Spider S - Skeleton T - Troll", Color::Red, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 9, "! - Potion / - Sword   $ - Gold", Color::Cyan, Color::Black)?;
        terminal.draw_text(0, map.height as u16 + 10, "& - Chest  _ - Altar   ^ - Trap", Color::Yellow, Color::Black)?;
        
        // Draw statistics
        let monster_count = world.read_storage::<Monster>().join().count();
        let item_count = world.read_storage::<Item>().join().count();
        
        terminal.draw_text(50, map.height as u16 + 3, 
            &format!("Depth: {}", map.depth), Color::White, Color::Black)?;
        terminal.draw_text(50, map.height as u16 + 4, 
            &format!("Rooms: {}", map.rooms.len()), Color::White, Color::Black)?;
        terminal.draw_text(50, map.height as u16 + 5, 
            &format!("Monsters: {}", monster_count), Color::White, Color::Black)?;
        terminal.draw_text(50, map.height as u16 + 6, 
            &format!("Items: {}", item_count), Color::White, Color::Black)?;
        
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
    }).unwrap();
}