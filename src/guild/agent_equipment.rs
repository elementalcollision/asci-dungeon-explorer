use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::guild::guild_core::{GuildMember, Guild};
use crate::guild::agent_progression::AgentStats;
use crate::items::{Item, ItemType, ArmorType, WeaponType};
use crate::components::Name;

/// Agent equipment preferences
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentEquipmentPreferences {
    pub preferred_weapon_types: HashSet<String>,
    pub preferred_armor_types: HashSet<String>,
    pub preferred_item_stats: HashSet<String>,
    pub item_value_threshold: u32,
    pub auto_equip_better_items: bool,
    pub keep_consumables: bool,
    pub prioritize_set_items: bool,
    pub equipment_style: EquipmentStyle,
    pub item_quality_threshold: ItemQuality,
}

/// Equipment style preferences
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipmentStyle {
    Balanced,    // Equal focus on offense and defense
    Offensive,   // Focus on damage output
    Defensive,   // Focus on survivability
    Utility,     // Focus on special abilities and effects
    Specialized, // Focus on specific damage types or resistances
}

impl Default for EquipmentStyle {
    fn default() -> Self {
        EquipmentStyle::Balanced
    }
}

/// Item quality thresholds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemQuality {
    Any,        // Use any quality
    Common,     // At least common quality
    Uncommon,   // At least uncommon quality
    Rare,       // At least rare quality
    Epic,       // At least epic quality
    Legendary,  // Only legendary quality
}

impl Default for ItemQuality {
    fn default() -> Self {
        ItemQuality::Common
    }
}

impl Default for AgentEquipmentPreferences {
    fn default() -> Self {
        let mut preferred_weapon_types = HashSet::new();
        preferred_weapon_types.insert("Sword".to_string());
        
        let mut preferred_armor_types = HashSet::new();
        preferred_armor_types.insert("Medium".to_string());
        
        let mut preferred_item_stats = HashSet::new();
        preferred_item_stats.insert("Strength".to_string());
        preferred_item_stats.insert("Constitution".to_string());
        
        AgentEquipmentPreferences {
            preferred_weapon_types,
            preferred_armor_types,
            preferred_item_stats,
            item_value_threshold: 10,
            auto_equip_better_items: true,
            keep_consumables: true,
            prioritize_set_items: false,
            equipment_style: EquipmentStyle::Balanced,
            item_quality_threshold: ItemQuality::Common,
        }
    }
}

