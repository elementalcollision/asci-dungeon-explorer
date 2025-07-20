use bevy::prelude::*;
use crate::ai::{AIComponent, AIBehaviorState, PathFollower, PathfindingRequest};
use crate::components::Position;
use crate::map::DungeonMap;

/// System that integrates AI behavior with pathfinding
pub fn ai_movement_system(
    mut ai_query: Query<(Entity, &mut AIComponent, &Position, Option<&mut PathFollower>)>,
    map: Res<DungeonMap>,
    mut commands: Commands,
) {
    for (entity, mut ai, position, path_follower) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        match ai.current_state {
            AIBehaviorState::Hunt => {
                handle_hunt_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            AIBehaviorState::Patrol => {
                handle_patrol_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            AIBehaviorState::Flee => {
                handle_flee_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            AIBehaviorState::Search => {
                handle_search_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            AIBehaviorState::Wander => {
                handle_wander_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            AIBehaviorState::Guard => {
                handle_guard_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            AIBehaviorState::Follow => {
                handle_follow_movement(&mut ai, entity, position, path_follower, &mut commands);
            }
            _ => {
                // For states that don't require movement, clear any existing path
                if let Some(mut follower) = path_follower {
                    if follower.has_path() {
                        follower.clear_path();
                    }
                }
            }
        }
    }
}

fn handle_hunt_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    if let Some(target_pos) = ai.memory.last_known_target_position {
        let current_pos = position.0;
        let distance = current_pos.distance(target_pos);

        // Only request new path if we don't have one or target moved significantly
        let should_request_path = if let Some(follower) = &path_follower {
            !follower.has_path() || 
            follower.current_target().map_or(true, |target| {
                target.as_vec2().distance(target_pos) > follower.path_recalculation_distance
            })
        } else {
            true
        };

        if should_request_path && distance > 1.0 {
            // Add PathFollower component if not present
            if path_follower.is_none() {
                commands.entity(entity).insert(PathFollower::new(ai.personality.aggression * 3.0 + 1.0));
            }
            
            // Request pathfinding to target
            commands.entity(entity).insert(PathfindingRequest::new(target_pos));
        }
    }
}

fn handle_patrol_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    if let Some(patrol_target) = ai.get_current_patrol_target() {
        let current_pos = position.0;
        let target_pos = patrol_target.as_vec2();
        let distance = current_pos.distance(target_pos);

        // Check if we've reached the patrol point
        if distance < 1.0 {
            ai.advance_patrol();
            // Clear current path so we can path to the next point
            if let Some(mut follower) = path_follower {
                follower.clear_path();
            }
        } else {
            // Request path to current patrol target if needed
            let should_request_path = if let Some(follower) = &path_follower {
                !follower.has_path()
            } else {
                true
            };

            if should_request_path {
                // Add PathFollower component if not present
                if path_follower.is_none() {
                    commands.entity(entity).insert(PathFollower::new(2.0)); // Moderate patrol speed
                }
                
                commands.entity(entity).insert(PathfindingRequest::new(target_pos));
            }
        }
    }
}

fn handle_flee_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    // Calculate flee direction
    let flee_target = if let Some(enemy_pos) = ai.memory.last_known_target_position {
        // Flee away from the enemy
        let current_pos = position.0;
        let flee_direction = (current_pos - enemy_pos).normalize_or_zero();
        current_pos + flee_direction * 10.0 // Flee 10 units away
    } else {
        // Flee to home position if no specific threat
        ai.memory.home_position
    };

    let should_request_path = if let Some(follower) = &path_follower {
        !follower.has_path() || 
        follower.current_target().map_or(true, |target| {
            target.as_vec2().distance(flee_target) > 2.0
        })
    } else {
        true
    };

    if should_request_path {
        // Add PathFollower component if not present (fast flee speed)
        if path_follower.is_none() {
            let flee_speed = 4.0 + ai.personality.courage * 2.0; // Cowards flee faster
            commands.entity(entity).insert(PathFollower::new(flee_speed));
        }
        
        commands.entity(entity).insert(PathfindingRequest::new(flee_target));
    }
}

fn handle_search_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    if let Some(search_center) = ai.memory.last_known_target_position {
        let current_pos = position.0;
        
        // Create a search pattern around the last known position
        let search_radius = 5.0;
        let time_factor = ai.state_timer * 0.5; // Slow search pattern
        let search_offset = Vec2::new(
            (time_factor.sin() * search_radius),
            (time_factor.cos() * search_radius),
        );
        let search_target = search_center + search_offset;

        let should_request_path = if let Some(follower) = &path_follower {
            !follower.has_path() || 
            follower.current_target().map_or(true, |target| {
                target.as_vec2().distance(search_target) > 2.0
            })
        } else {
            true
        };

        if should_request_path {
            // Add PathFollower component if not present
            if path_follower.is_none() {
                commands.entity(entity).insert(PathFollower::new(1.5)); // Slow search speed
            }
            
            commands.entity(entity).insert(PathfindingRequest::new(search_target));
        }
    }
}

fn handle_wander_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    // Generate a new wander target periodically
    if ai.state_timer > 5.0 || ai.memory.last_known_target_position.is_none() {
        let wander_radius = 8.0;
        let current_pos = position.0;
        
        // Generate pseudo-random direction based on entity ID and time
        let entity_seed = entity.index() as f32;
        let time_seed = ai.state_timer;
        let angle = (entity_seed + time_seed).sin() * std::f32::consts::TAU;
        
        let wander_offset = Vec2::new(angle.cos(), angle.sin()) * wander_radius;
        let wander_target = ai.memory.home_position + wander_offset;
        
        ai.memory.last_known_target_position = Some(wander_target);
        ai.state_timer = 0.0; // Reset timer
    }

    if let Some(wander_target) = ai.memory.last_known_target_position {
        let should_request_path = if let Some(follower) = &path_follower {
            !follower.has_path()
        } else {
            true
        };

        if should_request_path {
            // Add PathFollower component if not present
            if path_follower.is_none() {
                commands.entity(entity).insert(PathFollower::new(1.0)); // Slow wander speed
            }
            
            commands.entity(entity).insert(PathfindingRequest::new(wander_target));
        }
    }
}

