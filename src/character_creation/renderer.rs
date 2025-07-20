use crossterm::style::Color;
use crate::game_state::{RunState, GameState};
use crate::rendering::terminal::with_terminal;
use crate::components::AttributeType;
use super::CharacterCreationState;

pub fn render_character_creation(game_state: &GameState, creation_state: &CharacterCreationState) {
    match game_state.run_state {
        RunState::CharacterName => render_name_screen(creation_state),
        RunState::CharacterClass => render_class_screen(creation_state),
        RunState::CharacterBackground => render_background_screen(creation_state),
        RunState::CharacterAttributes => render_attributes_screen(creation_state),
        RunState::CharacterEquipment => render_equipment_screen(creation_state),
        RunState::CharacterConfirm => render_confirm_screen(creation_state),
        _ => {}
    }
}

fn render_name_screen(creation_state: &CharacterCreationState) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 10, "CHARACTER CREATION", Color::Yellow, Color::Black)?;
        
        // Draw name prompt
        terminal.draw_text_centered(center_y - 5, "Enter your character's name:", Color::White, Color::Black)?;
        
        // Draw name input box
        terminal.draw_box(center_x - 15, center_y - 3, 30, 3, Color::White, Color::Black)?;
        terminal.draw_text(center_x - 13, center_y - 2, &creation_state.player_name, Color::White, Color::Black)?;
        
        // Draw cursor
        terminal.draw_char_at(
            (center_x - 13 + creation_state.player_name.len()) as u16,
            (center_y - 2) as u16,
            '_',
            Color::White,
            Color::Black
        )?;
        
        // Draw instructions
        terminal.draw_text_centered(center_y + 5, "Press Enter to continue, Esc to return to main menu", Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}

