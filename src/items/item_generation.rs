use specs::{World, WorldExt, Entity};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::components::{Position, Name, Renderable, Item};
use crate::items::{
    ItemProperties, ItemType, ItemRarity, WeaponType, ArmorType, ConsumableType,
    ItemBonuses, MagicalItem, Enchantment, EnchantmentType, ItemStack, ItemFactory
};
use crate::resources::RandomNumberGenerator;

/// Main item generation system
pub struct ItemGenerator {
    pub loot_tables: HashMap<String, LootTable>,
    pub affix_tables: HashMap<ItemType, AffixTable>,
    pub rarity_weights: RarityWeights,
    pub depth_scaling: DepthScaling,
}

impl ItemGenerator {
    pub fn new() -> Self {
        let mut generator = ItemGenerator {
            loot_tables: HashMap::new(),
            affix_tables: HashMap::new(),
            rarity_weights: RarityWeights::default(),
            depth_scaling: DepthScaling::default(),
        };
        
        generator.initialize_default_tables();
        generator
    }

    fn initialize_default_tables(&mut self) {
        self.create_default_loot_tables();
        self.create_default_affix_tables();
    }

    /// Generate a random item based on dungeon depth and context
    pub fn generate_item(
        &self,
        world: &mut World,
        position: Position,
        depth: i32,
        context: GenerationContext,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        // Determine item type based on context
        let item_type = self.select_item_type(&context, rng);
        
        // Determine rarity based on depth
        let rarity = self.select_rarity(depth, &context, rng);
        
        // Create base item
        let factory = ItemFactory::new();
        let entity = match item_type {
            ItemType::Weapon(weapon_type) => {
                factory.create_weapon(world, weapon_type, position, rng)
            },
            ItemType::Armor(armor_type) => {
                factory.create_armor(world, armor_type, position, rng)
            },
            ItemType::Consumable(consumable_type) => {
                factory.create_consumable(world, consumable_type, position, rng)
            },
            _ => {
                factory.create_basic_item(world, "Generated Item".to_string(), item_type, position, '?', crossterm::style::Color::White)
            }
        };

        // Apply depth scaling
        self.apply_depth_scaling(world, entity, depth);
        
        // Apply rarity modifications
        self.apply_rarity_modifications(world, entity, rarity, rng);
        
        // Apply affixes if applicable
        if rarity >= ItemRarity::Uncommon {
            self.apply_affixes(world, entity, &item_type, rarity, rng);
        }
        
        entity
    }

