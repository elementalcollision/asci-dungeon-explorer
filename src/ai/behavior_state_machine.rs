use specs::{System, ReadStorage, WriteStorage, Entities, Entity, Join};
use std::time::{Duration, Instant};
use crate::ai::ai_components::{AI, AIBehaviorState, AIDecisionFactors, AITargetSelector};
use crate::components::{Position, Health, Player, Name};

/// Behavior state machine system that manages AI state transitions
pub struct BehaviorStateMachineSystem {
    last_update: Instant,
    update_frequency: Duration,
}

impl BehaviorStateMachineSystem {
    pub fn new() -> Self {
        BehaviorStateMachineSystem {
            last_update: Instant::now(),
            update_frequency: Duration::from_millis(100), // Update 10 times per second
        }
    }

    pub fn with_update_frequency(mut self, frequency: Duration) -> Self {
        self.update_frequency = frequency;
        self
    }
}

impl<'a> System<'a> for BehaviorStateMachineSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, AI>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, AITargetSelector>,
    );

    fn run(&mut self, (entities, mut ais, positions, healths, players, names, mut target_selectors): Self::SystemData) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_frequency {
            return;
        }

        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Collect player entities for targeting
        let player_entities: Vec<Entity> = (&entities, &players).join().map(|(e, _)| e).collect();

        for (entity, ai, position, health) in (&entities, &mut ais, &positions, &healths).join() {
            if !ai.enabled {
                continue;
            }

            // Update AI timer
            ai.update_timer(delta_time);

            // Clean up old memories
            ai.memory.cleanup_old_memories();

            // Update decision factors
            ai.decision_factors = self.calculate_decision_factors(
                entity,
                &ai,
                position,
                health,
                &player_entities,
                &positions,
                &healths,
            );

            // Make decision if enough time has passed
            if ai.can_make_decision() {
                if let Some(mut decision_system) = ai.decision_factors.clone().into() {
                    if let Some(new_state) = decision_system.make_decision(&ai.decision_factors, &ai.personality) {
                        if ai.can_interrupt_with(&new_state) {
                            ai.change_state(new_state);
                        }
                    }
                }
            }

            // Execute current state behavior
            self.execute_state_behavior(entity, ai, position, &player_entities, &positions);

            // Update target selector if present
            if let Some(target_selector) = target_selectors.get_mut(entity) {
                if target_selector.can_update_target() {
                    ai.current_target = self.select_target(
                        entity,
                        position,
                        target_selector,
                        &player_entities,
                        &positions,
                        &ai.memory,
                    );
                    target_selector.mark_target_updated();
                }
            }
        }
    }
}

impl BehaviorStateMachineSystem {
    /// Calculate decision factors for an AI entity
    fn calculate_decision_factors(
        &self,
        entity: Entity,
        ai: &AI,
        position: &Position,
        health: &Health,
        player_entities: &[Entity],
        positions: &ReadStorage<Position>,
        healths: &ReadStorage<Health>,
    ) -> AIDecisionFactors {
        let mut factors = AIDecisionFactors::default();

        // Health percentage
        factors.health_percentage = health.current as f32 / health.max as f32;

        // Distance to target and line of sight
        if let Some(target) = ai.current_target {
            if let Some(target_pos) = positions.get(target) {
                factors.distance_to_target = self.calculate_distance(position, target_pos);
                factors.has_line_of_sight = self.has_line_of_sight(position, target_pos);
            }
        }

        // Count nearby enemies and allies
        let detection_range = ai.behavior_params.detection_range as f32;
        let mut enemies_nearby = 0;
        let mut allies_nearby = 0;

        for &player_entity in player_entities {
            if let Some(player_pos) = positions.get(player_entity) {
                let distance = self.calculate_distance(position, player_pos);
                if distance <= detection_range {
                    enemies_nearby += 1;
                }
            }
        }

        // Count AI allies nearby
        // In a full implementation, this would check for other AI entities with the same faction
        for ally in &ai.memory.allies {
            if let Some(ally_pos) = positions.get(*ally) {
                let distance = self.calculate_distance(position, ally_pos);
                if distance <= detection_range {
                    allies_nearby += 1;
                }
            }
        }

        factors.number_of_enemies = enemies_nearby;
        factors.number_of_allies = allies_nearby;
        factors.is_outnumbered = enemies_nearby > allies_nearby + 1;

        // Time since last action
        factors.time_since_last_action = ai.time_in_current_state();

        // Current threat level based on nearby enemies and health
        factors.current_threat_level = if enemies_nearby > 0 {
            let health_factor = 1.0 - factors.health_percentage;
            let enemy_factor = enemies_nearby as f32 * 0.2;
            (health_factor + enemy_factor).min(1.0)
        } else {
            0.0
        };

        // Energy level (simplified - could be based on stamina, mana, etc.)
        factors.energy_level = 1.0; // Placeholder

        factors
    }

