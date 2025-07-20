use specs::{Component, VecStorage, System, WriteStorage, ReadStorage, Entities, Entity, Join, Write};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::components::{CombatStats, Player, Name};
use crate::items::{ItemProperties, ItemType, ArmorType, WeaponType, ItemBonuses};
use crate::resources::GameLog;

/// Equipment slots available for characters
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    MainHand, OffHand, Head, Chest, Legs, Feet, Hands,
    Ring1, Ring2, Amulet, Cloak, Belt,
}

impl EquipmentSlot {
    pub fn name(&self) -> &'static str {
        match self {
            EquipmentSlot::MainHand => "Main Hand",
            EquipmentSlot::OffHand => "Off Hand",
            EquipmentSlot::Head => "Head",
            EquipmentSlot::Chest => "Chest",
            EquipmentSlot::Legs => "Legs",
            EquipmentSlot::Feet => "Feet",
            EquipmentSlot::Hands => "Hands",
            EquipmentSlot::Ring1 => "Ring 1",
            EquipmentSlot::Ring2 => "Ring 2",
            EquipmentSlot::Amulet => "Amulet",
            EquipmentSlot::Cloak => "Cloak",
            EquipmentSlot::Belt => "Belt",
        }
    }

    pub fn can_equip_item_type(&self, item_type: &ItemType) -> bool {
        match (self, item_type) {
            (EquipmentSlot::MainHand, ItemType::Weapon(_)) => true,
            (EquipmentSlot::OffHand, ItemType::Weapon(WeaponType::Dagger)) => true,
            (EquipmentSlot::OffHand, ItemType::Armor(ArmorType::Shield)) => true,
            (EquipmentSlot::Head, ItemType::Armor(ArmorType::Helmet)) => true,
            (EquipmentSlot::Chest, ItemType::Armor(ArmorType::Chest)) => true,
            (EquipmentSlot::Legs, ItemType::Armor(ArmorType::Legs)) => true,
            (EquipmentSlot::Feet, ItemType::Armor(ArmorType::Boots)) => true,
            (EquipmentSlot::Hands, ItemType::Armor(ArmorType::Gloves)) => true,
            (EquipmentSlot::Ring1 | EquipmentSlot::Ring2, ItemType::Armor(ArmorType::Ring)) => true,
            (EquipmentSlot::Amulet, ItemType::Armor(ArmorType::Amulet)) => true,
            (EquipmentSlot::Cloak, ItemType::Armor(ArmorType::Cloak)) => true,
            _ => false,
        }
    }
}

/// Component for managing equipped items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Equipment {
    pub slots: HashMap<EquipmentSlot, Option<Entity>>,
    pub stat_cache: EquipmentStats,
    pub dirty: bool,
}

impl Equipment {
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        
        for slot in [
            EquipmentSlot::MainHand, EquipmentSlot::OffHand,
            EquipmentSlot::Head, EquipmentSlot::Chest, EquipmentSlot::Legs,
            EquipmentSlot::Feet, EquipmentSlot::Hands,
            EquipmentSlot::Ring1, EquipmentSlot::Ring2,
            EquipmentSlot::Amulet, EquipmentSlot::Cloak, EquipmentSlot::Belt,
        ] {
            slots.insert(slot, None);
        }

        Equipment {
            slots,
            stat_cache: EquipmentStats::default(),
            dirty: true,
        }
    }

    pub fn equip_item(&mut self, slot: EquipmentSlot, item: Entity) -> Option<Entity> {
        let old_item = self.slots.insert(slot, Some(item));
        self.dirty = true;
        old_item.flatten()
    }

    pub fn unequip_item(&mut self, slot: &EquipmentSlot) -> Option<Entity> {
        let item = self.slots.insert(slot.clone(), None);
        self.dirty = true;
        item.flatten()
    }

    pub fn get_equipped(&self, slot: &EquipmentSlot) -> Option<Entity> {
        self.slots.get(slot).and_then(|&item| item)
    }

    pub fn is_slot_empty(&self, slot: &EquipmentSlot) -> bool {
        self.slots.get(slot).map_or(true, |item| item.is_none())
    }

    pub fn get_all_equipped(&self) -> Vec<Entity> {
        self.slots.values().filter_map(|&item| item).collect()
    }

    pub fn find_item_slot(&self, item: Entity) -> Option<EquipmentSlot> {
        for (slot, &equipped_item) in &self.slots {
            if equipped_item == Some(item) {
                return Some(slot.clone());
            }
        }
        None
    }
}

