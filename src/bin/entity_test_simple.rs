use ascii_dungeon_explorer::map::entity_placement::{EntitySpawn, SpawnType};
use ascii_dungeon_explorer::map::{
    DungeonFeatureGenerator, EntityPlacementSystem, MapGenerator, RoomBasedDungeonGenerator,
    TileType,
};
use ascii_dungeon_explorer::resources::RandomNumberGenerator;

fn main() {
    // Create a random number generator
    let rng = RandomNumberGenerator::new_with_random_seed();

    // Create a dungeon generator
    let mut generator = RoomBasedDungeonGenerator::new(rng.clone());

    // Generate a base map
    let mut map = generator.generate_map(80, 50, 1);

    // Create a feature generator and add features
    let mut feature_gen = DungeonFeatureGenerator::new(rng.clone());
    feature_gen.add_features(&mut map);

    // Create an entity placement system and populate the map
    let mut entity_placer = EntityPlacementSystem::new(rng);
    let spawns = entity_placer.populate_map(&map, 3); // Difficulty level 3

    // Print statistics about the entities
    println!("Entity Placement Test");
    println!("=====================");
    println!("Map size: {}x{}", map.width, map.height);
    println!("Map theme: {:?}", map.theme);
    println!("Total rooms: {}", map.rooms.len());
    println!();

    println!("Entity Statistics:");
    println!("Total entities: {}", spawns.len());

    let enemy_count = spawns
        .iter()
        .filter(|s| matches!(s.entity_type, SpawnType::Enemy(_)))
        .count();
    let item_count = spawns
        .iter()
        .filter(|s| matches!(s.entity_type, SpawnType::Item(_)))
        .count();
    let special_count = spawns
        .iter()
        .filter(|s| matches!(s.entity_type, SpawnType::Special(_)))
        .count();

    println!("Enemies: {}", enemy_count);
    println!("Items: {}", item_count);
    println!("Special features: {}", special_count);
    println!();

    // Print details about each entity type
    println!("Enemy Types:");
    let mut enemy_types = std::collections::HashMap::new();
    for spawn in &spawns {
        if let SpawnType::Enemy(enemy_type) = spawn.entity_type {
            *enemy_types.entry(format!("{:?}", enemy_type)).or_insert(0) += 1;
        }
    }
    for (enemy_type, count) in enemy_types {
        println!("  {}: {}", enemy_type, count);
    }
    println!();

    println!("Item Types:");
    let mut item_types = std::collections::HashMap::new();
    for spawn in &spawns {
        if let SpawnType::Item(item_type) = spawn.entity_type {
            *item_types.entry(format!("{:?}", item_type)).or_insert(0) += 1;
        }
    }
    for (item_type, count) in item_types {
        println!("  {}: {}", item_type, count);
    }
    println!();

    println!("Special Feature Types:");
    let mut special_types = std::collections::HashMap::new();
    for spawn in &spawns {
        if let SpawnType::Special(special_type) = spawn.entity_type {
            *special_types
                .entry(format!("{:?}", special_type))
                .or_insert(0) += 1;
        }
    }
    for (special_type, count) in special_types {
        println!("  {}: {}", special_type, count);
    }

    // Print a simple ASCII representation of the map with entities
    println!("\nMap with Entities (E=Enemy, I=Item, S=Special):");
    for y in 0..map.height {
        for x in 0..map.width {
            let idx = map.xy_idx(x, y);

            // Check if there's an entity at this position
            let entity_here = spawns.iter().find(|spawn| spawn.x == x && spawn.y == y);

            if let Some(spawn) = entity_here {
                match spawn.entity_type {
                    SpawnType::Enemy(_) => print!("E"),
                    SpawnType::Item(_) => print!("I"),
                    SpawnType::Special(_) => print!("S"),
                }
            } else {
                // Otherwise print the map tile
                match map.tiles[idx] {
                    TileType::Wall => print!("#"),
                    TileType::Floor => print!("."),
                    TileType::DownStairs => print!(">"),
                    TileType::UpStairs => print!("<"),
                    TileType::Door(_) => print!("+"),
                    TileType::Water => print!("~"),
                    TileType::Lava => print!("^"),
                    TileType::Trap(_) => print!("^"),
                    TileType::Bridge => print!("="),
                    _ => print!(" "),
                }
            }
        }
        println!();
    }
}
