// Complete integration example for the consumable system

use specs::{World, WorldExt, Builder, Entity, Join};
use crate::components::{Position, Name, Player, CombatStats};
use crate::items::{
    ConsumableFactory, Consumable, ConsumableUsageSystem, ConsumableUpdateSystem,
    WantsToUseConsumable, ConsumableCooldowns, StatusEffects, PotionPotency,
    FoodType, ScrollType, ConsumableContext, StatusEffectType
};
use crate::resources::{GameLog, RandomNumberGenerator};

/// Complete example showing how to integrate the consumable system
pub struct ConsumableIntegration {
    pub factory: ConsumableFactory,
}

impl ConsumableIntegration {
    pub fn new() -> Self {
        ConsumableIntegration {
            factory: ConsumableFactory::new(),
        }
    }

    /// Set up the world with all necessary components
    pub fn setup_world(&self) -> World {
        let mut world = World::new();

        // Register all necessary components
        world.register::<Position>();
        world.register::<Name>();
        world.register::<Player>();
        world.register::<CombatStats>();
        world.register::<crate::components::Item>();
        world.register::<crate::items::ItemProperties>();
        world.register::<crate::items::ItemStack>();
        world.register::<crate::components::Renderable>();

        // Consumable-specific components
        world.register::<Consumable>();
        world.register::<WantsToUseConsumable>();
        world.register::<ConsumableCooldowns>();
        world.register::<StatusEffects>();

        // Add resources
        world.insert(GameLog::new());
        world.insert(RandomNumberGenerator::new());
        world.insert(0.016f32); // Delta time (60 FPS)

        world
    }

    /// Create a test player with some consumables
    pub fn create_test_player(&self, world: &mut World) -> Entity {
        let player = world.create_entity()
            .with(Position { x: 5, y: 5 })
            .with(Player)
            .with(Name { name: "Player".to_string() })
            .with(CombatStats { max_hp: 100, hp: 50, defense: 5, power: 10 })
            .with(ConsumableCooldowns::new())
            .with(StatusEffects::new())
            .build();

        // Create some consumables for the player
        self.create_test_consumables(world);

        player
    }

    fn create_test_consumables(&self, world: &mut World) {
        let positions = vec![
            Position { x: 4, y: 5 }, // Left of player
            Position { x: 6, y: 5 }, // Right of player
            Position { x: 5, y: 4 }, // Above player
            Position { x: 5, y: 6 }, // Below player
            Position { x: 3, y: 5 }, // Further left
        ];

        // Create different types of consumables
        self.factory.create_health_potion(world, positions[0], PotionPotency::Lesser);
        self.factory.create_mana_potion(world, positions[1], PotionPotency::Lesser);
        self.factory.create_food(world, positions[2], FoodType::Bread);
        self.factory.create_scroll(world, positions[3], ScrollType::Healing);
        self.factory.create_emergency_potion(world, positions[4]);
    }

    /// Demonstrate basic consumable usage
    pub fn demonstrate_basic_usage(&self, world: &mut World, player: Entity) {
        println!("=== BASIC CONSUMABLE USAGE ===\n");

        // Find a health potion
        let entities = world.entities();
        let consumables = world.read_storage::<Consumable>();
        let names = world.read_storage::<Name>();

        let mut health_potion = None;
        for (entity, consumable, name) in (&entities, &consumables, &names).join() {
            if name.name.contains("Health Potion") {
                health_potion = Some(entity);
                break;
            }
        }

        if let Some(potion) = health_potion {
            println!("Player attempts to use health potion...");
            
            // Show player's current health
            let combat_stats = world.read_storage::<CombatStats>();
            if let Some(stats) = combat_stats.get(player) {
                println!("Player health before: {}/{}", stats.hp, stats.max_hp);
            }

            // Create usage intent
            world.write_storage::<WantsToUseConsumable>()
                .insert(player, WantsToUseConsumable { item: potion, target: None })
                .expect("Failed to insert consumable usage intent");

            // Run the consumable usage system
            let mut usage_system = ConsumableUsageSystem;
            usage_system.run_now(world);
            world.maintain();

            // Show player's health after
            let combat_stats = world.read_storage::<CombatStats>();
            if let Some(stats) = combat_stats.get(player) {
                println!("Player health after: {}/{}", stats.hp, stats.max_hp);
            }

            // Show game log
            let gamelog = world.read_resource::<GameLog>();
            for entry in gamelog.entries.iter().rev().take(3) {
                println!("Log: {}", entry);
            }
        }
        println!();
    }

