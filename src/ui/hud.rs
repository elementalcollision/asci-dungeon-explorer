use crossterm::style::Color;
use specs::{World, Entity, Join, ReadStorage, WorldExt};
use crate::components::{Player, Position, CombatStats, Name, Viewshed};
use crate::items::{Equipment, StatusEffects};
use crate::map::Map;
use crate::resources::GameLog;
use crate::ui::{
    ui_components::{UIComponent, UIRenderCommand, UIPanel, UIText, TextAlignment},
};

/// In-game HUD component that displays player status, minimap, and messages
pub struct GameHUD {
    pub player_entity: Option<Entity>,
    pub show_minimap: bool,
    pub show_detailed_stats: bool,
    pub message_log_size: usize,
    pub hud_height: i32,
    pub minimap_size: i32,
}

impl GameHUD {
    pub fn new() -> Self {
        GameHUD {
            player_entity: None,
            show_minimap: true,
            show_detailed_stats: false,
            message_log_size: 5,
            hud_height: 8,
            minimap_size: 20,
        }
    }

    pub fn with_player(mut self, player: Entity) -> Self {
        self.player_entity = Some(player);
        self
    }

    pub fn toggle_minimap(&mut self) {
        self.show_minimap = !self.show_minimap;
    }

    pub fn toggle_detailed_stats(&mut self) {
        self.show_detailed_stats = !self.show_detailed_stats;
    }

    pub fn render_hud(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Calculate HUD layout
        let hud_y = screen_height - self.hud_height;
        let main_area_width = if self.show_minimap {
            screen_width - self.minimap_size - 2
        } else {
            screen_width
        };

        // Draw HUD background
        let hud_panel = UIPanel::new(
            "".to_string(),
            0,
            hud_y,
            screen_width,
            self.hud_height,
        ).with_colors(Color::DarkGrey, Color::Black, Color::White);
        
        commands.extend(hud_panel.render());

        // Render player status
        commands.extend(self.render_player_status(world, 1, hud_y + 1, main_area_width / 2 - 1));

        // Render message log
        commands.extend(self.render_message_log(world, main_area_width / 2 + 1, hud_y + 1, main_area_width / 2 - 1));

        // Render minimap if enabled
        if self.show_minimap {
            commands.extend(self.render_minimap(world, screen_width - self.minimap_size, hud_y + 1, self.minimap_size - 1));
        }

        // Render status effects
        commands.extend(self.render_status_effects(world, 1, hud_y - 1, screen_width - 2));

        commands
    }