impl AgentEquipmentPreferences {
    /// Create preferences based on agent role
    pub fn new(role: &str) -> Self {
        let mut preferences = AgentEquipmentPreferences::default();
        
        match role {
            "Fighter" => {
                preferences.preferred_weapon_types = ["Sword", "Axe", "Mace"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_armor_types = ["Heavy", "Medium"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_item_stats = ["Strength", "Constitution"].iter().map(|s| s.to_string()).collect();
                preferences.equipment_style = EquipmentStyle::Balanced;
            },
            "Rogue" => {
                preferences.preferred_weapon_types = ["Dagger", "Shortsword", "Bow"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_armor_types = ["Light"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_item_stats = ["Dexterity", "Agility"].iter().map(|s| s.to_string()).collect();
                preferences.equipment_style = EquipmentStyle::Offensive;
            },
            "Mage" => {
                preferences.preferred_weapon_types = ["Staff", "Wand"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_armor_types = ["Cloth", "Light"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_item_stats = ["Intelligence", "Wisdom"].iter().map(|s| s.to_string()).collect();
                preferences.equipment_style = EquipmentStyle::Utility;
            },
            "Ranger" => {
                preferences.preferred_weapon_types = ["Bow", "Spear", "Dagger"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_armor_types = ["Medium", "Light"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_item_stats = ["Dexterity", "Wisdom"].iter().map(|s| s.to_string()).collect();
                preferences.equipment_style = EquipmentStyle::Specialized;
            },
            "Cleric" => {
                preferences.preferred_weapon_types = ["Mace", "Staff", "Hammer"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_armor_types = ["Medium", "Heavy"].iter().map(|s| s.to_string()).collect();
                preferences.preferred_item_stats = ["Wisdom", "Constitution"].iter().map(|s| s.to_string()).collect();
                preferences.equipment_style = EquipmentStyle::Defensive;
            },
            _ => {}
        }
        
        preferences
    }
    
    /// Check if an item meets preferences
    pub fn meets_preferences(&self, item: &Item) -> bool {
        // Check quality threshold
        if !self.meets_quality_threshold(item) {
            return false;
        }
        
        // Check value threshold
        if item.value < self.item_value_threshold {
            return false;
        }
        
        // Check weapon type preferences
        if let ItemType::Weapon(weapon_type) = &item.item_type {
            if !self.preferred_weapon_types.is_empty() && 
               !self.preferred_weapon_types.contains(&weapon_type.to_string()) {
                return false;
            }
        }
        
        // Check armor type preferences
        if let ItemType::Armor(armor_type) = &item.item_type {
            if !self.preferred_armor_types.is_empty() && 
               !self.preferred_armor_types.contains(&armor_type.to_string()) {
                return false;
            }
        }
        
        // Check stat preferences
        if !self.preferred_item_stats.is_empty() {
            let mut has_preferred_stat = false;
            
            for stat in &item.stats {
                if self.preferred_item_stats.contains(&stat.name) {
                    has_preferred_stat = true;
                    break;
                }
            }
            
            if !has_preferred_stat {
                return false;
            }
        }
        
        true
    }
    
    /// Check if item meets quality threshold
    fn meets_quality_threshold(&self, item: &Item) -> bool {
        match self.item_quality_threshold {
            ItemQuality::Any => true,
            ItemQuality::Common => true, // All items are at least common
            ItemQuality::Uncommon => item.rarity >= 1,
            ItemQuality::Rare => item.rarity >= 2,
            ItemQuality::Epic => item.rarity >= 3,
            ItemQuality::Legendary => item.rarity >= 4,
        }
    }
    
    /// Compare two items and determine if the new one is better
    pub fn is_better_item(&self, new_item: &Item, current_item: &Item) -> bool {
        // If qualities differ significantly, use the higher quality one
        if new_item.rarity > current_item.rarity + 1 {
            return true;
        }
        
        if current_item.rarity > new_item.rarity + 1 {
            return false;
        }
        
        // Compare based on equipment style
        match self.equipment_style {
            EquipmentStyle::Offensive => {
                // Compare damage or offensive stats
                let new_offensive_value = self.calculate_offensive_value(new_item);
                let current_offensive_value = self.calculate_offensive_value(current_item);
                new_offensive_value > current_offensive_value
            },
            EquipmentStyle::Defensive => {
                // Compare armor or defensive stats
                let new_defensive_value = self.calculate_defensive_value(new_item);
                let current_defensive_value = self.calculate_defensive_value(current_item);
                new_defensive_value > current_defensive_value
            },
            EquipmentStyle::Utility => {
                // Compare special effects or utility
                let new_utility_value = self.calculate_utility_value(new_item);
                let current_utility_value = self.calculate_utility_value(current_item);
                new_utility_value > current_utility_value
            },
            EquipmentStyle::Specialized => {
                // Compare specialized stats
                let new_specialized_value = self.calculate_specialized_value(new_item);
                let current_specialized_value = self.calculate_specialized_value(current_item);
                new_specialized_value > current_specialized_value
            },
            EquipmentStyle::Balanced => {
                // Compare overall value
                let new_total = self.calculate_total_value(new_item);
                let current_total = self.calculate_total_value(current_item);
                new_total > current_total
            },
        }
    }
    
    /// Calculate offensive value of an item
    fn calculate_offensive_value(&self, item: &Item) -> f32 {
        let mut value = 0.0;
        
        // Base value from item type
        match &item.item_type {
            ItemType::Weapon(_) => {
                value += item.damage as f32 * 2.0;
            },
            _ => {}
        }
        
        // Add value from stats
        for stat in &item.stats {
            match stat.name.as_str() {
                "Strength" | "Dexterity" | "Intelligence" => {
                    value += stat.value as f32;
                },
                "CriticalChance" | "CriticalDamage" => {
                    value += stat.value as f32 * 1.5;
                },
                "DamageBonus" | "AttackSpeed" => {
                    value += stat.value as f32 * 2.0;
                },
                _ => {}
            }
        }
        
        // Rarity bonus
        value *= 1.0 + (item.rarity as f32 * 0.2);
        
        value
    }
    
    /// Calculate defensive value of an item
    fn calculate_defensive_value(&self, item: &Item) -> f32 {
        let mut value = 0.0;
        
        // Base value from item type
        match &item.item_type {
            ItemType::Armor(_) => {
                value += item.defense as f32 * 2.0;
            },
            _ => {}
        }
        
        // Add value from stats
        for stat in &item.stats {
            match stat.name.as_str() {
                "Constitution" | "Armor" | "Defense" => {
                    value += stat.value as f32;
                },
                "HealthBonus" | "HealthRegen" => {
                    value += stat.value as f32 * 0.5;
                },
                "DamageReduction" | "BlockChance" => {
                    value += stat.value as f32 * 2.0;
                },
                _ => {}
            }
        }
        
        // Rarity bonus
        value *= 1.0 + (item.rarity as f32 * 0.2);
        
        value
    }
    
    /// Calculate utility value of an item
    fn calculate_utility_value(&self, item: &Item) -> f32 {
        let mut value = 0.0;
        
        // Add value from stats
        for stat in &item.stats {
            match stat.name.as_str() {
                "Intelligence" | "Wisdom" | "Charisma" => {
                    value += stat.value as f32;
                },
                "ManaBonus" | "ManaRegen" => {
                    value += stat.value as f32 * 1.5;
                },
                "CooldownReduction" | "EffectDuration" => {
                    value += stat.value as f32 * 2.0;
                },
                _ => {}
            }
        }
        
        // Special effects add significant value
        value += item.special_effects.len() as f32 * 5.0;
        
        // Rarity bonus
        value *= 1.0 + (item.rarity as f32 * 0.2);
        
        value
    }
    
    /// Calculate specialized value of an item
    fn calculate_specialized_value(&self, item: &Item) -> f32 {
        let mut value = 0.0;
        
        // Check for preferred stats
        for stat in &item.stats {
            if self.preferred_item_stats.contains(&stat.name) {
                value += stat.value as f32 * 2.0;
            } else {
                value += stat.value as f32 * 0.5;
            }
        }
        
        // Check for set items
        if self.prioritize_set_items && !item.set_name.is_empty() {
            value *= 1.5;
        }
        
        // Rarity bonus
        value *= 1.0 + (item.rarity as f32 * 0.2);
        
        value
    }
    
    /// Calculate total value of an item
    fn calculate_total_value(&self, item: &Item) -> f32 {
        let offensive = self.calculate_offensive_value(item);
        let defensive = self.calculate_defensive_value(item);
        let utility = self.calculate_utility_value(item);
        
        offensive + defensive + utility
    }
}

/// Agent equipment manager component
#[derive(Component, Debug)]
pub struct AgentEquipmentManager {
    pub preferences: AgentEquipmentPreferences,
    pub equipped_items: HashMap<String, Entity>,
    pub inventory_items: Vec<Entity>,
    pub pending_equip: Option<Entity>,
    pub pending_unequip: Option<String>,
    pub equipment_score: f32,
    pub last_evaluation_time: f64,
}

impl Default for AgentEquipmentManager {
    fn default() -> Self {
        AgentEquipmentManager {
            preferences: AgentEquipmentPreferences::default(),
            equipped_items: HashMap::new(),
            inventory_items: Vec::new(),
            pending_equip: None,
            pending_unequip: None,
            equipment_score: 0.0,
            last_evaluation_time: 0.0,
        }
    }
}

impl AgentEquipmentManager {
    /// Create a new equipment manager with role-based preferences
    pub fn new(role: &str) -> Self {
        AgentEquipmentManager {
            preferences: AgentEquipmentPreferences::new(role),
            ..Default::default()
        }
    }
    
    /// Evaluate an item for potential use
    pub fn evaluate_item(&self, item: &Item) -> bool {
        self.preferences.meets_preferences(item)
    }
    
    /// Check if an item is better than currently equipped item
    pub fn is_better_than_equipped(&self, item: &Item, item_query: &Query<&Item>) -> Option<String> {
        let slot = self.get_slot_for_item(item);
        
        if let Some(slot_name) = slot {
            if let Some(equipped_entity) = self.equipped_items.get(&slot_name) {
                if let Ok(equipped_item) = item_query.get(*equipped_entity) {
                    if self.preferences.is_better_item(item, equipped_item) {
                        return Some(slot_name);
                    }
                }
            } else {
                // No item equipped in this slot
                return Some(slot_name);
            }
        }
        
        None
    }
    
    /// Get appropriate slot for an item
    fn get_slot_for_item(&self, item: &Item) -> Option<String> {
        match &item.item_type {
            ItemType::Weapon(_) => Some("Weapon".to_string()),
            ItemType::Armor(armor_type) => {
                match armor_type {
                    ArmorType::Helmet => Some("Head".to_string()),
                    ArmorType::Chest => Some("Chest".to_string()),
                    ArmorType::Legs => Some("Legs".to_string()),
                    ArmorType::Boots => Some("Feet".to_string()),
                    ArmorType::Gloves => Some("Hands".to_string()),
                    ArmorType::Shield => Some("OffHand".to_string()),
                    ArmorType::Ring => {
                        if self.equipped_items.contains_key("Ring1") {
                            Some("Ring2".to_string())
                        } else {
                            Some("Ring1".to_string())
                        }
                    },
                    ArmorType::Amulet => Some("Amulet".to_string()),
                    ArmorType::Cloak => Some("Cloak".to_string()),
                }
            },
            ItemType::Consumable(_) => None, // Consumables aren't equipped
            ItemType::Quest => None,
            ItemType::Material => None,
            ItemType::Miscellaneous => None,
        }
    }
    
    /// Calculate overall equipment score
    pub fn calculate_equipment_score(&mut self, item_query: &Query<&Item>) -> f32 {
        let mut total_score = 0.0;
        
        for (_, item_entity) in &self.equipped_items {
            if let Ok(item) = item_query.get(*item_entity) {
                total_score += self.preferences.calculate_total_value(item);
            }
        }
        
        self.equipment_score = total_score;
        total_score
    }
    
    /// Add item to inventory
    pub fn add_item(&mut self, item_entity: Entity) {
        if !self.inventory_items.contains(&item_entity) {
            self.inventory_items.push(item_entity);
        }
    }
    
    /// Remove item from inventory
    pub fn remove_item(&mut self, item_entity: Entity) -> bool {
        if let Some(index) = self.inventory_items.iter().position(|&i| i == item_entity) {
            self.inventory_items.remove(index);
            return true;
        }
        false
    }
    
    /// Equip an item
    pub fn equip_item(&mut self, item_entity: Entity, slot: String) -> Option<Entity> {
        let old_item = self.equipped_items.insert(slot, item_entity);
        
        // Remove from inventory if it was there
        self.remove_item(item_entity);
        
        // If we unequipped something, add it to inventory
        if let Some(old_item_entity) = old_item {
            self.add_item(old_item_entity);
        }
        
        old_item
    }
    
    /// Unequip an item
    pub fn unequip_item(&mut self, slot: &str) -> Option<Entity> {
        if let Some(item_entity) = self.equipped_items.remove(slot) {
            self.add_item(item_entity);
            Some(item_entity)
        } else {
            None
        }
    }
    
    /// Get best item for a slot from inventory
    pub fn get_best_item_for_slot(&self, slot: &str, item_query: &Query<&Item>) -> Option<Entity> {
        let mut best_item = None;
        let mut best_value = 0.0;
        
        for &item_entity in &self.inventory_items {
            if let Ok(item) = item_query.get(item_entity) {
                if let Some(item_slot) = self.get_slot_for_item(item) {
                    if item_slot == slot {
                        let value = self.preferences.calculate_total_value(item);
                        if best_item.is_none() || value > best_value {
                            best_item = Some(item_entity);
                            best_value = value;
                        }
                    }
                }
            }
        }
        
        best_item
    }
}

/// System for initializing agent equipment
pub fn agent_equipment_init_system(
    mut commands: Commands,
    query: Query<(Entity, &GuildMember), (Without<AgentEquipmentManager>, With<AgentStats>)>,
) {
    for (entity, guild_member) in query.iter() {
        let equipment_manager = AgentEquipmentManager::new(&guild_member.specialization);
        commands.entity(entity).insert(equipment_manager);
    }
}

/// System for managing agent equipment
pub fn agent_equipment_management_system(
    time: Res<Time>,
    mut agent_query: Query<(Entity, &mut AgentEquipmentManager)>,
    item_query: Query<&Item>,
    name_query: Query<&Name>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_seconds_f64();
    
    for (entity, mut equipment_manager) in agent_query.iter_mut() {
        // Only evaluate equipment periodically
        if current_time - equipment_manager.last_evaluation_time < 5.0 {
            continue;
        }
        
        equipment_manager.last_evaluation_time = current_time;
        
        // Calculate current equipment score
        equipment_manager.calculate_equipment_score(&item_query);
        
        // Check inventory for better items
        if equipment_manager.preferences.auto_equip_better_items {
            for &item_entity in &equipment_manager.inventory_items {
                if let Ok(item) = item_query.get(item_entity) {
                    if let Some(slot) = equipment_manager.is_better_than_equipped(item, &item_query) {
                        // Found a better item, equip it
                        equipment_manager.pending_equip = Some(item_entity);
                        equipment_manager.pending_unequip = Some(slot);
                        break;
                    }
                }
            }
        }
        
        // Handle pending equipment changes
        if let Some(item_entity) = equipment_manager.pending_equip {
            if let Some(slot) = &equipment_manager.pending_unequip {
                equipment_manager.equip_item(item_entity, slot.clone());
                
                // Log the equipment change
                if let Ok(item) = item_query.get(item_entity) {
                    if let Ok(name) = name_query.get(entity) {
                        // In a real implementation, you would log this to a game log
                        println!("{} equipped {} in {}", name.name, item.name, slot);
                    }
                }
                
                equipment_manager.pending_equip = None;
                equipment_manager.pending_unequip = None;
            }
        }
    }
}

/// System for sharing equipment between guild and agents
pub fn guild_agent_equipment_sharing_system(
    mut agent_query: Query<(Entity, &mut AgentEquipmentManager, &GuildMember)>,
    mut guild_manager: ResMut<Guild>,
    item_query: Query<&Item>,
) {
    // Check guild storage for better items for agents
    for (entity, mut equipment_manager, guild_member) in agent_query.iter_mut() {
        if guild_member.guild_id != guild_manager.id {
            continue;
        }
        
        let mut items_to_remove = Vec::new();
        
        for (i, item) in guild_manager.storage.iter().enumerate() {
            if equipment_manager.evaluate_item(item) {
                if let Some(slot) = equipment_manager.is_better_than_equipped(item, &item_query) {
                    // Found a better item in guild storage
                    // In a real implementation, you would create an entity for this item
                    // For now, we'll just remove it from guild storage
                    items_to_remove.push(i);
                    break;
                }
            }
        }
        
        // Remove items from guild storage (in reverse order to maintain indices)
        for i in items_to_remove.iter().rev() {
            guild_manager.storage.remove(*i);
        }
    }
}

/// Plugin for agent equipment systems
pub struct AgentEquipmentPlugin;

impl Plugin for AgentEquipmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            agent_equipment_init_system,
            agent_equipment_management_system,
            guild_agent_equipment_sharing_system,
        ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::{ItemStat, ItemEffect};

    // Helper function to create a test item
    fn create_test_item(
        name: &str,
        item_type: ItemType,
        rarity: u32,
        value: u32,
        stats: Vec<(&str, i32)>,
    ) -> Item {
        let mut item = Item {
            name: name.to_string(),
            item_type,
            rarity,
            value,
            weight: 1.0,
            damage: 0,
            defense: 0,
            stats: Vec::new(),
            special_effects: Vec::new(),
            set_name: "".to_string(),
            description: "Test item".to_string(),
        };
        
        for (stat_name, stat_value) in stats {
            item.stats.push(ItemStat {
                name: stat_name.to_string(),
                value: stat_value,
            });
        }
        
        item
    }

    #[test]
    fn test_equipment_preferences() {
        let fighter_prefs = AgentEquipmentPreferences::new("Fighter");
        assert!(fighter_prefs.preferred_weapon_types.contains("Sword"));
        assert!(fighter_prefs.preferred_armor_types.contains("Heavy"));
        assert!(fighter_prefs.preferred_item_stats.contains("Strength"));
        
        let mage_prefs = AgentEquipmentPreferences::new("Mage");
        assert!(mage_prefs.preferred_weapon_types.contains("Staff"));
        assert!(mage_prefs.preferred_armor_types.contains("Cloth"));
        assert!(mage_prefs.preferred_item_stats.contains("Intelligence"));
    }

    #[test]
    fn test_item_evaluation() {
        let fighter_prefs = AgentEquipmentPreferences::new("Fighter");
        
        // Create a sword with strength bonus
        let good_sword = create_test_item(
            "Steel Sword",
            ItemType::Weapon(WeaponType::Sword),
            2, // Rare
            50,
            vec![("Strength", 5), ("DamageBonus", 3)],
        );
        
        // Create a staff with intelligence bonus
        let bad_sword = create_test_item(
            "Wooden Staff",
            ItemType::Weapon(WeaponType::Staff),
            1, // Uncommon
            30,
            vec![("Intelligence", 5), ("ManaBonus", 10)],
        );
        
        assert!(fighter_prefs.meets_preferences(&good_sword));
        assert!(!fighter_prefs.meets_preferences(&bad_sword));
    }

    #[test]
    fn test_item_comparison() {
        let fighter_prefs = AgentEquipmentPreferences::new("Fighter");
        
        // Create two swords to compare
        let sword1 = create_test_item(
            "Iron Sword",
            ItemType::Weapon(WeaponType::Sword),
            1, // Uncommon
            30,
            vec![("Strength", 2), ("DamageBonus", 1)],
        );
        
        let sword2 = create_test_item(
            "Steel Sword",
            ItemType::Weapon(WeaponType::Sword),
            2, // Rare
            50,
            vec![("Strength", 4), ("DamageBonus", 2)],
        );
        
        assert!(fighter_prefs.is_better_item(&sword2, &sword1));
        assert!(!fighter_prefs.is_better_item(&sword1, &sword2));
    }

    #[test]
    fn test_equipment_manager() {
        let mut manager = AgentEquipmentManager::new("Fighter");
        
        // Create a mock entity
        let item_entity = Entity::from_raw(1);
        
        // Add item to inventory
        manager.add_item(item_entity);
        assert!(manager.inventory_items.contains(&item_entity));
        
        // Equip item
        let old_item = manager.equip_item(item_entity, "Weapon".to_string());
        assert!(old_item.is_none());
        assert_eq!(manager.equipped_items.get("Weapon"), Some(&item_entity));
        assert!(!manager.inventory_items.contains(&item_entity));
        
        // Unequip item
        let unequipped = manager.unequip_item("Weapon");
        assert_eq!(unequipped, Some(item_entity));
        assert!(manager.inventory_items.contains(&item_entity));
        assert!(!manager.equipped_items.contains_key("Weapon"));
    }
}