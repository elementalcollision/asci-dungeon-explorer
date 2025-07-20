use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use crate::guild::world_instance::{WorldInstance, WorldInstanceManager, InstanceStatus};
use crate::map::DungeonMap;

/// Serialized instance data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedInstance {
    pub instance: WorldInstance,
    pub map_data: Option<SerializedMap>,
    pub entities: Vec<SerializedEntity>,
    pub version: u32,
}

/// Serialized map data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedMap {
    pub width: i32,
    pub height: i32,
    pub tiles: Vec<u8>,
    pub revealed: Vec<bool>,
    pub visible: Vec<bool>,
    pub blocked: Vec<bool>,
    pub entry_points: HashMap<String, (i32, i32)>,
    pub exit_points: HashMap<String, (i32, i32)>,
    pub special_locations: HashMap<String, (i32, i32)>,
}

/// Serialized entity data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedEntity {
    pub entity_type: String,
    pub position: Option<(f32, f32)>,
    pub components: HashMap<String, String>, // Component name -> JSON serialized data
}

/// Instance serialization system
pub fn serialize_instance(
    instance_id: &str,
    instance_manager: &WorldInstanceManager,
    world: &World,
    save_dir: &Path,
) -> Result<(), String> {
    // Get the instance
    let instance = match instance_manager.get_instance(instance_id) {
        Some(instance) => instance,
        None => return Err(format!("Instance {} not found", instance_id)),
    };
    
    // Get the instance map
    let map_entity = instance_manager.get_instance_map_entity(instance_id);
    let map_data = if let Some(map_entity) = map_entity {
        if let Some(map) = world.get::<DungeonMap>(map_entity) {
            Some(serialize_map(map))
        } else {
            None
        }
    } else {
        None
    };
    
    // Serialize entities in the instance
    let mut entities = Vec::new();
    for &entity in &instance.connected_entities {
        if let Some(serialized) = serialize_entity(entity, world) {
            entities.push(serialized);
        }
    }
    
    // Create serialized instance
    let serialized = SerializedInstance {
        instance: instance.clone(),
        map_data,
        entities,
        version: 1, // Current version
    };
    
    // Create save directory if it doesn't exist
    if !save_dir.exists() {
        fs::create_dir_all(save_dir).map_err(|e| format!("Failed to create save directory: {}", e))?;
    }
    
    // Save to file
    let file_path = save_dir.join(format!("instance_{}.json", instance_id));
    let mut file = File::create(file_path).map_err(|e| format!("Failed to create file: {}", e))?;
    
    let json = serde_json::to_string_pretty(&serialized)
        .map_err(|e| format!("Failed to serialize instance: {}", e))?;
    
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(())
}

/// Instance deserialization function
pub fn deserialize_instance(
    file_path: &Path,
) -> Result<SerializedInstance, String> {
    // Read file
    let mut file = File::open(file_path)
        .map_err(|e| format!("Failed to open file: {}", e))?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    // Deserialize
    let serialized: SerializedInstance = serde_json::from_str(&contents)
        .map_err(|e| format!("Failed to deserialize instance: {}", e))?;
    
    Ok(serialized)
}

/// Map serialization function
fn serialize_map(map: &DungeonMap) -> SerializedMap {
    let mut entry_points = HashMap::new();
    let mut exit_points = HashMap::new();
    let mut special_locations = HashMap::new();
    
    // In a real implementation, you would convert Vec2 to (i32, i32) here
    // For now, we'll just create empty maps
    
    SerializedMap {
        width: map.width,
        height: map.height,
        tiles: map.tiles.clone(),
        revealed: map.revealed.clone(),
        visible: map.visible.clone(),
        blocked: map.blocked.clone(),
        entry_points,
        exit_points,
        special_locations,
    }
}

/// Entity serialization function
fn serialize_entity(entity: Entity, world: &World) -> Option<SerializedEntity> {
    // In a real implementation, you would serialize all relevant components
    // For now, we'll just create a placeholder
    
    let entity_type = if world.get::<Name>(entity).is_some() {
        "Named".to_string()
    } else {
        "Unknown".to_string()
    };
    
    let position = world.get::<Position>(entity).map(|pos| (pos.0.x, pos.0.y));
    
    Some(SerializedEntity {
        entity_type,
        position,
        components: HashMap::new(),
    })
}

/// System for auto-saving instances
pub fn auto_save_instances_system(
    instance_manager: Res<WorldInstanceManager>,
    world: &World,
    time: Res<Time>,
) {
    // In a real implementation, you would track last save time and only save periodically
    // For now, we'll just demonstrate the concept
    
    let save_dir = Path::new("saves/instances");
    
    // Save active instances
    for instance in instance_manager.instances.values() {
        if instance.is_active() {
            if let Err(err) = serialize_instance(&instance.id, &instance_manager, world, save_dir) {
                error!("Failed to save instance {}: {}", instance.id, err);
            }
        }
    }
}

/// System for loading instances on startup
pub fn load_instances_system(
    mut commands: Commands,
    mut instance_manager: ResMut<WorldInstanceManager>,
) {
    let save_dir = Path::new("saves/instances");
    
    // Check if directory exists
    if !save_dir.exists() {
        return;
    }
    
    // Read directory
    let entries = match fs::read_dir(save_dir) {
        Ok(entries) => entries,
        Err(e) => {
            error!("Failed to read save directory: {}", e);
            return;
        }
    };
    
    // Load each instance file
    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            
            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                match deserialize_instance(&path) {
                    Ok(serialized) => {
                        // Add instance to manager
                        let instance_id = serialized.instance.id.clone();
                        instance_manager.add_instance(serialized.instance);
                        
                        // Create map entity if map data exists
                        if let Some(map_data) = serialized.map_data {
                            let map = deserialize_map(&map_data);
                            let map_entity = commands.spawn(map).id();
                            instance_manager.register_instance_map(&instance_id, map_entity);
                        }
                        
                        // In a real implementation, you would also spawn entities
                        // For now, we'll just log that the instance was loaded
                        info!("Loaded instance {}", instance_id);
                    }
                    Err(e) => {
                        error!("Failed to load instance from {}: {}", path.display(), e);
                    }
                }
            }
        }
    }
}

/// Map deserialization function
fn deserialize_map(map_data: &SerializedMap) -> DungeonMap {
    DungeonMap {
        width: map_data.width,
        height: map_data.height,
        tiles: map_data.tiles.clone(),
        revealed: map_data.revealed.clone(),
        visible: map_data.visible.clone(),
        blocked: map_data.blocked.clone(),
    }
}

/// Plugin for instance serialization
pub struct InstanceSerializationPlugin;

impl Plugin for InstanceSerializationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_instances_system)
           .add_systems(Update, auto_save_instances_system);
    }
}