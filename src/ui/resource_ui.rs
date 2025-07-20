use specs::{World, WorldExt, Entity};
use crossterm::style::Color;
use crate::components::{PlayerResources, CombatStats, StatusEffects, Name};
use crate::rendering::terminal::with_terminal;

pub fn draw_resource_bars(world: &World, player_entity: Entity, x: u16, y: u16) -> Result<(), Box<dyn std::error::Error>> {
    let resources = world.read_storage::<PlayerResources>();
    let combat_stats = world.read_storage::<CombatStats>();
    
    if let (Some(resource), Some(stats)) = (resources.get(player_entity), combat_stats.get(player_entity)) {
        with_terminal(|terminal| {
            // Draw HP bar
            let hp_percentage = (stats.hp as f32 / stats.max_hp as f32) * 100.0;
            let hp_bar_width = 20;
            let hp_filled = ((hp_percentage / 100.0) * hp_bar_width as f32) as u16;
            
            terminal.draw_text(x, y, "HP: ", Color::White, Color::Black)?;
            
            // Draw HP bar background
            for i in 0..hp_bar_width {
                let bar_char = if i < hp_filled { '█' } else { '░' };
                let color = if i < hp_filled {
                    if hp_percentage > 60.0 { Color::Green }
                    else if hp_percentage > 30.0 { Color::Yellow }
                    else { Color::Red }
                } else {
                    Color::DarkGrey
                };
                terminal.draw_text(x + 4 + i, y, &bar_char.to_string(), color, Color::Black)?;
            }
            
            terminal.draw_text(x + 25, y, &format!("{}/{}", stats.hp, stats.max_hp), Color::White, Color::Black)?;
            
            // Draw Mana bar
            let mana_percentage = resource.mana_percentage();
            let mana_filled = ((mana_percentage / 100.0) * hp_bar_width as f32) as u16;
            
            terminal.draw_text(x, y + 1, "MP: ", Color::Blue, Color::Black)?;
            
            for i in 0..hp_bar_width {
                let bar_char = if i < mana_filled { '█' } else { '░' };
                let color = if i < mana_filled { Color::Blue } else { Color::DarkGrey };
                terminal.draw_text(x + 4 + i, y + 1, &bar_char.to_string(), color, Color::Black)?;
            }
            
            terminal.draw_text(x + 25, y + 1, &format!("{}/{}", resource.mana, resource.max_mana), Color::Blue, Color::Black)?;
            
            // Draw Stamina bar
            let stamina_percentage = resource.stamina_percentage();
            let stamina_filled = ((stamina_percentage / 100.0) * hp_bar_width as f32) as u16;
            
            terminal.draw_text(x, y + 2, "SP: ", Color::Green, Color::Black)?;
            
            for i in 0..hp_bar_width {
                let bar_char = if i < stamina_filled { '█' } else { '░' };
                let color = if i < stamina_filled { Color::Green } else { Color::DarkGrey };
                terminal.draw_text(x + 4 + i, y + 2, &bar_char.to_string(), color, Color::Black)?;
            }
            
            terminal.draw_text(x + 25, y + 2, &format!("{}/{}", resource.stamina, resource.max_stamina), Color::Green, Color::Black)?;
            
            Ok(())
        })?;
    }
    
    Ok(())
}

pub fn show_status_effects(world: &World, player_entity: Entity) -> Option<()> {
    let status_effects = world.read_storage::<StatusEffects>();
    let names = world.read_storage::<Name>();
    
    let player_name = names.get(player_entity)?.name.clone();
    let effects = status_effects.get(player_entity)?;
    
    if effects.effects.is_empty() {
        return None;
    }
    
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 15, &format!("{} - Status Effects", player_name), Color::Yellow, Color::Black)?;
        
        // Draw instructions
        terminal.draw_text_centered(
            center_y - 11,
            "Press any key to continue",
            Color::Grey,
            Color::Black
        )?;
        
        // Draw status effects
        for (i, effect) in effects.effects.iter().enumerate() {
            let y_pos = center_y - 8 + i as u16;
            
            let color = if effect.effect_type.is_beneficial() {
                Color::Green
            } else {
                Color::Red
            };
            
            let effect_text = format!("{} (Duration: {} turns, Magnitude: {})", 
                effect.effect_type.name(), effect.duration, effect.magnitude);
            
            terminal.draw_text_centered(y_pos, &effect_text, color, Color::Black)?;
        }
        
        // Flush the output
        terminal.flush()
    });
    
    Some(())
}

pub fn show_resource_management_screen(world: &World, player_entity: Entity) -> Option<ResourceAction> {
    let resources = world.read_storage::<PlayerResources>();
    let combat_stats = world.read_storage::<CombatStats>();
    let status_effects = world.read_storage::<StatusEffects>();
    let names = world.read_storage::<Name>();
    
    let player_name = names.get(player_entity)?.name.clone();
    let player_resources = resources.get(player_entity)?;
    let player_stats = combat_stats.get(player_entity)?;
    let player_effects = status_effects.get(player_entity);
    
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
        
        // Draw title
        terminal.draw_text_centered(center_y - 18, &format!("{} - Resource Management", player_name), Color::Yellow, Color::Black)?;
        
        // Draw resource bars
        draw_resource_bars(world, player_entity, center_x - 15, center_y - 15)?;
        
        // Draw regeneration info
        terminal.draw_text_centered(
            center_y - 10,
            &format!("Mana Regen: {} per 3 turns | Stamina Regen: {} per 2 turns", 
                player_resources.mana_regen_rate, player_resources.stamina_regen_rate),
            Color::Cyan,
            Color::Black
        )?;
        
        // Draw status effects count
        let effect_count = player_effects.map(|e| e.effects.len()).unwrap_or(0);
        terminal.draw_text_centered(
            center_y - 8,
            &format!("Active Status Effects: {}", effect_count),
            Color::White,
            Color::Black
        )?;
        
        // Draw options
        let options = [
            "Rest (Restore 25% of all resources)",
            "Meditate (Restore 50% mana, consume 25% stamina)",
            "Exercise (Restore 50% stamina, consume 25% mana)",
            "View Status Effects",
            "Exit"
        ];
        
        for (i, option) in options.iter().enumerate() {
            let y_pos = center_y - 5 + i as u16;
            let color = if i == selected_option { Color::Yellow } else { Color::White };
            
            // Draw selection indicator
            if i == selected_option {
                terminal.draw_text(center_x - 25, y_pos, "→", Color::Yellow, Color::Black)?;
            }
            
            terminal.draw_text(center_x - 23, y_pos, option, color, Color::Black)?;
        }
        
        // Draw instructions
        terminal.draw_text_centered(
            center_y + 8,
            "Up/Down to navigate, Enter to select, Esc to exit",
            Color::Grey,
            Color::Black
        )?;
        
        // Flush the output
        terminal.flush()
    });
    
    result
}

pub enum ResourceAction {
    Rest,
    Meditate,
    Exercise,
    ViewStatusEffects,
    Exit,
}