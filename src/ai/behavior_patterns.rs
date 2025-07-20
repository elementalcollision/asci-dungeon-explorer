use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ai::{AIComponent, AIBehaviorState, AIPersonality};
use crate::components::Position;

/// Different types of enemy behavior patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorPattern {
    /// Simple patrol between waypoints
    SimplePatrol,
    /// Random wandering in an area
    RandomWander,
    /// Aggressive hunter that actively seeks targets
    AggressiveHunter,
    /// Defensive guard that protects an area
    DefensiveGuard,
    /// Pack hunter that coordinates with others
    PackHunter,
    /// Ambush predator that waits for targets
    AmbushPredator,
    /// Cowardly enemy that flees from combat
    Coward,
    /// Berserker that becomes more aggressive when damaged
    Berserker,
    /// Support enemy that helps others
    Support,
    /// Elite enemy with complex behaviors
    Elite,
}

/// Configuration for behavior patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPatternConfig {
    pub pattern: BehaviorPattern,
    pub parameters: HashMap<String, f32>,
    pub group_id: Option<String>,
    pub priority_targets: Vec<String>,
    pub flee_threshold: f32,
    pub aggression_modifier: f32,
    pub detection_range: f32,
    pub communication_range: f32,
}

impl Default for BehaviorPatternConfig {
    fn default() -> Self {
        BehaviorPatternConfig {
            pattern: BehaviorPattern::SimplePatrol,
            parameters: HashMap::new(),
            priority_targets: vec!["Player".to_string()],
            flee_threshold: 0.2,
            aggression_modifier: 1.0,
            detection_range: 8.0,
            communication_range: 12.0,
            group_id: None,
        }
    }
}

impl BehaviorPatternConfig {
    /// Create a simple patrol pattern
    pub fn simple_patrol(patrol_radius: f32) -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::SimplePatrol;
        config.parameters.insert("patrol_radius".to_string(), patrol_radius);
        config.parameters.insert("patrol_speed".to_string(), 1.5);
        config.parameters.insert("wait_time".to_string(), 2.0);
        config
    }

    /// Create a random wander pattern
    pub fn random_wander(wander_radius: f32) -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::RandomWander;
        config.parameters.insert("wander_radius".to_string(), wander_radius);
        config.parameters.insert("wander_speed".to_string(), 1.0);
        config.parameters.insert("direction_change_time".to_string(), 3.0);
        config
    }

    /// Create an aggressive hunter pattern
    pub fn aggressive_hunter() -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::AggressiveHunter;
        config.detection_range = 12.0;
        config.aggression_modifier = 1.5;
        config.flee_threshold = 0.1;
        config.parameters.insert("hunt_speed".to_string(), 3.0);
        config.parameters.insert("persistence".to_string(), 15.0);
        config.parameters.insert("search_time".to_string(), 10.0);
        config
    }

    /// Create a defensive guard pattern
    pub fn defensive_guard(guard_radius: f32) -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::DefensiveGuard;
        config.detection_range = 10.0;
        config.aggression_modifier = 0.8;
        config.parameters.insert("guard_radius".to_string(), guard_radius);
        config.parameters.insert("return_speed".to_string(), 2.0);
        config.parameters.insert("alert_duration".to_string(), 8.0);
        config
    }

    /// Create a pack hunter pattern
    pub fn pack_hunter(group_id: String) -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::PackHunter;
        config.group_id = Some(group_id);
        config.communication_range = 15.0;
        config.detection_range = 10.0;
        config.aggression_modifier = 1.2;
        config.parameters.insert("pack_coordination".to_string(), 0.8);
        config.parameters.insert("flanking_distance".to_string(), 4.0);
        config.parameters.insert("pack_bonus".to_string(), 0.3);
        config
    }

    /// Create an ambush predator pattern
    pub fn ambush_predator() -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::AmbushPredator;
        config.detection_range = 6.0;
        config.aggression_modifier = 2.0;
        config.parameters.insert("ambush_range".to_string(), 3.0);
        config.parameters.insert("patience".to_string(), 20.0);
        config.parameters.insert("strike_speed".to_string(), 4.0);
        config
    }

    /// Create a coward pattern
    pub fn coward() -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::Coward;
        config.flee_threshold = 0.8;
        config.aggression_modifier = 0.3;
        config.detection_range = 12.0;
        config.parameters.insert("flee_speed".to_string(), 3.5);
        config.parameters.insert("hide_time".to_string(), 10.0);
        config.parameters.insert("call_for_help_chance".to_string(), 0.7);
        config
    }

    /// Create a berserker pattern
    pub fn berserker() -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::Berserker;
        config.flee_threshold = 0.0; // Never flees
        config.aggression_modifier = 1.0; // Increases as health decreases
        config.parameters.insert("rage_threshold".to_string(), 0.5);
        config.parameters.insert("max_rage_bonus".to_string(), 2.0);
        config.parameters.insert("berserker_speed".to_string(), 2.5);
        config
    }

    /// Create a support pattern
    pub fn support(group_id: String) -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::Support;
        config.group_id = Some(group_id);
        config.communication_range = 20.0;
        config.aggression_modifier = 0.5;
        config.parameters.insert("heal_range".to_string(), 5.0);
        config.parameters.insert("buff_range".to_string(), 8.0);
        config.parameters.insert("support_priority".to_string(), 0.8);
        config
    }

    /// Create an elite pattern
    pub fn elite() -> Self {
        let mut config = BehaviorPatternConfig::default();
        config.pattern = BehaviorPattern::Elite;
        config.detection_range = 15.0;
        config.communication_range = 25.0;
        config.aggression_modifier = 1.3;
        config.flee_threshold = 0.15;
        config.parameters.insert("tactical_intelligence".to_string(), 0.9);
        config.parameters.insert("ability_cooldown".to_string(), 5.0);
        config.parameters.insert("leadership_range".to_string(), 12.0);
        config
    }

    /// Get a parameter value with default
    pub fn get_parameter(&self, key: &str, default: f32) -> f32 {
        self.parameters.get(key).copied().unwrap_or(default)
    }
}