    /// Execute behavior for the current state
    fn execute_state_behavior(
        &self,
        entity: Entity,
        ai: &mut AI,
        position: &Position,
        player_entities: &[Entity],
        positions: &ReadStorage<Position>,
    ) {
        match ai.current_state {
            AIBehaviorState::Idle => {
                self.execute_idle_behavior(ai);
            },
            AIBehaviorState::Patrol => {
                self.execute_patrol_behavior(ai, position);
            },
            AIBehaviorState::Hunt => {
                self.execute_hunt_behavior(ai, position, positions);
            },
            AIBehaviorState::Attack => {
                self.execute_attack_behavior(ai, position, positions);
            },
            AIBehaviorState::Flee => {
                self.execute_flee_behavior(ai, position, player_entities, positions);
            },
            AIBehaviorState::Search => {
                self.execute_search_behavior(ai, position);
            },
            AIBehaviorState::Guard => {
                self.execute_guard_behavior(ai, position);
            },
            AIBehaviorState::Follow => {
                self.execute_follow_behavior(ai, position, positions);
            },
            AIBehaviorState::Wander => {
                self.execute_wander_behavior(ai, position);
            },
            AIBehaviorState::Dead => {
                // Do nothing when dead
            },
        }
    }

    fn execute_idle_behavior(&self, ai: &mut AI) {
        // In idle state, occasionally switch to wander or patrol
        if ai.time_in_current_state() > Duration::from_secs(5) {
            if !ai.behavior_params.patrol_points.is_empty() {
                ai.change_state(AIBehaviorState::Patrol);
            } else if ai.personality.curiosity > 0.5 {
                ai.change_state(AIBehaviorState::Wander);
            }
        }
    }

    fn execute_patrol_behavior(&self, ai: &mut AI, position: &Position) {
        if ai.behavior_params.patrol_points.is_empty() {
            ai.change_state(AIBehaviorState::Idle);
            return;
        }

        // Simple patrol logic - in a full implementation, this would move the entity
        // For now, we just cycle through patrol points conceptually
        if ai.time_in_current_state() > Duration::from_secs(3) {
            // Move to next patrol point (simplified)
            ai.state_timer = Duration::from_secs(0);
        }
    }

    fn execute_hunt_behavior(&self, ai: &mut AI, position: &Position, positions: &ReadStorage<Position>) {
        if let Some(target) = ai.current_target {
            if let Some(target_pos) = positions.get(target) {
                let distance = self.calculate_distance(position, target_pos);
                
                // If close enough, switch to attack
                if distance <= ai.behavior_params.attack_range as f32 {
                    ai.change_state(AIBehaviorState::Attack);
                }
                
                // Remember target position
                ai.memory.remember_entity(target, *target_pos);
                
                // If we've been hunting for too long without getting closer, search
                if ai.time_in_current_state() > Duration::from_secs(10) {
                    ai.change_state(AIBehaviorState::Search);
                }
            } else {
                // Target no longer exists, search for it
                ai.change_state(AIBehaviorState::Search);
            }
        } else {
            // No target, go back to patrol or idle
            ai.change_state(AIBehaviorState::Patrol);
        }
    }