fn render_class_screen(creation_state: &CharacterCreationState) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 15, "CHARACTER CREATION", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(center_y - 13, "Choose your class", Color::White, Color::Black)?;
        
        // Draw class options
        let classes = [
            (crate::components::ClassType::Fighter, "1", "Fighter - A skilled warrior with high strength and constitution"),
            (crate::components::ClassType::Rogue, "2", "Rogue - A nimble thief with high dexterity"),
            (crate::components::ClassType::Mage, "3", "Mage - A powerful spellcaster with high intelligence"),
            (crate::components::ClassType::Cleric, "4", "Cleric - A divine spellcaster with high wisdom"),
            (crate::components::ClassType::Ranger, "5", "Ranger - A skilled hunter with high dexterity and wisdom"),
        ];
        
        for (i, (class_type, key, desc)) in classes.iter().enumerate() {
            let y_pos = center_y - 10 + i as u16 * 2;
            let color = if *class_type == creation_state.selected_class { Color::Yellow } else { Color::White };
            terminal.draw_text(center_x - 30, y_pos, &format!("{} - {}", key, desc), color, Color::Black)?;
        }
        
        // Draw class description
        let desc_y = center_y + 2;
        terminal.draw_text_centered(desc_y, "Class Description:", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(desc_y + 2, creation_state.selected_class.description(), Color::White, Color::Black)?;
        
        // Draw primary and secondary attributes
        let primary = creation_state.selected_class.primary_attribute();
        let secondary = creation_state.selected_class.secondary_attribute();
        
        terminal.draw_text_centered(desc_y + 4, &format!("Primary Attribute: {:?} (+2)", primary), Color::Green, Color::Black)?;
        terminal.draw_text_centered(desc_y + 5, &format!("Secondary Attribute: {:?} (+1)", secondary), Color::Green, Color::Black)?;
        
        // Draw instructions
        terminal.draw_text_centered(height - 3, "Press Enter to continue, Esc to go back", Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}

fn render_background_screen(creation_state: &CharacterCreationState) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 15, "CHARACTER CREATION", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(center_y - 13, "Choose your background", Color::White, Color::Black)?;
        
        // Draw background options
        let backgrounds = [
            (crate::components::BackgroundType::Soldier, "1", "Soldier - Military training and discipline"),
            (crate::components::BackgroundType::Scholar, "2", "Scholar - Academic knowledge and arcane studies"),
            (crate::components::BackgroundType::Noble, "3", "Noble - Privileged upbringing and education"),
            (crate::components::BackgroundType::Outlaw, "4", "Outlaw - Life outside the law and society"),
            (crate::components::BackgroundType::Acolyte, "5", "Acolyte - Religious training and divine service"),
            (crate::components::BackgroundType::Merchant, "6", "Merchant - Trading, negotiation, and worldly knowledge"),
        ];
        
        for (i, (bg_type, key, desc)) in backgrounds.iter().enumerate() {
            let y_pos = center_y - 10 + i as u16 * 2;
            let color = if *bg_type == creation_state.selected_background { Color::Yellow } else { Color::White };
            terminal.draw_text(center_x - 30, y_pos, &format!("{} - {}", key, desc), color, Color::Black)?;
        }
        
        // Draw background description
        let desc_y = center_y + 2;
        terminal.draw_text_centered(desc_y, "Background Description:", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(desc_y + 2, creation_state.selected_background.description(), Color::White, Color::Black)?;
        
        // Draw attribute bonus
        let bonus_attr = creation_state.selected_background.attribute_bonus();
        terminal.draw_text_centered(desc_y + 4, &format!("Attribute Bonus: {:?} (+1)", bonus_attr), Color::Green, Color::Black)?;
        
        // Draw instructions
        terminal.draw_text_centered(height - 3, "Press Enter to continue, Esc to go back", Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}

fn render_attributes_screen(creation_state: &CharacterCreationState) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 15, "CHARACTER CREATION", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(center_y - 13, "Allocate your attributes", Color::White, Color::Black)?;
        
        // Draw attribute allocation instructions
        terminal.draw_text_centered(center_y - 11, &format!("Unspent Points: {}", creation_state.attributes.unspent_points), Color::Green, Color::Black)?;
        terminal.draw_text_centered(center_y - 10, "Use arrow keys or hjkl to navigate, left/right to adjust", Color::Grey, Color::Black)?;
        
        // Draw attributes
        let attributes = [
            (AttributeType::Strength, "Strength", creation_state.attributes.strength),
            (AttributeType::Dexterity, "Dexterity", creation_state.attributes.dexterity),
            (AttributeType::Constitution, "Constitution", creation_state.attributes.constitution),
            (AttributeType::Intelligence, "Intelligence", creation_state.attributes.intelligence),
            (AttributeType::Wisdom, "Wisdom", creation_state.attributes.wisdom),
            (AttributeType::Charisma, "Charisma", creation_state.attributes.charisma),
        ];
        
        for (i, (attr_type, name, value)) in attributes.iter().enumerate() {
            let y_pos = center_y - 7 + i as u16 * 2;
            let color = if *attr_type == creation_state.selected_attribute { Color::Yellow } else { Color::White };
            
            // Calculate modifier
            let modifier = creation_state.attributes.get_modifier(*attr_type);
            let modifier_str = if modifier >= 0 { format!("+{}", modifier) } else { format!("{}", modifier) };
            
            terminal.draw_text(center_x - 20, y_pos, name, color, Color::Black)?;
            terminal.draw_text(center_x + 5, y_pos, &format!("{} ({})", value, modifier_str), color, Color::Black)?;
            
            // Draw selection indicator
            if *attr_type == creation_state.selected_attribute {
                terminal.draw_text(center_x - 22, y_pos, "→", Color::Yellow, Color::Black)?;
            }
        }
        
        // Draw attribute descriptions
        let desc_y = center_y + 5;
        match creation_state.selected_attribute {
            AttributeType::Strength => {
                terminal.draw_text_centered(desc_y, "Strength", Color::Yellow, Color::Black)?;
                terminal.draw_text_centered(desc_y + 1, "Affects melee damage, carrying capacity, and physical tasks", Color::White, Color::Black)?;
            },
            AttributeType::Dexterity => {
                terminal.draw_text_centered(desc_y, "Dexterity", Color::Yellow, Color::Black)?;
                terminal.draw_text_centered(desc_y + 1, "Affects accuracy, defense, stealth, and reflexes", Color::White, Color::Black)?;
            },
            AttributeType::Constitution => {
                terminal.draw_text_centered(desc_y, "Constitution", Color::Yellow, Color::Black)?;
                terminal.draw_text_centered(desc_y + 1, "Affects health points, stamina, and resistance to poison", Color::White, Color::Black)?;
            },
            AttributeType::Intelligence => {
                terminal.draw_text_centered(desc_y, "Intelligence", Color::Yellow, Color::Black)?;
                terminal.draw_text_centered(desc_y + 1, "Affects spell power, learning ability, and puzzle solving", Color::White, Color::Black)?;
            },
            AttributeType::Wisdom => {
                terminal.draw_text_centered(desc_y, "Wisdom", Color::Yellow, Color::Black)?;
                terminal.draw_text_centered(desc_y + 1, "Affects perception, intuition, and divine magic", Color::White, Color::Black)?;
            },
            AttributeType::Charisma => {
                terminal.draw_text_centered(desc_y, "Charisma", Color::Yellow, Color::Black)?;
                terminal.draw_text_centered(desc_y + 1, "Affects social interactions, leadership, and certain magic", Color::White, Color::Black)?;
            },
        }
        
        // Draw instructions
        let instruction_text = if creation_state.attributes.unspent_points == 0 {
            "Press Enter to continue, Esc to go back"
        } else {
            "Allocate all points before continuing (Esc to go back)"
        };
        terminal.draw_text_centered(height - 3, instruction_text, Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}

fn render_equipment_screen(creation_state: &CharacterCreationState) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 15, "CHARACTER CREATION", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(center_y - 13, "Choose your starting equipment", Color::White, Color::Black)?;
        
        // Draw equipment selection instructions
        terminal.draw_text_centered(center_y - 11, "Select up to 3 items (Space to select, Enter to continue)", Color::Grey, Color::Black)?;
        terminal.draw_text_centered(center_y - 10, &format!("Selected: {}/3", creation_state.selected_equipment_indices.len()), Color::Green, Color::Black)?;
        
        // Draw equipment options
        for (i, (name, slot)) in creation_state.available_equipment.iter().enumerate() {
            let y_pos = center_y - 7 + i as u16;
            let is_selected = creation_state.selected_equipment_indices.contains(&i);
            let is_current = i == creation_state.selected_equipment;
            
            let color = if is_current { Color::Yellow } else { Color::White };
            let prefix = if is_selected { "[X] " } else { "[ ] " };
            
            terminal.draw_text(center_x - 20, y_pos, &format!("{}{} ({})", prefix, name, format!("{:?}", slot)), color, Color::Black)?;
            
            // Draw selection indicator
            if is_current {
                terminal.draw_text(center_x - 22, y_pos, "→", Color::Yellow, Color::Black)?;
            }
        }
        
        // Draw equipment description
        if creation_state.selected_equipment < creation_state.available_equipment.len() {
            let (name, slot) = &creation_state.available_equipment[creation_state.selected_equipment];
            let desc_y = center_y + 5;
            
            terminal.draw_text_centered(desc_y, name, Color::Yellow, Color::Black)?;
            
            let desc = match slot {
                crate::components::EquipmentSlot::Melee => "A melee weapon for close combat. Increases attack power.",
                crate::components::EquipmentSlot::Ranged => "A ranged weapon for attacking from a distance.",
                crate::components::EquipmentSlot::Shield => "A shield for protection. Increases defense.",
                crate::components::EquipmentSlot::Armor => "Body armor for protection. Significantly increases defense.",
                crate::components::EquipmentSlot::Helmet => "Head protection. Increases defense.",
                crate::components::EquipmentSlot::Boots => "Footwear for protection and mobility. Increases defense.",
                crate::components::EquipmentSlot::Gloves => "Hand protection. Increases attack power slightly.",
                crate::components::EquipmentSlot::Ring => "A magical ring. Provides minor bonuses to defense.",
                crate::components::EquipmentSlot::Amulet => "A magical amulet. Provides minor bonuses to attack and defense.",
            };
            
            terminal.draw_text_centered(desc_y + 1, desc, Color::White, Color::Black)?;
        }
        
        // Draw instructions
        terminal.draw_text_centered(height - 3, "Press Enter to continue, Esc to go back", Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}

fn render_confirm_screen(creation_state: &CharacterCreationState) {
    let _ = with_terminal(|terminal| {
        // Clear the screen
        terminal.clear()?;
        
        // Get terminal size
        let (width, height) = terminal.size();
        
        // Calculate center position
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Draw title
        terminal.draw_text_centered(center_y - 15, "CHARACTER CREATION", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(center_y - 13, "Confirm your character", Color::White, Color::Black)?;
        
        // Draw character summary
        terminal.draw_text(center_x - 30, center_y - 10, &format!("Name: {}", creation_state.player_name), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 30, center_y - 8, &format!("Class: {}", creation_state.selected_class.name()), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 30, center_y - 6, &format!("Background: {}", creation_state.selected_background.name()), Color::White, Color::Black)?;
        
        // Draw attributes
        terminal.draw_text(center_x - 30, center_y - 4, "Attributes:", Color::White, Color::Black)?;
        terminal.draw_text(center_x - 25, center_y - 2, &format!("STR: {}", creation_state.attributes.strength), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 25, center_y, &format!("DEX: {}", creation_state.attributes.dexterity), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 25, center_y + 2, &format!("CON: {}", creation_state.attributes.constitution), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 10, center_y - 2, &format!("INT: {}", creation_state.attributes.intelligence), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 10, center_y, &format!("WIS: {}", creation_state.attributes.wisdom), Color::White, Color::Black)?;
        terminal.draw_text(center_x - 10, center_y + 2, &format!("CHA: {}", creation_state.attributes.charisma), Color::White, Color::Black)?;
        
        // Draw equipment
        terminal.draw_text(center_x + 5, center_y - 4, "Equipment:", Color::White, Color::Black)?;
        for (i, &idx) in creation_state.selected_equipment_indices.iter().enumerate() {
            let (name, _) = &creation_state.available_equipment[idx];
            terminal.draw_text(center_x + 10, center_y - 2 + i as u16 * 2, name, Color::White, Color::Black)?;
        }
        
        // Draw confirmation prompt
        terminal.draw_text_centered(center_y + 8, "Are you ready to begin your adventure?", Color::Yellow, Color::Black)?;
        terminal.draw_text_centered(center_y + 10, "Press Y or Enter to confirm, N or Esc to go back", Color::Grey, Color::Black)?;
        
        terminal.flush()
    });
}