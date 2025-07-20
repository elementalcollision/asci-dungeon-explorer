use bevy::prelude::*;
use crate::ai::{AIComponent, AIBehaviorState, PerceptionComponent, PerceptionMemory};
use crate::components::Position;

/// System that integrates perception data with AI decision making
pub fn perception_ai_integration_system(
    time: Res<Time>,
    mut ai_query: Query<(Entity, &mut AIComponent, &PerceptionComponent, &Position)>,
) {
    let current_time = time.elapsed_seconds();

    for (entity, mut ai, perception, position) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        // Update AI decision factors based on perception
        update_ai_from_perception(&mut ai, perception, current_time);

        // Update AI behavior based on perceived threats
        update_ai_behavior_from_perception(&mut ai, perception, position, current_time);

        // Update AI memory with perception data
        sync_ai_memory_with_perception(&mut ai, perception, current_time);
    }
}

/// Update AI decision factors based on what the entity perceives
fn update_ai_from_perception(
    ai: &mut AIComponent,
    perception: &PerceptionComponent,
    current_time: f32,
) {
    let hostile_memories = perception.get_hostile_memories(current_time);
    
    // Count nearby enemies and allies
    let mut enemies_nearby = 0;
    let mut allies_nearby = 0;
    let mut closest_enemy_distance = f32::MAX;
    let mut strongest_threat: Option<&PerceptionMemory> = None;

    for memory in &hostile_memories {
        let distance = ai.memory.home_position.distance(memory.last_known_position.as_vec2());
        
        if distance <= perception.detection_range {
            if memory.was_hostile {
                enemies_nearby += 1;
                if distance < closest_enemy_distance {
                    closest_enemy_distance = distance;
                    strongest_threat = Some(memory);
                }
            } else {
                allies_nearby += 1;
            }
        }
    }

    // Update AI decision factors
    ai.decision_factors.enemies_nearby = enemies_nearby;
    ai.decision_factors.allies_nearby = allies_nearby;
    ai.decision_factors.distance_to_target = closest_enemy_distance;

    // Update target information
    if let Some(threat) = strongest_threat {
        ai.set_target(Some(threat.entity));
        ai.memory.last_known_target_position = Some(threat.last_known_position.as_vec2());
        ai.decision_factors.time_since_last_seen_target = current_time - threat.last_seen_time;
        
        // Calculate target strength based on memory
        let health_ratio = threat.health_when_last_seen as f32 / 100.0; // Assume max health of 100
        ai.decision_factors.target_strength = threat.threat_level * health_ratio;
    } else if ai.decision_factors.time_since_last_seen_target < 30.0 {
        // Increment time since last seen if no current target
        ai.decision_factors.time_since_last_seen_target += 0.1;
    }

    // Update noise and light levels (simplified)
    ai.decision_factors.noise_level = calculate_ambient_noise_level(perception);
    ai.decision_factors.light_level = 1.0; // TODO: Implement proper lighting
}

