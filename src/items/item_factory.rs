use specs::{World, WorldExt, Builder, Entity};
use crate::components::{Position, Renderable, Name, Item};
use crate::items::item_components::*;
use crate::resources::RandomNumberGenerator;

pub struct ItemFactory;

impl ItemFactory {
    pub fn new() -> Self {
        ItemFactory
    }

    // Create a basic item with minimal properties
    pub fn create_basic_item(
        &self,
        world: &mut World,
        name: String,
        item_type: ItemType,
        position: Position,
        glyph: char,
        color: crossterm::style::Color,
    ) -> Entity {
        world.create_entity()
            .with(Item)
            .with(Name { name: name.clone() })
            .with(ItemProperties::new(name, item_type))
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    // Create weapons
    pub fn create_weapon(
        &self,
        world: &mut World,
        weapon_type: WeaponType,
        position: Position,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        let (name, glyph, color, base_value, weight) = match weapon_type {
            WeaponType::Sword => ("Iron Sword", '/', crossterm::style::Color::Grey, 50, 3.0),
            WeaponType::Axe => ("Battle Axe", 'P', crossterm::style::Color::DarkGrey, 60, 4.0),
            WeaponType::Mace => ("War Mace", 'T', crossterm::style::Color::Grey, 45, 3.5),
            WeaponType::Dagger => ("Steel Dagger", '-', crossterm::style::Color::White, 25, 1.0),
            WeaponType::Spear => ("Iron Spear", '|', crossterm::style::Color::DarkYellow, 40, 2.5),
            WeaponType::Bow => ("Hunting Bow", ')', crossterm::style::Color::DarkYellow, 75, 2.0),
            WeaponType::Crossbow => ("Light Crossbow", '}', crossterm::style::Color::DarkGrey, 100, 4.0),
            WeaponType::Staff => ("Wooden Staff", '\\', crossterm::style::Color::DarkYellow, 30, 2.0),
            WeaponType::Wand => ("Magic Wand", '/', crossterm::style::Color::Magenta, 80, 0.5),
            WeaponType::Thrown => ("Throwing Knife", '-', crossterm::style::Color::Grey, 15, 0.5),
        };

        let rarity = self.generate_rarity(rng);
        let final_value = (base_value as f32 * rarity.value_multiplier()) as i32;

        let properties = ItemProperties::new(name.to_string(), ItemType::Weapon(weapon_type.clone()))
            .with_description(format!("A {} weapon suitable for combat.", weapon_type.name()))
            .with_rarity(rarity)
            .with_value(final_value)
            .with_weight(weight)
            .with_durability(100);

        let mut bonuses = ItemBonuses::new();
        bonuses.combat_bonuses.attack_bonus = match weapon_type {
            WeaponType::Sword => 5,
            WeaponType::Axe => 7,
            WeaponType::Mace => 6,
            WeaponType::Dagger => 3,
            WeaponType::Spear => 4,
            WeaponType::Bow => 6,
            WeaponType::Crossbow => 8,
            WeaponType::Staff => 2,
            WeaponType::Wand => 1,
            WeaponType::Thrown => 2,
        };

        bonuses.combat_bonuses.damage_bonus = match weapon_type {
            WeaponType::Sword => 8,
            WeaponType::Axe => 12,
            WeaponType::Mace => 10,
            WeaponType::Dagger => 4,
            WeaponType::Spear => 6,
            WeaponType::Bow => 7,
            WeaponType::Crossbow => 10,
            WeaponType::Staff => 3,
            WeaponType::Wand => 2,
            WeaponType::Thrown => 3,
        };

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(bonuses)
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    // Create armor
    pub fn create_armor(
        &self,
        world: &mut World,
        armor_type: ArmorType,
        position: Position,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        let (name, glyph, color, base_value, weight) = match armor_type {
            ArmorType::Helmet => ("Iron Helmet", '^', crossterm::style::Color::Grey, 40, 2.0),
            ArmorType::Chest => ("Chain Mail", '[', crossterm::style::Color::Grey, 80, 15.0),
            ArmorType::Legs => ("Iron Greaves", '[', crossterm::style::Color::DarkGrey, 60, 8.0),
            ArmorType::Boots => ("Leather Boots", '[', crossterm::style::Color::DarkYellow, 25, 2.0),
            ArmorType::Gloves => ("Leather Gloves", '[', crossterm::style::Color::DarkYellow, 20, 1.0),
            ArmorType::Shield => ("Iron Shield", ')', crossterm::style::Color::Grey, 50, 5.0),
            ArmorType::Cloak => ("Traveler's Cloak", '(', crossterm::style::Color::DarkGreen, 30, 2.0),
            ArmorType::Ring => ("Simple Ring", '=', crossterm::style::Color::Yellow, 100, 0.1),
            ArmorType::Amulet => ("Bone Amulet", '"', crossterm::style::Color::White, 75, 0.2),
        };

        let rarity = self.generate_rarity(rng);
        let final_value = (base_value as f32 * rarity.value_multiplier()) as i32;

        let properties = ItemProperties::new(name.to_string(), ItemType::Armor(armor_type.clone()))
            .with_description(format!("A piece of {} armor for protection.", armor_type.name()))
            .with_rarity(rarity)
            .with_value(final_value)
            .with_weight(weight)
            .with_durability(80);

        let mut bonuses = ItemBonuses::new();
        bonuses.combat_bonuses.defense_bonus = match armor_type {
            ArmorType::Helmet => 3,
            ArmorType::Chest => 8,
            ArmorType::Legs => 5,
            ArmorType::Boots => 2,
            ArmorType::Gloves => 1,
            ArmorType::Shield => 6,
            ArmorType::Cloak => 2,
            ArmorType::Ring => 1,
            ArmorType::Amulet => 1,
        };

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(bonuses)
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    // Create consumables
    pub fn create_consumable(
        &self,
        world: &mut World,
        consumable_type: ConsumableType,
        position: Position,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        let (name, glyph, color, base_value, weight, stack_size) = match consumable_type {
            ConsumableType::Potion => ("Health Potion", '!', crossterm::style::Color::Red, 25, 0.5, 10),
            ConsumableType::Food => ("Bread", '%', crossterm::style::Color::DarkYellow, 5, 0.2, 20),
            ConsumableType::Scroll => ("Magic Scroll", '?', crossterm::style::Color::White, 50, 0.1, 5),
            ConsumableType::Ammunition => ("Arrow", '|', crossterm::style::Color::DarkYellow, 1, 0.1, 50),
        };

        let rarity = if matches!(consumable_type, ConsumableType::Scroll) {
            self.generate_rarity(rng)
        } else {
            ItemRarity::Common
        };

        let final_value = (base_value as f32 * rarity.value_multiplier()) as i32;

        let properties = ItemProperties::new(name.to_string(), ItemType::Consumable(consumable_type.clone()))
            .with_description(format!("A consumable {} item.", consumable_type.name()))
            .with_rarity(rarity)
            .with_value(final_value)
            .with_weight(weight)
            .with_stack_size(stack_size);

        let stack = ItemStack::new(1, stack_size);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(stack)
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    // Create tools
    pub fn create_tool(
        &self,
        world: &mut World,
        tool_type: ToolType,
        position: Position,
    ) -> Entity {
        let (name, glyph, color, value, weight, description) = match tool_type {
            ToolType::Lockpick => ("Lockpick Set", '(', crossterm::style::Color::Grey, 30, 0.2, "Tools for picking locks."),
            ToolType::Torch => ("Torch", '|', crossterm::style::Color::Yellow, 5, 1.0, "A burning torch for light."),
            ToolType::Rope => ("Rope", '~', crossterm::style::Color::DarkYellow, 10, 2.0, "Strong rope for climbing."),
            ToolType::Key => ("Iron Key", '-', crossterm::style::Color::Yellow, 1, 0.1, "A key that opens something."),
            ToolType::Container => ("Wooden Chest", '=', crossterm::style::Color::DarkYellow, 50, 10.0, "A container for storing items."),
        };

        let properties = ItemProperties::new(name.to_string(), ItemType::Tool(tool_type))
            .with_description(description.to_string())
            .with_value(value)
            .with_weight(weight);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    // Create materials
    pub fn create_material(
        &self,
        world: &mut World,
        material_type: MaterialType,
        position: Position,
        quantity: i32,
    ) -> Entity {
        let (name, glyph, color, base_value, weight, stack_size) = match material_type {
            MaterialType::Metal => ("Iron Ore", '*', crossterm::style::Color::Grey, 10, 2.0, 25),
            MaterialType::Wood => ("Oak Wood", '|', crossterm::style::Color::DarkYellow, 5, 1.0, 50),
            MaterialType::Leather => ("Leather Hide", '&', crossterm::style::Color::DarkYellow, 15, 1.5, 20),
            MaterialType::Cloth => ("Linen Cloth", '~', crossterm::style::Color::White, 8, 0.5, 30),
            MaterialType::Gem => ("Raw Gem", '*', crossterm::style::Color::Cyan, 100, 0.2, 10),
            MaterialType::Herb => ("Healing Herb", ',', crossterm::style::Color::Green, 20, 0.1, 40),
            MaterialType::Bone => ("Animal Bone", '|', crossterm::style::Color::White, 5, 0.8, 25),
            MaterialType::Stone => ("Stone Block", '#', crossterm::style::Color::Grey, 2, 5.0, 20),
        };

        let properties = ItemProperties::new(name.to_string(), ItemType::Material(material_type.clone()))
            .with_description(format!("Raw {} material for crafting.", material_type.name()))
            .with_value(base_value)
            .with_weight(weight)
            .with_stack_size(stack_size);

        let stack = ItemStack::new(quantity, stack_size);

        world.create_entity()
            .with(Item)
            .with(Name { name: name.to_string() })
            .with(properties)
            .with(stack)
            .with(position)
            .with(Renderable {
                glyph,
                fg: color,
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build()
    }

    // Generate random rarity based on probability
    fn generate_rarity(&self, rng: &mut RandomNumberGenerator) -> ItemRarity {
        let roll = rng.roll_dice(1, 1000);
        match roll {
            1..=500 => ItemRarity::Common,      // 50%
            501..=750 => ItemRarity::Uncommon,  // 25%
            751..=900 => ItemRarity::Rare,      // 15%
            901..=970 => ItemRarity::Epic,      // 7%
            971..=995 => ItemRarity::Legendary, // 2.5%
            _ => ItemRarity::Artifact,          // 0.5%
        }
    }

    // Create a random item of a specific type
    pub fn create_random_weapon(&self, world: &mut World, position: Position, rng: &mut RandomNumberGenerator) -> Entity {
        let weapon_types = vec![
            WeaponType::Sword,
            WeaponType::Axe,
            WeaponType::Mace,
            WeaponType::Dagger,
            WeaponType::Spear,
            WeaponType::Bow,
            WeaponType::Staff,
        ];
        
        let weapon_type = weapon_types[rng.roll_dice(1, weapon_types.len()) - 1].clone();
        self.create_weapon(world, weapon_type, position, rng)
    }

    pub fn create_random_armor(&self, world: &mut World, position: Position, rng: &mut RandomNumberGenerator) -> Entity {
        let armor_types = vec![
            ArmorType::Helmet,
            ArmorType::Chest,
            ArmorType::Legs,
            ArmorType::Boots,
            ArmorType::Gloves,
            ArmorType::Shield,
            ArmorType::Cloak,
        ];
        
        let armor_type = armor_types[rng.roll_dice(1, armor_types.len()) - 1].clone();
        self.create_armor(world, armor_type, position, rng)
    }

    pub fn create_random_consumable(&self, world: &mut World, position: Position, rng: &mut RandomNumberGenerator) -> Entity {
        let consumable_types = vec![
            ConsumableType::Potion,
            ConsumableType::Food,
            ConsumableType::Scroll,
        ];
        
        let consumable_type = consumable_types[rng.roll_dice(1, consumable_types.len()) - 1].clone();
        self.create_consumable(world, consumable_type, position, rng)
    }

    // Create magical items with enchantments
    pub fn create_magical_item(
        &self,
        world: &mut World,
        base_item_type: ItemType,
        position: Position,
        magic_level: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        // Create base item first
        let entity = match base_item_type {
            ItemType::Weapon(weapon_type) => self.create_weapon(world, weapon_type, position, rng),
            ItemType::Armor(armor_type) => self.create_armor(world, armor_type, position, rng),
            _ => self.create_basic_item(world, "Magical Item".to_string(), base_item_type, position, '*', crossterm::style::Color::Magenta),
        };

        // Add magical properties
        let mut magical_item = MagicalItem::new(magic_level);
        
        // Add random enchantments based on magic level
        let num_enchantments = (magic_level / 2).max(1);
        for _ in 0..num_enchantments {
            let enchantment = self.generate_random_enchantment(rng);
            magical_item.add_enchantment(enchantment);
        }

        // Small chance for curse
        if rng.roll_dice(1, 20) == 1 {
            let curse = self.generate_random_curse(rng);
            magical_item.add_curse(curse);
        }

        world.write_storage::<MagicalItem>()
            .insert(entity, magical_item)
            .expect("Failed to add magical properties");

        entity
    }

    fn generate_random_enchantment(&self, rng: &mut RandomNumberGenerator) -> Enchantment {
        let enchantment_types = vec![
            EnchantmentType::Sharpness,
            EnchantmentType::Fire,
            EnchantmentType::Protection,
            EnchantmentType::AttributeBonus("Strength".to_string(), rng.roll_dice(1, 3)),
        ];

        let enchantment_type = enchantment_types[rng.roll_dice(1, enchantment_types.len()) - 1].clone();
        let power = rng.roll_dice(1, 5);

        Enchantment {
            name: format!("{:?}", enchantment_type),
            description: "A magical enchantment.".to_string(),
            enchantment_type,
            power,
            duration: None,
        }
    }

    fn generate_random_curse(&self, rng: &mut RandomNumberGenerator) -> Curse {
        let curse_types = vec![
            CurseType::Binding,
            CurseType::Fragility,
            CurseType::Weakness,
        ];

        let curse_type = curse_types[rng.roll_dice(1, curse_types.len()) - 1].clone();
        let power = rng.roll_dice(1, 3);

        Curse {
            name: format!("{:?}", curse_type),
            description: "A harmful curse.".to_string(),
            curse_type,
            power,
            removable: rng.roll_dice(1, 2) == 1,
        }
    }
}

// Extension traits for item types
impl WeaponType {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponType::Sword => "sword",
            WeaponType::Axe => "axe",
            WeaponType::Mace => "mace",
            WeaponType::Dagger => "dagger",
            WeaponType::Spear => "spear",
            WeaponType::Bow => "bow",
            WeaponType::Crossbow => "crossbow",
            WeaponType::Staff => "staff",
            WeaponType::Wand => "wand",
            WeaponType::Thrown => "thrown weapon",
        }
    }
}

impl ArmorType {
    pub fn name(&self) -> &'static str {
        match self {
            ArmorType::Helmet => "helmet",
            ArmorType::Chest => "chest armor",
            ArmorType::Legs => "leg armor",
            ArmorType::Boots => "boots",
            ArmorType::Gloves => "gloves",
            ArmorType::Shield => "shield",
            ArmorType::Cloak => "cloak",
            ArmorType::Ring => "ring",
            ArmorType::Amulet => "amulet",
        }
    }
}

impl ConsumableType {
    pub fn name(&self) -> &'static str {
        match self {
            ConsumableType::Potion => "potion",
            ConsumableType::Food => "food",
            ConsumableType::Scroll => "scroll",
            ConsumableType::Ammunition => "ammunition",
        }
    }
}

impl MaterialType {
    pub fn name(&self) -> &'static str {
        match self {
            MaterialType::Metal => "metal",
            MaterialType::Wood => "wood",
            MaterialType::Leather => "leather",
            MaterialType::Cloth => "cloth",
            MaterialType::Gem => "gem",
            MaterialType::Herb => "herb",
            MaterialType::Bone => "bone",
            MaterialType::Stone => "stone",
        }
    }
}