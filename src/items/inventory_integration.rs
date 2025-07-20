// Complete integration example for the inventory system

use specs::{World, WorldExt, Builder, Entity, Join};
use crossterm::event::{KeyCode, KeyEvent};
use crate::components::{Position, Player, Name, Item, WantsToPickupItem, WantsToDropItem};
use crate::items::{
    ItemFactory, AdvancedInventory, InventoryUI, InventoryAction, ItemPickupSystem, 
    ItemDropSystem, AutoPickupSystem, InventoryManagementSystem, Container, ContainerType,
    WeaponType, ConsumableType, ItemProperties
};
use crate::resources::{RandomNumberGenerator, GameLog};

/// Complete example showing how to integrate the inventory system
pub struct InventoryIntegrationExample {
    pub inventory_ui: InventoryUI,
    pub player_entity: Option<Entity>,
}

impl InventoryIntegrationExample {
    pub fn new() -> Self {
        InventoryIntegrationExample {
            inventory_ui: InventoryUI::new(),
            player_entity: None,
        }
    }

    /// Set up the world with all necessary components and systems
    pub fn setup_world(&mut self) -> World {
        let mut world = World::new();

        // Register all necessary components
        self.register_components(&mut world);

        // Add resources
        world.insert(GameLog::new());
        world.insert(RandomNumberGenerator::new());

        // Create a player with inventory
        self.player_entity = Some(self.create_player(&mut world));

        // Create some test items
        self.create_test_items(&mut world);

        // Create a test container
        self.create_test_container(&mut world);

        world
    }

    fn register_components(&self, world: &mut World) {
        // Basic components
        world.register::<Position>();
        world.register::<Player>();
        world.register::<Name>();
        world.register::<Item>();

        // Item components
        world.register::<ItemProperties>();
        world.register::<crate::items::ItemStack>();
        world.register::<crate::items::ItemIdentification>();
        world.register::<crate::items::MagicalItem>();
        world.register::<crate::items::ItemBonuses>();

        // Inventory components
        world.register::<AdvancedInventory>();
        world.register::<Container>();
        world.register::<crate::items::Pickupable>();
        world.register::<crate::items::InventoryBonus>();

        // Intent components
        world.register::<WantsToPickupItem>();
        world.register::<WantsToDropItem>();
    }

    fn create_player(&self, world: &mut World) -> Entity {
        world.create_entity()
            .with(Position { x: 5, y: 5 })
            .with(Player)
            .with(Name { name: "Player".to_string() })
            .with(AdvancedInventory::new(20, 100.0)) // 20 slots, 100 lbs capacity
            .build()
    }

    fn create_test_items(&self, world: &mut World) {
        let factory = ItemFactory::new();
        let mut rng = RandomNumberGenerator::new();

        // Create various items around the player
        let positions = vec![
            Position { x: 4, y: 5 }, // Left of player
            Position { x: 6, y: 5 }, // Right of player
            Position { x: 5, y: 4 }, // Above player
            Position { x: 5, y: 6 }, // Below player
        ];

        // Create different types of items
        factory.create_weapon(world, WeaponType::Sword, positions[0], &mut rng);
        factory.create_consumable(world, ConsumableType::Potion, positions[1], &mut rng);
        factory.create_random_armor(world, positions[2], &mut rng);
        factory.create_random_weapon(world, positions[3], &mut rng);

        // Create some materials
        for i in 0..3 {
            let pos = Position { x: 7 + i, y: 5 };
            factory.create_material(world, crate::items::MaterialType::Metal, pos, 5);
        }
    }

