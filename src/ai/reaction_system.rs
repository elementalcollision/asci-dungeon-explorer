use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use crate::ai::{AIComponent, AIBehaviorState};
use crate::components::{Position, Health};

/// Types of events that enemies can react to
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReactionTrigger {
    /// Player actions
    PlayerAction {
        action_type: String,
        position: Vec2,
        intensity: f32,
        timestamp: f32,
    },
    /// Environmental changes
    EnvironmentalChange {
        change_type: String,
        affected_area: Vec<IVec2>,
        severity: f32,
        timestamp: f32,
    },
    /// Combat events
    CombatEvent {
        event_type: String,
        attacker: Option<Entity>,
        target: Option<Entity>,
        damage: f32,
        position: Vec2,
        timestamp: f32,
    },
    /// Sound events
    SoundEvent {
        sound_type: String,
        position: Vec2,
        volume: f32,
        timestamp: f32,
    },
}

impl ReactionTrigger {
    /// Get the position associated with this trigger
    pub fn get_position(&self) -> Vec2 {
        match self {
            ReactionTrigger::PlayerAction { position, .. } => *position,
            ReactionTrigger::EnvironmentalChange { affected_area, .. } => {
                if let Some(first_tile) = affected_area.first() {
                    first_tile.as_vec2()
                } else {
                    Vec2::ZERO
                }
            }
            ReactionTrigger::CombatEvent { position, .. } => *position,
            ReactionTrigger::SoundEvent { position, .. } => *position,
        }
    }

    /// Get the timestamp of this trigger
    pub fn get_timestamp(&self) -> f32 {
        match self {
            ReactionTrigger::PlayerAction { timestamp, .. } => *timestamp,
            ReactionTrigger::EnvironmentalChange { timestamp, .. } => *timestamp,
            ReactionTrigger::CombatEvent { timestamp, .. } => *timestamp,
            ReactionTrigger::SoundEvent { timestamp, .. } => *timestamp,
        }
    }

    /// Get the intensity/severity of this trigger
    pub fn get_intensity(&self) -> f32 {
        match self {
            ReactionTrigger::PlayerAction { intensity, .. } => *intensity,
            ReactionTrigger::EnvironmentalChange { severity, .. } => *severity,
            ReactionTrigger::CombatEvent { damage, .. } => *damage / 100.0,
            ReactionTrigger::SoundEvent { volume, .. } => *volume,
        }
    }
}

/// Reaction response types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReactionResponse {
    /// Change behavior state
    BehaviorChange {
        new_state: AIBehaviorState,
        duration: Option<f32>,
        priority: u32,
    },
    /// Move to investigate
    MoveTo {
        target_position: Vec2,
        urgency: f32,
        investigate_time: f32,
    },
    /// Alert nearby allies
    AlertAllies {
        alert_radius: f32,
        alert_message: String,
        alert_duration: f32,
    },
    /// Flee from the area
    FleeArea {
        flee_distance: f32,
        flee_duration: f32,
        panic_level: f32,
    },
}

/// Conditions for reaction triggers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReactionCondition {
    /// Trigger type matches
    TriggerType(String),
    /// Distance from trigger is within range
    DistanceWithin(f32),
    /// Intensity/severity above threshold
    IntensityAbove(f32),
    /// Current AI state matches
    CurrentState(AIBehaviorState),
    /// Health percentage condition
    HealthBelow(f32),
}