    /// Demonstrate status effects
    pub fn demonstrate_status_effects(&self, world: &mut World, player: Entity) {
        println!("=== STATUS EFFECTS DEMONSTRATION ===\n");

        // Create a regeneration potion
        let regen_potion = self.factory.create_regeneration_potion(
            world,
            Position { x: 7, y: 5 },
            30.0, // 30 seconds
            3,    // 3 HP per second
        );

        println!("Player uses regeneration potion...");

        // Use the potion
        world.write_storage::<WantsToUseConsumable>()
            .insert(player, WantsToUseConsumable { item: regen_potion, target: None })
            .expect("Failed to insert consumable usage intent");

        let mut usage_system = ConsumableUsageSystem;
        usage_system.run_now(world);
        world.maintain();

        // Check status effects
        let status_effects = world.read_storage::<StatusEffects>();
        if let Some(effects) = status_effects.get(player) {
            if effects.has_effect(&StatusEffectType::Regeneration) {
                println!("Regeneration effect applied!");
                if let Some(effect) = effects.get_effect(&StatusEffectType::Regeneration) {
                    println!("Duration: {:.1}s, Power: {}", effect.duration, effect.power);
                }
            }
        }

        // Simulate time passing and status effect ticks
        println!("\nSimulating 5 seconds of regeneration...");
        let mut update_system = ConsumableUpdateSystem;
        
        for i in 1..=5 {
            // Update with 1 second delta time
            *world.write_resource::<f32>() = 1.0;
            update_system.run_now(world);
            world.maintain();

            let combat_stats = world.read_storage::<CombatStats>();
            if let Some(stats) = combat_stats.get(player) {
                println!("After {}s - Health: {}/{}", i, stats.hp, stats.max_hp);
            }
        }
        println!();
    }

    /// Demonstrate cooldown system
    pub fn demonstrate_cooldowns(&self, world: &mut World, player: Entity) {
        println!("=== COOLDOWN SYSTEM DEMONSTRATION ===\n");

        // Create two health potions
        let potion1 = self.factory.create_health_potion(world, Position { x: 8, y: 5 }, PotionPotency::Lesser);
        let potion2 = self.factory.create_health_potion(world, Position { x: 9, y: 5 }, PotionPotency::Lesser);

        println!("Player uses first health potion...");
        
        // Use first potion
        world.write_storage::<WantsToUseConsumable>()
            .insert(player, WantsToUseConsumable { item: potion1, target: None })
            .expect("Failed to insert consumable usage intent");

        let mut usage_system = ConsumableUsageSystem;
        usage_system.run_now(world);
        world.maintain();

        // Check cooldowns
        let cooldowns = world.read_storage::<ConsumableCooldowns>();
        if let Some(cd) = cooldowns.get(player) {
            let potion_cooldown = cd.get_cooldown("Potion");
            println!("Potion cooldown: {:.1}s", potion_cooldown);
        }

        println!("Player immediately tries to use second potion...");
        
        // Try to use second potion immediately
        world.write_storage::<WantsToUseConsumable>()
            .insert(player, WantsToUseConsumable { item: potion2, target: None })
            .expect("Failed to insert consumable usage intent");

        usage_system.run_now(world);
        world.maintain();

        // Show game log for cooldown message
        let gamelog = world.read_resource::<GameLog>();
        for entry in gamelog.entries.iter().rev().take(2) {
            println!("Log: {}", entry);
        }

        // Simulate time passing to clear cooldown
        println!("\nWaiting for cooldown to expire...");
        *world.write_resource::<f32>() = 3.0; // 3 seconds
        let mut update_system = ConsumableUpdateSystem;
        update_system.run_now(world);
        world.maintain();

        // Try again
        println!("Player tries to use second potion after cooldown...");
        world.write_storage::<WantsToUseConsumable>()
            .insert(player, WantsToUseConsumable { item: potion2, target: None })
            .expect("Failed to insert consumable usage intent");

        usage_system.run_now(world);
        world.maintain();

        let gamelog = world.read_resource::<GameLog>();
        println!("Log: {}", gamelog.entries.last().unwrap_or(&"No log entry".to_string()));
        println!();
    }

