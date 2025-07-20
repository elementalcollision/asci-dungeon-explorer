use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use crate::components::{
    Equipped, Equippable, WantsToUseItem, Name, MeleePowerBonus, DefenseBonus, 
    Inventory, CombatStats, EquipmentSlot
};
use crate::resources::GameLog;

pub struct EquipmentSystem {}

impl<'a> System<'a> for EquipmentSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        WriteStorage<'a, Inventory>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities, 
            mut wants_use, 
            names, 
            equippables, 
            mut equipped,
            melee_power_bonuses,
            defense_bonuses,
            mut inventories,
            mut gamelog
        ) = data;

        // Process equip/unequip requests
        let mut to_equip = Vec::new();
        for (entity, use_item) in (&entities, &wants_use).join() {
            // Check if the item is equippable
            if let Some(can_equip) = equippables.get(use_item.item) {
                let item_name = if let Some(name) = names.get(use_item.item) {
                    name.name.clone()
                } else {
                    "Unknown item".to_string()
                };
                
                // Check if the item is already equipped
                let already_equipped = equipped.get(use_item.item).is_some();
                
                if already_equipped {
                    // Unequip the item
                    equipped.remove(use_item.item);
                    gamelog.add_entry(format!("You unequip the {}.", item_name));
                } else {
                    // Check if something else is already equipped in this slot
                    let mut to_unequip: Option<Entity> = None;
                    
                    // Find any items in the same slot
                    if let Some(inv) = inventories.get(entity) {
                        for &item_entity in inv.items.iter() {
                            if let Some(item_equipped) = equipped.get(item_entity) {
                                if item_equipped.owner == entity && item_equipped.slot == can_equip.slot {
                                    to_unequip = Some(item_entity);
                                }
                            }
                        }
                    }
                    
                    // Unequip the previous item if any
                    if let Some(item_entity) = to_unequip {
                        equipped.remove(item_entity);
                        if let Some(name) = names.get(item_entity) {
                            gamelog.add_entry(format!("You unequip the {}.", name.name));
                        }
                    }
                    
                    // Equip the new item
                    equipped.insert(use_item.item, Equipped { owner: entity, slot: can_equip.slot })
                        .expect("Failed to equip item");
                    gamelog.add_entry(format!("You equip the {}.", item_name));
                }
                
                to_equip.push(entity);
            }
        }
        
        // Clean up use requests
        for entity in to_equip {
            wants_use.remove(entity);
        }
    }
}

// System to apply equipment bonuses to combat stats
pub struct EquipmentBonusSystem {}

impl<'a> System<'a> for EquipmentBonusSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Equipped>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Inventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_stats, equipped_items, melee_bonuses, defense_bonuses, inventories) = data;

        // Reset and recalculate equipment bonuses for all entities
        for (entity, stats, inventory) in (&entities, &mut combat_stats, &inventories).join() {
            // Reset bonuses (assuming base stats are stored elsewhere or we track base vs modified)
            let mut total_power_bonus = 0;
            let mut total_defense_bonus = 0;
            
            // Calculate bonuses from equipped items
            for &item_entity in inventory.items.iter() {
                if let Some(equipped) = equipped_items.get(item_entity) {
                    if equipped.owner == entity {
                        // Add power bonus
                        if let Some(power_bonus) = melee_bonuses.get(item_entity) {
                            total_power_bonus += power_bonus.power;
                        }
                        
                        // Add defense bonus
                        if let Some(defense_bonus) = defense_bonuses.get(item_entity) {
                            total_defense_bonus += defense_bonus.defense;
                        }
                    }
                }
            }
            
            // Apply bonuses (this is a simplified approach)
            // In a real system, you'd want to track base stats separately
            stats.power = 5 + total_power_bonus; // 5 is base power
            stats.defense = 2 + total_defense_bonus; // 2 is base defense
        }
    }
}