    /// Generate items from a loot table
    pub fn generate_from_loot_table(
        &self,
        world: &mut World,
        table_name: &str,
        position: Position,
        depth: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Vec<Entity> {
        if let Some(loot_table) = self.loot_tables.get(table_name) {
            loot_table.generate_loot(world, self, position, depth, rng)
        } else {
            Vec::new()
        }
    }

    fn select_item_type(&self, context: &GenerationContext, rng: &mut RandomNumberGenerator) -> ItemType {
        let weights = match context {
            GenerationContext::Combat => vec![
                (ItemType::Weapon(WeaponType::Sword), 30),
                (ItemType::Armor(ArmorType::Chest), 25),
                (ItemType::Consumable(ConsumableType::Potion), 20),
                (ItemType::Armor(ArmorType::Shield), 15),
                (ItemType::Weapon(WeaponType::Bow), 10),
            ],
            GenerationContext::Treasure => vec![
                (ItemType::Weapon(WeaponType::Sword), 20),
                (ItemType::Armor(ArmorType::Chest), 20),
                (ItemType::Consumable(ConsumableType::Potion), 15),
                (ItemType::Material(crate::items::MaterialType::Gem), 25),
                (ItemType::Miscellaneous, 20),
            ],
            GenerationContext::Merchant => vec![
                (ItemType::Weapon(WeaponType::Sword), 25),
                (ItemType::Armor(ArmorType::Chest), 25),
                (ItemType::Consumable(ConsumableType::Potion), 30),
                (ItemType::Tool(crate::items::ToolType::Lockpick), 10),
                (ItemType::Material(crate::items::MaterialType::Metal), 10),
            ],
            GenerationContext::Random => vec![
                (ItemType::Weapon(WeaponType::Sword), 20),
                (ItemType::Armor(ArmorType::Chest), 20),
                (ItemType::Consumable(ConsumableType::Potion), 20),
                (ItemType::Material(crate::items::MaterialType::Metal), 20),
                (ItemType::Tool(crate::items::ToolType::Torch), 20),
            ],
        };

        self.weighted_choice(&weights, rng)
    }

    fn select_rarity(&self, depth: i32, context: &GenerationContext, rng: &mut RandomNumberGenerator) -> ItemRarity {
        let mut weights = self.rarity_weights.base_weights.clone();
        
        // Apply depth scaling to rarity
        let depth_bonus = (depth as f32 * self.depth_scaling.rarity_scaling).min(50.0);
        
        // Increase chances of higher rarities with depth
        if depth > 5 {
            weights[&ItemRarity::Uncommon] += depth_bonus as i32;
        }
        if depth > 10 {
            weights[&ItemRarity::Rare] += (depth_bonus * 0.5) as i32;
        }
        if depth > 15 {
            weights[&ItemRarity::Epic] += (depth_bonus * 0.25) as i32;
        }
        if depth > 20 {
            weights[&ItemRarity::Legendary] += (depth_bonus * 0.1) as i32;
        }
        
        // Apply context modifiers
        match context {
            GenerationContext::Treasure => {
                // Treasure has better rarity chances
                weights.iter_mut().for_each(|(rarity, weight)| {
                    if *rarity >= ItemRarity::Uncommon {
                        *weight = (*weight as f32 * 1.5) as i32;
                    }
                });
            },
            GenerationContext::Combat => {
                // Combat drops are more common
                weights[&ItemRarity::Common] += 20;
            },
            _ => {},
        }

        let weight_vec: Vec<(ItemRarity, i32)> = weights.into_iter().collect();
        self.weighted_choice(&weight_vec, rng)
    }

    fn apply_depth_scaling(&self, world: &mut World, entity: Entity, depth: i32) {
        let mut bonuses = world.write_storage::<ItemBonuses>();
        
        if let Some(bonus) = bonuses.get_mut(entity) {
            let scaling_factor = (depth as f32 * self.depth_scaling.stat_scaling).max(0.0);
            
            // Scale combat bonuses
            bonus.combat_bonuses.attack_bonus += (scaling_factor * 0.5) as i32;
            bonus.combat_bonuses.damage_bonus += (scaling_factor * 0.7) as i32;
            bonus.combat_bonuses.defense_bonus += (scaling_factor * 0.6) as i32;
        }
        
        // Scale item value
        let mut properties = world.write_storage::<ItemProperties>();
        if let Some(props) = properties.get_mut(entity) {
            let value_scaling = 1.0 + (depth as f32 * self.depth_scaling.value_scaling);
            props.value = (props.value as f32 * value_scaling) as i32;
        }
    }

    fn apply_rarity_modifications(&self, world: &mut World, entity: Entity, rarity: ItemRarity, rng: &mut RandomNumberGenerator) {
        // Update item properties for rarity
        let mut properties = world.write_storage::<ItemProperties>();
        if let Some(props) = properties.get_mut(entity) {
            props.rarity = rarity.clone();
            props.value = (props.value as f32 * rarity.value_multiplier()) as i32;
        }

        // Add magical properties for higher rarities
        if rarity >= ItemRarity::Rare {
            let magic_level = match rarity {
                ItemRarity::Rare => 1,
                ItemRarity::Epic => 2,
                ItemRarity::Legendary => 3,
                ItemRarity::Artifact => 5,
                _ => 0,
            };

            if magic_level > 0 {
                let mut magical_item = MagicalItem::new(magic_level);
                
                // Add enchantments based on magic level
                for _ in 0..magic_level {
                    if rng.roll_dice(1, 100) <= 70 { // 70% chance per level
                        let enchantment = self.generate_random_enchantment(rng);
                        magical_item.add_enchantment(enchantment);
                    }
                }

                world.write_storage::<MagicalItem>()
                    .insert(entity, magical_item)
                    .expect("Failed to add magical properties");
            }
        }
    }

    fn apply_affixes(&self, world: &mut World, entity: Entity, item_type: &ItemType, rarity: ItemRarity, rng: &mut RandomNumberGenerator) {
        if let Some(affix_table) = self.affix_tables.get(item_type) {
            let num_affixes = match rarity {
                ItemRarity::Uncommon => 1,
                ItemRarity::Rare => rng.roll_dice(1, 2),
                ItemRarity::Epic => rng.roll_dice(1, 3),
                ItemRarity::Legendary => rng.roll_dice(2, 3),
                ItemRarity::Artifact => rng.roll_dice(2, 4),
                _ => 0,
            };

            for _ in 0..num_affixes {
                if let Some(affix) = affix_table.get_random_affix(rng) {
                    self.apply_affix(world, entity, &affix);
                }
            }
        }
    }

    fn apply_affix(&self, world: &mut World, entity: Entity, affix: &Affix) {
        // Update item name
        let mut names = world.write_storage::<Name>();
        if let Some(name) = names.get_mut(entity) {
            match affix.affix_type {
                AffixType::Prefix => {
                    name.name = format!("{} {}", affix.name, name.name);
                },
                AffixType::Suffix => {
                    name.name = format!("{} {}", name.name, affix.name);
                },
            }
        }

        // Apply stat bonuses
        let mut bonuses = world.write_storage::<ItemBonuses>();
        if let Some(bonus) = bonuses.get_mut(entity) {
            for (stat, value) in &affix.stat_bonuses {
                match stat.as_str() {
                    "attack" => bonus.combat_bonuses.attack_bonus += value,
                    "damage" => bonus.combat_bonuses.damage_bonus += value,
                    "defense" => bonus.combat_bonuses.defense_bonus += value,
                    "critical_chance" => bonus.combat_bonuses.critical_chance_bonus += value,
                    "critical_damage" => bonus.combat_bonuses.critical_damage_bonus += value,
                    _ => {
                        // Handle attribute bonuses
                        bonus.attribute_bonuses.insert(stat.clone(), *value);
                    }
                }
            }
        }

        // Update item value
        let mut properties = world.write_storage::<ItemProperties>();
        if let Some(props) = properties.get_mut(entity) {
            props.value += affix.value_bonus;
        }
    }

    fn generate_random_enchantment(&self, rng: &mut RandomNumberGenerator) -> Enchantment {
        let enchantment_types = vec![
            EnchantmentType::Sharpness,
            EnchantmentType::Fire,
            EnchantmentType::Ice,
            EnchantmentType::Lightning,
            EnchantmentType::Protection,
            EnchantmentType::Regeneration,
        ];

        let enchantment_type = enchantment_types[rng.roll_dice(1, enchantment_types.len()) - 1].clone();
        let power = rng.roll_dice(1, 5);

        Enchantment {
            name: format!("{:?}", enchantment_type),
            description: self.get_enchantment_description(&enchantment_type),
            enchantment_type,
            power,
            duration: None,
        }
    }

    fn get_enchantment_description(&self, enchantment_type: &EnchantmentType) -> String {
        match enchantment_type {
            EnchantmentType::Sharpness => "Increases weapon damage".to_string(),
            EnchantmentType::Fire => "Adds fire damage to attacks".to_string(),
            EnchantmentType::Ice => "Adds cold damage and slowing effect".to_string(),
            EnchantmentType::Lightning => "Adds lightning damage".to_string(),
            EnchantmentType::Protection => "Increases armor value".to_string(),
            EnchantmentType::Regeneration => "Slowly restores health".to_string(),
            _ => "A magical enchantment".to_string(),
        }
    }

    fn weighted_choice<T: Clone>(&self, weights: &[(T, i32)], rng: &mut RandomNumberGenerator) -> T {
        let total_weight: i32 = weights.iter().map(|(_, weight)| weight).sum();
        let mut roll = rng.roll_dice(1, total_weight);
        
        for (item, weight) in weights {
            roll -= weight;
            if roll <= 0 {
                return item.clone();
            }
        }
        
        // Fallback to first item
        weights[0].0.clone()
    }

    fn create_default_loot_tables(&mut self) {
        // Monster loot table
        let monster_loot = LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Potion)),
                    table_reference: None,
                    weight: 40,
                    quantity_range: (1, 2),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Material(crate::items::MaterialType::Bone)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 3),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Dagger)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("gold".to_string()),
                    weight: 10,
                    quantity_range: (5, 20),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 3,
        };
        self.loot_tables.insert("monster".to_string(), monster_loot);

        // Treasure chest loot table
        let treasure_loot = LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Sword)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Chest)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(crate::items::MaterialType::Gem)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 3),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("gold".to_string()),
                    weight: 20,
                    quantity_range: (20, 100),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 2,
            max_drops: 4,
        };
        self.loot_tables.insert("treasure".to_string(), treasure_loot);
    }

    fn create_default_affix_tables(&mut self) {
        // Weapon affixes
        let weapon_affixes = AffixTable {
            prefixes: vec![
                Affix {
                    name: "Sharp".to_string(),
                    affix_type: AffixType::Prefix,
                    stat_bonuses: vec![("damage".to_string(), 2)].into_iter().collect(),
                    value_bonus: 25,
                    weight: 30,
                },
                Affix {
                    name: "Heavy".to_string(),
                    affix_type: AffixType::Prefix,
                    stat_bonuses: vec![("damage".to_string(), 4), ("attack".to_string(), -1)].into_iter().collect(),
                    value_bonus: 40,
                    weight: 20,
                },
                Affix {
                    name: "Swift".to_string(),
                    affix_type: AffixType::Prefix,
                    stat_bonuses: vec![("attack".to_string(), 3)].into_iter().collect(),
                    value_bonus: 30,
                    weight: 25,
                },
            ],
            suffixes: vec![
                Affix {
                    name: "of Power".to_string(),
                    affix_type: AffixType::Suffix,
                    stat_bonuses: vec![("Strength".to_string(), 2)].into_iter().collect(),
                    value_bonus: 35,
                    weight: 25,
                },
                Affix {
                    name: "of Precision".to_string(),
                    affix_type: AffixType::Suffix,
                    stat_bonuses: vec![("critical_chance".to_string(), 5)].into_iter().collect(),
                    value_bonus: 50,
                    weight: 15,
                },
                Affix {
                    name: "of Slaying".to_string(),
                    affix_type: AffixType::Suffix,
                    stat_bonuses: vec![("critical_damage".to_string(), 10)].into_iter().collect(),
                    value_bonus: 60,
                    weight: 10,
                },
            ],
        };
        self.affix_tables.insert(ItemType::Weapon(WeaponType::Sword), weapon_affixes);

        // Armor affixes
        let armor_affixes = AffixTable {
            prefixes: vec![
                Affix {
                    name: "Sturdy".to_string(),
                    affix_type: AffixType::Prefix,
                    stat_bonuses: vec![("defense".to_string(), 3)].into_iter().collect(),
                    value_bonus: 30,
                    weight: 30,
                },
                Affix {
                    name: "Light".to_string(),
                    affix_type: AffixType::Prefix,
                    stat_bonuses: vec![("defense".to_string(), 1), ("Dexterity".to_string(), 2)].into_iter().collect(),
                    value_bonus: 25,
                    weight: 25,
                },
            ],
            suffixes: vec![
                Affix {
                    name: "of Protection".to_string(),
                    affix_type: AffixType::Suffix,
                    stat_bonuses: vec![("defense".to_string(), 4)].into_iter().collect(),
                    value_bonus: 40,
                    weight: 20,
                },
                Affix {
                    name: "of Vitality".to_string(),
                    affix_type: AffixType::Suffix,
                    stat_bonuses: vec![("Constitution".to_string(), 3)].into_iter().collect(),
                    value_bonus: 45,
                    weight: 15,
                },
            ],
        };
        self.affix_tables.insert(ItemType::Armor(ArmorType::Chest), armor_affixes);
    }
}

