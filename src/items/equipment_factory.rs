use specs::{World, WorldExt, Builder, Entity};
use crate::components::{Position, Name, Renderable, Item};
use crate::items::{
    ItemProperties, ItemType, WeaponType, ArmorType, ItemRarity, ItemBonuses,
    equipment_system::{Equippable, EquipmentSlot, EquipmentRequirements, EquipmentSet, SetBonus}
};
use crate::resources::RandomNumberGenerator;
use std::collections::HashMap;

/// Factory for creating different types of equipment
pub struct EquipmentFactory;

impl EquipmentFactory {
    pub fn new() -> Self {
        EquipmentFactory
    }

    /// Create a weapon
    pub fn create_weapon(
        &self,
        world: &mut World,
        position: Position,
        weapon_type: WeaponType,
        quality: EquipmentQuality,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        let (name, base_damage, base_value, weight, glyph, color) = match weapon_type {
            WeaponType::Sword => ("Sword", 8, 50, 3.0, '/', crossterm::style::Color::Grey),
            WeaponType::Axe => ("Axe", 10, 60, 4.0, 'P', crossterm::style::Color::DarkGrey),
            WeaponType::Mace => ("Mace", 9, 55, 5.0, 'T', crossterm::style::Color::Grey),
            WeaponType::Dagger => ("Dagger", 5, 30, 1.0, '-', crossterm::style::Color::Grey),
            WeaponType::Spear => ("Spear", 7, 45, 4.0, '|', crossterm::style::Color::DarkYellow),
            WeaponType::Bow => ("Bow", 6, 70, 2.0, ')', crossterm::style::Color::DarkYellow),
            WeaponType::Crossbow => ("Crossbow", 8, 90, 3.0, '}', crossterm::style::Color::DarkGrey),
            WeaponType::Staff => ("Staff", 5, 40, 3.0, '\\', crossterm::style::Color::DarkYellow),
            WeaponType::Wand => ("Wand", 3, 60, 0.5, '/', crossterm::style::Color::Magenta),
            WeaponType::Thrown => ("Throwing Knife", 4, 20, 0.5, '-', crossterm::style::Color::Grey),
        };

        // Apply quality modifiers
        let (quality_prefix, damage_mod, value_mod, rarity) = quality.get_modifiers();
        let damage = (base_damage as f32 * damage_mod) as i32;
        let value = (base_value as f32 * value_mod) as i32;
        let full_name = if quality_prefix.is_empty() {
            name.to_string()
        } else {
            format!("{} {}", quality_prefix, name)
        };

        // Create item properties
        let properties = ItemProperties::new(full_name.clone(), ItemType::Weapon(weapon_type))
            .with_description(format!("A {} weapon that deals {} damage.", quality_prefix.to_lowercase(), damage))
            .with_rarity(rarity)
            .with_value(value)
            .with_weight(weight)
            .with_durability(100);

        // Create item bonuses
        let mut bonuses = ItemBonuses::new();
        bonuses.combat_bonuses.attack_bonus = damage / 2;
        bonuses.combat_bonuses.damage_bonus = damage;

        // Add random attribute bonus for higher qualities
        if quality >= EquipmentQuality::Superior {
            let attributes = ["Strength", "Dexterity"];
            let attr = attributes[rng.roll_dice(1, attributes.len()) - 1];
            let bonus_value = rng.roll_dice(1, 3);
            bonuses.add_attribute_bonus(attr.to_string(), bonus_value);
        }

        // Create the entity
        let entity = world.create_entity()
            .with(Item)
            .with(Name { name: full_name })
            .with(properties)
            .with(bonuses)
            .with(Equippable::new(EquipmentSlot::MainHand))
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build();

        entity
    }

