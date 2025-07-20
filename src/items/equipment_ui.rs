use crossterm::{
    event::{KeyCode, KeyEvent},
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor},
};
use specs::{World, Entity, Join, WorldExt};
use crate::components::{Name, Player};
use crate::items::{
    ItemProperties, ItemBonuses, get_item_display_name, get_item_current_value,
    equipment_system::{Equipment, EquipmentSlot, EquipmentStats}
};
use std::io::{Write, stdout};

pub struct EquipmentUI {
    pub selected_slot: usize,
    pub show_comparison: bool,
    pub comparison_item: Option<Entity>,
}

impl EquipmentUI {
    pub fn new() -> Self {
        EquipmentUI {
            selected_slot: 0,
            show_comparison: false,
            comparison_item: None,
        }
    }

    pub fn render(&self, world: &World, player_entity: Entity, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        let equipment = world.read_storage::<Equipment>();
        
        if let Some(equipment) = equipment.get(player_entity) {
            self.render_equipment_screen(world, equipment, width, height)?;
        }
        
        Ok(())
    }

    fn render_equipment_screen(
        &self,
        world: &World,
        equipment: &Equipment,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        crossterm::execute!(stdout, SetForegroundColor(Color::Yellow))?;
        println!("=== EQUIPMENT ===");
        crossterm::execute!(stdout, ResetColor)?;
        
        println!();

        // Render equipment slots
        let slots = self.get_ordered_slots();
        for (index, slot) in slots.iter().enumerate() {
            let is_selected = index == self.selected_slot;
            
            if is_selected {
                crossterm::execute!(stdout, SetBackgroundColor(Color::DarkGrey))?;
            }
            
            self.render_equipment_slot(world, equipment, slot, is_selected)?;
            
            if is_selected {
                crossterm::execute!(stdout, ResetColor)?;
            }
        }

        // Show equipment stats
        self.render_equipment_stats(&equipment.stat_cache, height)?;

        // Show comparison if active
        if self.show_comparison {
            if let Some(comparison_item) = self.comparison_item {
                self.render_item_comparison(world, equipment, comparison_item, width, height)?;
            }
        }

        // Controls help
        self.render_controls_help(height)?;
        
        stdout.flush()?;
        Ok(())
    }

    fn render_equipment_slot(
        &self,
        world: &World,
        equipment: &Equipment,
        slot: &EquipmentSlot,
        is_selected: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let slot_name = format!("{:12}", slot.name());
        
        if let Some(item_entity) = equipment.get_equipped(slot) {
            let item_name = get_item_display_name(world, item_entity).unwrap_or("Unknown".to_string());
            let properties = world.read_storage::<ItemProperties>();
            
            if let Some(props) = properties.get(item_entity) {
                let (r, g, b) = props.rarity.color();
                crossterm::execute!(stdout(), SetForegroundColor(Color::Rgb { r, g, b }))?;
            }
            
            println!("{}: {}", slot_name, item_name);
            crossterm::execute!(stdout(), ResetColor)?;
        } else {
            crossterm::execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
            println!("{}: <empty>", slot_name);
            crossterm::execute!(stdout(), ResetColor)?;
        }
        
        Ok(())
    }