/// Aggregated stats from all equipped items
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct EquipmentStats {
    pub attack_bonus: i32,
    pub damage_bonus: i32,
    pub defense_bonus: i32,
    pub critical_chance_bonus: i32,
    pub critical_damage_bonus: i32,
    pub strength_bonus: i32,
    pub dexterity_bonus: i32,
    pub constitution_bonus: i32,
    pub intelligence_bonus: i32,
    pub wisdom_bonus: i32,
    pub charisma_bonus: i32,
    pub fire_resistance: i32,
    pub cold_resistance: i32,
    pub lightning_resistance: i32,
    pub poison_resistance: i32,
    pub magic_resistance: i32,
    pub movement_speed_bonus: i32,
    pub health_bonus: i32,
    pub mana_bonus: i32,
    pub stamina_bonus: i32,
}

impl EquipmentStats {
    pub fn add(&mut self, other: &EquipmentStats) {
        self.attack_bonus += other.attack_bonus;
        self.damage_bonus += other.damage_bonus;
        self.defense_bonus += other.defense_bonus;
        self.critical_chance_bonus += other.critical_chance_bonus;
        self.critical_damage_bonus += other.critical_damage_bonus;
        self.strength_bonus += other.strength_bonus;
        self.dexterity_bonus += other.dexterity_bonus;
        self.constitution_bonus += other.constitution_bonus;
        self.intelligence_bonus += other.intelligence_bonus;
        self.wisdom_bonus += other.wisdom_bonus;
        self.charisma_bonus += other.charisma_bonus;
        self.fire_resistance += other.fire_resistance;
        self.cold_resistance += other.cold_resistance;
        self.lightning_resistance += other.lightning_resistance;
        self.poison_resistance += other.poison_resistance;
        self.magic_resistance += other.magic_resistance;
        self.movement_speed_bonus += other.movement_speed_bonus;
        self.health_bonus += other.health_bonus;
        self.mana_bonus += other.mana_bonus;
        self.stamina_bonus += other.stamina_bonus;
    }
}

/// Intent component for equipping items
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToEquip {
    pub item: Entity,
    pub slot: Option<EquipmentSlot>,
}

/// Intent component for unequipping items
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToUnequip {
    pub slot: EquipmentSlot,
}

/// System for handling equipment changes
pub struct EquipmentSystem;

impl<'a> System<'a> for EquipmentSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Equipment>,
        WriteStorage<'a, WantsToEquip>,
        WriteStorage<'a, WantsToUnequip>,
        ReadStorage<'a, ItemProperties>,
        ReadStorage<'a, ItemBonuses>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut equipment,
            mut wants_to_equip,
            mut wants_to_unequip,
            item_properties,
            item_bonuses,
            names,
            players,
            mut gamelog,
        ) = data;

        // Handle equip requests
        let mut to_remove_equip = Vec::new();
        for (entity, equip_intent) in (&entities, &wants_to_equip).join() {
            if let Some(equipment) = equipment.get_mut(entity) {
                let item_entity = equip_intent.item;
                
                let slot = if let Some(slot) = &equip_intent.slot {
                    slot.clone()
                } else {
                    if let Some(props) = item_properties.get(item_entity) {
                        if let Some(detected_slot) = self.detect_equipment_slot(&props.item_type) {
                            detected_slot
                        } else {
                            gamelog.entries.push("Cannot determine equipment slot for this item".to_string());
                            to_remove_equip.push(entity);
                            continue;
                        }
                    } else {
                        gamelog.entries.push("Item has no properties".to_string());
                        to_remove_equip.push(entity);
                        continue;
                    }
                };

                if let Some(props) = item_properties.get(item_entity) {
                    if !slot.can_equip_item_type(&props.item_type) {
                        let item_name = names.get(item_entity)
                            .map(|n| n.name.clone())
                            .unwrap_or("Unknown Item".to_string());
                        gamelog.entries.push(format!("Cannot equip {} in {}", item_name, slot.name()));
                        to_remove_equip.push(entity);
                        continue;
                    }
                }

                let old_item = equipment.equip_item(slot.clone(), item_entity);
                
                let item_name = names.get(item_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Unknown Item".to_string());

                if players.get(entity).is_some() {
                    gamelog.entries.push(format!("You equip the {} in your {}", item_name, slot.name()));
                } else {
                    let entity_name = names.get(entity)
                        .map(|n| n.name.clone())
                        .unwrap_or("Someone".to_string());
                    gamelog.entries.push(format!("{} equips {}", entity_name, item_name));
                }

                if let Some(old_item_entity) = old_item {
                    let old_item_name = names.get(old_item_entity)
                        .map(|n| n.name.clone())
                        .unwrap_or("Unknown Item".to_string());
                    gamelog.entries.push(format!("Unequipped {}", old_item_name));
                }
            }

            to_remove_equip.push(entity);
        }

        // Handle unequip requests
        let mut to_remove_unequip = Vec::new();
        for (entity, unequip_intent) in (&entities, &wants_to_unequip).join() {
            if let Some(equipment) = equipment.get_mut(entity) {
                let slot = &unequip_intent.slot;
                
                if let Some(item_entity) = equipment.unequip_item(slot) {
                    let item_name = names.get(item_entity)
                        .map(|n| n.name.clone())
                        .unwrap_or("Unknown Item".to_string());

                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("You unequip the {} from your {}", item_name, slot.name()));
                    } else {
                        let entity_name = names.get(entity)
                            .map(|n| n.name.clone())
                            .unwrap_or("Someone".to_string());
                        gamelog.entries.push(format!("{} unequips {}", entity_name, item_name));
                    }
                } else {
                    gamelog.entries.push(format!("Nothing equipped in {}", slot.name()));
                }
            }

            to_remove_unequip.push(entity);
        }

        for entity in to_remove_equip {
            wants_to_equip.remove(entity);
        }
        for entity in to_remove_unequip {
            wants_to_unequip.remove(entity);
        }
    }
}