impl ReactionCondition {
    /// Evaluate condition against current state
    pub fn evaluate(
        &self,
        trigger: &ReactionTrigger,
        ai: &AIComponent,
        position: Vec2,
        _current_time: f32,
    ) -> bool {
        match self {
            ReactionCondition::TriggerType(expected_type) => {
                match trigger {
                    ReactionTrigger::PlayerAction { action_type, .. } => action_type == expected_type,
                    ReactionTrigger::EnvironmentalChange { change_type, .. } => change_type == expected_type,
                    ReactionTrigger::CombatEvent { event_type, .. } => event_type == expected_type,
                    ReactionTrigger::SoundEvent { sound_type, .. } => sound_type == expected_type,
                }
            }
            ReactionCondition::DistanceWithin(max_distance) => {
                let trigger_pos = trigger.get_position();
                position.distance(trigger_pos) <= *max_distance
            }
            ReactionCondition::IntensityAbove(threshold) => {
                trigger.get_intensity() > *threshold
            }
            ReactionCondition::CurrentState(expected_state) => {
                ai.current_state == *expected_state
            }
            ReactionCondition::HealthBelow(threshold) => {
                ai.decision_factors.health_percentage < *threshold
            }
        }
    }
}

/// Reaction rule defining trigger -> response mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactionRule {
    pub name: String,
    pub trigger_conditions: Vec<ReactionCondition>,
    pub response: ReactionResponse,
    pub cooldown: f32,
    pub priority: u32,
    pub max_distance: f32,
}

/// Component for enemy reaction system
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct ReactionComponent {
    pub reaction_rules: Vec<ReactionRule>,
    pub recent_triggers: VecDeque<ReactionTrigger>,
    pub last_reaction_times: HashMap<String, f32>,
    pub reaction_sensitivity: f32,
    pub memory_duration: f32,
    pub max_recent_triggers: usize,
}

impl Default for ReactionComponent {
    fn default() -> Self {
        let mut component = ReactionComponent {
            reaction_rules: Vec::new(),
            recent_triggers: VecDeque::new(),
            last_reaction_times: HashMap::new(),
            reaction_sensitivity: 1.0,
            memory_duration: 30.0,
            max_recent_triggers: 10,
        };

        component.add_default_rules();
        component
    }
}

impl ReactionComponent {
    /// Add default reaction rules
    fn add_default_rules(&mut self) {
        // React to combat by becoming alert
        self.reaction_rules.push(ReactionRule {
            name: "combat_alert".to_string(),
            trigger_conditions: vec![
                ReactionCondition::TriggerType("combat".to_string()),
                ReactionCondition::DistanceWithin(10.0),
            ],
            response: ReactionResponse::BehaviorChange {
                new_state: AIBehaviorState::Hunt,
                duration: Some(15.0),
                priority: 5,
            },
            cooldown: 2.0,
            priority: 5,
            max_distance: 10.0,
        });

        // React to loud sounds by investigating
        self.reaction_rules.push(ReactionRule {
            name: "investigate_sound".to_string(),
            trigger_conditions: vec![
                ReactionCondition::TriggerType("loud_sound".to_string()),
                ReactionCondition::IntensityAbove(0.5),
                ReactionCondition::CurrentState(AIBehaviorState::Idle),
            ],
            response: ReactionResponse::MoveTo {
                target_position: Vec2::ZERO,
                urgency: 0.7,
                investigate_time: 10.0,
            },
            cooldown: 5.0,
            priority: 3,
            max_distance: 12.0,
        });
    }

    /// Create reaction component for different enemy types
    pub fn for_enemy_type(enemy_type: &str) -> Self {
        let mut component = ReactionComponent::default();
        
        match enemy_type {
            "guard" => component.add_guard_rules(),
            "coward" => component.add_coward_rules(),
            "berserker" => component.add_berserker_rules(),
            _ => {}
        }
        
        component
    }

    /// Add guard-specific rules
    fn add_guard_rules(&mut self) {
        self.reaction_rules.push(ReactionRule {
            name: "guard_intrusion_response".to_string(),
            trigger_conditions: vec![
                ReactionCondition::TriggerType("intrusion".to_string()),
                ReactionCondition::DistanceWithin(8.0),
            ],
            response: ReactionResponse::BehaviorChange {
                new_state: AIBehaviorState::Hunt,
                duration: Some(20.0),
                priority: 8,
            },
            cooldown: 1.0,
            priority: 8,
            max_distance: 8.0,
        });
    }