    /// Create armor
    pub fn create_armor(
        &self,
        world: &mut World,
        position: Position,
        armor_type: ArmorType,
        quality: EquipmentQuality,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        let (name, base_defense, base_value, weight, glyph, color, slot) = match armor_type {
            ArmorType::Helmet => ("Helmet", 3, 40, 2.0, '^', crossterm::style::Color::Grey, EquipmentSlot::Head),
            ArmorType::Chest => ("Chestplate", 8, 80, 10.0, '[', crossterm::style::Color::Grey, EquipmentSlot::Chest),
            ArmorType::Legs => ("Greaves", 5, 60, 6.0, '[', crossterm::style::Color::DarkGrey, EquipmentSlot::Legs),
            ArmorType::Boots => ("Boots", 2, 30, 2.0, '[', crossterm::style::Color::DarkGrey, EquipmentSlot::Feet),
            ArmorType::Gloves => ("Gloves", 1, 25, 1.0, '[', crossterm::style::Color::DarkGrey, EquipmentSlot::Hands),
            ArmorType::Shield => ("Shield", 4, 50, 4.0, ')', crossterm::style::Color::Grey, EquipmentSlot::OffHand),
            ArmorType::Cloak => ("Cloak", 2, 35, 1.0, '(', crossterm::style::Color::DarkGreen, EquipmentSlot::Back),
            ArmorType::Ring => ("Ring", 0, 100, 0.1, '=', crossterm::style::Color::Yellow, EquipmentSlot::Ring1),
            ArmorType::Amulet => ("Amulet", 0, 120, 0.2, '"', crossterm::style::Color::Yellow, EquipmentSlot::Neck),
        };

        // Apply quality modifiers
        let (quality_prefix, defense_mod, value_mod, rarity) = quality.get_modifiers();
        let defense = (base_defense as f32 * defense_mod) as i32;
        let value = (base_value as f32 * value_mod) as i32;
        let full_name = if quality_prefix.is_empty() {
            name.to_string()
        } else {
            format!("{} {}", quality_prefix, name)
        };

        // Create item properties
        let properties = ItemProperties::new(full_name.clone(), ItemType::Armor(armor_type))
            .with_description(format!("A {} piece of armor that provides {} defense.", quality_prefix.to_lowercase(), defense))
            .with_rarity(rarity)
            .with_value(value)
            .with_weight(weight)
            .with_durability(100);

        // Create item bonuses
        let mut bonuses = ItemBonuses::new();
        bonuses.combat_bonuses.defense_bonus = defense;

        // Add random attribute bonus for higher qualities
        if quality >= EquipmentQuality::Superior {
            let attributes = ["Constitution", "Dexterity"];
            let attr = attributes[rng.roll_dice(1, attributes.len()) - 1];
            let bonus_value = rng.roll_dice(1, 3);
            bonuses.add_attribute_bonus(attr.to_string(), bonus_value);
        }

        // Create the entity
        let entity = world.create_entity()
            .with(Item)
            .with(Name { name: full_name })
            .with(properties)
            .with(bonuses)
            .with(Equippable::new(slot))
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build();

        entity
    }

    /// Create a random piece of equipment
    pub fn create_random_equipment(
        &self,
        world: &mut World,
        position: Position,
        min_quality: EquipmentQuality,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        // Determine if weapon or armor
        let is_weapon = rng.roll_dice(1, 2) == 1;

        // Determine quality (ensure at least min_quality)
        let quality = self.random_quality(min_quality, rng);

        if is_weapon {
            // Random weapon type
            let weapon_types = [
                WeaponType::Sword,
                WeaponType::Axe,
                WeaponType::Mace,
                WeaponType::Dagger,
                WeaponType::Spear,
                WeaponType::Bow,
                WeaponType::Staff,
            ];
            let weapon_type = weapon_types[rng.roll_dice(1, weapon_types.len()) - 1];
            self.create_weapon(world, position, weapon_type, quality, rng)
        } else {
            // Random armor type
            let armor_types = [
                ArmorType::Helmet,
                ArmorType::Chest,
                ArmorType::Legs,
                ArmorType::Boots,
                ArmorType::Gloves,
                ArmorType::Shield,
                ArmorType::Cloak,
            ];
            let armor_type = armor_types[rng.roll_dice(1, armor_types.len()) - 1];
            self.create_armor(world, position, armor_type, quality, rng)
        }
    }

    /// Generate a random quality based on minimum quality
    fn random_quality(&self, min_quality: EquipmentQuality, rng: &mut RandomNumberGenerator) -> EquipmentQuality {
        let qualities = [
            EquipmentQuality::Poor,
            EquipmentQuality::Common,
            EquipmentQuality::Uncommon,
            EquipmentQuality::Rare,
            EquipmentQuality::Epic,
            EquipmentQuality::Legendary,
        ];

        let min_index = qualities.iter().position(|&q| q == min_quality).unwrap_or(0);
        let roll = rng.roll_dice(1, 100);

        let quality_index = if roll <= 50 {
            min_index
        } else if roll <= 75 {
            (min_index + 1).min(qualities.len() - 1)
        } else if roll <= 90 {
            (min_index + 2).min(qualities.len() - 1)
        } else if roll <= 97 {
            (min_index + 3).min(qualities.len() - 1)
        } else if roll <= 99 {
            (min_index + 4).min(qualities.len() - 1)
        } else {
            qualities.len() - 1 // Legendary
        };

        qualities[quality_index]
    }
}

/// Equipment quality levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EquipmentQuality {
    Poor,
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
    Superior, // Alias for Rare
}