impl EquipmentSystem {
    fn detect_equipment_slot(&self, item_type: &ItemType) -> Option<EquipmentSlot> {
        match item_type {
            ItemType::Weapon(_) => Some(EquipmentSlot::MainHand),
            ItemType::Armor(armor_type) => match armor_type {
                ArmorType::Helmet => Some(EquipmentSlot::Head),
                ArmorType::Chest => Some(EquipmentSlot::Chest),
                ArmorType::Legs => Some(EquipmentSlot::Legs),
                ArmorType::Boots => Some(EquipmentSlot::Feet),
                ArmorType::Gloves => Some(EquipmentSlot::Hands),
                ArmorType::Shield => Some(EquipmentSlot::OffHand),
                ArmorType::Ring => Some(EquipmentSlot::Ring1),
                ArmorType::Amulet => Some(EquipmentSlot::Amulet),
                ArmorType::Cloak => Some(EquipmentSlot::Cloak),
            },
            _ => None,
        }
    }
}

/// System for calculating equipment bonuses
pub struct EquipmentBonusSystem;

impl<'a> System<'a> for EquipmentBonusSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Equipment>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, ItemBonuses>,
        ReadStorage<'a, ItemProperties>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut equipment, mut combat_stats, item_bonuses, item_properties) = data;

        for (entity, equipment, combat_stats) in (&entities, &mut equipment, &mut combat_stats).join() {
            if equipment.dirty {
                equipment.stat_cache = self.calculate_equipment_stats(equipment, &item_bonuses, &item_properties);
                equipment.dirty = false;

                combat_stats.power = 10 + equipment.stat_cache.attack_bonus + equipment.stat_cache.damage_bonus;
                combat_stats.defense = 5 + equipment.stat_cache.defense_bonus;
                combat_stats.max_hp = 100 + equipment.stat_cache.health_bonus + (equipment.stat_cache.constitution_bonus * 5);
                combat_stats.hp = combat_stats.hp.min(combat_stats.max_hp);
            }
        }
    }
}

impl EquipmentBonusSystem {
    fn calculate_equipment_stats(
        &self,
        equipment: &Equipment,
        item_bonuses: &ReadStorage<ItemBonuses>,
        item_properties: &ReadStorage<ItemProperties>,
    ) -> EquipmentStats {
        let mut total_stats = EquipmentStats::default();

        for &item_entity in equipment.slots.values().flatten() {
            if let Some(bonuses) = item_bonuses.get(item_entity) {
                let mut item_stats = EquipmentStats::default();
                
                item_stats.attack_bonus = bonuses.combat_bonuses.attack_bonus;
                item_stats.damage_bonus = bonuses.combat_bonuses.damage_bonus;
                item_stats.defense_bonus = bonuses.combat_bonuses.defense_bonus;
                item_stats.critical_chance_bonus = bonuses.combat_bonuses.critical_chance_bonus;
                item_stats.critical_damage_bonus = bonuses.combat_bonuses.critical_damage_bonus;

                for (attr, value) in &bonuses.attribute_bonuses {
                    match attr.as_str() {
                        "Strength" => item_stats.strength_bonus += value,
                        "Dexterity" => item_stats.dexterity_bonus += value,
                        "Constitution" => item_stats.constitution_bonus += value,
                        "Intelligence" => item_stats.intelligence_bonus += value,
                        "Wisdom" => item_stats.wisdom_bonus += value,
                        "Charisma" => item_stats.charisma_bonus += value,
                        _ => {}
                    }
                }

                total_stats.add(&item_stats);
            }
        }

        total_stats
    }
}