    fn execute_attack_behavior(&self, ai: &mut AI, position: &Position, positions: &ReadStorage<Position>) {
        if let Some(target) = ai.current_target {
            if let Some(target_pos) = positions.get(target) {
                let distance = self.calculate_distance(position, target_pos);
                
                // If target moved away, hunt it
                if distance > ai.behavior_params.attack_range as f32 {
                    ai.change_state(AIBehaviorState::Hunt);
                }
                
                // Attack logic would go here (damage dealing, etc.)
                // For now, we just stay in attack state
            } else {
                // Target no longer exists
                ai.current_target = None;
                ai.change_state(AIBehaviorState::Search);
            }
        } else {
            ai.change_state(AIBehaviorState::Idle);
        }
    }

    fn execute_flee_behavior(
        &self,
        ai: &mut AI,
        position: &Position,
        player_entities: &[Entity],
        positions: &ReadStorage<Position>,
    ) {
        // Find the nearest threat to flee from
        let mut nearest_threat_distance = f32::MAX;
        
        for &player_entity in player_entities {
            if let Some(player_pos) = positions.get(player_entity) {
                let distance = self.calculate_distance(position, player_pos);
                if distance < nearest_threat_distance {
                    nearest_threat_distance = distance;
                }
            }
        }

        // If we've fled far enough, consider other actions
        if nearest_threat_distance > ai.behavior_params.flee_distance as f32 {
            // Check if we should continue fleeing based on health and personality
            let flee_threshold = ai.get_flee_threshold();
            if ai.decision_factors.health_percentage > flee_threshold {
                ai.change_state(AIBehaviorState::Search);
            }
        }

        // Flee logic would move the entity away from threats
        // This would be implemented in a movement system
    }

    fn execute_search_behavior(&self, ai: &mut AI, position: &Position) {
        // Search for last known target position
        if let Some((last_pos, _)) = ai.memory.last_known_player_position {
            let distance = self.calculate_distance(position, &last_pos);
            
            // If we're at the last known position and haven't found anything, expand search
            if distance < 2.0 && ai.time_in_current_state() > Duration::from_secs(3) {
                // Expand search area or give up
                if ai.time_in_current_state() > Duration::from_secs(15) {
                    ai.change_state(AIBehaviorState::Patrol);
                }
            }
        } else {
            // No last known position, patrol or wander
            if ai.time_in_current_state() > Duration::from_secs(5) {
                ai.change_state(AIBehaviorState::Patrol);
            }
        }
    }

    fn execute_guard_behavior(&self, ai: &mut AI, position: &Position) {
        if let Some(guard_pos) = ai.behavior_params.guard_position {
            let distance = self.calculate_distance(position, &guard_pos);
            
            // If too far from guard position, return to it
            if distance > 3.0 {
                // Move back to guard position (would be handled by movement system)
            }
            
            // Look for threats while guarding
            // This would be handled by the perception system
        } else {
            // No guard position set, switch to patrol or idle
            ai.change_state(AIBehaviorState::Patrol);
        }
    }

    fn execute_follow_behavior(&self, ai: &mut AI, position: &Position, positions: &ReadStorage<Position>) {
        if let Some(follow_target) = ai.behavior_params.follow_target {
            if let Some(target_pos) = positions.get(follow_target) {
                let distance = self.calculate_distance(position, target_pos);
                
                // Maintain following distance
                let follow_distance = 3.0;
                if distance > follow_distance + 2.0 {
                    // Move closer (would be handled by movement system)
                } else if distance < follow_distance - 1.0 {
                    // Move away slightly
                }
            } else {
                // Follow target no longer exists
                ai.behavior_params.follow_target = None;
                ai.change_state(AIBehaviorState::Idle);
            }
        } else {
            ai.change_state(AIBehaviorState::Idle);
        }
    }