    fn create_test_container(&self, world: &mut World) -> Entity {
        let factory = ItemFactory::new();
        let mut rng = RandomNumberGenerator::new();

        // Create a chest
        let chest = world.create_entity()
            .with(Position { x: 10, y: 5 })
            .with(Name { name: "Wooden Chest".to_string() })
            .with(Container::new(10, ContainerType::Chest))
            .build();

        // Add some items to the chest
        let mut container = Container::new(10, ContainerType::Chest);
        
        // Create items and add them to the container
        let sword = factory.create_weapon(world, WeaponType::Dagger, Position { x: -1, y: -1 }, &mut rng);
        let potion = factory.create_consumable(world, ConsumableType::Potion, Position { x: -1, y: -1 }, &mut rng);
        
        container.add_item(sword);
        container.add_item(potion);

        // Update the chest's container
        world.write_storage::<Container>()
            .insert(chest, container)
            .expect("Failed to update container");

        chest
    }

    /// Run the inventory systems
    pub fn run_systems(&self, world: &mut World) {
        let mut pickup_system = ItemPickupSystem;
        let mut drop_system = ItemDropSystem;
        let mut auto_pickup_system = AutoPickupSystem;
        let mut management_system = InventoryManagementSystem;

        pickup_system.run_now(world);
        drop_system.run_now(world);
        auto_pickup_system.run_now(world);
        management_system.run_now(world);

        world.maintain();
    }

    /// Handle player input for inventory management
    pub fn handle_input(&mut self, world: &mut World, key: KeyEvent) -> bool {
        if let Some(player_entity) = self.player_entity {
            match key.code {
                KeyCode::Char('i') | KeyCode::Char('I') => {
                    // Open inventory
                    self.show_inventory(world, player_entity);
                    true
                },
                KeyCode::Char('g') | KeyCode::Char('G') => {
                    // Pick up item at player position
                    self.pickup_item_at_player_position(world, player_entity);
                    true
                },
                KeyCode::Char(',') => {
                    // Pick up all items at player position
                    self.pickup_all_items_at_player_position(world, player_entity);
                    true
                },
                _ => false,
            }
        } else {
            false
        }
    }

    fn show_inventory(&mut self, world: &World, player_entity: Entity) {
        println!("=== INVENTORY DEMO ===");
        
        let inventories = world.read_storage::<AdvancedInventory>();
        if let Some(inventory) = inventories.get(player_entity) {
            println!("Capacity: {}/{}", inventory.items.len(), inventory.capacity);
            println!("Weight: {:.1}/{:.1} lbs", inventory.current_weight, inventory.weight_limit);
            println!("Gold: {}", inventory.gold);
            println!();

            if inventory.items.is_empty() {
                println!("Inventory is empty.");
            } else {
                println!("Items:");
                for (index, slot) in inventory.items.iter().enumerate() {
                    let name = crate::items::get_item_display_name(world, slot.entity)
                        .unwrap_or("Unknown".to_string());
                    
                    if slot.quantity > 1 {
                        println!("  {}. {} x{}", index + 1, name, slot.quantity);
                    } else {
                        println!("  {}. {}", index + 1, name);
                    }
                }
            }
        }
        println!();
    }

    fn pickup_item_at_player_position(&self, world: &mut World, player_entity: Entity) {
        let player_pos = {
            let positions = world.read_storage::<Position>();
            positions.get(player_entity).cloned()
        };

        if let Some(pos) = player_pos {
            let items_at_pos = crate::items::find_items_at_position(world, pos.x, pos.y);
            
            if let Some(&first_item) = items_at_pos.first() {
                // Create pickup intent
                world.write_storage::<WantsToPickupItem>()
                    .insert(player_entity, WantsToPickupItem { item: first_item })
                    .expect("Failed to insert pickup intent");
                
                println!("Attempting to pick up item...");
            } else {
                println!("No items here to pick up.");
            }
        }
    }

