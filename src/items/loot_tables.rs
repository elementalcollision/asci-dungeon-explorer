use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use specs::{World, Entity};
use crate::components::Position;
use crate::items::{ItemType, ItemRarity, WeaponType, ArmorType, ConsumableType, MaterialType, ToolType};
use crate::items::item_generation::{ItemGenerator, GenerationContext, LootTable, LootEntry};
use crate::resources::RandomNumberGenerator;

/// Comprehensive loot table manager
pub struct LootTableManager {
    pub tables: HashMap<String, LootTable>,
    pub monster_tables: HashMap<String, String>, // Monster type -> table name
    pub depth_tables: HashMap<i32, String>,      // Depth -> table name
    pub special_tables: HashMap<String, String>, // Special locations -> table name
}

impl LootTableManager {
    pub fn new() -> Self {
        let mut manager = LootTableManager {
            tables: HashMap::new(),
            monster_tables: HashMap::new(),
            depth_tables: HashMap::new(),
            special_tables: HashMap::new(),
        };
        
        manager.initialize_default_tables();
        manager.setup_monster_mappings();
        manager.setup_depth_mappings();
        manager.setup_special_mappings();
        
        manager
    }

    fn initialize_default_tables(&mut self) {
        // Basic monster loot
        self.add_table("goblin", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Potion)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Dagger)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Bone)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 2),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("small_gold".to_string()),
                    weight: 25,
                    quantity_range: (3, 8),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 2,
        });

        // Skeleton loot
        self.add_table("skeleton", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Sword)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Shield)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Bone)),
                    table_reference: None,
                    weight: 35,
                    quantity_range: (2, 4),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Scroll)),
                    table_reference: None,
                    weight: 15,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("small_gold".to_string()),
                    weight: 5,
                    quantity_range: (1, 5),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 3,
        });

        // Orc loot
        self.add_table("orc", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Axe)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Chest)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Food)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 3),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Metal)),
                    table_reference: None,
                    weight: 15,
                    quantity_range: (1, 2),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("medium_gold".to_string()),
                    weight: 10,
                    quantity_range: (5, 15),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 3,
        });

        // Dragon loot (boss)
        self.add_table("dragon", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Sword)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Legendary),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Chest)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Epic),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Gem)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (3, 8),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Scroll)),
                    table_reference: None,
                    weight: 15,
                    quantity_range: (2, 4),
                    rarity_override: Some(ItemRarity::Epic),
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("large_gold".to_string()),
                    weight: 20,
                    quantity_range: (100, 500),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 3,
            max_drops: 6,
        });

        // Treasure chests by quality
        self.add_table("wooden_chest", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Potion)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 3),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Tool(ToolType::Lockpick)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Wood)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (2, 5),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("small_gold".to_string()),
                    weight: 25,
                    quantity_range: (10, 30),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 3,
        });

        self.add_table("iron_chest", LootTable {
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
                    item_type: Some(ItemType::Consumable(ConsumableType::Potion)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (2, 4),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Metal)),
                    table_reference: None,
                    weight: 15,
                    quantity_range: (3, 6),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("medium_gold".to_string()),
                    weight: 15,
                    quantity_range: (25, 75),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 2,
            max_drops: 4,
        });

        self.add_table("golden_chest", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Sword)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Chest)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Gem)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (2, 5),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Scroll)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 3),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("large_gold".to_string()),
                    weight: 15,
                    quantity_range: (50, 200),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 2,
            max_drops: 5,
        });

        // Depth-based general loot
        self.add_table("depth_1_5", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Dagger)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Boots)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Common),
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Potion)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 2),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Tool(ToolType::Torch)),
                    table_reference: None,
                    weight: 15,
                    quantity_range: (1, 3),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("small_gold".to_string()),
                    weight: 10,
                    quantity_range: (1, 10),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 2,
        });

        self.add_table("depth_6_10", LootTable {
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
                    item_type: Some(ItemType::Consumable(ConsumableType::Potion)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (2, 3),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Metal)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (2, 4),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("medium_gold".to_string()),
                    weight: 10,
                    quantity_range: (10, 25),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 1,
            max_drops: 3,
        });

        self.add_table("depth_11_20", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Sword)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Chest)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Scroll)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 2),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Gem)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 3),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("large_gold".to_string()),
                    weight: 10,
                    quantity_range: (25, 75),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 2,
            max_drops: 4,
        });

        // Special location loot
        self.add_table("library", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Scroll)),
                    table_reference: None,
                    weight: 50,
                    quantity_range: (2, 5),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Staff)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Herb)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (3, 6),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("knowledge".to_string()),
                    weight: 10,
                    quantity_range: (1, 1),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 2,
            max_drops: 4,
        });

        self.add_table("armory", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Sword)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 2),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Chest)),
                    table_reference: None,
                    weight: 30,
                    quantity_range: (1, 2),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Weapon(WeaponType::Bow)),
                    table_reference: None,
                    weight: 20,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Uncommon),
                },
                LootEntry {
                    item_type: Some(ItemType::Consumable(ConsumableType::Ammunition)),
                    table_reference: None,
                    weight: 15,
                    quantity_range: (10, 30),
                    rarity_override: None,
                },
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Metal)),
                    table_reference: None,
                    weight: 5,
                    quantity_range: (5, 10),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 2,
            max_drops: 5,
        });

        self.add_table("treasury", LootTable {
            entries: vec![
                LootEntry {
                    item_type: Some(ItemType::Material(MaterialType::Gem)),
                    table_reference: None,
                    weight: 40,
                    quantity_range: (3, 8),
                    rarity_override: Some(ItemRarity::Rare),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Ring)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 2),
                    rarity_override: Some(ItemRarity::Epic),
                },
                LootEntry {
                    item_type: Some(ItemType::Armor(ArmorType::Amulet)),
                    table_reference: None,
                    weight: 25,
                    quantity_range: (1, 1),
                    rarity_override: Some(ItemRarity::Epic),
                },
                LootEntry {
                    item_type: None,
                    table_reference: Some("huge_gold".to_string()),
                    weight: 10,
                    quantity_range: (200, 1000),
                    rarity_override: None,
                },
            ],
            guaranteed_drops: 3,
            max_drops: 6,
        });
    }

    fn setup_monster_mappings(&mut self) {
        self.monster_tables.insert("Goblin".to_string(), "goblin".to_string());
        self.monster_tables.insert("Skeleton".to_string(), "skeleton".to_string());
        self.monster_tables.insert("Orc".to_string(), "orc".to_string());
        self.monster_tables.insert("Dragon".to_string(), "dragon".to_string());
        
        // Add more monster mappings
        self.monster_tables.insert("Rat".to_string(), "goblin".to_string()); // Reuse goblin table
        self.monster_tables.insert("Spider".to_string(), "goblin".to_string());
        self.monster_tables.insert("Zombie".to_string(), "skeleton".to_string()); // Reuse skeleton table
        self.monster_tables.insert("Troll".to_string(), "orc".to_string()); // Reuse orc table
    }

    fn setup_depth_mappings(&mut self) {
        for depth in 1..=5 {
            self.depth_tables.insert(depth, "depth_1_5".to_string());
        }
        for depth in 6..=10 {
            self.depth_tables.insert(depth, "depth_6_10".to_string());
        }
        for depth in 11..=20 {
            self.depth_tables.insert(depth, "depth_11_20".to_string());
        }
        // Depths beyond 20 use the highest tier table
        for depth in 21..=50 {
            self.depth_tables.insert(depth, "depth_11_20".to_string());
        }
    }

    fn setup_special_mappings(&mut self) {
        self.special_tables.insert("Library".to_string(), "library".to_string());
        self.special_tables.insert("Armory".to_string(), "armory".to_string());
        self.special_tables.insert("Treasury".to_string(), "treasury".to_string());
        self.special_tables.insert("Altar".to_string(), "library".to_string()); // Reuse library for now
        self.special_tables.insert("Forge".to_string(), "armory".to_string()); // Reuse armory
    }

    pub fn add_table(&mut self, name: &str, table: LootTable) {
        self.tables.insert(name.to_string(), table);
    }

    pub fn get_table(&self, name: &str) -> Option<&LootTable> {
        self.tables.get(name)
    }

    pub fn get_monster_loot_table(&self, monster_name: &str) -> Option<&LootTable> {
        if let Some(table_name) = self.monster_tables.get(monster_name) {
            self.tables.get(table_name)
        } else {
            // Default to goblin table for unknown monsters
            self.tables.get("goblin")
        }
    }

    pub fn get_depth_loot_table(&self, depth: i32) -> Option<&LootTable> {
        if let Some(table_name) = self.depth_tables.get(&depth) {
            self.tables.get(table_name)
        } else {
            // For very deep levels, use the highest tier
            self.tables.get("depth_11_20")
        }
    }

    pub fn get_special_loot_table(&self, location_name: &str) -> Option<&LootTable> {
        if let Some(table_name) = self.special_tables.get(location_name) {
            self.tables.get(table_name)
        } else {
            None
        }
    }

    /// Generate loot for a specific monster
    pub fn generate_monster_loot(
        &self,
        world: &mut World,
        generator: &ItemGenerator,
        monster_name: &str,
        position: Position,
        depth: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Vec<Entity> {
        if let Some(table) = self.get_monster_loot_table(monster_name) {
            table.generate_loot(world, generator, position, depth, rng)
        } else {
            Vec::new()
        }
    }

    /// Generate loot for a container based on its type
    pub fn generate_container_loot(
        &self,
        world: &mut World,
        generator: &ItemGenerator,
        container_type: &str,
        position: Position,
        depth: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Vec<Entity> {
        let table_name = match container_type.to_lowercase().as_str() {
            "wooden_chest" | "chest" => "wooden_chest",
            "iron_chest" | "metal_chest" => "iron_chest",
            "golden_chest" | "gold_chest" | "treasure_chest" => "golden_chest",
            _ => "wooden_chest", // Default
        };

        if let Some(table) = self.tables.get(table_name) {
            table.generate_loot(world, generator, position, depth, rng)
        } else {
            Vec::new()
        }
    }

    /// Generate random loot based on depth
    pub fn generate_depth_loot(
        &self,
        world: &mut World,
        generator: &ItemGenerator,
        position: Position,
        depth: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Vec<Entity> {
        if let Some(table) = self.get_depth_loot_table(depth) {
            table.generate_loot(world, generator, position, depth, rng)
        } else {
            Vec::new()
        }
    }

    /// Generate loot for special locations
    pub fn generate_special_loot(
        &self,
        world: &mut World,
        generator: &ItemGenerator,
        location_name: &str,
        position: Position,
        depth: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Vec<Entity> {
        if let Some(table) = self.get_special_loot_table(location_name) {
            table.generate_loot(world, generator, position, depth, rng)
        } else {
            // Fallback to depth-based loot
            self.generate_depth_loot(world, generator, position, depth, rng)
        }
    }

    /// Save loot tables to file
    pub fn save_to_file(&self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let data = LootTableData {
            tables: self.tables.clone(),
            monster_tables: self.monster_tables.clone(),
            depth_tables: self.depth_tables.clone(),
            special_tables: self.special_tables.clone(),
        };
        
        let json = serde_json::to_string_pretty(&data)?;
        std::fs::write(filename, json)?;
        Ok(())
    }

    /// Load loot tables from file
    pub fn load_from_file(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filename)?;
        let data: LootTableData = serde_json::from_str(&json)?;
        
        Ok(LootTableManager {
            tables: data.tables,
            monster_tables: data.monster_tables,
            depth_tables: data.depth_tables,
            special_tables: data.special_tables,
        })
    }

    /// Get statistics about loot tables
    pub fn get_statistics(&self) -> LootTableStatistics {
        let total_tables = self.tables.len();
        let total_entries: usize = self.tables.values().map(|t| t.entries.len()).sum();
        let monster_mappings = self.monster_tables.len();
        let depth_mappings = self.depth_tables.len();
        let special_mappings = self.special_tables.len();

        LootTableStatistics {
            total_tables,
            total_entries,
            monster_mappings,
            depth_mappings,
            special_mappings,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LootTableData {
    tables: HashMap<String, LootTable>,
    monster_tables: HashMap<String, String>,
    depth_tables: HashMap<i32, String>,
    special_tables: HashMap<String, String>,
}

pub struct LootTableStatistics {
    pub total_tables: usize,
    pub total_entries: usize,
    pub monster_mappings: usize,
    pub depth_mappings: usize,
    pub special_mappings: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};
    use crate::items::item_generation::ItemGenerator;

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<crate::components::Name>();
        world.register::<crate::components::Renderable>();
        world.register::<crate::components::Item>();
        world.register::<crate::items::ItemProperties>();
        world.register::<crate::items::ItemBonuses>();
        world.register::<crate::items::MagicalItem>();
        world.register::<crate::items::ItemStack>();
        world
    }

    #[test]
    fn test_loot_table_manager_creation() {
        let manager = LootTableManager::new();
        assert!(!manager.tables.is_empty());
        assert!(!manager.monster_tables.is_empty());
        assert!(!manager.depth_tables.is_empty());
    }

    #[test]
    fn test_monster_loot_generation() {
        let mut world = setup_world();
        let manager = LootTableManager::new();
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let position = Position { x: 0, y: 0 };
        let loot = manager.generate_monster_loot(&mut world, &generator, "Goblin", position, 5, &mut rng);
        
        assert!(!loot.is_empty());
    }

    #[test]
    fn test_container_loot_generation() {
        let mut world = setup_world();
        let manager = LootTableManager::new();
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let position = Position { x: 0, y: 0 };
        let loot = manager.generate_container_loot(&mut world, &generator, "wooden_chest", position, 5, &mut rng);
        
        assert!(!loot.is_empty());
    }

    #[test]
    fn test_depth_scaling() {
        let manager = LootTableManager::new();
        
        // Test depth mappings
        assert!(manager.get_depth_loot_table(3).is_some());
        assert!(manager.get_depth_loot_table(8).is_some());
        assert!(manager.get_depth_loot_table(15).is_some());
        assert!(manager.get_depth_loot_table(25).is_some()); // Should fallback to highest tier
    }

    #[test]
    fn test_special_location_loot() {
        let mut world = setup_world();
        let manager = LootTableManager::new();
        let generator = ItemGenerator::new();
        let mut rng = RandomNumberGenerator::new();
        
        let position = Position { x: 0, y: 0 };
        let loot = manager.generate_special_loot(&mut world, &generator, "Library", position, 10, &mut rng);
        
        assert!(!loot.is_empty());
    }

    #[test]
    fn test_statistics() {
        let manager = LootTableManager::new();
        let stats = manager.get_statistics();
        
        assert!(stats.total_tables > 0);
        assert!(stats.total_entries > 0);
        assert!(stats.monster_mappings > 0);
        assert!(stats.depth_mappings > 0);
    }
}