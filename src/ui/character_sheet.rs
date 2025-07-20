use specs::{World, WorldExt, Entity, Join};
use crossterm::style::Color;
use crate::rendering::terminal::with_terminal;
use crate::components::*;

pub fn render_character_sheet(world: &World, player_entity: Entity) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        
        // Draw title
        terminal.draw_text_centered(2, "CHARACTER SHEET", Color::Yellow, Color::Black)?;
        
        // Get player components
        let names = world.read_storage::<Name>();
        let classes = world.read_storage::<CharacterClass>();
        let backgrounds = world.read_storage::<Background>();
        let attributes = world.read_storage::<Attributes>();
        let experiences = world.read_storage::<Experience>();
        let combat_stats = world.read_storage::<CombatStats>();
        let skills = world.read_storage::<Skills>();
        let abilities = world.read_storage::<Abilities>();
        
        // Draw character info
        if let Some(name) = names.get(player_entity) {
            terminal.draw_text(2, 4, &format!("Name: {}", name.name), Color::White, Color::Black)?;
        }
        
        if let Some(class) = classes.get(player_entity) {
            terminal.draw_text(2, 5, &format!("Class: {}", class.class_type.name()), Color::White, Color::Black)?;
        }
        
        if let Some(background) = backgrounds.get(player_entity) {
            terminal.draw_text(2, 6, &format!("Background: {}", background.background_type.name()), Color::White, Color::Black)?;
        }
        
        if let Some(exp) = experiences.get(player_entity) {
            terminal.draw_text(2, 7, &format!("Level: {}", exp.level), Color::White, Color::Black)?;
            terminal.draw_text(2, 8, &format!("Experience: {}/{} ({}%)", 
                exp.current, exp.level_up_target, exp.progress_percentage() as i32), Color::White, Color::Black)?;
            terminal.draw_text(2, 9, &format!("Unspent Points: {}", exp.unspent_points), Color::Green, Color::Black)?;
            
            // Draw experience bar
            let bar_width = 20;
            let filled = ((exp.current as f32 / exp.level_up_target as f32) * bar_width as f32) as usize;
            let bar = format!("[{}{}]", 
                "=".repeat(filled), 
                " ".repeat(bar_width - filled));
            terminal.draw_text(2, 10, &bar, Color::Yellow, Color::Black)?;
        }
        
        // Draw attributes
        if let Some(attr) = attributes.get(player_entity) {
            terminal.draw_text(2, 12, "Attributes:", Color::Yellow, Color::Black)?;
            terminal.draw_text(4, 13, &format!("STR: {} ({})", attr.strength, 
                format_modifier(attr.get_modifier(AttributeType::Strength))), Color::White, Color::Black)?;
            terminal.draw_text(4, 14, &format!("DEX: {} ({})", attr.dexterity, 
                format_modifier(attr.get_modifier(AttributeType::Dexterity))), Color::White, Color::Black)?;
            terminal.draw_text(4, 15, &format!("CON: {} ({})", attr.constitution, 
                format_modifier(attr.get_modifier(AttributeType::Constitution))), Color::White, Color::Black)?;
            terminal.draw_text(4, 16, &format!("INT: {} ({})", attr.intelligence, 
                format_modifier(attr.get_modifier(AttributeType::Intelligence))), Color::White, Color::Black)?;
            terminal.draw_text(4, 17, &format!("WIS: {} ({})", attr.wisdom, 
                format_modifier(attr.get_modifier(AttributeType::Wisdom))), Color::White, Color::Black)?;
            terminal.draw_text(4, 18, &format!("CHA: {} ({})", attr.charisma, 
                format_modifier(attr.get_modifier(AttributeType::Charisma))), Color::White, Color::Black)?;
        }
        
        // Draw combat stats
        if let Some(stats) = combat_stats.get(player_entity) {
            terminal.draw_text(center_x + 5, 12, "Combat Stats:", Color::Yellow, Color::Black)?;
            terminal.draw_text(center_x + 7, 13, &format!("HP: {}/{}", stats.hp, stats.max_hp), Color::White, Color::Black)?;
            terminal.draw_text(center_x + 7, 14, &format!("Attack: {}", stats.power), Color::White, Color::Black)?;
            terminal.draw_text(center_x + 7, 15, &format!("Defense: {}", stats.defense), Color::White, Color::Black)?;
        }
        
        // Draw skills
        if let Some(skill) = skills.get(player_entity) {
            terminal.draw_text(2, 20, "Skills:", Color::Yellow, Color::Black)?;
            terminal.draw_text(4, 21, &format!("Unspent Skill Points: {}", skill.unspent_skill_points), Color::Green, Color::Black)?;
            
            let mut row = 22;
            let mut col = 4;
            
            for skill_type in SkillType::all() {
                let level = skill.get_skill_level(skill_type);
                let stars = "*".repeat(level as usize);
                terminal.draw_text(col, row, &format!("{}: {}", skill_type.name(), stars), Color::White, Color::Black)?;
                
                row += 1;
                if row > 30 {
                    row = 22;
                    col += 25;
                }
            }
        }
        
        // Draw abilities
        if let Some(ability) = abilities.get(player_entity) {
            terminal.draw_text(center_x + 5, 20, "Abilities:", Color::Yellow, Color::Black)?;
            
            let mut row = 22;
            let mut col = center_x + 7;
            
            if let Some(class) = classes.get(player_entity) {
                let class_abilities = AbilityType::get_class_abilities(class.class_type);
                
                for &ability_type in &class_abilities {
                    let has_ability = ability.has_ability(ability_type);
                    let on_cooldown = ability.is_on_cooldown(ability_type);
                    let cooldown = ability.get_cooldown(ability_type);
                    let req_level = ability_type.required_level();
                    
                    let color = if has_ability {
                        if on_cooldown {
                            Color::DarkGrey
                        } else {
                            Color::Green
                        }
                    } else {
                        Color::DarkGrey
                    };
                    
                    let status = if has_ability {
                        if on_cooldown {
                            format!("(Cooldown: {})", cooldown)
                        } else {
                            "".to_string()
                        }
                    } else {
                        format!("(Unlocks at level {})", req_level)
                    };
                    
                    terminal.draw_text(col, row, &format!("{} {}", ability_type.name(), status), color, Color::Black)?;
                    
                    row += 1;
                }
            }
        }
        
        // Draw instructions
        terminal.draw_text_centered(height - 2, "Press Esc to return to game", Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}