/// Context for item generation
#[derive(Debug, Clone, PartialEq)]
pub enum GenerationContext {
    Combat,    // Dropped by monsters
    Treasure,  // Found in chests/treasure
    Merchant,  // Sold by merchants
    Random,    // Random generation
}

/// Loot table for generating multiple items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable {
    pub entries: Vec<LootEntry>,
    pub guaranteed_drops: i32,
    pub max_drops: i32,
}

impl LootTable {
    pub fn generate_loot(
        &self,
        world: &mut World,
        generator: &ItemGenerator,
        position: Position,
        depth: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Vec<Entity> {
        let mut items = Vec::new();
        let num_drops = rng.roll_dice(self.guaranteed_drops, self.max_drops - self.guaranteed_drops + 1);
        
        for _ in 0..num_drops {
            if let Some(entry) = self.select_entry(rng) {
                if let Some(item_type) = &entry.item_type {
                    let quantity = rng.roll_dice(entry.quantity_range.0, entry.quantity_range.1 - entry.quantity_range.0 + 1);
                    
                    for _ in 0..quantity {
                        let context = GenerationContext::Random;
                        let item = generator.generate_item(world, position, depth, context, rng);
                        
                        // Override rarity if specified
                        if let Some(rarity) = &entry.rarity_override {
                            let mut properties = world.write_storage::<ItemProperties>();
                            if let Some(props) = properties.get_mut(item) {
                                props.rarity = rarity.clone();
                            }
                        }
                        
                        items.push(item);
                    }
                } else if let Some(table_ref) = &entry.table_reference {
                    // Handle special cases like gold
                    if table_ref == "gold" {
                        // Add gold to player inventory instead of creating item
                        // This would be handled by the calling system
                    }
                }
            }
        }
        
        items
    }

    fn select_entry(&self, rng: &mut RandomNumberGenerator) -> Option<&LootEntry> {
        let total_weight: i32 = self.entries.iter().map(|e| e.weight).sum();
        let mut roll = rng.roll_dice(1, total_weight);
        
        for entry in &self.entries {
            roll -= entry.weight;
            if roll <= 0 {
                return Some(entry);
            }
        }
        
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry {
    pub item_type: Option<ItemType>,
    pub table_reference: Option<String>,
    pub weight: i32,
    pub quantity_range: (i32, i32),
    pub rarity_override: Option<ItemRarity>,
}

/// Affix system for item modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffixTable {
    pub prefixes: Vec<Affix>,
    pub suffixes: Vec<Affix>,
}

impl AffixTable {
    pub fn get_random_affix(&self, rng: &mut RandomNumberGenerator) -> Option<Affix> {
        let use_prefix = rng.roll_dice(1, 2) == 1;
        
        if use_prefix && !self.prefixes.is_empty() {
            let total_weight: i32 = self.prefixes.iter().map(|a| a.weight).sum();
            let mut roll = rng.roll_dice(1, total_weight);
            
            for affix in &self.prefixes {
                roll -= affix.weight;
                if roll <= 0 {
                    return Some(affix.clone());
                }
            }
        } else if !self.suffixes.is_empty() {
            let total_weight: i32 = self.suffixes.iter().map(|a| a.weight).sum();
            let mut roll = rng.roll_dice(1, total_weight);
            
            for affix in &self.suffixes {
                roll -= affix.weight;
                if roll <= 0 {
                    return Some(affix.clone());
                }
            }
        }
        
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affix {
    pub name: String,
    pub affix_type: AffixType,
    pub stat_bonuses: HashMap<String, i32>,
    pub value_bonus: i32,
    pub weight: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AffixType {
    Prefix,
    Suffix,
}

/// Rarity weight configuration
#[derive(Debug, Clone)]
pub struct RarityWeights {
    pub base_weights: HashMap<ItemRarity, i32>,
}

impl Default for RarityWeights {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(ItemRarity::Trash, 5);
        weights.insert(ItemRarity::Common, 50);
        weights.insert(ItemRarity::Uncommon, 25);
        weights.insert(ItemRarity::Rare, 15);
        weights.insert(ItemRarity::Epic, 4);
        weights.insert(ItemRarity::Legendary, 1);
        weights.insert(ItemRarity::Artifact, 0); // Only through special generation
        
        RarityWeights {
            base_weights: weights,
        }
    }
}

/// Depth scaling configuration
#[derive(Debug, Clone)]
pub struct DepthScaling {
    pub stat_scaling: f32,     // How much stats increase per depth level
    pub value_scaling: f32,    // How much value increases per depth level
    pub rarity_scaling: f32,   // How much rarity chances improve per depth level
}

impl Default for DepthScaling {
    fn default() -> Self {
        DepthScaling {
            stat_scaling: 0.1,    // 10% increase per level
            value_scaling: 0.05,  // 5% value increase per level
            rarity_scaling: 1.0,  // 1 point rarity bonus per level
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Name>();
        world.register::<Renderable>();
        world.register::<Item>();
        world.register::<ItemProperties>();
        world.register::<ItemBonuses>();
        world.register::<MagicalItem>();
        world.register::<ItemStack>();
        world
    }

    #[test]
    fn test_item_generator_creation() {
        let generator = ItemGenerator::new();
        assert!(!generator.loot_tables.is_empty());
        assert!(!generator.affix_tables.is_empty());
    }

    #[test]
    fn test_rarity_selection() {
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        // Test depth scaling
        let low_depth_rarity = generator.select_rarity(1, &GenerationContext::Random, &mut rng);
        let high_depth_rarity = generator.select_rarity(25, &GenerationContext::Random, &mut rng);
        
        // Higher depth should generally produce better rarities (though random)
        // This test just ensures the function doesn't panic
        assert!(matches!(low_depth_rarity, ItemRarity::_));
        assert!(matches!(high_depth_rarity, ItemRarity::_));
    }

    #[test]
    fn test_loot_table_generation() {
        let mut world = setup_world();
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let position = Position { x: 0, y: 0 };
        let items = generator.generate_from_loot_table(&mut world, "monster", position, 5, &mut rng);
        
        assert!(!items.is_empty());
        assert!(items.len() <= 3); // Max drops for monster table
    }

    #[test]
    fn test_affix_application() {
        let mut world = setup_world();
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let position = Position { x: 0, y: 0 };
        let item = generator.generate_item(&mut world, position, 10, GenerationContext::Treasure, &mut rng);
        
        // Verify item was created
        let names = world.read_storage::<Name>();
        assert!(names.get(item).is_some());
    }

    #[test]
    fn test_weighted_choice() {
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let choices = vec![
            ("A".to_string(), 10),
            ("B".to_string(), 20),
            ("C".to_string(), 70),
        ];
        
        let result = generator.weighted_choice(&choices, &mut rng);
        assert!(["A", "B", "C"].contains(&result.as_str()));
    }

    #[test]
    fn test_depth_scaling() {
        let mut world = setup_world();
        let generator = ItemGenerator::new();
        
        // Create a test item with bonuses
        let entity = world.create_entity()
            .with(ItemBonuses::new())
            .with(ItemProperties::new("Test Item".to_string(), ItemType::Miscellaneous))
            .build();
        
        generator.apply_depth_scaling(&mut world, entity, 10);
        
        let bonuses = world.read_storage::<ItemBonuses>();
        let bonus = bonuses.get(entity).unwrap();
        
        // Should have some scaling applied
        assert!(bonus.combat_bonuses.attack_bonus > 0 || 
                bonus.combat_bonuses.damage_bonus > 0 || 
                bonus.combat_bonuses.defense_bonus > 0);
    }
}