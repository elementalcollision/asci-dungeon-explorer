use bevy::prelude::*;
use std::collections::HashMap;
use crate::guild::world_instance::{WorldInstanceManager, WorldInstance, InstanceTransition, TransitionType, InstanceMap, InstanceType, InstanceStatus};
use crate::components::{Position, Player};
use crate::map::DungeonMap;
use crate::guild::mission::{Mission, MissionTracker};
use crate::guild::mission_board::MissionBoard;

/// Instance transition request component
#[derive(Component, Debug)]
pub struct TransitionRequest {
    pub target_instance: String,
    pub entry_point: Option<String>,
    pub transition_type: TransitionType,
}

/// Instance transition event
#[derive(Event, Debug)]
pub struct InstanceTransitionEvent {
    pub entity: Entity,
    pub from_instance: String,
    pub to_instance: String,
    pub transition_type: TransitionType,
}

/// System for handling instance transitions
pub fn instance_transition_system(
    mut commands: Commands,
    mut instance_manager: ResMut<WorldInstanceManager>,
    mut transition_events: EventWriter<InstanceTransitionEvent>,
    transition_query: Query<(Entity, &TransitionRequest)>,
    player_query: Query<Entity, With<Player>>,
    position_query: Query<&mut Position>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Process transition requests
    for (entity, request) in transition_query.iter() {
        // Get current instance
        let current_instance_id = if let Some(instance) = instance_manager.get_active_instance() {
            instance.id.clone()
        } else {
            // No active instance, can't transition
            commands.entity(entity).remove::<TransitionRequest>();
            continue;
        };
        
        // Get target instance
        let target_instance = if let Some(instance) = instance_manager.get_instance(&request.target_instance) {
            instance.clone()
        } else {
            // Target instance doesn't exist, remove request
            commands.entity(entity).remove::<TransitionRequest>();
            continue;
        };
        
        // Check if target instance is active
        if target_instance.status != InstanceStatus::Active && target_instance.status != InstanceStatus::Initializing {
            // Target instance is not active, activate it
            if let Some(instance) = instance_manager.get_instance_mut(&request.target_instance) {
                instance.activate(current_time);
            }
        }
        
        // Get target position
        let target_position = if let Some(entry_point) = &request.entry_point {
            // Get entry point from instance map
            if let Some(map_entity) = instance_manager.get_instance_map_entity(&request.target_instance) {
                if let Ok(instance_map) = commands.get_entity(map_entity).unwrap().get::<InstanceMap>() {
                    instance_map.entry_points.get(entry_point).cloned()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            // Use default entry point
            None
        };
        
        // Update entity position
        if let Ok(mut position) = position_query.get_mut(entity) {
            if let Some(target_pos) = target_position {
                position.0 = target_pos;
            } else {
                // Use a default position if no entry point is specified
                position.0 = Vec2::new(5.0, 5.0);
            }
        }
        
        // Update instance connections
        if let Some(current) = instance_manager.get_instance_mut(&current_instance_id) {
            current.remove_entity(&entity);
        }
        
        if let Some(target) = instance_manager.get_instance_mut(&request.target_instance) {
            target.add_entity(entity);
        }
        
        // If this is the player, update player instance
        if player_query.contains(entity) {
            instance_manager.set_player_instance(&request.target_instance);
            instance_manager.set_active_instance(&request.target_instance);
        }
        
        // Send transition event
        transition_events.send(InstanceTransitionEvent {
            entity,
            from_instance: current_instance_id,
            to_instance: request.target_instance.clone(),
            transition_type: request.transition_type,
        });
        
        // Remove transition request
        commands.entity(entity).remove::<TransitionRequest>();
    }
}

/// System for creating instance transitions from portals
pub fn portal_transition_system(
    mut commands: Commands,
    portal_query: Query<(Entity, &InstanceTransition, &Position)>,
    entity_query: Query<(Entity, &Position), Without<InstanceTransition>>,
    player_query: Query<Entity, With<Player>>,
) {
    // Check for entities near portals
    for (portal_entity, transition, portal_pos) in portal_query.iter() {
        for (entity, position) in entity_query.iter() {
            // Check if entity is near portal
            if position.0.distance(portal_pos.0) < 1.5 {
                // Only transition players for now
                if player_query.contains(entity) {
                    // Create transition request
                    commands.entity(entity).insert(TransitionRequest {
                        target_instance: transition.target_instance.clone(),
                        entry_point: None, // Use default entry point
                        transition_type: transition.transition_type,
                    });
                }
            }
        }
    }
}

/// System for creating mission instances
pub fn mission_instance_creation_system(
    mut commands: Commands,
    mut instance_manager: ResMut<WorldInstanceManager>,
    mission_board: Res<MissionBoard>,
    mission_tracker_query: Query<(Entity, &MissionTracker, Option<&Player>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Check for entities with active missions
    for (entity, tracker, player) in mission_tracker_query.iter() {
        if let Some(mission_id) = &tracker.active_mission {
            // Check if mission instance already exists
            let mission_instances = instance_manager.get_instances_by_mission(mission_id);
            if !mission_instances.is_empty() {
                continue; // Instance already exists
            }
            
            // Get mission
            if let Some(mission) = mission_board.get_mission(mission_id) {
                // Create mission instance
                let instance_id = instance_manager.create_mission_instance(mission, current_time);
                
                // Create dungeon map for the instance
                let map = create_mission_map(mission);
                let map_entity = commands.spawn(map.clone()).id();
                
                // Create instance map
                let instance_map = InstanceMap {
                    instance_id: instance_id.clone(),
                    map: map.clone(),
                    entry_points: HashMap::new(),
                    exit_points: HashMap::new(),
                    special_locations: HashMap::new(),
                };
                
                commands.entity(map_entity).insert(instance_map);
                instance_manager.register_instance_map(&instance_id, map_entity);
                
                // Activate instance
                if let Some(instance) = instance_manager.get_instance_mut(&instance_id) {
                    instance.activate(current_time);
                }
                
                // If this is the player, create transition request
                if player.is_some() {
                    commands.entity(entity).insert(TransitionRequest {
                        target_instance: instance_id,
                        entry_point: None,
                        transition_type: TransitionType::Mission,
                    });
                }
            }
        }
    }
}

/// Create a mission map
fn create_mission_map(mission: &Mission) -> DungeonMap {
    // In a real implementation, you would generate a map based on mission properties
    // For now, we'll just create a placeholder map
    
    let width = 50;
    let height = 50;
    let mut map = DungeonMap::new(width, height);
    
    // Create a simple room in the center
    for y in 20..30 {
        for x in 20..30 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = 1; // Floor
            map.blocked[idx] = false;
        }
    }
    
    map
}

/// System for returning from mission instances
pub fn mission_return_system(
    mut commands: Commands,
    mut instance_manager: ResMut<WorldInstanceManager>,
    mission_board: Res<MissionBoard>,
    mission_tracker_query: Query<(Entity, &MissionTracker, Option<&Player>)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    // Check for completed or failed missions
    for (entity, tracker, player) in mission_tracker_query.iter() {
        if let Some(mission_id) = &tracker.active_mission {
            if let Some(mission) = mission_board.get_mission(mission_id) {
                if mission.status == crate::guild::mission_types::MissionStatus::Completed || 
                   mission.status == crate::guild::mission_types::MissionStatus::Failed {
                    // Find mission instance
                    let mission_instances = instance_manager.get_instances_by_mission(mission_id);
                    
                    for instance in mission_instances {
                        // Archive instance
                        if let Some(instance) = instance_manager.get_instance_mut(&instance.id) {
                            instance.archive(current_time);
                        }
                        
                        // If this is the player, return to main world
                        if player.is_some() {
                            // Find main world instance
                            let main_worlds = instance_manager.get_instances_by_type(InstanceType::MainWorld);
                            
                            if let Some(main_world) = main_worlds.first() {
                                commands.entity(entity).insert(TransitionRequest {
                                    target_instance: main_world.id.clone(),
                                    entry_point: None,
                                    transition_type: TransitionType::Return,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Plugin for instance transition systems
pub struct InstanceTransitionPlugin;

impl Plugin for InstanceTransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InstanceTransitionEvent>()
           .add_systems(Update, (
               instance_transition_system,
               portal_transition_system,
               mission_instance_creation_system,
               mission_return_system,
           ).chain());
    }
}