    fn render_equipment_stats(&self, stats: &EquipmentStats, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        let stats_start_y = height / 2;
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y))?;
        
        crossterm::execute!(stdout(), SetForegroundColor(Color::Cyan))?;
        println!("Equipment Bonuses:");
        crossterm::execute!(stdout(), ResetColor)?;
        
        let mut line = 1;
        
        // Combat stats
        if stats.attack_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Attack: {:+}", stats.attack_bonus);
            line += 1;
        }
        if stats.damage_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Damage: {:+}", stats.damage_bonus);
            line += 1;
        }
        if stats.defense_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Defense: {:+}", stats.defense_bonus);
            line += 1;
        }
        if stats.critical_chance_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Crit Chance: {:+}%", stats.critical_chance_bonus);
            line += 1;
        }
        
        // Attribute bonuses
        if stats.strength_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Strength: {:+}", stats.strength_bonus);
            line += 1;
        }
        if stats.dexterity_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Dexterity: {:+}", stats.dexterity_bonus);
            line += 1;
        }
        if stats.constitution_bonus != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Constitution: {:+}", stats.constitution_bonus);
            line += 1;
        }
        
        // Resistances
        if stats.fire_resistance != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Fire Resist: {:+}%", stats.fire_resistance);
            line += 1;
        }
        if stats.cold_resistance != 0 {
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(40, stats_start_y + line))?;
            println!("Cold Resist: {:+}%", stats.cold_resistance);
            line += 1;
        }
        
        Ok(())
    }

    fn render_item_comparison(
        &self,
        world: &World,
        equipment: &Equipment,
        comparison_item: Entity,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let comparison_start_x = width - 40;
        let comparison_start_y = 2;
        
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y))?;
        crossterm::execute!(stdout(), SetForegroundColor(Color::Yellow))?;
        println!("Item Comparison:");
        crossterm::execute!(stdout(), ResetColor)?;
        
        // Get comparison item info
        let item_name = get_item_display_name(world, comparison_item).unwrap_or("Unknown".to_string());
        let item_value = get_item_current_value(world, comparison_item);
        
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + 2))?;
        println!("Item: {}", item_name);
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + 3))?;
        println!("Value: {} gold", item_value);
        
        // Show stat comparison
        let item_bonuses = world.read_storage::<ItemBonuses>();
        if let Some(bonuses) = item_bonuses.get(comparison_item) {
            let mut line = 5;
            
            crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + line))?;
            crossterm::execute!(stdout(), SetForegroundColor(Color::Green))?;
            println!("Bonuses:");
            crossterm::execute!(stdout(), ResetColor)?;
            line += 1;
            
            if bonuses.combat_bonuses.attack_bonus != 0 {
                crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + line))?;
                println!("  Attack: {:+}", bonuses.combat_bonuses.attack_bonus);
                line += 1;
            }
            if bonuses.combat_bonuses.damage_bonus != 0 {
                crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + line))?;
                println!("  Damage: {:+}", bonuses.combat_bonuses.damage_bonus);
                line += 1;
            }
            if bonuses.combat_bonuses.defense_bonus != 0 {
                crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + line))?;
                println!("  Defense: {:+}", bonuses.combat_bonuses.defense_bonus);
                line += 1;
            }
            
            for (attr, value) in &bonuses.attribute_bonuses {
                crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + line))?;
                println!("  {}: {:+}", attr, value);
                line += 1;
            }
        }
        
        // Show what would be replaced
        let properties = world.read_storage::<ItemProperties>();
        if let Some(props) = properties.get(comparison_item) {
            if let Some(slot) = self.detect_item_slot(&props.item_type) {
                if let Some(current_item) = equipment.get_equipped(&slot) {
                    crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + 15))?;
                    crossterm::execute!(stdout(), SetForegroundColor(Color::Red))?;
                    println!("Would replace:");
                    crossterm::execute!(stdout(), ResetColor)?;
                    
                    let current_name = get_item_display_name(world, current_item).unwrap_or("Unknown".to_string());
                    crossterm::execute!(stdout(), crossterm::cursor::MoveTo(comparison_start_x, comparison_start_y + 16))?;
                    println!("  {}", current_name);
                }
            }
        }
        
        Ok(())
    }

    fn render_controls_help(&self, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(0, height - 2))?;
        crossterm::execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
        if self.show_comparison {
            println!("Controls: ↑↓ Navigate | Enter Equip | U Unequip | C Close Comparison | ESC Exit");
        } else {
            println!("Controls: ↑↓ Navigate | Enter Equip | U Unequip | C Compare Item | ESC Exit");
        }
        crossterm::execute!(stdout(), ResetColor)?;
        Ok(())
    }

    fn get_ordered_slots(&self) -> Vec<EquipmentSlot> {
        vec![
            EquipmentSlot::MainHand,
            EquipmentSlot::OffHand,
            EquipmentSlot::Head,
            EquipmentSlot::Chest,
            EquipmentSlot::Legs,
            EquipmentSlot::Feet,
            EquipmentSlot::Hands,
            EquipmentSlot::Ring1,
            EquipmentSlot::Ring2,
            EquipmentSlot::Amulet,
            EquipmentSlot::Cloak,
            EquipmentSlot::Belt,
        ]
    }

    fn detect_item_slot(&self, item_type: &crate::items::ItemType) -> Option<EquipmentSlot> {
        match item_type {
            crate::items::ItemType::Weapon(_) => Some(EquipmentSlot::MainHand),
            crate::items::ItemType::Armor(armor_type) => match armor_type {
                crate::items::ArmorType::Helmet => Some(EquipmentSlot::Head),
                crate::items::ArmorType::Chest => Some(EquipmentSlot::Chest),
                crate::items::ArmorType::Legs => Some(EquipmentSlot::Legs),
                crate::items::ArmorType::Boots => Some(EquipmentSlot::Feet),
                crate::items::ArmorType::Gloves => Some(EquipmentSlot::Hands),
                crate::items::ArmorType::Shield => Some(EquipmentSlot::OffHand),
                crate::items::ArmorType::Ring => Some(EquipmentSlot::Ring1),
                crate::items::ArmorType::Amulet => Some(EquipmentSlot::Amulet),
                crate::items::ArmorType::Cloak => Some(EquipmentSlot::Cloak),
            },
            _ => None,
        }
    }

    pub fn handle_input(&mut self, key: KeyEvent, world: &mut World, player_entity: Entity) -> EquipmentAction {
        match key.code {
            KeyCode::Up => {
                if self.selected_slot > 0 {
                    self.selected_slot -= 1;
                }
                EquipmentAction::None
            },
            KeyCode::Down => {
                let slots = self.get_ordered_slots();
                if self.selected_slot < slots.len() - 1 {
                    self.selected_slot += 1;
                }
                EquipmentAction::None
            },
            KeyCode::Enter => {
                // Equip item from inventory (would need inventory integration)
                EquipmentAction::None
            },
            KeyCode::Char('u') | KeyCode::Char('U') => {
                let slots = self.get_ordered_slots();
                if self.selected_slot < slots.len() {
                    let slot = slots[self.selected_slot].clone();
                    return EquipmentAction::Unequip(slot);
                }
                EquipmentAction::None
            },
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if self.show_comparison {
                    self.show_comparison = false;
                    self.comparison_item = None;
                } else {
                    // Would need to select an item to compare
                    self.show_comparison = true;
                }
                EquipmentAction::None
            },
            KeyCode::Esc => {
                if self.show_comparison {
                    self.show_comparison = false;
                    self.comparison_item = None;
                    EquipmentAction::None
                } else {
                    EquipmentAction::Close
                }
            },
            _ => EquipmentAction::None,
        }
    }

    pub fn set_comparison_item(&mut self, item: Entity) {
        self.comparison_item = Some(item);
        self.show_comparison = true;
    }
}