    /// Demonstrate restricted consumables
    pub fn demonstrate_restrictions(&self, world: &mut World, player: Entity) {
        println!("=== CONSUMABLE RESTRICTIONS DEMONSTRATION ===\n");

        // Find the emergency potion (has health threshold restriction)
        let entities = world.entities();
        let consumables = world.read_storage::<Consumable>();
        let names = world.read_storage::<Name>();

        let mut emergency_potion = None;
        for (entity, _consumable, name) in (&entities, &consumables, &names).join() {
            if name.name.contains("Emergency") {
                emergency_potion = Some(entity);
                break;
            }
        }

        if let Some(potion) = emergency_potion {
            // Set player health to high (above threshold)
            {
                let mut combat_stats = world.write_storage::<CombatStats>();
                if let Some(stats) = combat_stats.get_mut(player) {
                    stats.hp = 80; // 80% health
                }
            }

            println!("Player at 80% health tries to use emergency potion...");
            
            world.write_storage::<WantsToUseConsumable>()
                .insert(player, WantsToUseConsumable { item: potion, target: None })
                .expect("Failed to insert consumable usage intent");

            let mut usage_system = ConsumableUsageSystem;
            usage_system.run_now(world);
            world.maintain();

            let gamelog = world.read_resource::<GameLog>();
            println!("Log: {}", gamelog.entries.last().unwrap_or(&"No log entry".to_string()));

            // Lower player health below threshold
            {
                let mut combat_stats = world.write_storage::<CombatStats>();
                if let Some(stats) = combat_stats.get_mut(player) {
                    stats.hp = 20; // 20% health
                }
            }

            println!("Player at 20% health tries to use emergency potion...");
            
            world.write_storage::<WantsToUseConsumable>()
                .insert(player, WantsToUseConsumable { item: potion, target: None })
                .expect("Failed to insert consumable usage intent");

            usage_system.run_now(world);
            world.maintain();

            let gamelog = world.read_resource::<GameLog>();
            println!("Log: {}", gamelog.entries.last().unwrap_or(&"No log entry".to_string()));
        }
        println!();
    }

    /// Demonstrate different consumable types
    pub fn demonstrate_consumable_types(&self, world: &mut World) {
        println!("=== CONSUMABLE TYPES DEMONSTRATION ===\n");

        let mut rng = RandomNumberGenerator::new();
        let position = Position { x: 10, y: 10 };

        // Create examples of each type
        println!("Creating different consumable types:");

        // Health potions of different potencies
        for potency in [PotionPotency::Minor, PotionPotency::Lesser, PotionPotency::Greater, PotionPotency::Superior] {
            let potion = self.factory.create_health_potion(world, position, potency);
            let names = world.read_storage::<Name>();
            let properties = world.read_storage::<crate::items::ItemProperties>();
            
            if let (Some(name), Some(props)) = (names.get(potion), properties.get(potion)) {
                println!("  {} - {} gold", name.name, props.value);
            }
        }

        // Different food types
        for food_type in [FoodType::Bread, FoodType::Cheese, FoodType::Meat, FoodType::Rations] {
            let food = self.factory.create_food(world, position, food_type);
            let names = world.read_storage::<Name>();
            
            if let Some(name) = names.get(food) {
                println!("  {}", name.name);
            }
        }

        // Different scroll types
        for scroll_type in [ScrollType::Healing, ScrollType::Fireball, ScrollType::Teleport, ScrollType::Identify] {
            let scroll = self.factory.create_scroll(world, position, scroll_type);
            let names = world.read_storage::<Name>();
            
            if let Some(name) = names.get(scroll) {
                println!("  {}", name.name);
            }
        }

        // Random consumables by context
        println!("\nRandom consumables by context:");
        for context in [ConsumableContext::Combat, ConsumableContext::Exploration, ConsumableContext::Treasure] {
            let item = self.factory.create_random_consumable(world, position, context, &mut rng);
            let names = world.read_storage::<Name>();
            
            if let Some(name) = names.get(item) {
                println!("  {:?}: {}", context, name.name);
            }
        }
        println!();
    }