    fn execute_wander_behavior(&self, ai: &mut AI, position: &Position) {
        // Wander around randomly
        if ai.time_in_current_state() > Duration::from_secs(8) {
            // Pick a new random direction or return to patrol
            if !ai.behavior_params.patrol_points.is_empty() && ai.personality.curiosity < 0.7 {
                ai.change_state(AIBehaviorState::Patrol);
            } else {
                // Continue wandering (movement would be handled by movement system)
                ai.state_timer = Duration::from_secs(0);
            }
        }
    }

    /// Select a target based on the target selector configuration
    fn select_target(
        &self,
        entity: Entity,
        position: &Position,
        target_selector: &AITargetSelector,
        player_entities: &[Entity],
        positions: &ReadStorage<Position>,
        memory: &crate::ai::ai_components::AIMemory,
    ) -> Option<Entity> {
        let mut candidates = Vec::new();

        // Collect potential targets based on target types
        for target_type in &target_selector.target_types {
            match target_type {
                crate::ai::ai_components::AITargetType::Player => {
                    for &player_entity in player_entities {
                        if let Some(player_pos) = positions.get(player_entity) {
                            let distance = self.calculate_distance(position, player_pos);
                            if distance <= target_selector.max_target_distance {
                                candidates.push((player_entity, distance, 1.0)); // priority
                            }
                        }
                    }
                },
                // Other target types would be implemented here
                _ => {},
            }
        }

        if candidates.is_empty() {
            return None;
        }

        // Select target based on strategy
        match target_selector.selection_strategy {
            crate::ai::ai_components::TargetSelectionStrategy::Nearest => {
                candidates.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                Some(candidates[0].0)
            },
            crate::ai::ai_components::TargetSelectionStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..candidates.len());
                Some(candidates[index].0)
            },
            crate::ai::ai_components::TargetSelectionStrategy::LastSeen => {
                // Find the most recently seen target
                let mut best_target = None;
                let mut most_recent_time = None;

                for (candidate_entity, _, _) in candidates {
                    if let Some((_, seen_time)) = memory.seen_entities.get(&candidate_entity) {
                        if most_recent_time.is_none() || seen_time > most_recent_time.as_ref().unwrap() {
                            most_recent_time = Some(seen_time);
                            best_target = Some(candidate_entity);
                        }
                    }
                }

                best_target.or_else(|| Some(candidates[0].0))
            },
            crate::ai::ai_components::TargetSelectionStrategy::HighestPriority => {
                candidates.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
                Some(candidates[0].0)
            },
            // Other strategies would be implemented here
            _ => Some(candidates[0].0),
        }
    }

    /// Calculate distance between two positions
    fn calculate_distance(&self, pos1: &Position, pos2: &Position) -> f32 {
        let dx = (pos1.x - pos2.x) as f32;
        let dy = (pos1.y - pos2.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Check if there's line of sight between two positions (simplified)
    fn has_line_of_sight(&self, pos1: &Position, pos2: &Position) -> bool {
        // Simplified line of sight - in a real implementation, this would
        // check for walls and obstacles between the positions
        let distance = self.calculate_distance(pos1, pos2);
        distance <= 10.0 // Can see up to 10 units away
    }
}

/// AI state transition rules
pub struct StateTransitionRules {
    pub rules: Vec<StateTransitionRule>,
}

pub struct StateTransitionRule {
    pub from_state: AIBehaviorState,
    pub to_state: AIBehaviorState,
    pub condition: Box<dyn Fn(&AIDecisionFactors, &crate::ai::ai_components::AIPersonality) -> bool>,
    pub priority: u32,
}

impl StateTransitionRules {
    pub fn new() -> Self {
        StateTransitionRules {
            rules: Vec::new(),
        }
    }

    pub fn add_rule<F>(&mut self, from: AIBehaviorState, to: AIBehaviorState, condition: F, priority: u32)
    where
        F: Fn(&AIDecisionFactors, &crate::ai::ai_components::AIPersonality) -> bool + 'static,
    {
        self.rules.push(StateTransitionRule {
            from_state: from,
            to_state: to,
            condition: Box::new(condition),
            priority,
        });
    }

    pub fn evaluate_transitions(
        &self,
        current_state: &AIBehaviorState,
        factors: &AIDecisionFactors,
        personality: &crate::ai::ai_components::AIPersonality,
    ) -> Option<AIBehaviorState> {
        let mut applicable_rules: Vec<&StateTransitionRule> = self.rules
            .iter()
            .filter(|rule| &rule.from_state == current_state)
            .filter(|rule| (rule.condition)(factors, personality))
            .collect();

        // Sort by priority (higher priority first)
        applicable_rules.sort_by(|a, b| b.priority.cmp(&a.priority));

        applicable_rules.first().map(|rule| rule.to_state.clone())
    }
}

impl Default for StateTransitionRules {
    fn default() -> Self {
        let mut rules = StateTransitionRules::new();

        // Add default transition rules
        rules.add_rule(
            AIBehaviorState::Idle,
            AIBehaviorState::Hunt,
            |factors, _| factors.has_line_of_sight && factors.distance_to_target < 10.0,
            80,
        );

        rules.add_rule(
            AIBehaviorState::Hunt,
            AIBehaviorState::Attack,
            |factors, _| factors.distance_to_target <= 2.0,
            90,
        );

        rules.add_rule(
            AIBehaviorState::Attack,
            AIBehaviorState::Flee,
            |factors, personality| {
                factors.health_percentage < (1.0 - personality.courage) * 0.5
            },
            100,
        );

        rules.add_rule(
            AIBehaviorState::Hunt,
            AIBehaviorState::Search,
            |factors, _| !factors.has_line_of_sight && factors.distance_to_target > 15.0,
            60,
        );

        rules
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::ai::ai_components::{AI, AIPersonality};

    #[test]
    fn test_behavior_state_machine_system() {
        let mut world = World::new();
        world.register::<AI>();
        world.register::<Position>();
        world.register::<Health>();
        world.register::<Player>();
        world.register::<Name>();
        world.register::<AITargetSelector>();

        let system = BehaviorStateMachineSystem::new();
        assert_eq!(system.update_frequency, Duration::from_millis(100));
    }

    #[test]
    fn test_distance_calculation() {
        let system = BehaviorStateMachineSystem::new();
        let pos1 = Position { x: 0, y: 0, z: 0 };
        let pos2 = Position { x: 3, y: 4, z: 0 };
        
        let distance = system.calculate_distance(&pos1, &pos2);
        assert_eq!(distance, 5.0); // 3-4-5 triangle
    }

    #[test]
    fn test_line_of_sight() {
        let system = BehaviorStateMachineSystem::new();
        let pos1 = Position { x: 0, y: 0, z: 0 };
        let pos2 = Position { x: 5, y: 5, z: 0 };
        let pos3 = Position { x: 20, y: 20, z: 0 };
        
        assert!(system.has_line_of_sight(&pos1, &pos2));
        assert!(!system.has_line_of_sight(&pos1, &pos3));
    }

    #[test]
    fn test_state_transition_rules() {
        let rules = StateTransitionRules::default();
        let factors = AIDecisionFactors {
            has_line_of_sight: true,
            distance_to_target: 5.0,
            ..Default::default()
        };
        let personality = AIPersonality::default();
        
        let transition = rules.evaluate_transitions(&AIBehaviorState::Idle, &factors, &personality);
        assert_eq!(transition, Some(AIBehaviorState::Hunt));
    }

    #[test]
    fn test_decision_factors_calculation() {
        let system = BehaviorStateMachineSystem::new();
        let ai = AI::default();
        let position = Position { x: 0, y: 0, z: 0 };
        let health = Health { current: 50, max: 100 };
        let player_entities = vec![];
        
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Health>();
        
        let factors = system.calculate_decision_factors(
            Entity::from_raw_index(0),
            &ai,
            &position,
            &health,
            &player_entities,
            &world.read_storage::<Position>(),
            &world.read_storage::<Health>(),
        );
        
        assert_eq!(factors.health_percentage, 0.5);
        assert_eq!(factors.number_of_enemies, 0);
    }
}