/// Update AI behavior state based on perception
fn update_ai_behavior_from_perception(
    ai: &mut AIComponent,
    perception: &PerceptionComponent,
    position: &Position,
    current_time: f32,
) {
    let hostile_memories = perception.get_hostile_memories(current_time);
    let reliable_memories = perception.get_reliable_memories(current_time);

    match ai.current_state {
        AIBehaviorState::Idle => {
            // Transition to alert states based on perception
            if !hostile_memories.is_empty() {
                let closest_threat = find_closest_threat(&hostile_memories, position.0);
                if let Some(threat) = closest_threat {
                    let distance = position.0.distance(threat.last_known_position.as_vec2());
                    
                    if distance <= 3.0 && ai.personality.aggression > 0.6 {
                        ai.transition_to_state(AIBehaviorState::Attack);
                    } else if distance <= 8.0 {
                        ai.transition_to_state(AIBehaviorState::Hunt);
                    } else {
                        ai.transition_to_state(AIBehaviorState::Search);
                    }
                }
            } else if !reliable_memories.is_empty() {
                // Investigate non-hostile but interesting entities
                if ai.personality.curiosity > 0.5 {
                    ai.transition_to_state(AIBehaviorState::Search);
                }
            }
        }

        AIBehaviorState::Patrol => {
            // Interrupt patrol if threats are detected
            if !hostile_memories.is_empty() {
                let closest_threat = find_closest_threat(&hostile_memories, position.0);
                if let Some(threat) = closest_threat {
                    let distance = position.0.distance(threat.last_known_position.as_vec2());
                    
                    if distance <= perception.detection_range * 0.8 {
                        ai.transition_to_state(AIBehaviorState::Hunt);
                    }
                }
            }
        }

        AIBehaviorState::Hunt => {
            // Continue hunting or escalate to attack
            if let Some(current_target) = ai.current_target {
                if let Some(target_memory) = perception.get_memory(current_target) {
                    let distance = position.0.distance(target_memory.last_known_position.as_vec2());
                    
                    if distance <= 2.0 && ai.can_react() {
                        ai.transition_to_state(AIBehaviorState::Attack);
                    } else if !target_memory.is_reliable(current_time, perception.memory_duration) {
                        // Target memory is stale, switch to search
                        ai.transition_to_state(AIBehaviorState::Search);
                    }
                } else {
                    // No memory of current target, search for new threats
                    if hostile_memories.is_empty() {
                        ai.transition_to_state(AIBehaviorState::Search);
                    } else {
                        // Switch to new threat
                        let new_threat = find_closest_threat(&hostile_memories, position.0);
                        if let Some(threat) = new_threat {
                            ai.set_target(Some(threat.entity));
                        }
                    }
                }
            }
        }

        AIBehaviorState::Attack => {
            // Verify target is still in attack range
            if let Some(current_target) = ai.current_target {
                if let Some(target_memory) = perception.get_memory(current_target) {
                    let distance = position.0.distance(target_memory.last_known_position.as_vec2());
                    
                    if distance > 3.0 {
                        ai.transition_to_state(AIBehaviorState::Hunt);
                    } else if !target_memory.is_reliable(current_time, 2.0) {
                        // Lost sight of target during combat
                        ai.transition_to_state(AIBehaviorState::Search);
                    }
                }
            } else {
                // No current target in attack state
                if !hostile_memories.is_empty() {
                    let new_threat = find_closest_threat(&hostile_memories, position.0);
                    if let Some(threat) = new_threat {
                        ai.set_target(Some(threat.entity));
                        ai.transition_to_state(AIBehaviorState::Hunt);
                    }
                } else {
                    ai.transition_to_state(AIBehaviorState::Search);
                }
            }
        }

        AIBehaviorState::Search => {
            // Look for new targets or return to patrol
            if !hostile_memories.is_empty() {
                let new_threat = find_closest_threat(&hostile_memories, position.0);
                if let Some(threat) = new_threat {
                    ai.set_target(Some(threat.entity));
                    ai.transition_to_state(AIBehaviorState::Hunt);
                }
            } else if ai.state_timer > 10.0 {
                // Give up searching after a while
                if ai.memory.patrol_points.is_empty() {
                    ai.transition_to_state(AIBehaviorState::Idle);
                } else {
                    ai.transition_to_state(AIBehaviorState::Patrol);
                }
            }
        }

        AIBehaviorState::Flee => {
            // Check if it's safe to stop fleeing
            let immediate_threats = hostile_memories.iter()
                .filter(|memory| {
                    let distance = position.0.distance(memory.last_known_position.as_vec2());
                    distance <= perception.detection_range * 0.5
                })
                .count();

            if immediate_threats == 0 && ai.state_timer > 5.0 {
                // Safe to stop fleeing
                if ai.decision_factors.health_percentage > 0.5 {
                    ai.transition_to_state(AIBehaviorState::Guard);
                } else {
                    ai.transition_to_state(AIBehaviorState::Idle);
                }
            }
        }

        _ => {
            // Handle other states with basic threat response
            if !hostile_memories.is_empty() && ai.personality.alertness > 0.5 {
                let closest_threat = find_closest_threat(&hostile_memories, position.0);
                if let Some(threat) = closest_threat {
                    let distance = position.0.distance(threat.last_known_position.as_vec2());
                    
                    if distance <= perception.detection_range * 0.6 {
                        if ai.should_flee() {
                            ai.transition_to_state(AIBehaviorState::Flee);
                        } else if ai.should_be_aggressive() {
                            ai.set_target(Some(threat.entity));
                            ai.transition_to_state(AIBehaviorState::Hunt);
                        }
                    }
                }
            }
        }
    }
}