fn handle_guard_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    let current_pos = position.0;
    let home_pos = ai.memory.home_position;
    let distance_from_home = current_pos.distance(home_pos);

    // Return to guard position if too far away
    if distance_from_home > 3.0 {
        let should_request_path = if let Some(follower) = &path_follower {
            !follower.has_path()
        } else {
            true
        };

        if should_request_path {
            // Add PathFollower component if not present
            if path_follower.is_none() {
                commands.entity(entity).insert(PathFollower::new(2.0)); // Moderate guard speed
            }
            
            commands.entity(entity).insert(PathfindingRequest::new(home_pos));
        }
    } else {
        // Close enough to guard position, stop moving
        if let Some(mut follower) = path_follower {
            if follower.has_path() {
                follower.clear_path();
            }
        }
    }
}

fn handle_follow_movement(
    ai: &mut AIComponent,
    entity: Entity,
    position: &Position,
    path_follower: Option<&mut PathFollower>,
    commands: &mut Commands,
) {
    if let Some(target_pos) = ai.memory.last_known_target_position {
        let current_pos = position.0;
        let distance = current_pos.distance(target_pos);

        // Maintain following distance
        let follow_distance = 2.0;
        
        if distance > follow_distance + 1.0 {
            let should_request_path = if let Some(follower) = &path_follower {
                !follower.has_path() || 
                follower.current_target().map_or(true, |target| {
                    target.as_vec2().distance(target_pos) > follower.path_recalculation_distance
                })
            } else {
                true
            };

            if should_request_path {
                // Add PathFollower component if not present
                if path_follower.is_none() {
                    let follow_speed = 2.5 + ai.personality.loyalty; // Loyal followers move faster
                    commands.entity(entity).insert(PathFollower::new(follow_speed));
                }
                
                // Path to a position near the target, not exactly on it
                let follow_direction = (current_pos - target_pos).normalize_or_zero();
                let follow_target = target_pos + follow_direction * follow_distance;
                
                commands.entity(entity).insert(PathfindingRequest::new(follow_target));
            }
        } else if distance < follow_distance - 0.5 {
            // Too close, stop moving
            if let Some(mut follower) = path_follower {
                if follower.has_path() {
                    follower.clear_path();
                }
            }
        }
    }
}

/// System for handling stuck AI entities
pub fn ai_stuck_handling_system(
    mut ai_query: Query<(Entity, &mut AIComponent, &mut PathFollower, &Position)>,
    mut pathfinding_res: ResMut<crate::ai::PathfindingResource>,
    mut commands: Commands,
) {
    for (entity, mut ai, mut path_follower, position) in ai_query.iter_mut() {
        if path_follower.is_stuck() {
            // Entity is stuck, try alternative approaches
            
            // 1. Clear current path
            path_follower.clear_path();
            
            // 2. Invalidate cached paths through current position
            pathfinding_res.pathfinder.invalidate_paths_through(position.0.as_ivec2());
            
            // 3. Change AI behavior to try to get unstuck
            match ai.current_state {
                AIBehaviorState::Hunt | AIBehaviorState::Patrol | AIBehaviorState::Search => {
                    // Switch to wander to try a different approach
                    ai.transition_to_state(AIBehaviorState::Wander);
                }
                AIBehaviorState::Wander => {
                    // If wandering and still stuck, try returning home
                    ai.transition_to_state(AIBehaviorState::Guard);
                }
                _ => {
                    // For other states, try wandering
                    ai.transition_to_state(AIBehaviorState::Wander);
                }
            }
            
            // 4. Reset stuck timer
            path_follower.stuck_timer = 0.0;
        }
    }
}

/// Plugin for AI movement integration
pub struct AIMovementPlugin;

impl Plugin for AIMovementPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ai_movement_system,
            ai_stuck_handling_system,
        ).after(crate::ai::ai_behavior_system));
    }
}