/// Component for behavior pattern configuration
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorPatternComponent {
    pub config: BehaviorPatternConfig,
    pub state_data: HashMap<String, f32>,
    pub last_update: f32,
    pub pattern_timer: f32,
}

impl BehaviorPatternComponent {
    pub fn new(config: BehaviorPatternConfig) -> Self {
        BehaviorPatternComponent {
            config,
            state_data: HashMap::new(),
            last_update: 0.0,
            pattern_timer: 0.0,
        }
    }

    /// Get state data with default
    pub fn get_state(&self, key: &str, default: f32) -> f32 {
        self.state_data.get(key).copied().unwrap_or(default)
    }

    /// Set state data
    pub fn set_state(&mut self, key: String, value: f32) {
        self.state_data.insert(key, value);
    }

    /// Update pattern timer
    pub fn update_timer(&mut self, delta_time: f32) {
        self.pattern_timer += delta_time;
        self.last_update += delta_time;
    }
}

/// Group coordination data
#[derive(Debug, Clone)]
pub struct GroupCoordination {
    pub group_id: String,
    pub members: Vec<Entity>,
    pub leader: Option<Entity>,
    pub shared_target: Option<Entity>,
    pub formation_center: Vec2,
    pub coordination_level: f32,
    pub last_communication: f32,
}

impl GroupCoordination {
    pub fn new(group_id: String) -> Self {
        GroupCoordination {
            group_id,
            members: Vec::new(),
            leader: None,
            shared_target: None,
            formation_center: Vec2::ZERO,
            coordination_level: 0.5,
            last_communication: 0.0,
        }
    }

    /// Add a member to the group
    pub fn add_member(&mut self, entity: Entity) {
        if !self.members.contains(&entity) {
            self.members.push(entity);
            
            // Set first member as leader if no leader exists
            if self.leader.is_none() {
                self.leader = Some(entity);
            }
        }
    }

    /// Remove a member from the group
    pub fn remove_member(&mut self, entity: Entity) {
        self.members.retain(|&e| e != entity);
        
        // Choose new leader if current leader was removed
        if self.leader == Some(entity) {
            self.leader = self.members.first().copied();
        }
    }

    /// Check if entity is the leader
    pub fn is_leader(&self, entity: Entity) -> bool {
        self.leader == Some(entity)
    }

    /// Get group size
    pub fn size(&self) -> usize {
        self.members.len()
    }
}

/// Resource for managing group coordination
#[derive(Resource, Default)]
pub struct GroupCoordinationResource {
    pub groups: HashMap<String, GroupCoordination>,
}

impl GroupCoordinationResource {
    /// Get or create a group
    pub fn get_or_create_group(&mut self, group_id: String) -> &mut GroupCoordination {
        self.groups.entry(group_id.clone()).or_insert_with(|| GroupCoordination::new(group_id))
    }

