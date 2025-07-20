use specs::{World, WorldExt, Entity, Join};
use crossterm::style::Color;
use crate::components::{DeathState, GameSettings, Name, RevivalItem, Inventory, GameMode};
use crate::rendering::terminal::with_terminal;

pub fn show_death_screen(world: &World, player_entity: Entity) -> Option<DeathAction> {
    let death_states = world.read_storage::<DeathState>();
    let game_settings = world.read_storage::<GameSettings>();
    let names = world.read_storage::<Name>();
    let revival_items = world.read_storage::<RevivalItem>();
    let inventories = world.read_storage::<Inventory>();
    
    let death_state = death_states.get(player_entity)?;
    let player_name = names.get(player_entity)?.name.clone();
    let settings = game_settings.get(player_entity);
    let inventory = inventories.get(player_entity);
    
    if !death_state.is_dead {
        return None;
    }
    
    // Find available revival items
    let mut revival_options = Vec::new();
    if let Some(inv) = inventory {
        for &item_entity in &inv.items {
            if let Some(revival_item) = revival_items.get(item_entity) {
                if !revival_item.auto_use {
                    let item_name = names.get(item_entity).map_or("Unknown Item".to_string(), |n| n.name.clone());
                    revival_options.push((item_entity, item_name, revival_item.revival_power));
                }
            }
        }
    }
    
    let mut selected_option = 0;
    let mut result = None;
    
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw death screen border
        for x in center_x - 30..center_x + 30 {
            terminal.draw_text(x, center_y - 15, "═", Color::Red, Color::Black)?;
            terminal.draw_text(x, center_y + 15, "═", Color::Red, Color::Black)?;
        }
        for y in center_y - 15..center_y + 15 {
            terminal.draw_text(center_x - 30, y, "║", Color::Red, Color::Black)?;
            terminal.draw_text(center_x + 30, y, "║", Color::Red, Color::Black)?;
        }
        
        // Draw corners
        terminal.draw_text(center_x - 30, center_y - 15, "╔", Color::Red, Color::Black)?;
        terminal.draw_text(center_x + 30, center_y - 15, "╗", Color::Red, Color::Black)?;
        terminal.draw_text(center_x - 30, center_y + 15, "╚", Color::Red, Color::Black)?;
        terminal.draw_text(center_x + 30, center_y + 15, "╝", Color::Red, Color::Black)?;
        
        // Draw death message
        terminal.draw_text_centered(center_y - 12, "YOU HAVE DIED", Color::Red, Color::Black)?;
        terminal.draw_text_centered(center_y - 10, &format!("{} has fallen!", player_name), Color::White, Color::Black)?;
        terminal.draw_text_centered(center_y - 8, &format!("Cause: {}", death_state.death_cause), Color::Grey, Color::Black)?;
        
        // Draw game mode info
        if let Some(settings) = settings {
            terminal.draw_text_centered(center_y - 6, &format!("Game Mode: {}", settings.game_mode.name()), Color::Yellow, Color::Black)?;
            
            match settings.game_mode {
                GameMode::Permadeath => {
                    terminal.draw_text_centered(center_y - 4, "PERMADEATH - GAME OVER", Color::Red, Color::Black)?;
                    terminal.draw_text_centered(center_y + 10, "Press any key to return to main menu", Color::Grey, Color::Black)?;
                    return Ok(());
                },
                _ => {
                    terminal.draw_text_centered(center_y - 4, 
                        &format!("Revival attempts: {}/{}", death_state.revival_attempts, death_state.max_revival_attempts), 
                        Color::Cyan, Color::Black)?;
                }
            }
        }
        
        // Draw revival options
        if death_state.can_revive() {
            terminal.draw_text_centered(center_y - 2, "Revival Options:", Color::Green, Color::Black)?;
            
            let mut option_index = 0;
            
            // Option to revive with penalty
            let color = if selected_option == option_index { Color::Yellow } else { Color::White };
            if selected_option == option_index {
                terminal.draw_text(center_x - 25, center_y + option_index as u16, "→", Color::Yellow, Color::Black)?;
            }
            terminal.draw_text(center_x - 23, center_y + option_index as u16, "Revive with penalty (25% HP)", color, Color::Black)?;
            option_index += 1;
            
            // Revival item options
            for (i, (_, item_name, revival_power)) in revival_options.iter().enumerate() {
                let color = if selected_option == option_index { Color::Yellow } else { Color::Green };
                if selected_option == option_index {
                    terminal.draw_text(center_x - 25, center_y + option_index as u16, "→", Color::Yellow, Color::Black)?;
                }
                terminal.draw_text(center_x - 23, center_y + option_index as u16, 
                    &format!("Use {} (Restore {} HP)", item_name, revival_power), color, Color::Black)?;
                option_index += 1;
            }
            
            // Option to give up
            let color = if selected_option == option_index { Color::Yellow } else { Color::Red };
            if selected_option == option_index {
                terminal.draw_text(center_x - 25, center_y + option_index as u16, "→", Color::Yellow, Color::Black)?;
            }
            terminal.draw_text(center_x - 23, center_y + option_index as u16, "Give up (Game Over)", color, Color::Black)?;
            
        } else {
            terminal.draw_text_centered(center_y + 2, "No revival attempts remaining", Color::Red, Color::Black)?;
            terminal.draw_text_centered(center_y + 4, "GAME OVER", Color::Red, Color::Black)?;
        }
        
        // Draw instructions
        if death_state.can_revive() {
            terminal.draw_text_centered(center_y + 12, "Up/Down to select, Enter to confirm", Color::Grey, Color::Black)?;
        } else {
            terminal.draw_text_centered(center_y + 12, "Press any key to continue", Color::Grey, Color::Black)?;
        }
        
        // Flush the output
        terminal.flush()
    });
    
    result
}