fn format_modifier(modifier: i32) -> String {
    if modifier >= 0 {
        format!("+{}", modifier)
    } else {
        format!("{}", modifier)
    }
}

pub fn render_level_up_screen(world: &World, player_entity: Entity) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        
        // Draw title
        terminal.draw_text_centered(2, "LEVEL UP!", Color::Yellow, Color::Black)?;
        
        // Get player components
        let names = world.read_storage::<Name>();
        let classes = world.read_storage::<CharacterClass>();
        let experiences = world.read_storage::<Experience>();
        let attributes = world.read_storage::<Attributes>();
        
        // Draw character info
        if let Some(name) = names.get(player_entity) {
            terminal.draw_text(2, 4, &format!("Name: {}", name.name), Color::White, Color::Black)?;
        }
        
        if let Some(class) = classes.get(player_entity) {
            terminal.draw_text(2, 5, &format!("Class: {}", class.class_type.name()), Color::White, Color::Black)?;
        }
        
        if let Some(exp) = experiences.get(player_entity) {
            terminal.draw_text(2, 6, &format!("New Level: {}", exp.level), Color::White, Color::Black)?;
            terminal.draw_text(2, 7, &format!("Unspent Points: {}", exp.unspent_points), Color::Green, Color::Black)?;
        }
        
        // Draw attributes
        if let Some(attr) = attributes.get(player_entity) {
            terminal.draw_text(2, 9, "Attributes:", Color::Yellow, Color::Black)?;
            terminal.draw_text(4, 10, &format!("STR: {} ({})", attr.strength, 
                format_modifier(attr.get_modifier(AttributeType::Strength))), Color::White, Color::Black)?;
            terminal.draw_text(4, 11, &format!("DEX: {} ({})", attr.dexterity, 
                format_modifier(attr.get_modifier(AttributeType::Dexterity))), Color::White, Color::Black)?;
            terminal.draw_text(4, 12, &format!("CON: {} ({})", attr.constitution, 
                format_modifier(attr.get_modifier(AttributeType::Constitution))), Color::White, Color::Black)?;
            terminal.draw_text(4, 13, &format!("INT: {} ({})", attr.intelligence, 
                format_modifier(attr.get_modifier(AttributeType::Intelligence))), Color::White, Color::Black)?;
            terminal.draw_text(4, 14, &format!("WIS: {} ({})", attr.wisdom, 
                format_modifier(attr.get_modifier(AttributeType::Wisdom))), Color::White, Color::Black)?;
            terminal.draw_text(4, 15, &format!("CHA: {} ({})", attr.charisma, 
                format_modifier(attr.get_modifier(AttributeType::Charisma))), Color::White, Color::Black)?;
        }
        
        // Draw instructions
        terminal.draw_text_centered(18, "Allocate your attribute points:", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(19, "1-6: Increase attribute (STR, DEX, CON, INT, WIS, CHA)", Color::White, Color::Black)?;
        terminal.draw_text_centered(20, "Enter: Continue when done", Color::White, Color::Black)?;
        
        // Draw new abilities
        if let (Some(exp), Some(class)) = (experiences.get(player_entity), classes.get(player_entity)) {
            let class_abilities = AbilityType::get_class_abilities(class.class_type);
            
            let mut new_abilities = Vec::new();
            for &ability_type in &class_abilities {
                if ability_type.required_level() == exp.level {
                    new_abilities.push(ability_type);
                }
            }
            
            if !new_abilities.is_empty() {
                terminal.draw_text_centered(22, "New Abilities Unlocked:", Color::Yellow, Color::Black)?;
                
                for (i, &ability) in new_abilities.iter().enumerate() {
                    terminal.draw_text_centered(24 + i as u16, &format!("{}: {}", ability.name(), ability.description()), 
                        Color::Green, Color::Black)?;
                }
            }
        }
        
        terminal.flush()
    });
}