    /// Add entity to group
    pub fn add_to_group(&mut self, group_id: String, entity: Entity) {
        let group = self.get_or_create_group(group_id);
        group.add_member(entity);
    }

    /// Remove entity from group
    pub fn remove_from_group(&mut self, group_id: &str, entity: Entity) {
        if let Some(group) = self.groups.get_mut(group_id) {
            group.remove_member(entity);
            
            // Remove empty groups
            if group.members.is_empty() {
                self.groups.remove(group_id);
            }
        }
    }

    /// Get group for entity
    pub fn get_group_for_entity(&self, entity: Entity) -> Option<&GroupCoordination> {
        self.groups.values().find(|group| group.members.contains(&entity))
    }

    /// Get mutable group for entity
    pub fn get_group_for_entity_mut(&mut self, entity: Entity) -> Option<&mut GroupCoordination> {
        self.groups.values_mut().find(|group| group.members.contains(&entity))
    }
}

/// System for updating behavior patterns
pub fn behavior_pattern_system(
    time: Res<Time>,
    mut pattern_query: Query<(Entity, &mut AIComponent, &mut BehaviorPatternComponent, &Position)>,
    mut group_resource: ResMut<GroupCoordinationResource>,
) {
    let delta_time = time.delta_seconds();
    let current_time = time.elapsed_seconds();

    for (entity, mut ai, mut pattern, position) in pattern_query.iter_mut() {
        if !ai.enabled {
            continue;
        }

        pattern.update_timer(delta_time);

        // Apply behavior pattern logic
        match pattern.config.pattern {
            BehaviorPattern::SimplePatrol => {
                apply_simple_patrol(&mut ai, &mut pattern, position);
            }
            BehaviorPattern::RandomWander => {
                apply_random_wander(&mut ai, &mut pattern, position, current_time);
            }
            BehaviorPattern::AggressiveHunter => {
                apply_aggressive_hunter(&mut ai, &mut pattern);
            }
            BehaviorPattern::DefensiveGuard => {
                apply_defensive_guard(&mut ai, &mut pattern, position);
            }
            BehaviorPattern::PackHunter => {
                apply_pack_hunter(&mut ai, &mut pattern, entity, &mut group_resource);
            }
            BehaviorPattern::AmbushPredator => {
                apply_ambush_predator(&mut ai, &mut pattern, position);
            }
            BehaviorPattern::Coward => {
                apply_coward(&mut ai, &mut pattern);
            }
            BehaviorPattern::Berserker => {
                apply_berserker(&mut ai, &mut pattern);
            }
            BehaviorPattern::Support => {
                apply_support(&mut ai, &mut pattern, entity, &mut group_resource);
            }
            BehaviorPattern::Elite => {
                apply_elite(&mut ai, &mut pattern, entity, &mut group_resource);
            }
        }

        // Apply pattern-specific personality modifications
        apply_personality_modifications(&mut ai, &pattern);
    }
}

fn apply_simple_patrol(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, position: &Position) {
    let patrol_radius = pattern.config.get_parameter("patrol_radius", 5.0);
    let wait_time = pattern.config.get_parameter("wait_time", 2.0);

    match ai.current_state {
        AIBehaviorState::Idle => {
            // Start patrolling if no patrol points exist
            if ai.memory.patrol_points.is_empty() {
                generate_patrol_points(ai, position.0, patrol_radius, 4);
            }
            ai.transition_to_state(AIBehaviorState::Patrol);
        }
        AIBehaviorState::Patrol => {
            // Check if we should wait at patrol point
            if let Some(patrol_target) = ai.get_current_patrol_target() {
                let distance = position.0.distance(patrol_target.as_vec2());
                if distance < 1.0 {
                    if pattern.get_state("wait_timer", 0.0) >= wait_time {
                        ai.advance_patrol();
                        pattern.set_state("wait_timer".to_string(), 0.0);
                    } else {
                        pattern.set_state("wait_timer".to_string(), pattern.get_state("wait_timer", 0.0) + 0.1);
                    }
                }
            }
        }
        _ => {
            // Return to patrol if not in combat
            if ai.current_target.is_none() && ai.decision_factors.time_since_last_seen_target > 5.0 {
                ai.transition_to_state(AIBehaviorState::Patrol);
            }
        }
    }
}

