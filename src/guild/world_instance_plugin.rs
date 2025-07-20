use bevy::prelude::*;
use crate::guild::world_instance::{WorldInstanceManager, WorldInstance, InstanceType, InstanceStatus};
use crate::guild::instance_serialization::InstanceSerializationPlugin;
use crate::guild::instance_transition::InstanceTransitionPlugin;
use crate::map::DungeonMap;

/// System for initializing the world instance manager
pub fn initialize_world_instance_manager(
    mut commands: Commands,
    mut instance_manager: ResMut<WorldInstanceManager>,
    time: Res<Time>,
) {
    // Only run once
    if !instance_manager.instances.is_empty() {
        return;
    }

    let current_time = time.elapsed_seconds_f64();

    // Create main world instance
    let mut main_world = WorldInstance::new("Main World", InstanceType::MainWorld, current_time);
    main_world.activate(current_time);
    
    let main_world_id = main_world.id.clone();
    instance_manager.add_instance(main_world);
    
    // Set as active and player instance
    instance_manager.set_active_instance(&main_world_id);
    instance_manager.set_player_instance(&main_world_id);
    
    // Create main world map
    let map = create_main_world_map();
    let map_entity = commands.spawn(map).id();
    
    // Register map with instance
    instance_manager.register_instance_map(&main_world_id, map_entity);
    
    // Create guild hall instance
    let mut guild_hall = WorldInstance::new("Guild Hall", InstanceType::GuildHall, current_time);
    guild_hall.activate(current_time);
    
    let guild_hall_id = guild_hall.id.clone();
    instance_manager.add_instance(guild_hall);
    
    // Create guild hall map
    let map = create_guild_hall_map();
    let map_entity = commands.spawn(map).id();
    
    // Register map with instance
    instance_manager.register_instance_map(&guild_hall_id, map_entity);
    
    info!("Initialized world instance manager");
}

/// Create a main world map
fn create_main_world_map() -> DungeonMap {
    // In a real implementation, you would generate a proper world map
    // For now, we'll just create a placeholder map
    
    let width = 80;
    let height = 50;
    let mut map = DungeonMap::new(width, height);
    
    // Create a simple open area
    for y in 0..height {
        for x in 0..width {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = 1; // Floor
            map.blocked[idx] = false;
        }
    }
    
    // Add some walls around the edges
    for x in 0..width {
        let idx = map.xy_idx(x, 0);
        map.tiles[idx] = 2; // Wall
        map.blocked[idx] = true;
        
        let idx = map.xy_idx(x, height - 1);
        map.tiles[idx] = 2; // Wall
        map.blocked[idx] = true;
    }
    
    for y in 0..height {
        let idx = map.xy_idx(0, y);
        map.tiles[idx] = 2; // Wall
        map.blocked[idx] = true;
        
        let idx = map.xy_idx(width - 1, y);
        map.tiles[idx] = 2; // Wall
        map.blocked[idx] = true;
    }
    
    map
}

/// Create a guild hall map
fn create_guild_hall_map() -> DungeonMap {
    // In a real implementation, you would generate a proper guild hall map
    // For now, we'll just create a placeholder map
    
    let width = 40;
    let height = 30;
    let mut map = DungeonMap::new(width, height);
    
    // Create a simple room
    for y in 0..height {
        for x in 0..width {
            let idx = map.xy_idx(x, y);
            
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                map.tiles[idx] = 2; // Wall
                map.blocked[idx] = true;
            } else {
                map.tiles[idx] = 1; // Floor
                map.blocked[idx] = false;
            }
        }
    }
    
    map
}

/// System for updating world instances
pub fn update_world_instances_system(
    mut instance_manager: ResMut<WorldInstanceManager>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Update instance statuses
    instance_manager.update_instances(current_time);
    
    // Clean up archived instances (keep for 30 days)
    instance_manager.clean_up_archived_instances(30, current_time);
}

/// Plugin for world instance management
pub struct WorldInstancePlugin;

impl Plugin for WorldInstancePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldInstanceManager>()
           .add_plugins((
               InstanceSerializationPlugin,
               InstanceTransitionPlugin,
           ))
           .add_systems(Startup, initialize_world_instance_manager)
           .add_systems(Update, update_world_instances_system);
    }
}