impl EquipmentQuality {
    pub fn name(&self) -> &'static str {
        match self {
            EquipmentQuality::Poor => "Poor",
            EquipmentQuality::Common => "Common",
            EquipmentQuality::Uncommon => "Uncommon",
            EquipmentQuality::Rare | EquipmentQuality::Superior => "Superior",
            EquipmentQuality::Epic => "Epic",
            EquipmentQuality::Legendary => "Legendary",
        }
    }

    pub fn get_modifiers(&self) -> (&'static str, f32, f32, ItemRarity) {
        match self {
            EquipmentQuality::Poor => ("Crude", 0.7, 0.5, ItemRarity::Common),
            EquipmentQuality::Common => ("", 1.0, 1.0, ItemRarity::Common),
            EquipmentQuality::Uncommon => ("Fine", 1.2, 1.5, ItemRarity::Uncommon),
            EquipmentQuality::Rare | EquipmentQuality::Superior => ("Superior", 1.5, 2.5, ItemRarity::Rare),
            EquipmentQuality::Epic => ("Masterwork", 2.0, 5.0, ItemRarity::Epic),
            EquipmentQuality::Legendary => ("Legendary", 3.0, 10.0, ItemRarity::Legendary),
        }
    }
}

impl ArmorType {
    pub fn to_string(&self) -> &'static str {
        match self {
            ArmorType::Helmet => "Helmet",
            ArmorType::Chest => "Chestplate",
            ArmorType::Legs => "Greaves",
            ArmorType::Boots => "Boots",
            ArmorType::Gloves => "Gloves",
            ArmorType::Shield => "Shield",
            ArmorType::Cloak => "Cloak",
            ArmorType::Ring => "Ring",
            ArmorType::Amulet => "Amulet",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};
    use crate::components::{Position, Name, Renderable, Item};
    use crate::resources::RandomNumberGenerator;

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Item>();
        world.register::<Name>();
        world.register::<ItemProperties>();
        world.register::<ItemBonuses>();
        world.register::<Equippable>();
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<EquipmentSet>();
        world
    }

    #[test]
    fn test_weapon_creation() {
        let mut world = setup_world();
        let mut rng = RandomNumberGenerator::new();
        let factory = EquipmentFactory::new();

        let position = Position { x: 0, y: 0 };
        let sword = factory.create_weapon(
            &mut world,
            position,
            WeaponType::Sword,
            EquipmentQuality::Common,
            &mut rng,
        );

        // Verify the weapon was created correctly
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        let equippables = world.read_storage::<Equippable>();

        if let Some(name) = names.get(sword) {
            assert!(name.name.contains("Sword"));
        } else {
            panic!("Weapon should have a name");
        }

        if let Some(props) = properties.get(sword) {
            assert!(matches!(props.item_type, ItemType::Weapon(WeaponType::Sword)));
        } else {
            panic!("Weapon should have properties");
        }

        if let Some(equippable) = equippables.get(sword) {
            assert_eq!(equippable.slot, EquipmentSlot::MainHand);
        } else {
            panic!("Weapon should be equippable");
        }
    }

    #[test]
    fn test_armor_creation() {
        let mut world = setup_world();
        let mut rng = RandomNumberGenerator::new();
        let factory = EquipmentFactory::new();

        let position = Position { x: 0, y: 0 };
        let helmet = factory.create_armor(
            &mut world,
            position,
            ArmorType::Helmet,
            EquipmentQuality::Rare,
            &mut rng,
        );

        // Verify the armor was created correctly
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        let equippables = world.read_storage::<Equippable>();

        if let Some(name) = names.get(helmet) {
            assert!(name.name.contains("Helmet"));
            assert!(name.name.contains("Superior")); // Rare quality prefix
        } else {
            panic!("Armor should have a name");
        }

        if let Some(props) = properties.get(helmet) {
            assert!(matches!(props.item_type, ItemType::Armor(ArmorType::Helmet)));
            assert_eq!(props.rarity, ItemRarity::Rare);
        } else {
            panic!("Armor should have properties");
        }

        if let Some(equippable) = equippables.get(helmet) {
            assert_eq!(equippable.slot, EquipmentSlot::Head);
        } else {
            panic!("Armor should be equippable");
        }
    }

    #[test]
    fn test_random_equipment() {
        let mut world = setup_world();
        let mut rng = RandomNumberGenerator::new();
        let factory = EquipmentFactory::new();

        let position = Position { x: 0, y: 0 };
        
        // Create several random pieces of equipment
        for _ in 0..10 {
            let equipment = factory.create_random_equipment(
                &mut world,
                position,
                EquipmentQuality::Common,
                &mut rng,
            );

            // Verify basic components exist
            let names = world.read_storage::<Name>();
            let properties = world.read_storage::<ItemProperties>();
            let equippables = world.read_storage::<Equippable>();

            assert!(names.get(equipment).is_some());
            assert!(properties.get(equipment).is_some());
            assert!(equippables.get(equipment).is_some());
        }
    }

    #[test]
    fn test_equipment_quality_modifiers() {
        let (prefix, damage_mod, value_mod, rarity) = EquipmentQuality::Legendary.get_modifiers();
        
        assert_eq!(prefix, "Legendary");
        assert_eq!(damage_mod, 3.0);
        assert_eq!(value_mod, 10.0);
        assert_eq!(rarity, ItemRarity::Legendary);
    }
}