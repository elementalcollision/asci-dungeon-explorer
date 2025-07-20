use ascii_dungeon_explorer::rendering::{RenderContext, VisualEffect};
use ascii_dungeon_explorer::map::{Map, TileType};
use crossterm::style::Color;
use std::{thread, time::Duration};

fn main() {
    // Create a render context
    let mut context = RenderContext::new();
    
    // Create a simple map
    let mut map = Map::new(80, 50, 1);
    
    // Create a simple room
    for y in 10..40 {
        for x in 10..70 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
            map.blocked[idx] = false;
            map.visible_tiles[idx] = true;
            map.revealed_tiles[idx] = true;
        }
    }
    
    // Player position
    let player_pos = (40, 25);
    
    // Add some effects
    
    // Particle effect
    context.add_effect(VisualEffect::particle(
        (30, 20),
        (50, 30),
        '*',
        Color::Yellow,
        Duration::from_secs(2)
    ));
    
    // Flash effect
    context.add_effect(VisualEffect::flash(
        (40, 20),
        '!',
        vec![Color::Red, Color::Yellow, Color::White],
        Duration::from_secs(3)
    ));
    
    // Text effect
    context.add_effect(VisualEffect::text(
        (40, 25),
        "Critical Hit!".to_string(),
        Color::Red,
        Duration::from_secs(2),
        true
    ));
    
    // Explosion effect
    context.add_effect(VisualEffect::explosion(
        (50, 25),
        5,
        Color::Red,
        '*',
        Duration::from_secs(2)
    ));
    
    // Render loop
    let start_time = std::time::Instant::now();
    while start_time.elapsed() < Duration::from_secs(5) {
        // Clear the screen
        context.clear();
        
        // Render the map
        context.render_map(&map, player_pos);
        
        // Update and render effects
        context.update_effects();
        context.render_effects(&map, player_pos);
        
        // Sleep for a bit
        thread::sleep(Duration::from_millis(50));
    }
}