#[derive(Debug, Clone)]
pub enum EquipmentAction {
    None,
    Equip(Entity, EquipmentSlot),
    Unequip(EquipmentSlot),
    Compare(Entity),
    Close,
}

/// Equipment comparison utility
pub struct EquipmentComparison;

impl EquipmentComparison {
    pub fn compare_items(
        world: &World,
        current_item: Option<Entity>,
        new_item: Entity,
    ) -> ItemComparisonResult {
        let item_bonuses = world.read_storage::<ItemBonuses>();
        let item_properties = world.read_storage::<ItemProperties>();
        
        let new_bonuses = item_bonuses.get(new_item);
        let new_props = item_properties.get(new_item);
        
        let mut comparison = ItemComparisonResult {
            new_item_name: get_item_display_name(world, new_item).unwrap_or("Unknown".to_string()),
            current_item_name: None,
            stat_changes: Vec::new(),
            is_upgrade: false,
            value_difference: 0,
        };
        
        if let Some(current) = current_item {
            comparison.current_item_name = Some(get_item_display_name(world, current).unwrap_or("Unknown".to_string()));
            
            let current_bonuses = item_bonuses.get(current);
            let current_props = item_properties.get(current);
            
            // Compare stats
            if let (Some(new_b), Some(curr_b)) = (new_bonuses, current_bonuses) {
                Self::compare_combat_stats(&mut comparison, &new_b.combat_bonuses, &curr_b.combat_bonuses);
                Self::compare_attribute_bonuses(&mut comparison, &new_b.attribute_bonuses, &curr_b.attribute_bonuses);
            }
            
            // Compare values
            if let (Some(new_p), Some(curr_p)) = (new_props, current_props) {
                comparison.value_difference = new_p.value - curr_p.value;
            }
        } else {
            // No current item, so any stats are improvements
            if let Some(bonuses) = new_bonuses {
                if bonuses.combat_bonuses.attack_bonus > 0 {
                    comparison.stat_changes.push(StatChange {
                        stat_name: "Attack".to_string(),
                        change: bonuses.combat_bonuses.attack_bonus,
                        is_improvement: true,
                    });
                }
                if bonuses.combat_bonuses.damage_bonus > 0 {
                    comparison.stat_changes.push(StatChange {
                        stat_name: "Damage".to_string(),
                        change: bonuses.combat_bonuses.damage_bonus,
                        is_improvement: true,
                    });
                }
                if bonuses.combat_bonuses.defense_bonus > 0 {
                    comparison.stat_changes.push(StatChange {
                        stat_name: "Defense".to_string(),
                        change: bonuses.combat_bonuses.defense_bonus,
                        is_improvement: true,
                    });
                }
            }
            
            if let Some(props) = new_props {
                comparison.value_difference = props.value;
            }
        }
        
        // Determine if it's an overall upgrade
        comparison.is_upgrade = comparison.stat_changes.iter()
            .filter(|change| change.is_improvement)
            .count() > comparison.stat_changes.iter()
            .filter(|change| !change.is_improvement)
            .count();
        
        comparison
    }
    
