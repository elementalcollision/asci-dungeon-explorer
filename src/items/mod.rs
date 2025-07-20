pub mod item_components;
pub mod item_factory;
pub mod item_serialization;
pub mod inventory_system;
pub mod inventory_ui;
pub mod inventory_integration;
pub mod item_generation;
pub mod loot_tables;
pub mod name_generator;
pub mod consumable_system;
pub mod consumable_factory;
pub mod consumable_integration;
pub mod generation_integration;
pub mod integration_example;
pub mod equipment_system;
pub mod equipment_factory;
pub mod containers;

#[cfg(test)]
mod tests;

pub use item_components::*;
pub use item_factory::ItemFactory;
pub use item_serialization::{
    SerializableItem, ItemDatabase, ItemTemplate, ItemCollection,
    serialize_items_in_area, count_items_by_type
};
pub use inventory_system::{
    AdvancedInventory, InventorySlot, InventorySortMode, Container, ContainerType,
    ItemPickupSystem, ItemDropSystem, AutoPickupSystem, InventoryManagementSystem,
    Pickupable, InventoryBonus
};
pub use inventory_ui::{
    InventoryUI, InventoryAction, InventoryFilter, ContainerUI, ContainerAction, ContainerPanel
};
pub use inventory_integration::{
    InventoryIntegrationExample, InventoryGameState
};
pub use item_generation::{
    ItemGenerator, GenerationContext, LootTable, LootEntry, AffixTable, Affix, AffixType,
    RarityWeights, DepthScaling
};
pub use loot_tables::{
    LootTableManager, LootTableStatistics
};
pub use name_generator::{
    ItemNameGenerator, NameAffix, AffixApplicability
};
pub use generation_integration::ItemGenerationIntegration;
pub use consumable_system::{
    Consumable, ConsumableEffect, StatusEffectType, StatusEffect, ConsumableRequirements,
    ConsumableRestriction, ConsumableCooldowns, StatusEffects, WantsToUseConsumable,
    ConsumableUsageSystem, ConsumableUpdateSystem
};
pub use consumable_factory::{
    ConsumableFactory, PotionPotency, FoodType, ScrollType, ConsumableContext
};
pub use consumable_integration::ConsumableIntegration;
pub use equipment_system::{
    Equippable, Equipment, EquipmentSlot, EquipmentRequirements, EquipmentSet, SetBonus,
    WantsToEquip, WantsToUnequip, EquipmentSystem, EquipmentStatsSystem, EquipmentSetSystem
};
pub use equipment_factory::{EquipmentFactory, EquipmentQuality};
pub use containers::{
    Container, ContainerType, TrapType, WantsToOpenContainer, WantsToCloseContainer,
    WantsToTakeFromContainer, WantsToPutInContainer, ContainerSystem, LootTable, LootEntry,
    LootResult, ContainerFactory
};

// Re-export commonly used types
pub use item_components::{
    ItemProperties, ItemType, ItemRarity, WeaponType, ArmorType, ConsumableType,
    ToolType, MaterialType, ItemStack, ItemIdentification, MagicalItem,
    ItemBonuses, Enchantment, EnchantmentType, Curse, CurseType
};

// Utility functions for working with items
use specs::{World, Entity, Join, WorldExt};
use crate::components::{Item, Name, Position};

/// Get the display name of an item, considering identification status
pub fn get_item_display_name(world: &World, entity: Entity) -> Option<String> {
    let names = world.read_storage::<Name>();
    let identifications = world.read_storage::<ItemIdentification>();
    
    if let Some(name) = names.get(entity) {
        if let Some(identification) = identifications.get(entity) {
            if identification.identified {
                Some(name.name.clone())
            } else {
                Some(identification.unidentified_name.clone())
            }
        } else {
            Some(name.name.clone())
        }
    } else {
        None
    }
}

/// Get the display description of an item, considering identification status
pub fn get_item_display_description(world: &World, entity: Entity) -> Option<String> {
    let properties = world.read_storage::<ItemProperties>();
    let identifications = world.read_storage::<ItemIdentification>();
    
    if let Some(props) = properties.get(entity) {
        if let Some(identification) = identifications.get(entity) {
            if identification.identified {
                Some(props.description.clone())
            } else {
                Some(identification.unidentified_description.clone())
            }
        } else {
            Some(props.description.clone())
        }
    } else {
        None
    }
}

/// Check if an item can be stacked with another item
pub fn can_stack_items(world: &World, item1: Entity, item2: Entity) -> bool {
    let properties = world.read_storage::<ItemProperties>();
    let stacks = world.read_storage::<ItemStack>();
    
    if let (Some(props1), Some(props2)) = (properties.get(item1), properties.get(item2)) {
        // Items must be the same type and have the same properties
        if props1.name == props2.name && 
           props1.item_type == props2.item_type &&
           props1.rarity == props2.rarity {
            
            // Both items must have stack components
            if let (Some(stack1), Some(stack2)) = (stacks.get(item1), stacks.get(item2)) {
                return !stack1.is_full() && !stack2.is_full();
            }
        }
    }
    
    false
}