fn apply_random_wander(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, position: &Position, current_time: f32) {
    let wander_radius = pattern.config.get_parameter("wander_radius", 8.0);
    let direction_change_time = pattern.config.get_parameter("direction_change_time", 3.0);

    match ai.current_state {
        AIBehaviorState::Idle | AIBehaviorState::Wander => {
            // Change direction periodically
            if pattern.pattern_timer >= direction_change_time {
                let angle = (current_time + position.0.x + position.0.y).sin() * std::f32::consts::TAU;
                let wander_offset = Vec2::new(angle.cos(), angle.sin()) * wander_radius;
                let wander_target = ai.memory.home_position + wander_offset;
                
                ai.memory.last_known_target_position = Some(wander_target);
                ai.transition_to_state(AIBehaviorState::Wander);
                pattern.pattern_timer = 0.0;
            }
        }
        _ => {
            // Return to wandering if not in combat
            if ai.current_target.is_none() && ai.decision_factors.time_since_last_seen_target > 3.0 {
                ai.transition_to_state(AIBehaviorState::Wander);
            }
        }
    }
}

fn apply_aggressive_hunter(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent) {
    let persistence = pattern.config.get_parameter("persistence", 15.0);
    let search_time = pattern.config.get_parameter("search_time", 10.0);

    // Increase aggression and detection
    ai.personality.aggression = (ai.personality.aggression * 1.2).min(1.0);
    ai.personality.alertness = (ai.personality.alertness * 1.1).min(1.0);

    match ai.current_state {
        AIBehaviorState::Idle => {
            // Actively look for targets
            ai.transition_to_state(AIBehaviorState::Search);
        }
        AIBehaviorState::Hunt => {
            // Persist in hunting longer than normal
            pattern.set_state("hunt_timer".to_string(), pattern.get_state("hunt_timer", 0.0) + 0.1);
        }
        AIBehaviorState::Search => {
            // Search for longer periods
            if pattern.pattern_timer > search_time {
                ai.transition_to_state(AIBehaviorState::Patrol);
                pattern.pattern_timer = 0.0;
            }
        }
        _ => {}
    }
}

fn apply_defensive_guard(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, position: &Position) {
    let guard_radius = pattern.config.get_parameter("guard_radius", 6.0);
    let alert_duration = pattern.config.get_parameter("alert_duration", 8.0);

    let distance_from_home = position.0.distance(ai.memory.home_position);

    match ai.current_state {
        AIBehaviorState::Hunt | AIBehaviorState::Attack => {
            // Don't chase too far from guard position
            if distance_from_home > guard_radius * 1.5 {
                ai.transition_to_state(AIBehaviorState::Guard);
                pattern.set_state("alert_timer".to_string(), alert_duration);
            }
        }
        AIBehaviorState::Guard => {
            // Stay alert for a while after combat
            let alert_timer = pattern.get_state("alert_timer", 0.0);
            if alert_timer > 0.0 {
                pattern.set_state("alert_timer".to_string(), alert_timer - 0.1);
            } else if distance_from_home <= 1.0 {
                ai.transition_to_state(AIBehaviorState::Idle);
            }
        }
        AIBehaviorState::Idle => {
            // Return to guard position if too far
            if distance_from_home > guard_radius {
                ai.transition_to_state(AIBehaviorState::Guard);
            }
        }
        _ => {}
    }
}

fn apply_pack_hunter(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, entity: Entity, group_resource: &mut ResMut<GroupCoordinationResource>) {
    if let Some(group_id) = &pattern.config.group_id {
        let group = group_resource.get_or_create_group(group_id.clone());
        group.add_member(entity);

        let pack_coordination = pattern.config.get_parameter("pack_coordination", 0.8);
        let flanking_distance = pattern.config.get_parameter("flanking_distance", 4.0);

        // Share target information with pack
        if let Some(target) = ai.current_target {
            group.shared_target = Some(target);
        }

        // Coordinate attacks
        if group.size() > 1 {
            let pack_bonus = pattern.config.get_parameter("pack_bonus", 0.3);
            ai.personality.aggression = (ai.personality.aggression * (1.0 + pack_bonus)).min(1.0);
            ai.personality.courage = (ai.personality.courage * (1.0 + pack_bonus * 0.5)).min(1.0);

            // Implement flanking behavior
            if ai.current_state == AIBehaviorState::Hunt && group.is_leader(entity) {
                // Leader coordinates the attack
                pattern.set_state("coordinating_attack".to_string(), 1.0);
            }
        }
    }
}

