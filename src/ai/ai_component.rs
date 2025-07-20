use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// AI behavior states
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AIBehaviorState {
    Idle,
    Patrol,
    Hunt,
    Attack,
    Flee,
    Search,
    Guard,
    Follow,
    Wander,
    Dead,
}

impl Default for AIBehaviorState {
    fn default() -> Self {
        AIBehaviorState::Idle
    }
}

/// AI decision factors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIDecisionFactors {
    pub health_percentage: f32,
    pub distance_to_target: f32,
    pub target_strength: f32,
    pub allies_nearby: u32,
    pub enemies_nearby: u32,
    pub time_since_last_seen_target: f32,
    pub noise_level: f32,
    pub light_level: f32,
}

impl Default for AIDecisionFactors {
    fn default() -> Self {
        AIDecisionFactors {
            health_percentage: 1.0,
            distance_to_target: f32::MAX,
            target_strength: 0.0,
            allies_nearby: 0,
            enemies_nearby: 0,
            time_since_last_seen_target: f32::MAX,
            noise_level: 0.0,
            light_level: 1.0,
        }
    }
}

/// AI personality traits that affect decision making
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIPersonality {
    pub aggression: f32,      // 0.0 = passive, 1.0 = very aggressive
    pub courage: f32,         // 0.0 = cowardly, 1.0 = fearless
    pub intelligence: f32,    // 0.0 = simple, 1.0 = very smart
    pub curiosity: f32,       // 0.0 = ignores distractions, 1.0 = easily distracted
    pub loyalty: f32,         // 0.0 = abandons allies, 1.0 = fights to the death for allies
    pub alertness: f32,       // 0.0 = oblivious, 1.0 = very perceptive
}

impl Default for AIPersonality {
    fn default() -> Self {
        AIPersonality {
            aggression: 0.5,
            courage: 0.5,
            intelligence: 0.5,
            curiosity: 0.5,
            loyalty: 0.5,
            alertness: 0.5,
        }
    }
}

/// AI memory of entities and locations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIMemory {
    pub last_known_target_position: Option<Vec2>,
    pub last_seen_target_time: f32,
    pub patrol_points: Vec<Vec2>,
    pub current_patrol_index: usize,
    pub home_position: Vec2,
    pub known_enemies: HashMap<Entity, (Vec2, f32)>, // Entity -> (last_position, last_seen_time)
    pub known_allies: HashMap<Entity, (Vec2, f32)>,
    pub interesting_locations: Vec<(Vec2, f32)>, // (position, interest_level)
}

impl Default for AIMemory {
    fn default() -> Self {
        AIMemory {
            last_known_target_position: None,
            last_seen_target_time: 0.0,
            patrol_points: Vec::new(),
            current_patrol_index: 0,
            home_position: Vec2::ZERO,
            known_enemies: HashMap::new(),
            known_allies: HashMap::new(),
            interesting_locations: Vec::new(),
        }
    }
}

/// AI state machine transition conditions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIStateTransition {
    pub from_state: AIBehaviorState,
    pub to_state: AIBehaviorState,
    pub condition: AITransitionCondition,
    pub priority: u32,
}

/// Conditions for state transitions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AITransitionCondition {
    HealthBelow(f32),
    HealthAbove(f32),
    TargetInRange(f32),
    TargetOutOfRange(f32),
    NoTargetFor(f32), // seconds
    AlliesNearby(u32),
    EnemiesNearby(u32),
    TimerExpired(f32),
    Always,
    Never,
    And(Box<AITransitionCondition>, Box<AITransitionCondition>),
    Or(Box<AITransitionCondition>, Box<AITransitionCondition>),
    Not(Box<AITransitionCondition>),
}

