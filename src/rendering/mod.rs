pub mod terminal;
pub mod camera;
pub mod effects;

use crossterm::style::Color;
use crate::map::{Map, TileType};
use crate::components::{Position, Renderable};
pub use terminal::{Terminal, with_terminal};
pub use camera::{Camera, create_camera_for_map};
pub use effects::{VisualEffect, EffectType, EffectManager};

pub struct RenderContext {
    pub width: u16,
    pub height: u16,
    pub camera: Option<Camera>,
    pub effect_manager: EffectManager,
}

impl RenderContext {
    pub fn new() -> Self {
        let (width, height) = with_terminal(|terminal| {
            Ok(terminal.size())
        }).unwrap_or((80, 24));
        
        RenderContext { 
            width, 
            height,
            camera: None,
            effect_manager: EffectManager::new(),
        }
    }
    
    pub fn clear(&self) {
        let _ = with_terminal(|terminal| {
            terminal.clear()?;
            terminal.flush()
        });
    }
    
    pub fn render_map(&self, map: &Map, player_pos: (i32, i32)) {
        let _ = with_terminal(|terminal| {
            // Create camera
            let mut camera = self.camera.clone().unwrap_or_else(|| {
                create_camera_for_map(map, self.width as i32, self.height as i32, player_pos)
            });
            
            // Center camera on player
            camera.center_on(player_pos.0, player_pos.1);
            
            // Render the map
            for screen_y in 0..camera.height {
                for screen_x in 0..camera.width {
                    let world_pos = camera.screen_to_world(screen_x, screen_y);
                    let map_x = world_pos.0;
                    let map_y = world_pos.1;
                    
                    if map_x >= 0 && map_x < map.width && map_y >= 0 && map_y < map.height {
                        let idx = map.xy_idx(map_x, map_y);
                        if map.visible_tiles[idx] {
                            let tile = map.tiles[idx];
                            let glyph = tile.glyph();
                            
                            let fg = match tile {
                                TileType::Floor => Color::Grey,
                                TileType::Wall => Color::White,
                                TileType::DownStairs => Color::Cyan,
                                TileType::UpStairs => Color::Cyan,
                                TileType::Door(_) => Color::Yellow,
                                TileType::Water => Color::Blue,
                                TileType::Lava => Color::Red,
                                TileType::Grass => Color::Green,
                                TileType::Tree => Color::DarkGreen,
                                TileType::Rock => Color::DarkGrey,
                                TileType::Sand => Color::Yellow,
                                TileType::Ice => Color::Cyan,
                                TileType::Void => Color::Black,
                    TileType::Trap(_) => Color::Magenta,
                    TileType::Bridge => Color::DarkYellow,
                            };
                            
                            terminal.draw_char_at(screen_x as u16, screen_y as u16, glyph, fg, Color::Black)?;
                        } else if map.revealed_tiles[idx] {
                            let glyph = match map.tiles[idx] {
                                TileType::Floor => '.',
                                TileType::Wall => '#',
                                TileType::DownStairs => '>',
                                TileType::UpStairs => '<',
                                _ => map.tiles[idx].glyph(),
                            };
                            
                            terminal.draw_char_at(screen_x as u16, screen_y as u16, glyph, Color::DarkGrey, Color::Black)?;
                        }
                    }
                }
            }
            
            terminal.flush()
        });
    }
    
    pub fn render_entities(&self, entities: &[(Position, Renderable)], map: &Map, player_pos: (i32, i32)) {
        let _ = with_terminal(|terminal| {
            // Create camera
            let mut camera = self.camera.clone().unwrap_or_else(|| {
                create_camera_for_map(map, self.width as i32, self.height as i32, player_pos)
            });
            
            // Center camera on player
            camera.center_on(player_pos.0, player_pos.1);
            
            // Render entities
            for (pos, render) in entities.iter() {
                // Convert world position to screen position
                let screen_pos = camera.world_to_screen(pos.x, pos.y);
                
                if camera.is_visible(pos.x, pos.y) {
                    let idx = map.xy_idx(pos.x, pos.y);
                    if map.visible_tiles[idx] {
                        let (r, g, b) = render.fg;
                        let (br, bg, bb) = render.bg;
                        
                        terminal.draw_char_at(
                            screen_pos.0 as u16, 
                            screen_pos.1 as u16, 
                            render.glyph, 
                            Color::Rgb { r, g, b }, 
                            Color::Rgb { r: br, g: bg, b: bb }
                        )?;
                    }
                }
            }
            
            terminal.flush()
        });
    }
    
    pub fn render_ui(&self, player_stats: &str, log_messages: &[String]) {
        let _ = with_terminal(|terminal| {
            // Render player stats at the top
            terminal.draw_text(0, 0, player_stats, Color::White, Color::Black)?;
            
            // Render log messages at the bottom
            let log_start_y = self.height.saturating_sub(log_messages.len() as u16);
            for (i, message) in log_messages.iter().enumerate() {
                terminal.draw_text(0, log_start_y + i as u16, message, Color::White, Color::Black)?;
            }
            
            terminal.flush()
        });
    }
    
