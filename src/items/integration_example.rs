// Example of how to integrate the item system into the main game

use specs::{World, WorldExt, Builder};
use crate::components::{Position, Item, Name};
use crate::items::{ItemFactory, ItemDatabase, ItemType, WeaponType, ConsumableType};
use crate::resources::RandomNumberGenerator;

/// Example function showing how to set up the item system
pub fn setup_item_system(world: &mut World) {
    // Register all item-related components
    world.register::<Item>();
    world.register::<crate::items::ItemProperties>();
    world.register::<crate::items::ItemStack>();
    world.register::<crate::items::ItemIdentification>();
    world.register::<crate::items::MagicalItem>();
    world.register::<crate::items::ItemBonuses>();
    
    println!("Item system components registered!");
}

/// Example function showing how to create items using the factory
pub fn create_example_items(world: &mut World) {
    let factory = ItemFactory::new();
    let mut rng = RandomNumberGenerator::new();
    
    // Create a sword at position (5, 5)
    let sword_pos = Position { x: 5, y: 5 };
    let sword = factory.create_weapon(world, WeaponType::Sword, sword_pos, &mut rng);
    println!("Created sword entity: {:?}", sword);
    
    // Create a health potion at position (6, 6)
    let potion_pos = Position { x: 6, y: 6 };
    let potion = factory.create_consumable(world, ConsumableType::Potion, potion_pos, &mut rng);
    println!("Created potion entity: {:?}", potion);
    
    // Create a random weapon at position (7, 7)
    let random_pos = Position { x: 7, y: 7 };
    let random_weapon = factory.create_random_weapon(world, random_pos, &mut rng);
    println!("Created random weapon entity: {:?}", random_weapon);
}

/// Example function showing how to use the item database
pub fn demonstrate_item_database(world: &mut World) {
    // Create the default item database
    let db = ItemDatabase::create_default_database();
    
    // Get a template and create an item from it
    if let Some(template) = db.get_item_template("iron_sword") {
        let position = Position { x: 10, y: 10 };
        let entity = template.create_item(world, position);
        println!("Created item from template: {:?}", entity);
        
        // Display item information
        let info = crate::items::get_item_info_string(world, entity);
        println!("Item info:\n{}", info);
    }
    
    // Save the database to a file (in a real game)
    if let Err(e) = db.save_to_file("item_database.json") {
        println!("Failed to save item database: {}", e);
    } else {
        println!("Item database saved to item_database.json");
    }
}

/// Example function showing how to work with item collections
pub fn demonstrate_item_serialization(world: &World) {
    // Serialize all items in the world
    let collection = crate::items::ItemCollection::from_world(world);
    println!("Found {} items in the world", collection.items.len());
    
    // Save items to a file
    if let Err(e) = collection.save_to_file("world_items.json") {
        println!("Failed to save items: {}", e);
    } else {
        println!("Items saved to world_items.json");
    }
    
    // Get item counts by type
    let counts = crate::items::count_items_by_type(world);
    println!("Item counts by type:");
    for (item_type, count) in counts {
        println!("  {}: {}", item_type, count);
    }
}

/// Example function showing how to find and interact with items
pub fn demonstrate_item_interaction(world: &World) {
    // Find all items at a specific position
    let items_at_5_5 = crate::items::find_items_at_position(world, 5, 5);
    println!("Found {} items at position (5, 5)", items_at_5_5.len());
    
    for entity in items_at_5_5 {
        if let Some(name) = crate::items::get_item_display_name(world, entity) {
            println!("  - {}", name);
            
            let value = crate::items::get_item_current_value(world, entity);
            println!("    Value: {} gold", value);
        }
    }
    
    // Calculate total weight at a position
    let total_weight = crate::items::get_total_weight_at_position(world, 5, 5);
    println!("Total weight at (5, 5): {:.1} lbs", total_weight);
}

/// Complete example showing the full item system workflow
pub fn run_complete_example() {
    println!("=== Item System Integration Example ===\n");
    
    // Create a new world
    let mut world = World::new();
    
    // Set up the item system
    setup_item_system(&mut world);
    
    // Create some example items
    create_example_items(&mut world);
    
    // Demonstrate the item database
    demonstrate_item_database(&mut world);
    
    // Show item serialization
    demonstrate_item_serialization(&world);
    
    // Demonstrate item interaction
    demonstrate_item_interaction(&world);
    
    println!("\n=== Example Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt};

    #[test]
    fn test_integration_example() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Name>();
        
        // This should not panic
        setup_item_system(&mut world);
        create_example_items(&mut world);
        
        // Verify items were created
        let entities = world.entities();
        let items = world.read_storage::<Item>();
        let item_count = (&entities, &items).join().count();
        
        assert!(item_count >= 3, "Should have created at least 3 items");
    }
}