impl AITransitionCondition {
    /// Evaluate the condition against current AI factors
    pub fn evaluate(&self, factors: &AIDecisionFactors, timer: f32) -> bool {
        match self {
            AITransitionCondition::HealthBelow(threshold) => factors.health_percentage < *threshold,
            AITransitionCondition::HealthAbove(threshold) => factors.health_percentage > *threshold,
            AITransitionCondition::TargetInRange(range) => factors.distance_to_target <= *range,
            AITransitionCondition::TargetOutOfRange(range) => factors.distance_to_target > *range,
            AITransitionCondition::NoTargetFor(time) => factors.time_since_last_seen_target > *time,
            AITransitionCondition::AlliesNearby(count) => factors.allies_nearby >= *count,
            AITransitionCondition::EnemiesNearby(count) => factors.enemies_nearby >= *count,
            AITransitionCondition::TimerExpired(duration) => timer >= *duration,
            AITransitionCondition::Always => true,
            AITransitionCondition::Never => false,
            AITransitionCondition::And(a, b) => a.evaluate(factors, timer) && b.evaluate(factors, timer),
            AITransitionCondition::Or(a, b) => a.evaluate(factors, timer) || b.evaluate(factors, timer),
            AITransitionCondition::Not(condition) => !condition.evaluate(factors, timer),
        }
    }
}

/// AI behavior configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AIBehaviorConfig {
    pub state_transitions: Vec<AIStateTransition>,
    pub default_state: AIBehaviorState,
    pub update_interval: f32, // seconds between AI updates
    pub reaction_time: f32,   // delay before reacting to stimuli
}

impl Default for AIBehaviorConfig {
    fn default() -> Self {
        let mut config = AIBehaviorConfig {
            state_transitions: Vec::new(),
            default_state: AIBehaviorState::Idle,
            update_interval: 0.1,
            reaction_time: 0.2,
        };

        // Add default state transitions
        config.add_default_transitions();
        config
    }
}

impl AIBehaviorConfig {
    /// Add default state transitions for basic AI behavior
    fn add_default_transitions(&mut self) {
        // Idle -> Hunt when enemy spotted
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Idle,
            to_state: AIBehaviorState::Hunt,
            condition: AITransitionCondition::TargetInRange(10.0),
            priority: 5,
        });

        // Hunt -> Attack when close to target
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Hunt,
            to_state: AIBehaviorState::Attack,
            condition: AITransitionCondition::TargetInRange(1.5),
            priority: 8,
        });

        // Attack -> Hunt when target moves away
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Attack,
            to_state: AIBehaviorState::Hunt,
            condition: AITransitionCondition::TargetOutOfRange(2.0),
            priority: 6,
        });

        // Any -> Flee when health is low
        for state in [AIBehaviorState::Idle, AIBehaviorState::Hunt, AIBehaviorState::Attack, AIBehaviorState::Patrol] {
            self.state_transitions.push(AIStateTransition {
                from_state: state,
                to_state: AIBehaviorState::Flee,
                condition: AITransitionCondition::HealthBelow(0.25),
                priority: 10,
            });
        }

        // Flee -> Hunt when health recovers
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Flee,
            to_state: AIBehaviorState::Hunt,
            condition: AITransitionCondition::And(
                Box::new(AITransitionCondition::HealthAbove(0.5)),
                Box::new(AITransitionCondition::TargetInRange(8.0))
            ),
            priority: 4,
        });

        // Hunt -> Search when target lost
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Hunt,
            to_state: AIBehaviorState::Search,
            condition: AITransitionCondition::NoTargetFor(2.0),
            priority: 3,
        });

        // Search -> Idle when giving up
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Search,
            to_state: AIBehaviorState::Idle,
            condition: AITransitionCondition::TimerExpired(10.0),
            priority: 2,
        });

        // Idle -> Patrol for wandering enemies
        self.state_transitions.push(AIStateTransition {
            from_state: AIBehaviorState::Idle,
            to_state: AIBehaviorState::Patrol,
            condition: AITransitionCondition::TimerExpired(5.0),
            priority: 1,
        });
    }

    /// Get the next state based on current state and factors
    pub fn get_next_state(&self, current_state: &AIBehaviorState, factors: &AIDecisionFactors, timer: f32) -> Option<AIBehaviorState> {
        let mut valid_transitions: Vec<&AIStateTransition> = self.state_transitions
            .iter()
            .filter(|transition| {
                transition.from_state == *current_state && 
                transition.condition.evaluate(factors, timer)
            })
            .collect();

        // Sort by priority (higher priority first)
        valid_transitions.sort_by(|a, b| b.priority.cmp(&a.priority));

        valid_transitions.first().map(|transition| transition.to_state.clone())
    }
}