    /// Add coward-specific rules
    fn add_coward_rules(&mut self) {
        self.reaction_rules.push(ReactionRule {
            name: "coward_flee".to_string(),
            trigger_conditions: vec![
                ReactionCondition::TriggerType("combat".to_string()),
                ReactionCondition::DistanceWithin(8.0),
            ],
            response: ReactionResponse::FleeArea {
                flee_distance: 20.0,
                flee_duration: 15.0,
                panic_level: 1.0,
            },
            cooldown: 2.0,
            priority: 10,
            max_distance: 8.0,
        });
    }

    /// Add berserker-specific rules
    fn add_berserker_rules(&mut self) {
        self.reaction_rules.push(ReactionRule {
            name: "berserker_rage".to_string(),
            trigger_conditions: vec![
                ReactionCondition::TriggerType("damage_taken".to_string()),
                ReactionCondition::HealthBelow(0.5),
            ],
            response: ReactionResponse::BehaviorChange {
                new_state: AIBehaviorState::Attack,
                duration: Some(20.0),
                priority: 7,
            },
            cooldown: 15.0,
            priority: 7,
            max_distance: f32::MAX,
        });
    }

    /// Add a trigger to recent memory
    pub fn add_trigger(&mut self, trigger: ReactionTrigger) {
        self.recent_triggers.push_back(trigger);
        
        while self.recent_triggers.len() > self.max_recent_triggers {
            self.recent_triggers.pop_front();
        }
    }

    /// Clean up old triggers
    pub fn cleanup_old_triggers(&mut self, current_time: f32) {
        self.recent_triggers.retain(|trigger| {
            current_time - trigger.get_timestamp() <= self.memory_duration
        });
    }

    /// Check if a rule can be triggered
    pub fn can_trigger_rule(&self, rule: &ReactionRule, current_time: f32) -> bool {
        if let Some(&last_time) = self.last_reaction_times.get(&rule.name) {
            current_time - last_time >= rule.cooldown
        } else {
            true
        }
    }

    /// Record that a rule was triggered
    pub fn record_rule_trigger(&mut self, rule_name: String, current_time: f32) {
        self.last_reaction_times.insert(rule_name, current_time);
    }
}

/// Resource for managing global reaction events
#[derive(Resource, Default)]
pub struct ReactionEventResource {
    pub pending_triggers: Vec<ReactionTrigger>,
    pub global_alert_level: f32,
    pub difficulty_modifier: f32,
}

impl ReactionEventResource {
    /// Add a global trigger event
    pub fn add_trigger(&mut self, trigger: ReactionTrigger) {
        self.pending_triggers.push(trigger);
    }

    /// Adjust difficulty based on player performance
    pub fn adjust_difficulty(&mut self, player_performance: f32) {
        if player_performance > 0.8 {
            self.difficulty_modifier = (self.difficulty_modifier + 0.01).min(2.0);
        } else if player_performance < 0.3 {
            self.difficulty_modifier = (self.difficulty_modifier - 0.01).max(0.5);
        }
    }
}

/// System for processing reaction triggers
pub fn reaction_trigger_system(
    time: Res<Time>,
    mut reaction_resource: ResMut<ReactionEventResource>,
    mut reaction_query: Query<(Entity, &mut ReactionComponent, &mut AIComponent, &Position)>,
) {
    let current_time = time.elapsed_seconds();
    
    let pending_triggers = std::mem::take(&mut reaction_resource.pending_triggers);
    
    for trigger in pending_triggers {
        let trigger_pos = trigger.get_position();
        
        for (entity, mut reaction, mut ai, position) in reaction_query.iter_mut() {
            if !ai.enabled {
                continue;
            }

            let distance = position.0.distance(trigger_pos);
            reaction.add_trigger(trigger.clone());
            
            for rule in &reaction.reaction_rules.clone() {
                if !reaction.can_trigger_rule(rule, current_time) {
                    continue;
                }
                
                if distance > rule.max_distance {
                    continue;
                }
                
                let all_conditions_met = rule.trigger_conditions.iter().all(|condition| {
                    condition.evaluate(&trigger, &ai, position.0, current_time)
                });
                
                if all_conditions_met {
                    execute_reaction_response(&mut ai, &rule.response, &trigger);
                    reaction.record_rule_trigger(rule.name.clone(), current_time);
                }
            }
        }
    }
    
    for (_, mut reaction, _, _) in reaction_query.iter_mut() {
        reaction.cleanup_old_triggers(current_time);
    }
}