/// Get the total weight of all items at a position
pub fn get_total_weight_at_position(world: &World, x: i32, y: i32) -> f32 {
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    let properties = world.read_storage::<ItemProperties>();
    let stacks = world.read_storage::<ItemStack>();
    
    let mut total_weight = 0.0;
    
    for (_entity, _item, position, props) in (&entities, &items, &positions, &properties).join() {
        if position.x == x && position.y == y {
            let quantity = if let Some(stack) = stacks.get(_entity) {
                stack.quantity as f32
            } else {
                1.0
            };
            total_weight += props.weight * quantity;
        }
    }
    
    total_weight
}

/// Find all items at a specific position
pub fn find_items_at_position(world: &World, x: i32, y: i32) -> Vec<Entity> {
    let entities = world.entities();
    let items = world.read_storage::<Item>();
    let positions = world.read_storage::<Position>();
    
    let mut found_items = Vec::new();
    
    for (entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == x && position.y == y {
            found_items.push(entity);
        }
    }
    
    found_items
}

/// Get the value of an item considering its condition and rarity
pub fn get_item_current_value(world: &World, entity: Entity) -> i32 {
    let properties = world.read_storage::<ItemProperties>();
    
    if let Some(props) = properties.get(entity) {
        let mut value = props.value;
        
        // Adjust for durability
        if let Some(durability) = &props.durability {
            let condition_multiplier = durability.percentage();
            value = (value as f32 * condition_multiplier) as i32;
        }
        
        // Magical items might have additional value
        let magical_items = world.read_storage::<MagicalItem>();
        if let Some(magical) = magical_items.get(entity) {
            let enchantment_bonus = magical.total_enchantment_power() * 10;
            value += enchantment_bonus;
            
            // Cursed items are worth less
            if magical.is_cursed() {
                value = (value as f32 * 0.5) as i32;
            }
        }
        
        value.max(1) // Minimum value of 1
    } else {
        0
    }
}

/// Check if an item meets the requirements for a character
pub fn meets_requirements(
    world: &World,
    item_entity: Entity,
    character_entity: Entity,
) -> bool {
    let properties = world.read_storage::<ItemProperties>();
    
    if let Some(props) = properties.get(item_entity) {
        // For now, just return true - requirements checking would need
        // character attributes and skills to be implemented
        // TODO: Implement actual requirements checking
        true
    } else {
        false
    }
}

/// Get a formatted string describing an item's properties
pub fn get_item_info_string(world: &World, entity: Entity) -> String {
    let mut info = String::new();
    
    if let Some(name) = get_item_display_name(world, entity) {
        info.push_str(&format!("Name: {}\n", name));
    }
    
    if let Some(description) = get_item_display_description(world, entity) {
        info.push_str(&format!("Description: {}\n", description));
    }
    
    let properties = world.read_storage::<ItemProperties>();
    if let Some(props) = properties.get(entity) {
        info.push_str(&format!("Type: {:?}\n", props.item_type));
        info.push_str(&format!("Rarity: {}\n", props.rarity.name()));
        info.push_str(&format!("Value: {} gold\n", get_item_current_value(world, entity)));
        info.push_str(&format!("Weight: {:.1} lbs\n", props.weight));
        
        if let Some(durability) = &props.durability {
            info.push_str(&format!("Condition: {} ({}/{})\n", 
                durability.condition_name(), 
                durability.current, 
                durability.max));
        }
        
        if !props.tags.is_empty() {
            info.push_str("Tags: ");
            for (i, tag) in props.tags.iter().enumerate() {
                if i > 0 { info.push_str(", "); }
                info.push_str(&format!("{:?}", tag));
            }
            info.push('\n');
        }
    }
    
    // Add magical properties if present
    let magical_items = world.read_storage::<MagicalItem>();
    if let Some(magical) = magical_items.get(entity) {
        info.push_str("\nMagical Properties:\n");
        for enchantment in &magical.enchantments {
            info.push_str(&format!("  {}: {}\n", enchantment.name, enchantment.description));
        }
        
        if let Some(curse) = &magical.curse {
            info.push_str(&format!("Cursed: {} - {}\n", curse.name, curse.description));
        }
    }
    
    // Add bonus information
    let bonuses = world.read_storage::<ItemBonuses>();
    if let Some(bonus) = bonuses.get(entity) {
        if bonus.combat_bonuses.attack_bonus != 0 ||
           bonus.combat_bonuses.damage_bonus != 0 ||
           bonus.combat_bonuses.defense_bonus != 0 {
            info.push_str("\nCombat Bonuses:\n");
            if bonus.combat_bonuses.attack_bonus != 0 {
                info.push_str(&format!("  Attack: +{}\n", bonus.combat_bonuses.attack_bonus));
            }
            if bonus.combat_bonuses.damage_bonus != 0 {
                info.push_str(&format!("  Damage: +{}\n", bonus.combat_bonuses.damage_bonus));
            }
            if bonus.combat_bonuses.defense_bonus != 0 {
                info.push_str(&format!("  Defense: +{}\n", bonus.combat_bonuses.defense_bonus));
            }
        }
        
        if !bonus.attribute_bonuses.is_empty() {
            info.push_str("\nAttribute Bonuses:\n");
            for (attr, value) in &bonus.attribute_bonuses {
                info.push_str(&format!("  {}: +{}\n", attr, value));
            }
        }
        
        if !bonus.skill_bonuses.is_empty() {
            info.push_str("\nSkill Bonuses:\n");
            for (skill, value) in &bonus.skill_bonuses {
                info.push_str(&format!("  {}: +{}\n", skill, value));
            }
        }
    }
    
    info
}