    fn render_player_status(&self, world: &World, x: i32, y: i32, width: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if let Some(player_entity) = self.player_entity {
            let names = world.read_storage::<Name>();
            let combat_stats = world.read_storage::<CombatStats>();
            let positions = world.read_storage::<Position>();

            if let (Some(name), Some(stats), Some(pos)) = (
                names.get(player_entity),
                combat_stats.get(player_entity),
                positions.get(player_entity),
            ) {
                let mut status_lines = Vec::new();

                // Player name and level (placeholder level)
                status_lines.push(format!("{} (Lvl 1)", name.name));

                // Health bar
                let health_percentage = (stats.hp as f32 / stats.max_hp as f32 * 100.0) as i32;
                let health_bar = self.create_bar(stats.hp, stats.max_hp, 20, '█', '░');
                let health_color = if health_percentage > 75 {
                    Color::Green
                } else if health_percentage > 25 {
                    Color::Yellow
                } else {
                    Color::Red
                };
                status_lines.push(format!("HP: {} {}/{}", health_bar, stats.hp, stats.max_hp));

                // Combat stats
                status_lines.push(format!("ATK: {}  DEF: {}", stats.power, stats.defense));

                // Position
                status_lines.push(format!("Pos: ({}, {})", pos.x, pos.y));

                // Equipment summary (if available)
                let equipment = world.read_storage::<Equipment>();
                if let Some(equip) = equipment.get(player_entity) {
                    let equipped_count = equip.get_all_equipped_items().len();
                    status_lines.push(format!("Equipment: {}/12", equipped_count));
                }

                // Render status lines
                for (i, line) in status_lines.iter().enumerate() {
                    let line_y = y + i as i32;
                    if line_y < y + 6 { // Limit to available space
                        let color = if i == 1 { health_color } else { Color::White };
                        commands.push(UIRenderCommand::DrawText {
                            x,
                            y: line_y,
                            text: format!("{:<width$}", line, width = width as usize),
                            fg: color,
                            bg: Color::Black,
                        });
                    }
                }
            }
        } else {
            // No player found
            commands.push(UIRenderCommand::DrawText {
                x,
                y,
                text: "No Player".to_string(),
                fg: Color::Red,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_message_log(&self, world: &World, x: i32, y: i32, width: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let gamelog = world.fetch::<GameLog>();
        let messages = &gamelog.entries;

        // Get the last N messages
        let start_index = if messages.len() > self.message_log_size {
            messages.len() - self.message_log_size
        } else {
            0
        };

        let recent_messages = &messages[start_index..];

        // Render messages (newest at bottom)
        for (i, message) in recent_messages.iter().enumerate() {
            let line_y = y + i as i32;
            if line_y < y + 6 { // Limit to available space
                // Truncate message if too long
                let display_message = if message.len() > width as usize {
                    format!("{}...", &message[..width as usize - 3])
                } else {
                    message.clone()
                };

                // Color messages based on content
                let color = self.get_message_color(message);

                commands.push(UIRenderCommand::DrawText {
                    x,
                    y: line_y,
                    text: format!("{:<width$}", display_message, width = width as usize),
                    fg: color,
                    bg: Color::Black,
                });
            }
        }

        // Fill remaining space
        for i in recent_messages.len()..6 {
            let line_y = y + i as i32;
            commands.push(UIRenderCommand::DrawText {
                x,
                y: line_y,
                text: " ".repeat(width as usize),
                fg: Color::White,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_minimap(&self, world: &World, x: i32, y: i32, width: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Draw minimap border
        commands.push(UIRenderCommand::DrawText {
            x,
            y: y - 1,
            text: "Minimap".to_string(),
            fg: Color::Yellow,
            bg: Color::Black,
        });

        if let Some(player_entity) = self.player_entity {
            let positions = world.read_storage::<Position>();
            let viewsheds = world.read_storage::<Viewshed>();

            if let (Some(player_pos), Some(viewshed)) = (
                positions.get(player_entity),
                viewsheds.get(player_entity),
            ) {
                let map = world.fetch::<Map>();
                let minimap_size = (width - 2).min(6); // Small minimap

                // Calculate minimap bounds around player
                let start_x = (player_pos.x - minimap_size / 2).max(0);
                let start_y = (player_pos.y - minimap_size / 2).max(0);
                let end_x = (start_x + minimap_size).min(map.width);
                let end_y = (start_y + minimap_size).min(map.height);

                // Render minimap tiles
                for map_y in start_y..end_y {
                    for map_x in start_x..end_x {
                        let screen_x = x + 1 + (map_x - start_x);
                        let screen_y = y + (map_y - start_y);

                        let idx = map.xy_idx(map_x, map_y);
                        let tile = map.tiles[idx];

                        let (glyph, color) = if map_x == player_pos.x && map_y == player_pos.y {
                            ('@', Color::Yellow) // Player
                        } else if viewshed.visible_tiles.contains(&(map_x, map_y)) {
                            match tile {
                                crate::map::TileType::Floor => ('.', Color::Grey),
                                crate::map::TileType::Wall => ('#', Color::DarkGrey),
                                crate::map::TileType::DownStairs => ('>', Color::Cyan),
                                crate::map::TileType::UpStairs => ('<', Color::Cyan),
                                _ => ('.', Color::Grey), // Default for other tile types
                            }
                        } else if map.revealed_tiles[idx] {
                            match tile {
                                crate::map::TileType::Floor => ('.', Color::DarkGrey),
                                crate::map::TileType::Wall => ('#', Color::Black),
                                crate::map::TileType::DownStairs => ('>', Color::DarkCyan),
                                crate::map::TileType::UpStairs => ('<', Color::DarkCyan),
                                _ => ('.', Color::DarkGrey), // Default for other tile types
                            }
                        } else {
                            (' ', Color::Black) // Unexplored
                        };

                        commands.push(UIRenderCommand::DrawText {
                            x: screen_x,
                            y: screen_y,
                            text: glyph.to_string(),
                            fg: color,
                            bg: Color::Black,
                        });
                    }
                }
            }
        }

        commands
    }

    fn render_status_effects(&self, world: &World, x: i32, y: i32, width: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if let Some(player_entity) = self.player_entity {
            let status_effects = world.read_storage::<StatusEffects>();

            if let Some(effects) = status_effects.get(player_entity) {
                let mut effect_strings = Vec::new();

                for effect in &effects.effects {
                    let effect_str = format!("{}({})", effect.effect_type.name(), effect.duration);
                    effect_strings.push((effect_str, effect.effect_type.color()));
                }

                if !effect_strings.is_empty() {
                    // Render status effects in a single line
                    let mut current_x = x;
                    let mut effect_text = String::new();

                    for (i, (effect_str, color)) in effect_strings.iter().enumerate() {
                        if i > 0 {
                            effect_text.push_str(" | ");
                        }
                        effect_text.push_str(effect_str);

                        // Check if we have space
                        if effect_text.len() > width as usize {
                            break;
                        }
                    }

                    commands.push(UIRenderCommand::DrawText {
                        x: current_x,
                        y,
                        text: format!("Effects: {}", effect_text),
                        fg: Color::Magenta,
                        bg: Color::Black,
                    });
                }
            }
        }

        commands
    }

    fn create_bar(&self, current: i32, max: i32, width: usize, fill_char: char, empty_char: char) -> String {
        let filled = ((current as f32 / max as f32) * width as f32) as usize;
        let empty = width - filled;
        
        format!("{}{}",
            fill_char.to_string().repeat(filled),
            empty_char.to_string().repeat(empty)
        )
    }

    fn get_message_color(&self, message: &str) -> Color {
        let message_lower = message.to_lowercase();
        
        if message_lower.contains("damage") || message_lower.contains("hit") || message_lower.contains("attack") {
            Color::Red
        } else if message_lower.contains("heal") || message_lower.contains("restore") {
            Color::Green
        } else if message_lower.contains("pick up") || message_lower.contains("found") {
            Color::Yellow
        } else if message_lower.contains("level") || message_lower.contains("experience") {
            Color::Cyan
        } else if message_lower.contains("die") || message_lower.contains("death") {
            Color::DarkRed
        } else {
            Color::White
        }
    }
}

impl UIComponent for GameHUD {
    fn render(&self, _x: i32, _y: i32, width: i32, height: i32) -> Vec<UIRenderCommand> {
        // This would need access to the World, so we use render_hud instead
        vec![]
    }

    fn handle_input(&mut self, input: char) -> bool {
        match input {
            'm' | 'M' => {
                self.toggle_minimap();
                true
            }
            'i' | 'I' => {
                self.toggle_detailed_stats();
                true
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool {
        false // HUD is not focusable
    }

    fn set_focus(&mut self, _focused: bool) {
        // HUD cannot be focused
    }
}

/// HUD manager for handling HUD updates and rendering
pub struct HUDManager {
    pub hud: GameHUD,
    pub last_update_frame: u64,
    pub update_frequency: u64,
}

impl HUDManager {
    pub fn new(player_entity: Entity) -> Self {
        HUDManager {
            hud: GameHUD::new().with_player(player_entity),
            last_update_frame: 0,
            update_frequency: 1, // Update every frame
        }
    }

    pub fn update(&mut self, frame: u64) {
        if frame - self.last_update_frame >= self.update_frequency {
            // Update HUD state if needed
            self.last_update_frame = frame;
        }
    }

    pub fn render(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        self.hud.render_hud(world, screen_width, screen_height)
    }

    pub fn handle_input(&mut self, input: char) -> bool {
        self.hud.handle_input(input)
    }

    pub fn set_player(&mut self, player_entity: Entity) {
        self.hud.player_entity = Some(player_entity);
    }
}

// Extension trait for status effect types to provide colors
trait StatusEffectDisplay {
    fn name(&self) -> &'static str;
    fn color(&self) -> Color;
}

impl StatusEffectDisplay for crate::items::StatusEffectType {
    fn name(&self) -> &'static str {
        match self {
            crate::items::StatusEffectType::Poison => "Poison",
            crate::items::StatusEffectType::Regeneration => "Regen",
            crate::items::StatusEffectType::Strength => "Str+",
            crate::items::StatusEffectType::Weakness => "Weak",
            crate::items::StatusEffectType::Speed => "Fast",
            crate::items::StatusEffectType::Slow => "Slow",
            crate::items::StatusEffectType::Protection => "Prot",
            crate::items::StatusEffectType::Vulnerability => "Vuln",
        }
    }

    fn color(&self) -> Color {
        match self {
            crate::items::StatusEffectType::Poison => Color::Green,
            crate::items::StatusEffectType::Regeneration => Color::DarkGreen,
            crate::items::StatusEffectType::Strength => Color::Red,
            crate::items::StatusEffectType::Weakness => Color::DarkRed,
            crate::items::StatusEffectType::Speed => Color::Yellow,
            crate::items::StatusEffectType::Slow => Color::DarkYellow,
            crate::items::StatusEffectType::Protection => Color::Blue,
            crate::items::StatusEffectType::Vulnerability => Color::Magenta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::components::{Player, Position, CombatStats, Name};

    fn setup_test_world() -> (World, Entity) {
        let mut world = World::new();
        world.register::<Player>();
        world.register::<Position>();
        world.register::<CombatStats>();
        world.register::<Name>();
        world.register::<Equipment>();
        world.register::<StatusEffects>();
        world.register::<Viewshed>();
        world.insert(GameLog::new());
        world.insert(Map::new(80, 50, 1));

        let player = world.create_entity()
            .with(Player)
            .with(Position { x: 10, y: 10 })
            .with(CombatStats { max_hp: 100, hp: 75, defense: 10, power: 15 })
            .with(Name { name: "Hero".to_string() })
            .build();

        (world, player)
    }

    #[test]
    fn test_hud_creation() {
        let hud = GameHUD::new();
        
        assert!(hud.player_entity.is_none());
        assert!(hud.show_minimap);
        assert!(!hud.show_detailed_stats);
        assert_eq!(hud.message_log_size, 5);
        assert_eq!(hud.hud_height, 8);
    }

    #[test]
    fn test_hud_with_player() {
        let (world, player) = setup_test_world();
        let hud = GameHUD::new().with_player(player);
        
        assert_eq!(hud.player_entity, Some(player));
    }

    #[test]
    fn test_hud_toggles() {
        let mut hud = GameHUD::new();
        
        assert!(hud.show_minimap);
        hud.toggle_minimap();
        assert!(!hud.show_minimap);
        
        assert!(!hud.show_detailed_stats);
        hud.toggle_detailed_stats();
        assert!(hud.show_detailed_stats);
    }

    #[test]
    fn test_hud_input_handling() {
        let mut hud = GameHUD::new();
        
        assert!(hud.handle_input('m'));
        assert!(!hud.show_minimap);
        
        assert!(hud.handle_input('i'));
        assert!(hud.show_detailed_stats);
        
        assert!(!hud.handle_input('x')); // Unknown input
    }

    #[test]
    fn test_create_bar() {
        let hud = GameHUD::new();
        
        let bar = hud.create_bar(50, 100, 10, '█', '░');
        assert_eq!(bar.len(), 10);
        assert!(bar.contains('█'));
        assert!(bar.contains('░'));
        
        let full_bar = hud.create_bar(100, 100, 10, '█', '░');
        assert_eq!(full_bar, "██████████");
        
        let empty_bar = hud.create_bar(0, 100, 10, '█', '░');
        assert_eq!(empty_bar, "░░░░░░░░░░");
    }

    #[test]
    fn test_message_color() {
        let hud = GameHUD::new();
        
        assert_eq!(hud.get_message_color("You take damage!"), Color::Red);
        assert_eq!(hud.get_message_color("You heal yourself"), Color::Green);
        assert_eq!(hud.get_message_color("You pick up a sword"), Color::Yellow);
        assert_eq!(hud.get_message_color("You gain a level!"), Color::Cyan);
        assert_eq!(hud.get_message_color("The orc dies"), Color::DarkRed);
        assert_eq!(hud.get_message_color("You walk north"), Color::White);
    }

    #[test]
    fn test_hud_manager() {
        let (world, player) = setup_test_world();
        let mut hud_manager = HUDManager::new(player);
        
        assert_eq!(hud_manager.hud.player_entity, Some(player));
        
        hud_manager.update(1);
        assert_eq!(hud_manager.last_update_frame, 1);
        
        assert!(hud_manager.handle_input('m'));
        assert!(!hud_manager.hud.show_minimap);
    }

    #[test]
    fn test_hud_rendering() {
        let (world, player) = setup_test_world();
        let hud = GameHUD::new().with_player(player);
        
        let commands = hud.render_hud(&world, 80, 24);
        assert!(!commands.is_empty());
        
        // Check that we have various types of render commands
        let has_text = commands.iter().any(|cmd| matches!(cmd, UIRenderCommand::DrawText { .. }));
        assert!(has_text);
    }
}