fn apply_ambush_predator(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, position: &Position) {
    let ambush_range = pattern.config.get_parameter("ambush_range", 3.0);
    let patience = pattern.config.get_parameter("patience", 20.0);

    match ai.current_state {
        AIBehaviorState::Idle => {
            // Wait in ambush
            pattern.set_state("ambush_timer".to_string(), pattern.get_state("ambush_timer", 0.0) + 0.1);
            
            // Check for targets in ambush range
            if ai.decision_factors.distance_to_target <= ambush_range && ai.current_target.is_some() {
                ai.transition_to_state(AIBehaviorState::Attack);
                pattern.set_state("ambush_timer".to_string(), 0.0);
            }
        }
        AIBehaviorState::Hunt => {
            // Don't chase far, return to ambush position
            let distance_from_home = position.0.distance(ai.memory.home_position);
            if distance_from_home > ambush_range * 2.0 {
                ai.transition_to_state(AIBehaviorState::Guard);
            }
        }
        AIBehaviorState::Guard => {
            // Return to ambush position
            if position.0.distance(ai.memory.home_position) <= 1.0 {
                ai.transition_to_state(AIBehaviorState::Idle);
            }
        }
        _ => {}
    }

    // Relocate ambush position if waiting too long
    if pattern.get_state("ambush_timer", 0.0) > patience {
        let new_ambush = ai.memory.home_position + Vec2::new(
            (pattern.pattern_timer.sin() * 5.0),
            (pattern.pattern_timer.cos() * 5.0),
        );
        ai.memory.home_position = new_ambush;
        pattern.set_state("ambush_timer".to_string(), 0.0);
    }
}

fn apply_coward(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent) {
    let call_for_help_chance = pattern.config.get_parameter("call_for_help_chance", 0.7);
    let hide_time = pattern.config.get_parameter("hide_time", 10.0);

    // Increase fear and reduce aggression
    ai.personality.courage = (ai.personality.courage * 0.5).max(0.1);
    ai.personality.aggression = (ai.personality.aggression * 0.3).max(0.1);

    match ai.current_state {
        AIBehaviorState::Hunt | AIBehaviorState::Attack => {
            // Flee at higher health thresholds
            if ai.decision_factors.health_percentage < 0.8 {
                ai.transition_to_state(AIBehaviorState::Flee);
                
                // Chance to call for help
                if pattern.get_state("called_for_help", 0.0) == 0.0 && 
                   pattern.pattern_timer.sin().abs() < call_for_help_chance {
                    pattern.set_state("called_for_help".to_string(), 1.0);
                    pattern.set_state("help_called_timer".to_string(), 0.0);
                }
            }
        }
        AIBehaviorState::Flee => {
            // Hide for a while after fleeing
            if ai.decision_factors.distance_to_target > 10.0 {
                ai.transition_to_state(AIBehaviorState::Guard);
                pattern.set_state("hide_timer".to_string(), hide_time);
            }
        }
        AIBehaviorState::Guard => {
            // Stay hidden
            let hide_timer = pattern.get_state("hide_timer", 0.0);
            if hide_timer > 0.0 {
                pattern.set_state("hide_timer".to_string(), hide_timer - 0.1);
            } else {
                ai.transition_to_state(AIBehaviorState::Idle);
            }
        }
        _ => {}
    }
}

fn apply_berserker(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent) {
    let rage_threshold = pattern.config.get_parameter("rage_threshold", 0.5);
    let max_rage_bonus = pattern.config.get_parameter("max_rage_bonus", 2.0);

    // Calculate rage based on missing health
    let health_lost = 1.0 - ai.decision_factors.health_percentage;
    let rage_level = if ai.decision_factors.health_percentage < rage_threshold {
        (health_lost / (1.0 - rage_threshold)).min(1.0)
    } else {
        0.0
    };

    // Apply rage bonuses
    let rage_bonus = rage_level * max_rage_bonus;
    ai.personality.aggression = (ai.personality.aggression + rage_bonus).min(1.0);
    ai.personality.courage = (ai.personality.courage + rage_bonus * 0.5).min(1.0);

    // Never flee when in rage
    if rage_level > 0.3 {
        match ai.current_state {
            AIBehaviorState::Flee => {
                ai.transition_to_state(AIBehaviorState::Hunt);
            }
            AIBehaviorState::Idle => {
                if ai.current_target.is_some() {
                    ai.transition_to_state(AIBehaviorState::Hunt);
                }
            }
            _ => {}
        }
    }

    pattern.set_state("rage_level".to_string(), rage_level);
}