/// Main AI component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AIComponent {
    pub current_state: AIBehaviorState,
    pub previous_state: AIBehaviorState,
    pub state_timer: f32,
    pub update_timer: f32,
    pub reaction_timer: f32,
    pub personality: AIPersonality,
    pub memory: AIMemory,
    pub decision_factors: AIDecisionFactors,
    pub behavior_config: AIBehaviorConfig,
    pub current_target: Option<Entity>,
    pub enabled: bool,
}

impl Default for AIComponent {
    fn default() -> Self {
        AIComponent {
            current_state: AIBehaviorState::Idle,
            previous_state: AIBehaviorState::Idle,
            state_timer: 0.0,
            update_timer: 0.0,
            reaction_timer: 0.0,
            personality: AIPersonality::default(),
            memory: AIMemory::default(),
            decision_factors: AIDecisionFactors::default(),
            behavior_config: AIBehaviorConfig::default(),
            current_target: None,
            enabled: true,
        }
    }
}

impl AIComponent {
    /// Create a new AI component with specific personality
    pub fn new(personality: AIPersonality) -> Self {
        AIComponent {
            personality,
            ..Default::default()
        }
    }

    /// Create an aggressive AI
    pub fn aggressive() -> Self {
        AIComponent::new(AIPersonality {
            aggression: 0.8,
            courage: 0.7,
            intelligence: 0.6,
            curiosity: 0.3,
            loyalty: 0.5,
            alertness: 0.7,
        })
    }

    /// Create a defensive AI
    pub fn defensive() -> Self {
        AIComponent::new(AIPersonality {
            aggression: 0.3,
            courage: 0.4,
            intelligence: 0.7,
            curiosity: 0.4,
            loyalty: 0.8,
            alertness: 0.8,
        })
    }

    /// Create a cowardly AI
    pub fn cowardly() -> Self {
        AIComponent::new(AIPersonality {
            aggression: 0.2,
            courage: 0.1,
            intelligence: 0.5,
            curiosity: 0.6,
            loyalty: 0.2,
            alertness: 0.9,
        })
    }

    /// Create a berserker AI
    pub fn berserker() -> Self {
        AIComponent::new(AIPersonality {
            aggression: 1.0,
            courage: 0.9,
            intelligence: 0.3,
            curiosity: 0.1,
            loyalty: 0.4,
            alertness: 0.5,
        })
    }

    /// Update the AI state based on current factors
    pub fn update_state(&mut self, delta_time: f32) {
        self.state_timer += delta_time;
        self.update_timer += delta_time;
        self.reaction_timer += delta_time;

        // Check if it's time to update AI decisions
        if self.update_timer >= self.behavior_config.update_interval {
            self.update_timer = 0.0;

            // Check for state transitions
            if let Some(new_state) = self.behavior_config.get_next_state(
                &self.current_state,
                &self.decision_factors,
                self.state_timer
            ) {
                if new_state != self.current_state {
                    self.transition_to_state(new_state);
                }
            }
        }
    }

    /// Transition to a new state
    pub fn transition_to_state(&mut self, new_state: AIBehaviorState) {
        self.previous_state = self.current_state.clone();
        self.current_state = new_state;
        self.state_timer = 0.0;
        self.reaction_timer = 0.0;
    }

    /// Check if the AI can react (reaction time has passed)
    pub fn can_react(&self) -> bool {
        self.reaction_timer >= self.behavior_config.reaction_time
    }

    /// Set the current target
    pub fn set_target(&mut self, target: Option<Entity>) {
        self.current_target = target;
        if target.is_some() {
            self.decision_factors.time_since_last_seen_target = 0.0;
        }
    }

    /// Update decision factors
    pub fn update_decision_factors(&mut self, factors: AIDecisionFactors) {
        self.decision_factors = factors;
    }

