use specs::{World, WorldExt, Entity, Join};
use crossterm::style::Color;
use crate::components::{Experience, Attributes, Skills, Abilities, AttributeType, SkillType, AbilityType, Name};
use crate::rendering::terminal::with_terminal;

pub fn show_character_progression(world: &World, player_entity: Entity) -> Option<ProgressionAction> {
    // Get player's progression data
    let experiences = world.read_storage::<Experience>();
    let attributes = world.read_storage::<Attributes>();
    let skills = world.read_storage::<Skills>();
    let abilities = world.read_storage::<Abilities>();
    let names = world.read_storage::<Name>();
    
    let player_exp = experiences.get(player_entity)?;
    let player_attrs = attributes.get(player_entity)?;
    let player_skills = skills.get(player_entity)?;
    let player_abilities = abilities.get(player_entity)?;
    let player_name = names.get(player_entity)?;
    
    let mut selected_tab = 0; // 0 = Attributes, 1 = Skills, 2 = Abilities
    let mut selected_item = 0;
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
        terminal.draw_text_centered(center_y - 18, &format!("{} - Character Progression", player_name.name), Color::Yellow, Color::Black)?;
        
        // Draw level and experience info
        terminal.draw_text_centered(
            center_y - 16,
            &format!("Level: {} | XP: {}/{} | Unspent Points: {}", 
                player_exp.level, player_exp.current, player_exp.level_up_target, player_exp.unspent_points),
            Color::White,
            Color::Black
        )?;
        
        // Draw tabs
        let tabs = ["Attributes", "Skills", "Abilities"];
        for (i, tab) in tabs.iter().enumerate() {
            let color = if i == selected_tab { Color::Yellow } else { Color::White };
            let x_pos = center_x - 20 + (i as u16 * 15);
            terminal.draw_text(x_pos, center_y - 14, tab, color, Color::Black)?;
        }
        
        // Draw content based on selected tab
        match selected_tab {
            0 => draw_attributes_tab(terminal, center_x, center_y, player_attrs, selected_item)?,
            1 => draw_skills_tab(terminal, center_x, center_y, player_skills, selected_item)?,
            2 => draw_abilities_tab(terminal, center_x, center_y, player_abilities, selected_item)?,
            _ => {}
        }
        
        // Draw instructions
        terminal.draw_text_centered(
            center_y + 15,
            "Tab/Shift+Tab: Switch tabs | Up/Down: Navigate | Enter: Allocate point | Esc: Exit",
            Color::Grey,
            Color::Black
        )?;
        
        // Flush the output
        terminal.flush()
    });
    
    result
}

fn draw_attributes_tab(terminal: &mut crate::rendering::terminal::Terminal, center_x: u16, center_y: u16, attrs: &Attributes, selected: usize) -> Result<(), Box<dyn std::error::Error>> {
    let attributes = [
        (AttributeType::Strength, attrs.strength),
        (AttributeType::Dexterity, attrs.dexterity),
        (AttributeType::Constitution, attrs.constitution),
        (AttributeType::Intelligence, attrs.intelligence),
        (AttributeType::Wisdom, attrs.wisdom),
        (AttributeType::Charisma, attrs.charisma),
    ];
    
    terminal.draw_text_centered(center_y - 10, &format!("Unspent Attribute Points: {}", attrs.unspent_points), Color::Green, Color::Black)?;
    
    for (i, (attr_type, value)) in attributes.iter().enumerate() {
        let y_pos = center_y - 8 + i as u16;
        let color = if i == selected { Color::Yellow } else { Color::White };
        
        // Draw selection indicator
        if i == selected {
            terminal.draw_text(center_x - 22, y_pos, "→", Color::Yellow, Color::Black)?;
        }
        
        // Draw attribute name and value
        let modifier = (value - 10) / 2;
        let modifier_str = if modifier >= 0 { format!("+{}", modifier) } else { format!("{}", modifier) };
        
        terminal.draw_text(center_x - 20, y_pos, &format!("{:?}: {} ({})", attr_type, value, modifier_str), color, Color::Black)?;
    }
    
    Ok(())
}

fn draw_skills_tab(terminal: &mut crate::rendering::terminal::Terminal, center_x: u16, center_y: u16, skills: &Skills, selected: usize) -> Result<(), Box<dyn std::error::Error>> {
    let skill_types = SkillType::all();
    
    terminal.draw_text_centered(center_y - 10, &format!("Unspent Skill Points: {}", skills.unspent_skill_points), Color::Green, Color::Black)?;
    
    for (i, skill_type) in skill_types.iter().enumerate() {
        let y_pos = center_y - 8 + i as u16;
        let color = if i == selected { Color::Yellow } else { Color::White };
        let level = skills.get_skill_level(*skill_type);
        
        // Draw selection indicator
        if i == selected {
            terminal.draw_text(center_x - 22, y_pos, "→", Color::Yellow, Color::Black)?;
        }
        
        // Draw skill name and level
        terminal.draw_text(center_x - 20, y_pos, &format!("{}: {}/5", skill_type.name(), level), color, Color::Black)?;
    }
    
    Ok(())
}

fn draw_abilities_tab(terminal: &mut crate::rendering::terminal::Terminal, center_x: u16, center_y: u16, abilities: &Abilities, selected: usize) -> Result<(), Box<dyn std::error::Error>> {
    terminal.draw_text_centered(center_y - 10, "Available Abilities", Color::Green, Color::Black)?;
    
    let mut ability_list: Vec<AbilityType> = abilities.abilities.iter().cloned().collect();
    ability_list.sort_by_key(|a| a.required_level());
    
    for (i, ability_type) in ability_list.iter().enumerate() {
        let y_pos = center_y - 8 + i as u16;
        let color = if i == selected { Color::Yellow } else { Color::White };
        let cooldown = abilities.get_cooldown(*ability_type);
        
        // Draw selection indicator
        if i == selected {
            terminal.draw_text(center_x - 22, y_pos, "→", Color::Yellow, Color::Black)?;
        }
        
        // Draw ability name and cooldown
        let cooldown_str = if cooldown > 0 { format!(" (Cooldown: {})", cooldown) } else { String::new() };
        terminal.draw_text(center_x - 20, y_pos, &format!("{}{}", ability_type.name(), cooldown_str), color, Color::Black)?;
    }
    
    Ok(())
}

pub enum ProgressionAction {
    AllocateAttribute(AttributeType),
    AllocateSkill(SkillType),
    UseAbility(AbilityType),
    Exit,
}