fn apply_support(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, entity: Entity, group_resource: &mut ResMut<GroupCoordinationResource>) {
    if let Some(group_id) = &pattern.config.group_id {
        let group = group_resource.get_or_create_group(group_id.clone());
        group.add_member(entity);

        let heal_range = pattern.config.get_parameter("heal_range", 5.0);
        let support_priority = pattern.config.get_parameter("support_priority", 0.8);

        // Reduce aggression, increase support behavior
        ai.personality.aggression = (ai.personality.aggression * 0.5).max(0.1);
        ai.personality.loyalty = (ai.personality.loyalty * 1.5).min(1.0);

        // Stay back from combat
        match ai.current_state {
            AIBehaviorState::Attack => {
                ai.transition_to_state(AIBehaviorState::Hunt);
            }
            AIBehaviorState::Hunt => {
                // Maintain distance from target
                if ai.decision_factors.distance_to_target < heal_range {
                    ai.transition_to_state(AIBehaviorState::Guard);
                }
            }
            AIBehaviorState::Idle => {
                // Look for allies to support
                if group.size() > 1 {
                    pattern.set_state("supporting".to_string(), 1.0);
                }
            }
            _ => {}
        }
    }
}

fn apply_elite(ai: &mut AIComponent, pattern: &mut BehaviorPatternComponent, entity: Entity, group_resource: &mut ResMut<GroupCoordinationResource>) {
    let tactical_intelligence = pattern.config.get_parameter("tactical_intelligence", 0.9);
    let leadership_range = pattern.config.get_parameter("leadership_range", 12.0);

    // Enhance all personality traits
    ai.personality.intelligence = (ai.personality.intelligence * 1.3).min(1.0);
    ai.personality.alertness = (ai.personality.alertness * 1.2).min(1.0);
    ai.personality.aggression = (ai.personality.aggression * 1.1).min(1.0);

    // Elite enemies use more complex decision making
    match ai.current_state {
        AIBehaviorState::Hunt => {
            // Use tactical positioning
            if ai.decision_factors.distance_to_target > 2.0 && ai.decision_factors.distance_to_target < 6.0 {
                // Optimal engagement range
                pattern.set_state("tactical_position".to_string(), 1.0);
            }
        }
        AIBehaviorState::Attack => {
            // Use abilities more frequently
            let ability_cooldown = pattern.config.get_parameter("ability_cooldown", 5.0);
            if pattern.get_state("last_ability_use", 0.0) + ability_cooldown < pattern.pattern_timer {
                pattern.set_state("use_ability".to_string(), 1.0);
                pattern.set_state("last_ability_use".to_string(), pattern.pattern_timer);
            }
        }
        _ => {}
    }

    // Provide leadership to nearby allies
    if let Some(group_id) = &pattern.config.group_id {
        let group = group_resource.get_or_create_group(group_id.clone());
        group.add_member(entity);
        group.leader = Some(entity); // Elite becomes leader
        group.coordination_level = tactical_intelligence;
    }
}

fn apply_personality_modifications(ai: &mut AIComponent, pattern: &BehaviorPatternComponent) {
    // Apply pattern-specific modifiers
    let aggression_mod = pattern.config.aggression_modifier;
    ai.personality.aggression = (ai.personality.aggression * aggression_mod).clamp(0.0, 1.0);

    // Adjust flee threshold based on pattern
    if ai.decision_factors.health_percentage < pattern.config.flee_threshold {
        if ai.current_state != AIBehaviorState::Flee && ai.personality.courage < 0.5 {
            ai.transition_to_state(AIBehaviorState::Flee);
        }
    }
}

fn generate_patrol_points(ai: &mut AIComponent, center: Vec2, radius: f32, count: usize) {
    ai.memory.patrol_points.clear();
    
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let offset = Vec2::new(angle.cos(), angle.sin()) * radius;
        let patrol_point = center + offset;
        ai.add_patrol_point(patrol_point);
    }
}

/// Plugin for behavior patterns
pub struct BehaviorPatternsPlugin;

impl Plugin for BehaviorPatternsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GroupCoordinationResource>()
            .add_systems(Update, behavior_pattern_system.after(crate::ai::ai_behavior_system));
    }
}