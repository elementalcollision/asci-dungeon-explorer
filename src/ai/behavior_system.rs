use bevy::prelude::*;
use crate::ai::ai_component::*;
use crate::components::{Position, Health, Viewshed};
use std::collections::HashMap;

/// System for updating AI behavior states
pub fn ai_behavior_system(
    time: Res<Time>,
    mut ai_query: Query<(Entity, &mut AIComponent, &Position, &Health), With<AIComponent>>,
    target_query: Query<(Entity, &Position, &Health), Without<AIComponent>>,
    viewshed_query: Query<&Viewshed>,
) {
    let delta_time = time.delta_seconds();

    for (entity, mut ai, position, health) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        // Update AI timers and decision factors
        update_ai_decision_factors(&mut ai, position, health, &target_query, &viewshed_query, entity);
        
        // Update AI state machine
        ai.update_state(delta_time);
    }
}

/// Update AI decision factors based on current game state
fn update_ai_decision_factors(
    ai: &mut AIComponent,
    position: &Position,
    health: &Health,
    target_query: &Query<(Entity, &Position, &Health), Without<AIComponent>>,
    viewshed_query: &Query<&Viewshed>,
    ai_entity: Entity,
) {
    // Update health percentage
    ai.decision_factors.health_percentage = health.current as f32 / health.max as f32;

    // Find potential targets and update target-related factors
    let mut closest_target: Option<(Entity, f32)> = None;
    let mut enemies_nearby = 0;
    let detection_range = 10.0; // TODO: Make this configurable

    for (target_entity, target_position, target_health) in target_query.iter() {
        let distance = position.0.distance(target_position.0);
        
        if distance <= detection_range {
            // Check if we can see this target
            if let Ok(viewshed) = viewshed_query.get(ai_entity) {
                if viewshed.visible_tiles.contains(&target_position.0.as_ivec2()) {
                    enemies_nearby += 1;
                    
                    // Update closest target
                    if closest_target.is_none() || distance < closest_target.unwrap().1 {
                        closest_target = Some((target_entity, distance));
                    }
                    
                    // Remember this enemy
                    ai.remember_enemy(target_entity, target_position.0, 0.0);
                }
            }
        }
    }

    // Update target-related factors
    if let Some((target_entity, distance)) = closest_target {
        ai.set_target(Some(target_entity));
        ai.decision_factors.distance_to_target = distance;
        ai.memory.last_known_target_position = Some(
            target_query.get(target_entity).unwrap().1.0
        );
    } else {
        // No target visible, increment time since last seen
        ai.decision_factors.time_since_last_seen_target += 0.1; // Approximate update interval
        ai.decision_factors.distance_to_target = f32::MAX;
        
        // Clear target if we haven't seen it for too long
        if ai.decision_factors.time_since_last_seen_target > 5.0 {
            ai.set_target(None);
            ai.memory.last_known_target_position = None;
        }
    }

    ai.decision_factors.enemies_nearby = enemies_nearby;
    
    // TODO: Update allies_nearby, noise_level, light_level based on game state
    ai.decision_factors.allies_nearby = 0;
    ai.decision_factors.noise_level = 0.0;
    ai.decision_factors.light_level = 1.0;
}

/// System for AI target selection
pub fn ai_target_selection_system(
    mut ai_query: Query<(Entity, &mut AIComponent, &mut AITargetSelector, &Position), With<AITargetSelector>>,
    potential_targets: Query<(Entity, &Position, &Health), Without<AIComponent>>,
    viewshed_query: Query<&Viewshed>,
) {
    for (ai_entity, mut ai, mut target_selector, ai_position) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        let mut best_target: Option<(Entity, f32)> = None;

        // Evaluate all potential targets
        for (target_entity, target_position, target_health) in potential_targets.iter() {
            let distance = ai_position.0.distance(target_position.0);
            
            // Skip targets outside detection range
            if distance > target_selector.detection_range {
                continue;
            }

            // Check if target is visible
            if let Ok(viewshed) = viewshed_query.get(ai_entity) {
                if !viewshed.visible_tiles.contains(&target_position.0.as_ivec2()) {
                    continue;
                }
            }

            // Calculate target priority
            let health_percentage = target_health.current as f32 / target_health.max as f32;
            let threat_level = 1.0 - health_percentage; // Simple threat calculation
            let visibility = 1.0; // If we can see it, it's fully visible
            
            let priority = target_selector.calculate_target_priority(
                distance,
                health_percentage,
                threat_level,
                visibility,
            );

            // Update best target if this one has higher priority
            if best_target.is_none() || priority > best_target.unwrap().1 {
                best_target = Some((target_entity, priority));
            }
        }

        // Update AI target
        if let Some((target_entity, _)) = best_target {
            ai.set_target(Some(target_entity));
            target_selector.preferred_target = Some(target_entity);
        } else {
            ai.set_target(None);
            target_selector.preferred_target = None;
        }
    }
}