/// Execute a reaction response
fn execute_reaction_response(
    ai: &mut AIComponent,
    response: &ReactionResponse,
    trigger: &ReactionTrigger,
) {
    match response {
        ReactionResponse::BehaviorChange { new_state, .. } => {
            ai.transition_to_state(new_state.clone());
        }
        ReactionResponse::MoveTo { .. } => {
            ai.memory.last_known_target_position = Some(trigger.get_position());
            ai.transition_to_state(AIBehaviorState::Search);
        }
        ReactionResponse::AlertAllies { .. } => {
            info!("Enemy alerting allies about: {:?}", trigger);
        }
        ReactionResponse::FleeArea { .. } => {
            ai.transition_to_state(AIBehaviorState::Flee);
            let flee_direction = (ai.memory.home_position - trigger.get_position()).normalize_or_zero();
            ai.memory.last_known_target_position = Some(ai.memory.home_position + flee_direction * 10.0);
        }
    }
}

/// System for adaptive difficulty adjustment
pub fn adaptive_difficulty_system(
    mut reaction_resource: ResMut<ReactionEventResource>,
    player_query: Query<&Health, With<crate::components::Player>>,
    enemy_query: Query<&Health, (With<AIComponent>, Without<crate::components::Player>)>,
) {
    if let Ok(player_health) = player_query.get_single() {
        let player_health_ratio = player_health.current as f32 / player_health.max as f32;
        let enemy_count = enemy_query.iter().count();
        
        let performance = if enemy_count > 0 {
            player_health_ratio * (1.0 - (enemy_count as f32 * 0.1).min(0.5))
        } else {
            player_health_ratio
        };
        
        reaction_resource.adjust_difficulty(performance);
    }
}

/// Plugin for reaction system
pub struct ReactionSystemPlugin;

impl Plugin for ReactionSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ReactionEventResource>()
            .add_systems(Update, (
                reaction_trigger_system,
                adaptive_difficulty_system,
            ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_trigger() {
        let trigger = ReactionTrigger::PlayerAction {
            action_type: "attack".to_string(),
            position: Vec2::new(5.0, 5.0),
            intensity: 0.8,
            timestamp: 10.0,
        };
        
        assert_eq!(trigger.get_position(), Vec2::new(5.0, 5.0));
        assert_eq!(trigger.get_timestamp(), 10.0);
        assert_eq!(trigger.get_intensity(), 0.8);
    }

    #[test]
    fn test_reaction_condition() {
        let trigger = ReactionTrigger::CombatEvent {
            event_type: "damage".to_string(),
            attacker: None,
            target: None,
            damage: 50.0,
            position: Vec2::new(0.0, 0.0),
            timestamp: 0.0,
        };
        
        let ai = AIComponent::default();
        
        let condition = ReactionCondition::TriggerType("damage".to_string());
        assert!(condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        let distance_condition = ReactionCondition::DistanceWithin(5.0);
        assert!(distance_condition.evaluate(&trigger, &ai, Vec2::new(3.0, 0.0), 0.0));
    }

    #[test]
    fn test_reaction_component() {
        let mut reaction = ReactionComponent::default();
        
        let trigger = ReactionTrigger::SoundEvent {
            sound_type: "footstep".to_string(),
            position: Vec2::new(1.0, 1.0),
            volume: 0.5,
            timestamp: 0.0,
        };
        
        reaction.add_trigger(trigger);
        assert_eq!(reaction.recent_triggers.len(), 1);
        
        assert!(reaction.can_trigger_rule(&reaction.reaction_rules[0], 0.0));
    }
}