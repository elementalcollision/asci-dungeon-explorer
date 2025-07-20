// Complete integration example for the item generation system

use specs::{World, WorldExt, Builder, Entity};
use crate::components::{Position, Name, Renderable, Item};
use crate::items::{
    ItemGenerator, ItemNameGenerator, LootTableManager, GenerationContext,
    ItemProperties, ItemType, ItemRarity, WeaponType, ArmorType, ConsumableType
};
use crate::resources::RandomNumberGenerator;

/// Complete example showing how to integrate the item generation system
pub struct ItemGenerationIntegration {
    pub item_generator: ItemGenerator,
    pub name_generator: ItemNameGenerator,
    pub loot_manager: LootTableManager,
}

impl ItemGenerationIntegration {
    pub fn new() -> Self {
        ItemGenerationIntegration {
            item_generator: ItemGenerator::new(),
            name_generator: ItemNameGenerator::new(),
            loot_manager: LootTableManager::new(),
        }
    }

    /// Set up the world with all necessary components
    pub fn setup_world(&self) -> World {
        let mut world = World::new();

        // Register all necessary components
        world.register::<Position>();
        world.register::<Name>();
        world.register::<Renderable>();
        world.register::<Item>();
        world.register::<ItemProperties>();
        world.register::<crate::items::ItemStack>();
        world.register::<crate::items::ItemBonuses>();
        world.register::<crate::items::MagicalItem>();

        world
    }