    pub fn render_effects(&self, map: &Map, player_pos: (i32, i32)) {
        let _ = with_terminal(|terminal| {
            // Create camera
            let mut camera = self.camera.clone().unwrap_or_else(|| {
                create_camera_for_map(map, self.width as i32, self.height as i32, player_pos)
            });
            
            // Center camera on player
            camera.center_on(player_pos.0, player_pos.1);
            
            // Render each effect
            for effect in &self.effect_manager.effects {
                // Skip completed effects
                if effect.completed {
                    continue;
                }
                
                // Handle different effect types
                match &effect.effect_type {
                    EffectType::Particle { glyph, color, .. } => {
                        // Convert world position to screen position
                        let screen_pos = camera.world_to_screen(effect.position.0, effect.position.1);
                        
                        // Check if the position is visible on screen
                        if camera.is_visible(effect.position.0, effect.position.1) {
                            // Check if the position is visible in the map
                            let idx = map.xy_idx(effect.position.0, effect.position.1);
                            if idx < map.visible_tiles.len() && map.visible_tiles[idx] {
                                terminal.draw_char_at(
                                    screen_pos.0 as u16,
                                    screen_pos.1 as u16,
                                    *glyph,
                                    *color,
                                    Color::Black
                                )?;
                            }
                        }
                    },
                    EffectType::Flash { glyph, colors, interval } => {
                        // Convert world position to screen position
                        let screen_pos = camera.world_to_screen(effect.position.0, effect.position.1);
                        
                        // Check if the position is visible on screen
                        if camera.is_visible(effect.position.0, effect.position.1) {
                            // Check if the position is visible in the map
                            let idx = map.xy_idx(effect.position.0, effect.position.1);
                            if idx < map.visible_tiles.len() && map.visible_tiles[idx] {
                                // Calculate which color to use based on time
                                let interval_secs = interval.as_secs_f32();
                                let elapsed = effect.start_time.elapsed().as_secs_f32();
                                let index = (elapsed / interval_secs) as usize % colors.len();
                                
                                terminal.draw_char_at(
                                    screen_pos.0 as u16,
                                    screen_pos.1 as u16,
                                    *glyph,
                                    colors[index],
                                    Color::Black
                                )?;
                            }
                        }
                    },
                    EffectType::Text { text, color, offset_y, .. } => {
                        // Convert world position to screen position
                        let screen_pos = camera.world_to_screen(effect.position.0, effect.position.1);
                        
                        // Check if the position is visible on screen
                        if camera.is_visible(effect.position.0, effect.position.1) {
                            // Check if the position is visible in the map
                            let idx = map.xy_idx(effect.position.0, effect.position.1);
                            if idx < map.visible_tiles.len() && map.visible_tiles[idx] {
                                // Calculate text position
                                let text_x = screen_pos.0 as u16 - (text.len() as u16 / 2);
                                let text_y = screen_pos.1 as u16 + *offset_y as u16;
                                
                                terminal.draw_text(
                                    text_x,
                                    text_y,
                                    text,
                                    *color,
                                    Color::Black
                                )?;
                            }
                        }
                    },
                    EffectType::Explosion { glyph, color, radius } => {
                        // Calculate the progress of the effect
                        let progress = effect.start_time.elapsed().as_secs_f32() / effect.duration.as_secs_f32();
                        let current_radius = (progress * *radius as f32) as i32;
                        
                        // Render the explosion
                        for y in -current_radius..=current_radius {
                            for x in -current_radius..=current_radius {
                                let distance = ((x * x + y * y) as f32).sqrt();
                                if distance <= current_radius as f32 && distance > (current_radius - 1) as f32 {
                                    let world_pos = (effect.position.0 + x, effect.position.1 + y);
                                    let screen_pos = camera.world_to_screen(world_pos.0, world_pos.1);
                                    
                                    // Check if the position is visible on screen
                                    if camera.is_visible(world_pos.0, world_pos.1) {
                                        // Check if the position is visible in the map
                                        let idx = map.xy_idx(world_pos.0, world_pos.1);
                                        if idx < map.visible_tiles.len() && map.visible_tiles[idx] {
                                            terminal.draw_char_at(
                                                screen_pos.0 as u16,
                                                screen_pos.1 as u16,
                                                *glyph,
                                                *color,
                                                Color::Black
                                            )?;
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
            
            terminal.flush()
        });
    }
    
    pub fn add_effect(&mut self, effect: VisualEffect) {
        self.effect_manager.add_effect(effect);
    }
    
    pub fn update_effects(&mut self) {
        self.effect_manager.update();
    }
    
    pub fn clear_effects(&mut self) {
        self.effect_manager.clear();
    }
}