    fn pickup_all_items_at_player_position(&self, world: &mut World, player_entity: Entity) {
        let player_pos = {
            let positions = world.read_storage::<Position>();
            positions.get(player_entity).cloned()
        };

        if let Some(pos) = player_pos {
            let items_at_pos = crate::items::find_items_at_position(world, pos.x, pos.y);
            
            if items_at_pos.is_empty() {
                println!("No items here to pick up.");
                return;
            }

            // Create pickup intents for all items
            let mut pickup_storage = world.write_storage::<WantsToPickupItem>();
            for &item_entity in &items_at_pos {
                pickup_storage.insert(player_entity, WantsToPickupItem { item: item_entity })
                    .expect("Failed to insert pickup intent");
            }
            
            println!("Attempting to pick up {} items...", items_at_pos.len());
        }
    }

    /// Demonstrate inventory operations
    pub fn demonstrate_inventory_operations(&mut self, world: &mut World) {
        println!("=== INVENTORY SYSTEM DEMONSTRATION ===\n");

        if let Some(player_entity) = self.player_entity {
            // Show initial state
            println!("1. Initial inventory state:");
            self.show_inventory(world, player_entity);

            // Pick up some items
            println!("2. Picking up items...");
            self.pickup_all_items_at_player_position(world, player_entity);
            self.run_systems(world);
            self.show_inventory(world, player_entity);

            // Demonstrate sorting
            println!("3. Sorting inventory by name...");
            {
                let mut inventories = world.write_storage::<AdvancedInventory>();
                if let Some(inventory) = inventories.get_mut(player_entity) {
                    inventory.sort_mode = crate::items::InventorySortMode::Name;
                    inventory.sort_inventory(world);
                }
            }
            self.show_inventory(world, player_entity);

            // Show inventory statistics
            println!("4. Inventory statistics:");
            self.show_inventory_statistics(world, player_entity);

            // Demonstrate filtering
            println!("5. Filtering weapons:");
            self.show_filtered_items(world, player_entity, crate::items::ItemType::Weapon(WeaponType::Sword));
        }

        println!("=== DEMONSTRATION COMPLETE ===\n");
    }

    fn show_inventory_statistics(&self, world: &World, player_entity: Entity) {
        let inventories = world.read_storage::<AdvancedInventory>();
        if let Some(inventory) = inventories.get(player_entity) {
            let total_value = inventory.get_total_value(world);
            let weight_percentage = (inventory.current_weight / inventory.weight_limit) * 100.0;
            let capacity_percentage = (inventory.items.len() as f32 / inventory.capacity as f32) * 100.0;

            println!("  Total value: {} gold", total_value);
            println!("  Weight usage: {:.1}%", weight_percentage);
            println!("  Capacity usage: {:.1}%", capacity_percentage);
            
            if inventory.is_overweight() {
                println!("  WARNING: Overweight!");
            }
        }
        println!();
    }

    fn show_filtered_items(&self, world: &World, player_entity: Entity, filter_type: crate::items::ItemType) {
        let inventories = world.read_storage::<AdvancedInventory>();
        if let Some(inventory) = inventories.get(player_entity) {
            let filtered_indices = inventory.get_items_by_type(world, &filter_type);
            
            if filtered_indices.is_empty() {
                println!("  No items of this type found.");
            } else {
                println!("  Found {} items:", filtered_indices.len());
                for &index in &filtered_indices {
                    if let Some(slot) = inventory.items.get(index) {
                        let name = crate::items::get_item_display_name(world, slot.entity)
                            .unwrap_or("Unknown".to_string());
                        println!("    - {}", name);
                    }
                }
            }
        }
        println!();
    }

    /// Run a complete interactive demo
    pub fn run_interactive_demo(&mut self) {
        println!("Starting Interactive Inventory Demo...\n");
        
        let mut world = self.setup_world();
        
        // Run the demonstration
        self.demonstrate_inventory_operations(&mut world);
        
        // Show available commands
        println!("Available commands:");
        println!("  'i' - Show inventory");
        println!("  'g' - Pick up item");
        println!("  ',' - Pick up all items");
        println!("  'q' - Quit demo");
        println!();
        
        // In a real game, you would have an input loop here
        // For this demo, we'll just show what the commands would do
        println!("Demo complete. In a real game, you would use the input handling system.");
    }
}