    /// Add a patrol point
    pub fn add_patrol_point(&mut self, point: Vec2) {
        self.memory.patrol_points.push(point);
    }

    /// Set home position
    pub fn set_home_position(&mut self, position: Vec2) {
        self.memory.home_position = position;
    }

    /// Remember an enemy
    pub fn remember_enemy(&mut self, entity: Entity, position: Vec2, time: f32) {
        self.memory.known_enemies.insert(entity, (position, time));
    }

    /// Remember an ally
    pub fn remember_ally(&mut self, entity: Entity, position: Vec2, time: f32) {
        self.memory.known_allies.insert(entity, (position, time));
    }

    /// Get the current patrol target
    pub fn get_current_patrol_target(&self) -> Option<Vec2> {
        if self.memory.patrol_points.is_empty() {
            return None;
        }
        
        let index = self.memory.current_patrol_index % self.memory.patrol_points.len();
        Some(self.memory.patrol_points[index])
    }

    /// Advance to the next patrol point
    pub fn advance_patrol(&mut self) {
        if !self.memory.patrol_points.is_empty() {
            self.memory.current_patrol_index = (self.memory.current_patrol_index + 1) % self.memory.patrol_points.len();
        }
    }

    /// Check if the AI should be aggressive based on personality and factors
    pub fn should_be_aggressive(&self) -> bool {
        let base_aggression = self.personality.aggression;
        let health_factor = self.decision_factors.health_percentage;
        let ally_factor = if self.decision_factors.allies_nearby > 0 { 1.2 } else { 0.8 };
        
        let effective_aggression = base_aggression * health_factor * ally_factor;
        effective_aggression > 0.5
    }

    /// Check if the AI should flee based on personality and factors
    pub fn should_flee(&self) -> bool {
        let base_courage = self.personality.courage;
        let health_factor = 1.0 - self.decision_factors.health_percentage;
        let enemy_factor = if self.decision_factors.enemies_nearby > 1 { 1.5 } else { 1.0 };
        
        let fear_level = health_factor * enemy_factor * (1.0 - base_courage);
        fear_level > 0.6
    }
}

/// AI target selection component
#[derive(Component, Debug, Clone)]
pub struct AITargetSelector {
    pub target_types: Vec<String>, // Types of entities this AI will target
    pub detection_range: f32,
    pub preferred_target: Option<Entity>,
    pub target_priority_factors: TargetPriorityFactors,
}

/// Factors used to determine target priority
#[derive(Debug, Clone)]
pub struct TargetPriorityFactors {
    pub distance_weight: f32,     // Lower distance = higher priority
    pub health_weight: f32,       // Lower health = higher priority
    pub threat_weight: f32,       // Higher threat = higher priority
    pub visibility_weight: f32,   // More visible = higher priority
}

impl Default for TargetPriorityFactors {
    fn default() -> Self {
        TargetPriorityFactors {
            distance_weight: 1.0,
            health_weight: 0.5,
            threat_weight: 0.8,
            visibility_weight: 0.3,
        }
    }
}

impl Default for AITargetSelector {
    fn default() -> Self {
        AITargetSelector {
            target_types: vec!["Player".to_string()],
            detection_range: 8.0,
            preferred_target: None,
            target_priority_factors: TargetPriorityFactors::default(),
        }
    }
}

impl AITargetSelector {
    /// Create a new target selector
    pub fn new(target_types: Vec<String>, detection_range: f32) -> Self {
        AITargetSelector {
            target_types,
            detection_range,
            preferred_target: None,
            target_priority_factors: TargetPriorityFactors::default(),
        }
    }

    /// Calculate priority score for a potential target
    pub fn calculate_target_priority(&self, distance: f32, health_percentage: f32, threat_level: f32, visibility: f32) -> f32 {
        let distance_score = (1.0 - (distance / self.detection_range).min(1.0)) * self.target_priority_factors.distance_weight;
        let health_score = (1.0 - health_percentage) * self.target_priority_factors.health_weight;
        let threat_score = threat_level * self.target_priority_factors.threat_weight;
        let visibility_score = visibility * self.target_priority_factors.visibility_weight;

        distance_score + health_score + threat_score + visibility_score
    }
}

