#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::resources::RandomNumberGenerator;

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Item>();
        world.register::<Name>();
        world.register::<ItemProperties>();
        world.register::<Position>();
        world.register::<Renderable>();
        world.register::<ItemStack>();
        world.register::<ItemIdentification>();
        world.register::<MagicalItem>();
        world.register::<ItemBonuses>();
        world
    }

    #[test]
    fn test_item_properties_creation() {
        let props = ItemProperties::new("Test Sword".to_string(), ItemType::Weapon(WeaponType::Sword))
            .with_description("A test sword".to_string())
            .with_rarity(ItemRarity::Rare)
            .with_value(100)
            .with_weight(3.5)
            .with_durability(80);

        assert_eq!(props.name, "Test Sword");
        assert_eq!(props.rarity, ItemRarity::Rare);
        assert_eq!(props.value, 100);
        assert_eq!(props.weight, 3.5);
        assert!(props.durability.is_some());
        assert_eq!(props.durability.as_ref().unwrap().max, 80);
    }

    #[test]
    fn test_item_rarity_properties() {
        assert_eq!(ItemRarity::Common.name(), "Common");
        assert_eq!(ItemRarity::Legendary.name(), "Legendary");
        
        assert!(ItemRarity::Legendary.value_multiplier() > ItemRarity::Common.value_multiplier());
        
        let common_color = ItemRarity::Common.color();
        let legendary_color = ItemRarity::Legendary.color();
        assert_ne!(common_color, legendary_color);
    }

    #[test]
    fn test_item_stack() {
        let mut stack = ItemStack::new(5, 10);
        
        assert_eq!(stack.quantity, 5);
        assert_eq!(stack.max_stack, 10);
        assert!(!stack.is_full());
        assert!(stack.can_add(3));
        assert!(!stack.can_add(10));
        
        let overflow = stack.add(7);
        assert_eq!(stack.quantity, 10);
        assert_eq!(overflow, 2);
        assert!(stack.is_full());
        
        let removed = stack.remove(3);
        assert_eq!(removed, 3);
        assert_eq!(stack.quantity, 7);
        assert!(!stack.is_full());
    }

    #[test]
    fn test_item_durability() {
        let mut durability = ItemDurability { current: 80, max: 100 };
        
        assert_eq!(durability.percentage(), 0.8);
        assert_eq!(durability.condition_name(), "Good");
        assert!(!durability.is_broken());
        
        // Test damage
        let mut props = ItemProperties::new("Test Item".to_string(), ItemType::Miscellaneous)
            .with_durability(100);
        
        props.damage(30);
        assert_eq!(props.durability.as_ref().unwrap().current, 70);
        
        props.repair(20);
        assert_eq!(props.durability.as_ref().unwrap().current, 90);
        
        props.damage(200); // Excessive damage
        assert_eq!(props.durability.as_ref().unwrap().current, 0);
        assert!(props.is_broken());
    }

    #[test]
    fn test_item_factory_weapon_creation() {
        let mut world = setup_world();
        let mut rng = RandomNumberGenerator::new();
        let factory = ItemFactory::new();
        
        let position = Position { x: 5, y: 5 };
        let entity = factory.create_weapon(&mut world, WeaponType::Sword, position, &mut rng);
        
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        let bonuses = world.read_storage::<ItemBonuses>();
        
        let name = names.get(entity).unwrap();
        let props = properties.get(entity).unwrap();
        let bonus = bonuses.get(entity).unwrap();
        
        assert_eq!(name.name, "Iron Sword");
        assert!(matches!(props.item_type, ItemType::Weapon(WeaponType::Sword)));
        assert!(bonus.combat_bonuses.attack_bonus > 0);
        assert!(bonus.combat_bonuses.damage_bonus > 0);
    }

    #[test]
    fn test_item_factory_consumable_creation() {
        let mut world = setup_world();
        let mut rng = RandomNumberGenerator::new();
        let factory = ItemFactory::new();
        
        let position = Position { x: 3, y: 3 };
        let entity = factory.create_consumable(&mut world, ConsumableType::Potion, position, &mut rng);
        
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        let stacks = world.read_storage::<ItemStack>();
        
        let name = names.get(entity).unwrap();
        let props = properties.get(entity).unwrap();
        let stack = stacks.get(entity).unwrap();
        
        assert_eq!(name.name, "Health Potion");
        assert!(matches!(props.item_type, ItemType::Consumable(ConsumableType::Potion)));
        assert_eq!(stack.quantity, 1);
        assert!(stack.max_stack > 1);
    }

    #[test]
    fn test_magical_item() {
        let mut magical_item = MagicalItem::new(5);
        
        let enchantment = Enchantment {
            name: "Sharpness".to_string(),
            description: "Increases damage".to_string(),
            enchantment_type: EnchantmentType::Sharpness,
            power: 3,
            duration: None,
        };
        
        magical_item.add_enchantment(enchantment);
        assert_eq!(magical_item.enchantments.len(), 1);
        assert_eq!(magical_item.total_enchantment_power(), 3);
        assert!(!magical_item.is_cursed());
        
        let curse = Curse {
            name: "Binding".to_string(),
            description: "Cannot be unequipped".to_string(),
            curse_type: CurseType::Binding,
            power: 1,
            removable: false,
        };
        
        magical_item.add_curse(curse);
        assert!(magical_item.is_cursed());
    }

    #[test]
    fn test_item_identification() {
        let mut identification = ItemIdentification::new("Mysterious Sword".to_string())
            .with_description("A sword of unknown origin".to_string());
        
        assert!(!identification.identified);
        assert_eq!(identification.unidentified_name, "Mysterious Sword");
        
        identification.identify();
        assert!(identification.identified);
    }

    #[test]
    fn test_item_requirements() {
        let requirements = ItemRequirements::new()
            .with_level(5)
            .with_strength(12)
            .with_skill("Swords".to_string(), 3);
        
        assert_eq!(requirements.level, 5);
        assert_eq!(requirements.strength, 12);
        assert_eq!(requirements.skills.get("Swords"), Some(&3));
    }

    #[test]
    fn test_item_bonuses() {
        let mut bonuses = ItemBonuses::new();
        
        bonuses.add_attribute_bonus("Strength".to_string(), 2);
        bonuses.add_skill_bonus("Swords".to_string(), 1);
        
        assert_eq!(bonuses.attribute_bonuses.get("Strength"), Some(&2));
        assert_eq!(bonuses.skill_bonuses.get("Swords"), Some(&1));
        
        bonuses.add_attribute_bonus("Strength".to_string(), 1); // Should stack
        assert_eq!(bonuses.attribute_bonuses.get("Strength"), Some(&3));
    }

    #[test]
    fn test_item_database() {
        let db = ItemDatabase::create_default_database();
        
        assert!(db.get_item_template("iron_sword").is_some());
        assert!(db.get_item_template("health_potion").is_some());
        assert!(db.get_item_template("leather_armor").is_some());
        assert!(db.get_item_template("nonexistent").is_none());
        
        let sword_template = db.get_item_template("iron_sword").unwrap();
        assert_eq!(sword_template.name, "Iron Sword");
        assert!(matches!(sword_template.item_type, ItemType::Weapon(WeaponType::Sword)));
    }

    #[test]
    fn test_item_template_creation() {
        let mut world = setup_world();
        let db = ItemDatabase::create_default_database();
        let template = db.get_item_template("health_potion").unwrap();
        
        let position = Position { x: 0, y: 0 };
        let entity = template.create_item(&mut world, position);
        
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        let stacks = world.read_storage::<ItemStack>();
        
        let name = names.get(entity).unwrap();
        let props = properties.get(entity).unwrap();
        let stack = stacks.get(entity);
        
        assert_eq!(name.name, "Health Potion");
        assert!(matches!(props.item_type, ItemType::Consumable(ConsumableType::Potion)));
        assert!(stack.is_some());
    }

    #[test]
    fn test_utility_functions() {
        let mut world = setup_world();
        let factory = ItemFactory::new();
        let mut rng = RandomNumberGenerator::new();
        
        // Create some test items
        let pos1 = Position { x: 5, y: 5 };
        let pos2 = Position { x: 5, y: 5 }; // Same position
        let pos3 = Position { x: 6, y: 6 }; // Different position
        
        let item1 = factory.create_consumable(&mut world, ConsumableType::Potion, pos1, &mut rng);
        let item2 = factory.create_weapon(&mut world, WeaponType::Sword, pos2, &mut rng);
        let item3 = factory.create_armor(&mut world, ArmorType::Helmet, pos3, &mut rng);
        
        // Test finding items at position
        let items_at_5_5 = find_items_at_position(&world, 5, 5);
        assert_eq!(items_at_5_5.len(), 2);
        assert!(items_at_5_5.contains(&item1));
        assert!(items_at_5_5.contains(&item2));
        
        let items_at_6_6 = find_items_at_position(&world, 6, 6);
        assert_eq!(items_at_6_6.len(), 1);
        assert!(items_at_6_6.contains(&item3));
        
        // Test weight calculation
        let weight_at_5_5 = get_total_weight_at_position(&world, 5, 5);
        assert!(weight_at_5_5 > 0.0);
        
        // Test item value
        let value = get_item_current_value(&world, item1);
        assert!(value > 0);
        
        // Test display name
        let display_name = get_item_display_name(&world, item1);
        assert!(display_name.is_some());
        assert_eq!(display_name.unwrap(), "Health Potion");
    }

    #[test]
    fn test_item_serialization() {
        let mut world = setup_world();
        let factory = ItemFactory::new();
        let mut rng = RandomNumberGenerator::new();
        
        // Create a test item
        let position = Position { x: 10, y: 10 };
        let entity = factory.create_weapon(&mut world, WeaponType::Sword, position, &mut rng);
        
        // Serialize the item
        let serializable = SerializableItem::from_entity(&world, entity);
        assert!(serializable.is_some());
        
        let serializable = serializable.unwrap();
        assert_eq!(serializable.name, "Iron Sword");
        assert_eq!(serializable.position.x, 10);
        assert_eq!(serializable.position.y, 10);
        
        // Create a new world and deserialize
        let mut new_world = setup_world();
        let new_entity = serializable.to_entity(&mut new_world);
        
        let names = new_world.read_storage::<Name>();
        let positions = new_world.read_storage::<Position>();
        
        let name = names.get(new_entity).unwrap();
        let pos = positions.get(new_entity).unwrap();
        
        assert_eq!(name.name, "Iron Sword");
        assert_eq!(pos.x, 10);
        assert_eq!(pos.y, 10);
    }
}