    /// Show system statistics
    pub fn show_statistics(&self, world: &World) {
        println!("=== CONSUMABLE SYSTEM STATISTICS ===\n");

        let entities = world.entities();
        let consumables = world.read_storage::<Consumable>();
        let status_effects = world.read_storage::<StatusEffects>();
        let cooldowns = world.read_storage::<ConsumableCooldowns>();

        let consumable_count = (&entities, &consumables).join().count();
        let entities_with_effects = (&entities, &status_effects).join().count();
        let entities_with_cooldowns = (&entities, &cooldowns).join().count();

        println!("System Statistics:");
        println!("  Total consumables: {}", consumable_count);
        println!("  Entities with status effects: {}", entities_with_effects);
        println!("  Entities with cooldowns: {}", entities_with_cooldowns);

        // Count consumables by type
        let mut type_counts = std::collections::HashMap::new();
        for (_entity, consumable) in (&entities, &consumables).join() {
            let type_name = format!("{:?}", consumable.consumable_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        println!("\nConsumables by type:");
        for (consumable_type, count) in type_counts {
            println!("  {}: {}", consumable_type, count);
        }

        // Count active status effects
        let mut effect_counts = std::collections::HashMap::new();
        for (_entity, effects) in (&entities, &status_effects).join() {
            for effect_type in effects.effects.keys() {
                let type_name = format!("{:?}", effect_type);
                *effect_counts.entry(type_name).or_insert(0) += 1;
            }
        }

        if !effect_counts.is_empty() {
            println!("\nActive status effects:");
            for (effect_type, count) in effect_counts {
                println!("  {}: {}", effect_type, count);
            }
        }
        println!();
    }

    /// Run complete demonstration
    pub fn run_complete_demonstration(&self) {
        println!("=== CONSUMABLE SYSTEM DEMONSTRATION ===\n");

        let mut world = self.setup_world();
        let player = self.create_test_player(&mut world);

        self.demonstrate_consumable_types(&mut world);
        self.demonstrate_basic_usage(&mut world, player);
        self.demonstrate_status_effects(&mut world, player);
        self.demonstrate_cooldowns(&mut world, player);
        self.demonstrate_restrictions(&mut world, player);
        self.show_statistics(&world);

        println!("=== DEMONSTRATION COMPLETE ===");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consumable_integration_setup() {
        let integration = ConsumableIntegration::new();
        let world = integration.setup_world();
        
        // Verify world is set up correctly
        assert!(world.has_component::<Consumable>());
        assert!(world.has_component::<StatusEffects>());
        assert!(world.has_component::<ConsumableCooldowns>());
    }

    #[test]
    fn test_player_creation() {
        let integration = ConsumableIntegration::new();
        let mut world = integration.setup_world();
        
        let player = integration.create_test_player(&mut world);
        
        let players = world.read_storage::<Player>();
        let combat_stats = world.read_storage::<CombatStats>();
        let cooldowns = world.read_storage::<ConsumableCooldowns>();
        let status_effects = world.read_storage::<StatusEffects>();
        
        assert!(players.get(player).is_some());
        assert!(combat_stats.get(player).is_some());
        assert!(cooldowns.get(player).is_some());
        assert!(status_effects.get(player).is_some());
    }

    #[test]
    fn test_consumable_creation() {
        let integration = ConsumableIntegration::new();
        let mut world = integration.setup_world();
        
        integration.create_test_consumables(&mut world);
        
        let entities = world.entities();
        let consumables = world.read_storage::<Consumable>();
        
        let consumable_count = (&entities, &consumables).join().count();
        assert!(consumable_count >= 5); // Should have created at least 5 consumables
    }
}