/// AI decision system component for complex decision making
#[derive(Component, Debug, Clone)]
pub struct AIDecisionSystem {
    pub decision_tree: DecisionNode,
    pub context_variables: HashMap<String, f32>,
}

/// Decision tree node for AI decision making
#[derive(Debug, Clone)]
pub enum DecisionNode {
    Condition {
        test: DecisionTest,
        true_branch: Box<DecisionNode>,
        false_branch: Box<DecisionNode>,
    },
    Action {
        action: AIAction,
    },
}

/// Tests used in decision trees
#[derive(Debug, Clone)]
pub enum DecisionTest {
    HealthBelow(f32),
    DistanceToTargetBelow(f32),
    HasTarget,
    AlliesNearby(u32),
    EnemiesNearby(u32),
    ContextVariable(String, f32), // variable_name, threshold
}

/// Actions that can be taken by AI
#[derive(Debug, Clone, PartialEq)]
pub enum AIAction {
    SetState(AIBehaviorState),
    MoveToTarget,
    MoveToPosition(Vec2),
    Attack,
    Flee,
    CallForHelp,
    UseItem(String),
    Wait(f32),
    Custom(String),
}

impl DecisionTest {
    /// Evaluate the test against current AI state
    pub fn evaluate(&self, ai: &AIComponent, context: &HashMap<String, f32>) -> bool {
        match self {
            DecisionTest::HealthBelow(threshold) => ai.decision_factors.health_percentage < *threshold,
            DecisionTest::DistanceToTargetBelow(threshold) => ai.decision_factors.distance_to_target < *threshold,
            DecisionTest::HasTarget => ai.current_target.is_some(),
            DecisionTest::AlliesNearby(count) => ai.decision_factors.allies_nearby >= *count,
            DecisionTest::EnemiesNearby(count) => ai.decision_factors.enemies_nearby >= *count,
            DecisionTest::ContextVariable(name, threshold) => {
                context.get(name).map_or(false, |value| *value >= *threshold)
            }
        }
    }
}

impl DecisionNode {
    /// Evaluate the decision tree and return the resulting action
    pub fn evaluate(&self, ai: &AIComponent, context: &HashMap<String, f32>) -> AIAction {
        match self {
            DecisionNode::Condition { test, true_branch, false_branch } => {
                if test.evaluate(ai, context) {
                    true_branch.evaluate(ai, context)
                } else {
                    false_branch.evaluate(ai, context)
                }
            }
            DecisionNode::Action { action } => action.clone(),
        }
    }
}

impl Default for AIDecisionSystem {
    fn default() -> Self {
        // Create a simple default decision tree
        let decision_tree = DecisionNode::Condition {
            test: DecisionTest::HealthBelow(0.3),
            true_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Flee),
            }),
            false_branch: Box::new(DecisionNode::Condition {
                test: DecisionTest::HasTarget,
                true_branch: Box::new(DecisionNode::Condition {
                    test: DecisionTest::DistanceToTargetBelow(2.0),
                    true_branch: Box::new(DecisionNode::Action {
                        action: AIAction::SetState(AIBehaviorState::Attack),
                    }),
                    false_branch: Box::new(DecisionNode::Action {
                        action: AIAction::SetState(AIBehaviorState::Hunt),
                    }),
                }),
                false_branch: Box::new(DecisionNode::Action {
                    action: AIAction::SetState(AIBehaviorState::Patrol),
                }),
            }),
        };

        AIDecisionSystem {
            decision_tree,
            context_variables: HashMap::new(),
        }
    }
}

impl AIDecisionSystem {
    /// Make a decision based on current AI state
    pub fn make_decision(&self, ai: &AIComponent) -> AIAction {
        self.decision_tree.evaluate(ai, &self.context_variables)
    }

    /// Set a context variable
    pub fn set_context_variable(&mut self, name: String, value: f32) {
        self.context_variables.insert(name, value);
    }

    /// Get a context variable
    pub fn get_context_variable(&self, name: &str) -> Option<f32> {
        self.context_variables.get(name).copied()
    }
}