pub fn show_revival_confirmation(world: &World, player_entity: Entity, action: &DeathAction) -> bool {
    let names = world.read_storage::<Name>();
    let player_name = names.get(player_entity).map_or("Player".to_string(), |n| n.name.clone());
    
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw confirmation message
        terminal.draw_text_centered(center_y - 5, "REVIVAL CONFIRMATION", Color::Yellow, Color::Black)?;
        
        match action {
            DeathAction::ReviveWithPenalty => {
                terminal.draw_text_centered(center_y - 2, &format!("{} will be revived with:", player_name), Color::White, Color::Black)?;
                terminal.draw_text_centered(center_y, "• 25% HP restored", Color::Green, Color::Black)?;
                terminal.draw_text_centered(center_y + 1, "• Experience penalty applied", Color::Red, Color::Black)?;
                terminal.draw_text_centered(center_y + 2, "• Temporary stat reduction", Color::Red, Color::Black)?;
            },
            DeathAction::UseRevivalItem(item_entity) => {
                let revival_items = world.read_storage::<crate::components::RevivalItem>();
                if let Some(revival_item) = revival_items.get(*item_entity) {
                    let item_name = names.get(*item_entity).map_or("Revival Item".to_string(), |n| n.name.clone());
                    terminal.draw_text_centered(center_y - 2, &format!("Use {} to revive {}:", item_name, player_name), Color::White, Color::Black)?;
                    terminal.draw_text_centered(center_y, &format!("• {} HP will be restored", revival_item.revival_power), Color::Green, Color::Black)?;
                    if revival_item.consumed_on_use {
                        terminal.draw_text_centered(center_y + 1, "• Item will be consumed", Color::Yellow, Color::Black)?;
                    }
                }
            },
            DeathAction::GiveUp => {
                terminal.draw_text_centered(center_y - 2, "Are you sure you want to give up?", Color::Red, Color::Black)?;
                terminal.draw_text_centered(center_y, "This will end your game!", Color::Red, Color::Black)?;
            },
            DeathAction::GameOver => {
                return Ok(());
            }
        }
        
        terminal.draw_text_centered(center_y + 5, "Press Y to confirm, N to cancel", Color::Grey, Color::Black)?;
        
        // Flush the output
        terminal.flush()
    });
    
    false // This would be determined by user input in a real implementation
}

pub enum DeathAction {
    ReviveWithPenalty,
    UseRevivalItem(Entity),
    GiveUp,
    GameOver,
}