use specs::{System, ReadStorage, ReadExpect, Join};
use crate::components::{Position, Renderable, Player};
use crate::map::Map;
use crate::resources::GameLog;
use crate::rendering::RenderContext;

pub struct RenderSystem {
    pub context: RenderContext,
}

impl RenderSystem {
    pub fn new() -> Self {
        RenderSystem {
            context: RenderContext::new(),
        }
    }
    
    pub fn handle_resize(&mut self, width: u16, height: u16) {
        self.context.width = width;
        self.context.height = height;
        
        // Update camera if it exists
        if let Some(camera) = &mut self.context.camera {
            camera.resize(width as i32, height as i32);
        }
    }
}

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Renderable>,
        ReadStorage<'a, Player>,
        ReadExpect<'a, Map>,
        ReadExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (positions, renderables, players, map, game_log) = data;

        // Clear the screen
        self.context.clear();

        // Find the player position
        let mut player_pos = (0, 0);
        for (_player, pos) in (&players, &positions).join() {
            player_pos = (pos.x, pos.y);
            break;
        }

        // Render the map
        self.context.render_map(&map, player_pos);

        // Collect entities with position and renderable components
        let mut rendering_data = Vec::new();
        for (pos, render) in (&positions, &renderables).join() {
            rendering_data.push((pos.clone(), render.clone()));
        }

        // Sort by render order
        rendering_data.sort_by(|a, b| a.1.render_order.cmp(&b.1.render_order));

        // Render entities
        self.context.render_entities(&rendering_data, &map, player_pos);
        
        // Update and render effects
        self.context.update_effects();
        self.context.render_effects(&map, player_pos);

        // Get player stats (placeholder for now)
        let player_stats = "HP: 30/30 | Mana: 10/10";

        // Get log messages
        let messages: Vec<String> = game_log.entries.iter().cloned().collect();

        // Render UI
        self.context.render_ui(player_stats, &messages);
    }
}