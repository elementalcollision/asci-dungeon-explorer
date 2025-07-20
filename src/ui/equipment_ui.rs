use specs::{World, WorldExt, Entity, Join};
use crossterm::style::Color;
use crate::components::{Name, Equipped, Inventory, EquipmentSlot, Equippable, MeleePowerBonus, DefenseBonus};
use crate::rendering::terminal::with_terminal;

pub fn show_equipment_screen(world: &World, player_entity: Entity) -> Option<EquipmentAction> {
    // Get player's equipped items
    let equipped_items = world.read_storage::<Equipped>();
    let names = world.read_storage::<Name>();
    let inventories = world.read_storage::<Inventory>();
    let equippables = world.read_storage::<Equippable>();
    let melee_bonuses = world.read_storage::<MeleePowerBonus>();
    let defense_bonuses = world.read_storage::<DefenseBonus>();
    
    // Get player inventory
    let player_inventory = inventories.get(player_entity)?;
    
    // Build a map of equipped items by slot
    let mut equipment_by_slot = std::collections::HashMap::new();
    
    for &item_entity in &player_inventory.items {
        if let Some(equipped) = equipped_items.get(item_entity) {
            if equipped.owner == player_entity {
                let name = names.get(item_entity).map_or("Unknown Item".to_string(), |name| name.name.clone());
                
                // Get item bonuses
                let power_bonus = melee_bonuses.get(item_entity).map(|b| b.power).unwrap_or(0);
                let defense_bonus = defense_bonuses.get(item_entity).map(|b| b.defense).unwrap_or(0);
                
                equipment_by_slot.insert(equipped.slot, (item_entity, name, power_bonus, defense_bonus));
            }
        }
    }
    
    let mut selected_slot = 0;
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
        terminal.draw_text_centered(center_y - 15, "EQUIPMENT", Color::Yellow, Color::Black)?;
        
        // Draw instructions
        terminal.draw_text_centered(
            center_y - 11,
            "Up/Down to navigate, Enter to unequip, Tab for inventory, Esc to exit",
            Color::Grey,
            Color::Black
        )?;
        
        // Draw equipment slots
        let slots = [
            (EquipmentSlot::Helmet, "Head"),
            (EquipmentSlot::Amulet, "Neck"),
            (EquipmentSlot::Armor, "Body"),
            (EquipmentSlot::Melee, "Main Hand"),
            (EquipmentSlot::Shield, "Off Hand"),
            (EquipmentSlot::Gloves, "Hands"),
            (EquipmentSlot::Ring, "Finger"),
            (EquipmentSlot::Boots, "Feet"),
            (EquipmentSlot::Ranged, "Ranged"),
        ];
        
        for (i, (slot, slot_name)) in slots.iter().enumerate() {
            let y_pos = center_y - 8 + i as u16;
            let color = if i == selected_slot { Color::Yellow } else { Color::White };
            
            // Draw selection indicator
            if i == selected_slot {
                terminal.draw_text(center_x - 25, y_pos, "→", Color::Yellow, Color::Black)?;
            }
            
            // Draw slot name
            terminal.draw_text(center_x - 23, y_pos, slot_name, color, Color::Black)?;
            
            // Draw equipped item or empty
            if let Some((_, name, power_bonus, defense_bonus)) = equipment_by_slot.get(slot) {
                let mut item_text = name.clone();
                
                // Add bonus information
                if *power_bonus > 0 {
                    item_text.push_str(&format!(" (+{} Pow)", power_bonus));
                }
                if *defense_bonus > 0 {
                    item_text.push_str(&format!(" (+{} Def)", defense_bonus));
                }
                
                terminal.draw_text(center_x - 10, y_pos, &item_text, Color::Green, Color::Black)?;
            } else {
                terminal.draw_text(center_x - 10, y_pos, "[Empty]", Color::Grey, Color::Black)?;
            }
        }
        
        // Draw total bonuses
        let mut total_power = 0;
        let mut total_defense = 0;
        
        for (_, _, power, defense) in equipment_by_slot.values() {
            total_power += power;
            total_defense += defense;
        }
        
        terminal.draw_text_centered(
            center_y + 5,
            &format!("Total Bonuses: +{} Power, +{} Defense", total_power, total_defense),
            Color::Cyan,
            Color::Black
        )?;
        
        // Flush the output
        terminal.flush()
    });
    
    result
}

pub fn show_equippable_items(world: &World, player_entity: Entity, slot: EquipmentSlot) -> Option<Entity> {
    // Get player's inventory
    let inventories = world.read_storage::<Inventory>();
    let player_inventory = inventories.get(player_entity)?;
    
    // Get item data
    let names = world.read_storage::<Name>();
    let equippables = world.read_storage::<Equippable>();
    let equipped_items = world.read_storage::<Equipped>();
    
    // Find items that can be equipped in this slot
    let mut equippable_items = Vec::new();
    
    for &item_entity in &player_inventory.items {
        if let Some(equippable) = equippables.get(item_entity) {
            if equippable.slot == slot {
                let name = names.get(item_entity).map_or("Unknown Item".to_string(), |name| name.name.clone());
                let is_equipped = equipped_items.get(item_entity).is_some();
                equippable_items.push((item_entity, name, is_equipped));
            }
        }
    }
    
    if equippable_items.is_empty() {
        return None;
    }
    
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
        terminal.draw_text_centered(center_y - 15, &format!("EQUIP {:?}", slot), Color::Yellow, Color::Black)?;
        
        // Draw instructions
        terminal.draw_text_centered(
            center_y - 11,
            "Up/Down to navigate, Enter to equip, Esc to cancel",
            Color::Grey,
            Color::Black
        )?;
        
        // Draw equippable items
        for (i, (_, name, is_equipped)) in equippable_items.iter().enumerate() {
            let y_pos = center_y - 8 + i as u16;
            let color = if i == selected_item { Color::Yellow } else { Color::White };
            
            // Draw selection indicator
            if i == selected_item {
                terminal.draw_text(center_x - 22, y_pos, "→", Color::Yellow, Color::Black)?;
            }
            
            // Draw item name with equipped status
            let item_text = if *is_equipped {
                format!("{} [EQUIPPED]", name)
            } else {
                name.clone()
            };
            
            terminal.draw_text(center_x - 20, y_pos, &item_text, color, Color::Black)?;
        }
        
        // Flush the output
        terminal.flush()
    });
    
    // Return the selected item entity
    if selected_item < equippable_items.len() {
        result = Some(equippable_items[selected_item].0);
    }
    
    result
}

pub enum EquipmentAction {
    UnequipItem(EquipmentSlot),
    ShowEquippableItems(EquipmentSlot),
    Exit,
}