/// Synchronize AI memory with perception data
fn sync_ai_memory_with_perception(
    ai: &mut AIComponent,
    perception: &PerceptionComponent,
    current_time: f32,
) {
    let reliable_memories = perception.get_reliable_memories(current_time);

    // Update AI's known enemies and allies
    ai.memory.known_enemies.clear();
    ai.memory.known_allies.clear();

    for memory in reliable_memories {
        let position = memory.last_known_position.as_vec2();
        let time = memory.last_seen_time;

        if memory.was_hostile {
            ai.remember_enemy(memory.entity, position, time);
        } else {
            ai.remember_ally(memory.entity, position, time);
        }
    }

    // Update interesting locations based on where entities were seen
    ai.memory.interesting_locations.clear();
    for memory in perception.perceived_entities.values() {
        if memory.confidence > 0.5 {
            let interest_level = memory.threat_level * memory.confidence;
            ai.memory.interesting_locations.push((memory.last_known_position.as_vec2(), interest_level));
        }
    }

    // Sort interesting locations by interest level
    ai.memory.interesting_locations.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    
    // Keep only top 5 interesting locations
    ai.memory.interesting_locations.truncate(5);
}

/// Find the closest threat from a list of hostile memories
fn find_closest_threat(hostile_memories: &[&PerceptionMemory], position: Vec2) -> Option<&PerceptionMemory> {
    hostile_memories.iter()
        .min_by(|a, b| {
            let dist_a = position.distance(a.last_known_position.as_vec2());
            let dist_b = position.distance(b.last_known_position.as_vec2());
            dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
        })
        .copied()
}

/// Calculate ambient noise level based on perception
fn calculate_ambient_noise_level(perception: &PerceptionComponent) -> f32 {
    // Simple calculation based on number of perceived entities
    let entity_count = perception.perceived_entities.len() as f32;
    (entity_count * 0.1).min(1.0)
}

/// System for updating AI alertness based on perception
pub fn perception_alertness_system(
    mut ai_query: Query<(&mut AIComponent, &PerceptionComponent)>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    for (mut ai, perception) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        let hostile_count = perception.get_hostile_memories(current_time).len();
        let total_perceived = perception.get_reliable_memories(current_time).len();

        // Increase alertness based on perceived threats
        let threat_alertness = (hostile_count as f32 * 0.2).min(0.5);
        let general_alertness = (total_perceived as f32 * 0.05).min(0.2);
        
        let target_alertness = ai.personality.alertness + threat_alertness + general_alertness;
        
        // Gradually adjust alertness towards target
        let alertness_change_rate = 2.0 * time.delta_seconds();
        if target_alertness > ai.personality.alertness {
            ai.personality.alertness = (ai.personality.alertness + alertness_change_rate).min(target_alertness).min(1.0);
        } else {
            ai.personality.alertness = (ai.personality.alertness - alertness_change_rate * 0.5).max(target_alertness).max(0.1);
        }
    }
}

/// System for handling perception-based communication between AI entities
pub fn perception_communication_system(
    mut ai_query: Query<(Entity, &mut PerceptionComponent, &Position, &AIComponent)>,
    group_resource: Res<crate::ai::GroupCoordinationResource>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    // Share urgent threat information within communication range
    let mut urgent_threats: Vec<(Vec2, PerceptionMemory)> = Vec::new();

    // Collect urgent threats
    for (entity, perception, position, ai) in ai_query.iter() {
        if !ai.enabled {
            continue;
        }

        for memory in perception.get_hostile_memories(current_time) {
            if memory.threat_level > 0.7 && current_time - memory.last_seen_time < 5.0 {
                urgent_threats.push((position.0, memory.clone()));
            }
        }
    }

    // Share urgent threats with nearby entities
    for (entity, mut perception, position, ai) in ai_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        for (threat_reporter_pos, threat_memory) in &urgent_threats {
            let distance = position.0.distance(*threat_reporter_pos);
            
            if distance <= perception.hearing_range {
                // Add shared threat information with reduced confidence
                if !perception.perceived_entities.contains_key(&threat_memory.entity) {
                    let mut shared_memory = threat_memory.clone();
                    shared_memory.confidence *= 0.6; // Shared information is less reliable
                    shared_memory.last_seen_time = current_time; // Update to current time
                    perception.perceived_entities.insert(threat_memory.entity, shared_memory);
                }
            }
        }
    }
}

/// Plugin for perception integration
pub struct PerceptionIntegrationPlugin;

impl Plugin for PerceptionIntegrationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            perception_ai_integration_system,
            perception_alertness_system,
            perception_communication_system,
        ).after(crate::ai::perception_system));
    }
}