/// System for AI decision making using decision trees
pub fn ai_decision_system(
    mut ai_query: Query<(&mut AIComponent, &mut AIDecisionSystem), With<AIDecisionSystem>>,
) {
    for (mut ai, mut decision_system) in ai_query.iter_mut() {
        if !ai.enabled || !ai.can_react() {
            continue;
        }

        // Make a decision based on current state
        let action = decision_system.make_decision(&ai);
        
        // Execute the action
        execute_ai_action(&mut ai, &mut decision_system, action);
    }
}

/// Execute an AI action
fn execute_ai_action(
    ai: &mut AIComponent,
    decision_system: &mut AIDecisionSystem,
    action: AIAction,
) {
    match action {
        AIAction::SetState(new_state) => {
            if new_state != ai.current_state {
                ai.transition_to_state(new_state);
            }
        }
        AIAction::MoveToTarget => {
            // This would be handled by movement systems
            // For now, just ensure we're in the right state
            if ai.current_state != AIBehaviorState::Hunt && ai.current_state != AIBehaviorState::Attack {
                ai.transition_to_state(AIBehaviorState::Hunt);
            }
        }
        AIAction::MoveToPosition(position) => {
            // Store the target position in memory for movement systems to use
            ai.memory.last_known_target_position = Some(position);
            if ai.current_state == AIBehaviorState::Idle {
                ai.transition_to_state(AIBehaviorState::Patrol);
            }
        }
        AIAction::Attack => {
            ai.transition_to_state(AIBehaviorState::Attack);
        }
        AIAction::Flee => {
            ai.transition_to_state(AIBehaviorState::Flee);
        }
        AIAction::CallForHelp => {
            // Set a context variable that other systems can read
            decision_system.set_context_variable("calling_for_help".to_string(), 1.0);
        }
        AIAction::UseItem(item_name) => {
            // Store the item to use in context variables
            decision_system.set_context_variable("use_item".to_string(), 1.0);
            // In a real implementation, you'd store the item name somewhere accessible
        }
        AIAction::Wait(duration) => {
            decision_system.set_context_variable("wait_timer".to_string(), duration);
        }
        AIAction::Custom(action_name) => {
            // Handle custom actions
            decision_system.set_context_variable(format!("custom_{}", action_name), 1.0);
        }
    }
}

/// System for handling AI state-specific behaviors
pub fn ai_state_behavior_system(
    mut ai_query: Query<(Entity, &mut AIComponent, &Position), With<AIComponent>>,
    mut commands: Commands,
) {
    for (entity, mut ai, position) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        match ai.current_state {
            AIBehaviorState::Idle => {
                // Do nothing, just wait
            }
            AIBehaviorState::Patrol => {
                handle_patrol_behavior(&mut ai, position);
            }
            AIBehaviorState::Hunt => {
                handle_hunt_behavior(&mut ai, position);
            }
            AIBehaviorState::Attack => {
                handle_attack_behavior(&mut ai, entity, &mut commands);
            }
            AIBehaviorState::Flee => {
                handle_flee_behavior(&mut ai, position);
            }
            AIBehaviorState::Search => {
                handle_search_behavior(&mut ai, position);
            }
            AIBehaviorState::Guard => {
                handle_guard_behavior(&mut ai, position);
            }
            AIBehaviorState::Follow => {
                handle_follow_behavior(&mut ai, position);
            }
            AIBehaviorState::Wander => {
                handle_wander_behavior(&mut ai, position);
            }
            AIBehaviorState::Dead => {
                // AI is disabled when dead
                ai.enabled = false;
            }
        }
    }
}