    /// Demonstrate basic item generation
    pub fn demonstrate_basic_generation(&self, world: &mut World) {
        println!("=== BASIC ITEM GENERATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 0, y: 0 };

        // Generate items of different rarities
        let rarities = vec![
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::Epic,
            ItemRarity::Legendary,
        ];

        for rarity in rarities {
            println!("Generating {} items:", rarity.name());
            
            for _ in 0..3 {
                let item = self.item_generator.generate_item(
                    world,
                    position,
                    10, // depth
                    GenerationContext::Random,
                    &mut rng,
                );

                // Generate appropriate name
                let item_type = {
                    let properties = world.read_storage::<ItemProperties>();
                    properties.get(item).map(|p| p.item_type.clone())
                };

                if let Some(item_type) = item_type {
                    let has_enchantments = world.read_storage::<crate::items::MagicalItem>()
                        .get(item).is_some();
                    
                    let generated_name = self.name_generator.generate_name(
                        &item_type,
                        &rarity,
                        has_enchantments,
                        &mut rng,
                    );

                    // Update the item's name
                    let mut names = world.write_storage::<Name>();
                    if let Some(name) = names.get_mut(item) {
                        name.name = generated_name.clone();
                    }

                    println!("  - {}", generated_name);
                }
            }
            println!();
        }
    }

    /// Demonstrate depth-based scaling
    pub fn demonstrate_depth_scaling(&self, world: &mut World) {
        println!("=== DEPTH SCALING DEMONSTRATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 0, y: 0 };
        let depths = vec![1, 5, 10, 15, 20, 25];

        for depth in depths {
            println!("Depth {} items:", depth);
            
            for _ in 0..2 {
                let item = self.item_generator.generate_item(
                    world,
                    position,
                    depth,
                    GenerationContext::Random,
                    &mut rng,
                );

                // Show item stats
                let properties = world.read_storage::<ItemProperties>();
                let bonuses = world.read_storage::<crate::items::ItemBonuses>();
                let names = world.read_storage::<Name>();

                if let (Some(props), Some(name)) = (properties.get(item), names.get(item)) {
                    print!("  {} ({})", name.name, props.rarity.name());
                    
                    if let Some(bonus) = bonuses.get(item) {
                        if bonus.combat_bonuses.attack_bonus > 0 {
                            print!(" +{} Attack", bonus.combat_bonuses.attack_bonus);
                        }
                        if bonus.combat_bonuses.damage_bonus > 0 {
                            print!(" +{} Damage", bonus.combat_bonuses.damage_bonus);
                        }
                        if bonus.combat_bonuses.defense_bonus > 0 {
                            print!(" +{} Defense", bonus.combat_bonuses.defense_bonus);
                        }
                    }
                    
                    println!(" - {} gold", props.value);
                }
            }
            println!();
        }
    }

    /// Demonstrate loot table generation
    pub fn demonstrate_loot_tables(&self, world: &mut World) {
        println!("=== LOOT TABLE DEMONSTRATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 0, y: 0 };

        // Monster loot
        let monsters = vec!["Goblin", "Skeleton", "Orc", "Dragon"];
        for monster in monsters {
            println!("{}:", monster);
            let loot = self.loot_manager.generate_monster_loot(
                world,
                &self.item_generator,
                monster,
                position,
                10,
                &mut rng,
            );

            for item_entity in loot {
                let names = world.read_storage::<Name>();
                if let Some(name) = names.get(item_entity) {
                    println!("  - {}", name.name);
                }
            }
            println!();
        }

        // Container loot
        let containers = vec!["wooden_chest", "iron_chest", "golden_chest"];
        for container in containers {
            println!("{}:", container);
            let loot = self.loot_manager.generate_container_loot(
                world,
                &self.item_generator,
                container,
                position,
                10,
                &mut rng,
            );

            for item_entity in loot {
                let names = world.read_storage::<Name>();
                if let Some(name) = names.get(item_entity) {
                    println!("  - {}", name.name);
                }
            }
            println!();
        }

        // Special location loot
        let locations = vec!["Library", "Armory", "Treasury"];
        for location in locations {
            println!("{}:", location);
            let loot = self.loot_manager.generate_special_loot(
                world,
                &self.item_generator,
                location,
                position,
                15,
                &mut rng,
            );

            for item_entity in loot {
                let names = world.read_storage::<Name>();
                if let Some(name) = names.get(item_entity) {
                    println!("  - {}", name.name);
                }
            }
            println!();
        }
    }

    /// Demonstrate name generation
    pub fn demonstrate_name_generation(&self) {
        println!("=== NAME GENERATION DEMONSTRATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let item_types = vec![
            ItemType::Weapon(WeaponType::Sword),
            ItemType::Armor(ArmorType::Chest),
            ItemType::Consumable(ConsumableType::Potion),
        ];

        let rarities = vec![
            ItemRarity::Common,
            ItemRarity::Uncommon,
            ItemRarity::Rare,
            ItemRarity::Epic,
            ItemRarity::Legendary,
            ItemRarity::Artifact,
        ];

        for item_type in &item_types {
            println!("{:?} names:", item_type);
            
            for rarity in &rarities {
                let name = self.name_generator.generate_name(
                    item_type,
                    rarity,
                    rarity >= &ItemRarity::Uncommon,
                    &mut rng,
                );
                println!("  {} - {}", rarity.name(), name);
            }
            println!();
        }
    }

    /// Demonstrate affix system
    pub fn demonstrate_affix_system(&self, world: &mut World) {
        println!("=== AFFIX SYSTEM DEMONSTRATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 0, y: 0 };

        // Generate items with different numbers of affixes
        for rarity in vec![ItemRarity::Uncommon, ItemRarity::Rare, ItemRarity::Epic, ItemRarity::Legendary] {
            println!("{} items with affixes:", rarity.name());
            
            for _ in 0..3 {
                let item = self.item_generator.generate_item(
                    world,
                    position,
                    15, // Higher depth for better affixes
                    GenerationContext::Treasure,
                    &mut rng,
                );

                let names = world.read_storage::<Name>();
                let properties = world.read_storage::<ItemProperties>();
                let bonuses = world.read_storage::<crate::items::ItemBonuses>();

                if let (Some(name), Some(props)) = (names.get(item), properties.get(item)) {
                    print!("  {} ({})", name.name, props.rarity.name());
                    
                    if let Some(bonus) = bonuses.get(item) {
                        let mut stat_bonuses = Vec::new();
                        
                        if bonus.combat_bonuses.attack_bonus != 0 {
                            stat_bonuses.push(format!("Attack: {:+}", bonus.combat_bonuses.attack_bonus));
                        }
                        if bonus.combat_bonuses.damage_bonus != 0 {
                            stat_bonuses.push(format!("Damage: {:+}", bonus.combat_bonuses.damage_bonus));
                        }
                        if bonus.combat_bonuses.defense_bonus != 0 {
                            stat_bonuses.push(format!("Defense: {:+}", bonus.combat_bonuses.defense_bonus));
                        }
                        
                        for (attr, value) in &bonus.attribute_bonuses {
                            stat_bonuses.push(format!("{}: {:+}", attr, value));
                        }
                        
                        if !stat_bonuses.is_empty() {
                            print!(" [{}]", stat_bonuses.join(", "));
                        }
                    }
                    
                    println!();
                }
            }
            println!();
        }
    }

    /// Demonstrate context-based generation
    pub fn demonstrate_context_generation(&self, world: &mut World) {
        println!("=== CONTEXT-BASED GENERATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 0, y: 0 };
        let contexts = vec![
            GenerationContext::Combat,
            GenerationContext::Treasure,
            GenerationContext::Merchant,
            GenerationContext::Random,
        ];

        for context in contexts {
            println!("{:?} context items:", context);
            
            for _ in 0..5 {
                let item = self.item_generator.generate_item(
                    world,
                    position,
                    10,
                    context.clone(),
                    &mut rng,
                );

                let names = world.read_storage::<Name>();
                let properties = world.read_storage::<ItemProperties>();

                if let (Some(name), Some(props)) = (names.get(item), properties.get(item)) {
                    println!("  {} ({:?})", name.name, props.item_type);
                }
            }
            println!();
        }
    }

    /// Show system statistics
    pub fn show_statistics(&self) {
        println!("=== SYSTEM STATISTICS ===\n");

        let loot_stats = self.loot_manager.get_statistics();
        println!("Loot Table Statistics:");
        println!("  Total tables: {}", loot_stats.total_tables);
        println!("  Total entries: {}", loot_stats.total_entries);
        println!("  Monster mappings: {}", loot_stats.monster_mappings);
        println!("  Depth mappings: {}", loot_stats.depth_mappings);
        println!("  Special mappings: {}", loot_stats.special_mappings);
        println!();

        println!("Item Generator Features:");
        println!("  - Procedural item generation");
        println!("  - Depth-based scaling");
        println!("  - Rarity-based modifications");
        println!("  - Affix system with prefixes and suffixes");
        println!("  - Context-aware generation");
        println!("  - Magical item creation");
        println!();

        println!("Name Generator Features:");
        println!("  - Rarity-appropriate naming");
        println!("  - Type-specific base names");
        println!("  - Magical affixes");
        println!("  - Legendary and artifact names");
        println!("  - Contextual name generation");
        println!();
    }

    /// Run complete demonstration
    pub fn run_complete_demonstration(&self) {
        println!("=== ITEM GENERATION SYSTEM DEMONSTRATION ===\n");

        let mut world = self.setup_world();

        self.demonstrate_basic_generation(&mut world);
        self.demonstrate_depth_scaling(&mut world);
        self.demonstrate_loot_tables(&mut world);
        self.demonstrate_name_generation();
        self.demonstrate_affix_system(&mut world);
        self.demonstrate_context_generation(&mut world);
        self.show_statistics();

        println!("=== DEMONSTRATION COMPLETE ===");
    }

    /// Example of how to integrate into a game system
    pub fn example_dungeon_population(&self, world: &mut World, dungeon_depth: i32) -> Vec<Entity> {
        let mut rng = RandomNumberGenerator::new();
        let mut generated_items = Vec::new();

        // Generate random loot scattered throughout the dungeon
        for x in 0..20 {
            for y in 0..20 {
                if rng.roll_dice(1, 100) <= 5 { // 5% chance per tile
                    let position = Position { x, y };
                    let item = self.item_generator.generate_item(
                        world,
                        position,
                        dungeon_depth,
                        GenerationContext::Random,
                        &mut rng,
                    );
                    generated_items.push(item);
                }
            }
        }

        // Generate treasure chests
        for _ in 0..3 {
            let x = rng.roll_dice(1, 20) - 1;
            let y = rng.roll_dice(1, 20) - 1;
            let position = Position { x, y };
            
            let chest_loot = self.loot_manager.generate_container_loot(
                world,
                &self.item_generator,
                "iron_chest",
                position,
                dungeon_depth,
                &mut rng,
            );
            generated_items.extend(chest_loot);
        }

        // Generate monster loot (simulated)
        let monsters = vec!["Goblin", "Skeleton", "Orc"];
        for _ in 0..5 {
            let monster = &monsters[rng.roll_dice(1, monsters.len()) - 1];
            let x = rng.roll_dice(1, 20) - 1;
            let y = rng.roll_dice(1, 20) - 1;
            let position = Position { x, y };
            
            let monster_loot = self.loot_manager.generate_monster_loot(
                world,
                &self.item_generator,
                monster,
                position,
                dungeon_depth,
                &mut rng,
            );
            generated_items.extend(monster_loot);
        }

        println!("Generated {} items for dungeon depth {}", generated_items.len(), dungeon_depth);
        generated_items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_setup() {
        let integration = ItemGenerationIntegration::new();
        let world = integration.setup_world();
        
        // Verify world is set up correctly
        assert!(world.has_component::<Position>());
        assert!(world.has_component::<Name>());
        assert!(world.has_component::<Item>());
    }

    #[test]
    fn test_dungeon_population() {
        let integration = ItemGenerationIntegration::new();
        let mut world = integration.setup_world();
        
        let items = integration.example_dungeon_population(&mut world, 5);
        assert!(!items.is_empty());
        
        // Verify items were created properly
        let names = world.read_storage::<Name>();
        let properties = world.read_storage::<ItemProperties>();
        
        for &item in &items {
            assert!(names.get(item).is_some());
            assert!(properties.get(item).is_some());
        }
    }

    #[test]
    fn test_generation_contexts() {
        let integration = ItemGenerationIntegration::new();
        let mut world = integration.setup_world();
        let mut rng = RandomNumberGenerator::new();
        
        let contexts = vec![
            GenerationContext::Combat,
            GenerationContext::Treasure,
            GenerationContext::Merchant,
            GenerationContext::Random,
        ];
        
        for context in contexts {
            let item = integration.item_generator.generate_item(
                &mut world,
                Position { x: 0, y: 0 },
                10,
                context,
                &mut rng,
            );
            
            let names = world.read_storage::<Name>();
            assert!(names.get(item).is_some());
        }
    }
}