    fn compare_combat_stats(
        comparison: &mut ItemComparisonResult,
        new_stats: &crate::items::CombatBonuses,
        current_stats: &crate::items::CombatBonuses,
    ) {
        let changes = [
            ("Attack", new_stats.attack_bonus - current_stats.attack_bonus),
            ("Damage", new_stats.damage_bonus - current_stats.damage_bonus),
            ("Defense", new_stats.defense_bonus - current_stats.defense_bonus),
            ("Crit Chance", new_stats.critical_chance_bonus - current_stats.critical_chance_bonus),
            ("Crit Damage", new_stats.critical_damage_bonus - current_stats.critical_damage_bonus),
        ];
        
        for (stat_name, change) in changes {
            if change != 0 {
                comparison.stat_changes.push(StatChange {
                    stat_name: stat_name.to_string(),
                    change,
                    is_improvement: change > 0,
                });
            }
        }
    }
    
    fn compare_attribute_bonuses(
        comparison: &mut ItemComparisonResult,
        new_attrs: &std::collections::HashMap<String, i32>,
        current_attrs: &std::collections::HashMap<String, i32>,
    ) {
        let mut all_attrs = std::collections::HashSet::new();
        all_attrs.extend(new_attrs.keys());
        all_attrs.extend(current_attrs.keys());
        
        for attr in all_attrs {
            let new_value = *new_attrs.get(attr).unwrap_or(&0);
            let current_value = *current_attrs.get(attr).unwrap_or(&0);
            let change = new_value - current_value;
            
            if change != 0 {
                comparison.stat_changes.push(StatChange {
                    stat_name: attr.clone(),
                    change,
                    is_improvement: change > 0,
                });
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemComparisonResult {
    pub new_item_name: String,
    pub current_item_name: Option<String>,
    pub stat_changes: Vec<StatChange>,
    pub is_upgrade: bool,
    pub value_difference: i32,
}

#[derive(Debug, Clone)]
pub struct StatChange {
    pub stat_name: String,
    pub change: i32,
    pub is_improvement: bool,
}