fn handle_patrol_behavior(ai: &mut AIComponent, position: &Position) {
    if let Some(patrol_target) = ai.get_current_patrol_target() {
        let distance_to_patrol_point = position.0.distance(patrol_target);
        
        // If we've reached the patrol point, move to the next one
        if distance_to_patrol_point < 1.0 {
            ai.advance_patrol();
        }
    } else {
        // No patrol points, switch to wander
        ai.transition_to_state(AIBehaviorState::Wander);
    }
}

fn handle_hunt_behavior(ai: &mut AIComponent, _position: &Position) {
    // Hunting behavior is primarily handled by movement systems
    // Here we just ensure the AI stays focused on the target
    if ai.current_target.is_none() {
        ai.transition_to_state(AIBehaviorState::Search);
    }
}

fn handle_attack_behavior(ai: &mut AIComponent, entity: Entity, commands: &mut Commands) {
    // Trigger attack action
    if ai.can_react() {
        // In a real implementation, this would trigger combat systems
        // For now, we'll just add a marker component
        commands.entity(entity).insert(AttackIntent);
        ai.reaction_timer = 0.0; // Reset reaction timer
    }
}

fn handle_flee_behavior(ai: &mut AIComponent, position: &Position) {
    // Set flee target away from enemies
    if let Some(target_pos) = ai.memory.last_known_target_position {
        let flee_direction = (position.0 - target_pos).normalize();
        let flee_target = position.0 + flee_direction * 10.0;
        ai.memory.last_known_target_position = Some(flee_target);
    } else {
        // Flee towards home position
        ai.memory.last_known_target_position = Some(ai.memory.home_position);
    }
}

fn handle_search_behavior(ai: &mut AIComponent, position: &Position) {
    // Search around the last known target position
    if let Some(last_pos) = ai.memory.last_known_target_position {
        let distance_to_search_area = position.0.distance(last_pos);
        
        // If we're at the search location, look around
        if distance_to_search_area < 2.0 {
            // Create a search pattern around the last known position
            let search_offset = Vec2::new(
                (ai.state_timer.sin() * 3.0),
                (ai.state_timer.cos() * 3.0),
            );
            ai.memory.last_known_target_position = Some(last_pos + search_offset);
        }
    }
}

fn handle_guard_behavior(ai: &mut AIComponent, position: &Position) {
    // Stay near the guard position (home position)
    let distance_to_home = position.0.distance(ai.memory.home_position);
    
    if distance_to_home > 3.0 {
        // Return to guard position
        ai.memory.last_known_target_position = Some(ai.memory.home_position);
    }
}

fn handle_follow_behavior(ai: &mut AIComponent, _position: &Position) {
    // Following behavior would be handled by movement systems
    // based on the current target
    if ai.current_target.is_none() {
        ai.transition_to_state(AIBehaviorState::Idle);
    }
}

fn handle_wander_behavior(ai: &mut AIComponent, position: &Position) {
    // Random wandering around the home position
    if ai.state_timer > 3.0 {
        let wander_radius = 5.0;
        let random_angle = ai.state_timer; // Simple pseudo-random based on time
        let wander_offset = Vec2::new(
            random_angle.sin() * wander_radius,
            random_angle.cos() * wander_radius,
        );
        let wander_target = ai.memory.home_position + wander_offset;
        ai.memory.last_known_target_position = Some(wander_target);
        ai.state_timer = 0.0;
    }
}

/// Marker component for entities that want to attack
#[derive(Component)]
pub struct AttackIntent;

/// System for cleaning up AI components when entities die
pub fn ai_cleanup_system(
    mut ai_query: Query<&mut AIComponent>,
    health_query: Query<&Health>,
) {
    for mut ai in ai_query.iter_mut() {
        if let Some(target) = ai.current_target {
            // Check if target still exists and is alive
            if let Ok(target_health) = health_query.get(target) {
                if target_health.current <= 0 {
                    ai.set_target(None);
                }
            } else {
                // Target entity no longer exists
                ai.set_target(None);
            }
        }
    }
}

/// Plugin for AI behavior systems
pub struct AIBehaviorPlugin;

impl Plugin for AIBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            ai_behavior_system,
            ai_target_selection_system,
            ai_decision_system,
            ai_state_behavior_system,
            ai_cleanup_system,
        ).chain());
    }
}