/// Example of how to integrate inventory UI into a game state
pub struct InventoryGameState {
    pub inventory_ui: InventoryUI,
    pub world: World,
    pub player_entity: Entity,
    pub active: bool,
}

impl InventoryGameState {
    pub fn new(world: World, player_entity: Entity) -> Self {
        InventoryGameState {
            inventory_ui: InventoryUI::new(),
            world,
            player_entity,
            active: false,
        }
    }

    pub fn enter(&mut self) {
        self.active = true;
        self.inventory_ui.selected_index = 0;
        self.inventory_ui.scroll_offset = 0;
    }

    pub fn exit(&mut self) {
        self.active = false;
    }

    pub fn update(&mut self, key: KeyEvent) -> bool {
        if !self.active {
            return false;
        }

        let action = self.inventory_ui.handle_input(key, &mut self.world, self.player_entity);
        
        match action {
            InventoryAction::None => {},
            InventoryAction::UseItem(entity) => {
                self.use_item(entity);
            },
            InventoryAction::DropItem(entity) => {
                self.drop_item(entity);
            },
            InventoryAction::SortInventory(sort_mode) => {
                self.sort_inventory(sort_mode);
            },
            InventoryAction::ToggleAutoPickup => {
                self.toggle_auto_pickup();
            },
            InventoryAction::Close => {
                self.exit();
                return false;
            },
        }

        true
    }

    fn use_item(&mut self, item_entity: Entity) {
        // Implementation would depend on item type
        println!("Using item: {:?}", item_entity);
        
        // For consumables, you might:
        // 1. Apply item effects
        // 2. Remove item from inventory
        // 3. Log the action
    }

    fn drop_item(&mut self, item_entity: Entity) {
        // Create drop intent
        self.world.write_storage::<WantsToDropItem>()
            .insert(self.player_entity, WantsToDropItem { item: item_entity })
            .expect("Failed to insert drop intent");
    }

    fn sort_inventory(&mut self, sort_mode: crate::items::InventorySortMode) {
        let mut inventories = self.world.write_storage::<AdvancedInventory>();
        if let Some(inventory) = inventories.get_mut(self.player_entity) {
            inventory.sort_mode = sort_mode;
            inventory.sort_inventory(&self.world);
        }
    }

    fn toggle_auto_pickup(&mut self) {
        let mut inventories = self.world.write_storage::<AdvancedInventory>();
        if let Some(inventory) = inventories.get_mut(self.player_entity) {
            inventory.auto_pickup = !inventory.auto_pickup;
            
            let mut gamelog = self.world.write_resource::<GameLog>();
            if inventory.auto_pickup {
                gamelog.entries.push("Auto-pickup enabled.".to_string());
            } else {
                gamelog.entries.push("Auto-pickup disabled.".to_string());
            }
        }
    }

    pub fn render(&self, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        if self.active {
            self.inventory_ui.render(&self.world, self.player_entity, width, height)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_integration_setup() {
        let mut integration = InventoryIntegrationExample::new();
        let world = integration.setup_world();
        
        assert!(integration.player_entity.is_some());
        
        // Verify components are registered
        let entities = world.entities();
        let players = world.read_storage::<Player>();
        let inventories = world.read_storage::<AdvancedInventory>();
        
        let player_count = (&entities, &players, &inventories).join().count();
        assert_eq!(player_count, 1);
    }

    #[test]
    fn test_inventory_game_state() {
        let mut world = World::new();
        world.register::<Player>();
        world.register::<AdvancedInventory>();
        
        let player = world.create_entity()
            .with(Player)
            .with(AdvancedInventory::new(10, 50.0))
            .build();
        
        let mut game_state = InventoryGameState::new(world, player);
        
        assert!(!game_state.active);
        
        game_state.enter();
        assert!(game_state.active);
        
        game_state.exit();
        assert!(!game_state.active);
    }
}