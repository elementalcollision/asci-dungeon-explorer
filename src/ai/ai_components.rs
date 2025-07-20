use specs::{Component, VecStorage, Entity};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

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

/// AI personality traits that affect behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIPersonality {
    pub aggression: f32,      // 0.0 = passive, 1.0 = very aggressive
    pub courage: f32,         // 0.0 = cowardly, 1.0 = fearless
    pub intelligence: f32,    // 0.0 = simple, 1.0 = very smart
    pub alertness: f32,       // 0.0 = oblivious, 1.0 = very alert
    pub loyalty: f32,         // 0.0 = selfish, 1.0 = very loyal
    pub curiosity: f32,       // 0.0 = incurious, 1.0 = very curious
}

impl Default for AIPersonality {
    fn default() -> Self {
        AIPersonality {
            aggression: 0.5,
            courage: 0.5,
            intelligence: 0.5,
            alertness: 0.5,
            loyalty: 0.5,
            curiosity: 0.5,
        }
    }
}

impl AIPersonality {
    pub fn aggressive() -> Self {
        AIPersonality {
            aggression: 0.8,
            courage: 0.7,
            intelligence: 0.6,
            alertness: 0.7,
            loyalty: 0.4,
            curiosity: 0.3,
        }
    }

    pub fn defensive() -> Self {
        AIPersonality {
            aggression: 0.3,
            courage: 0.6,
            intelligence: 0.7,
            alertness: 0.8,
            loyalty: 0.7,
            curiosity: 0.4,
        }
    }

    pub fn cowardly() -> Self {
        AIPersonality {
            aggression: 0.2,
            courage: 0.1,
            intelligence: 0.6,
            alertness: 0.9,
            loyalty: 0.3,
            curiosity: 0.2,
        }
    }

    pub fn intelligent() -> Self {
        AIPersonality {
            aggression: 0.4,
            courage: 0.6,
            intelligence: 0.9,
            alertness: 0.8,
            loyalty: 0.6,
            curiosity: 0.8,
        }
    }
}

/// AI memory for remembering entities and events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMemory {
    pub seen_entities: HashMap<Entity, (crate::components::Position, Instant)>,
    pub last_known_player_position: Option<(crate::components::Position, Instant)>,
    pub interesting_locations: Vec<(crate::components::Position, String, Instant)>,
    pub threats: Vec<(Entity, f32, Instant)>, // Entity, threat level, when seen
    pub allies: Vec<Entity>,
    pub memory_duration: Duration,
}

impl Default for AIMemory {
    fn default() -> Self {
        AIMemory {
            seen_entities: HashMap::new(),
            last_known_player_position: None,
            interesting_locations: Vec::new(),
            threats: Vec::new(),
            allies: Vec::new(),
            memory_duration: Duration::from_secs(30), // 30 seconds default memory
        }
    }
}

impl AIMemory {
    pub fn remember_entity(&mut self, entity: Entity, position: crate::components::Position) {
        self.seen_entities.insert(entity, (position, Instant::now()));
    }

    pub fn remember_player_position(&mut self, position: crate::components::Position) {
        self.last_known_player_position = Some((position, Instant::now()));
    }

    pub fn add_threat(&mut self, entity: Entity, threat_level: f32) {
        self.threats.push((entity, threat_level, Instant::now()));
    }

    pub fn add_interesting_location(&mut self, position: crate::components::Position, description: String) {
        self.interesting_locations.push((position, description, Instant::now()));
    }

    pub fn cleanup_old_memories(&mut self) {
        let now = Instant::now();
        
        // Clean up old entity memories
        self.seen_entities.retain(|_, (_, time)| now.duration_since(*time) < self.memory_duration);
        
        // Clean up old player position
        if let Some((_, time)) = self.last_known_player_position {
            if now.duration_since(time) >= self.memory_duration {
                self.last_known_player_position = None;
            }
        }
        
        // Clean up old threats
        self.threats.retain(|(_, _, time)| now.duration_since(*time) < self.memory_duration);
        
        // Clean up old interesting locations
        self.interesting_locations.retain(|(_, _, time)| now.duration_since(*time) < self.memory_duration);
    }

    pub fn get_last_known_position(&self, entity: Entity) -> Option<crate::components::Position> {
        self.seen_entities.get(&entity).map(|(pos, _)| *pos)
    }

    pub fn has_seen_recently(&self, entity: Entity, within_seconds: u64) -> bool {
        if let Some((_, time)) = self.seen_entities.get(&entity) {
            Instant::now().duration_since(*time).as_secs() <= within_seconds
        } else {
            false
        }
    }
}

/// AI decision factors for behavior selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDecisionFactors {
    pub health_percentage: f32,
    pub distance_to_target: f32,
    pub number_of_enemies: u32,
    pub number_of_allies: u32,
    pub time_since_last_action: Duration,
    pub current_threat_level: f32,
    pub energy_level: f32,
    pub has_line_of_sight: bool,
    pub is_outnumbered: bool,
}

impl Default for AIDecisionFactors {
    fn default() -> Self {
        AIDecisionFactors {
            health_percentage: 1.0,
            distance_to_target: f32::MAX,
            number_of_enemies: 0,
            number_of_allies: 0,
            time_since_last_action: Duration::from_secs(0),
            current_threat_level: 0.0,
            energy_level: 1.0,
            has_line_of_sight: false,
            is_outnumbered: false,
        }
    }
}

/// AI behavior parameters for different states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIBehaviorParams {
    pub patrol_points: Vec<crate::components::Position>,
    pub guard_position: Option<crate::components::Position>,
    pub follow_target: Option<Entity>,
    pub search_area: Option<(crate::components::Position, u32)>, // center, radius
    pub flee_distance: u32,
    pub attack_range: u32,
    pub detection_range: u32,
    pub patrol_speed: f32,
    pub chase_speed: f32,
    pub reaction_time: Duration,
}

impl Default for AIBehaviorParams {
    fn default() -> Self {
        AIBehaviorParams {
            patrol_points: Vec::new(),
            guard_position: None,
            follow_target: None,
            search_area: None,
            flee_distance: 10,
            attack_range: 1,
            detection_range: 8,
            patrol_speed: 1.0,
            chase_speed: 1.5,
            reaction_time: Duration::from_millis(500),
        }
    }
}

/// Main AI component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct AI {
    pub current_state: AIBehaviorState,
    pub previous_state: AIBehaviorState,
    pub personality: AIPersonality,
    pub memory: AIMemory,
    pub decision_factors: AIDecisionFactors,
    pub behavior_params: AIBehaviorParams,
    pub current_target: Option<Entity>,
    pub state_timer: Duration,
    pub last_decision_time: Instant,
    pub decision_cooldown: Duration,
    pub enabled: bool,
}

impl Default for AI {
    fn default() -> Self {
        AI {
            current_state: AIBehaviorState::Idle,
            previous_state: AIBehaviorState::Idle,
            personality: AIPersonality::default(),
            memory: AIMemory::default(),
            decision_factors: AIDecisionFactors::default(),
            behavior_params: AIBehaviorParams::default(),
            current_target: None,
            state_timer: Duration::from_secs(0),
            last_decision_time: Instant::now(),
            decision_cooldown: Duration::from_millis(200), // 200ms between decisions
            enabled: true,
        }
    }
}

impl AI {
    pub fn new(personality: AIPersonality) -> Self {
        AI {
            personality,
            ..Default::default()
        }
    }

    pub fn with_behavior_params(mut self, params: AIBehaviorParams) -> Self {
        self.behavior_params = params;
        self
    }

    pub fn with_memory_duration(mut self, duration: Duration) -> Self {
        self.memory.memory_duration = duration;
        self
    }

    pub fn with_decision_cooldown(mut self, cooldown: Duration) -> Self {
        self.decision_cooldown = cooldown;
        self
    }

    /// Check if enough time has passed to make a new decision
    pub fn can_make_decision(&self) -> bool {
        self.enabled && Instant::now().duration_since(self.last_decision_time) >= self.decision_cooldown
    }

    /// Change to a new behavior state
    pub fn change_state(&mut self, new_state: AIBehaviorState) {
        if new_state != self.current_state {
            self.previous_state = self.current_state.clone();
            self.current_state = new_state;
            self.state_timer = Duration::from_secs(0);
            self.last_decision_time = Instant::now();
        }
    }

    /// Update state timer
    pub fn update_timer(&mut self, delta_time: Duration) {
        self.state_timer += delta_time;
    }

    /// Get time spent in current state
    pub fn time_in_current_state(&self) -> Duration {
        self.state_timer
    }

    /// Check if the AI should react based on personality
    pub fn should_react_to_threat(&self, threat_level: f32) -> bool {
        let reaction_threshold = 1.0 - self.personality.alertness;
        threat_level > reaction_threshold
    }

    /// Calculate flee threshold based on personality
    pub fn get_flee_threshold(&self) -> f32 {
        1.0 - self.personality.courage
    }

    /// Calculate aggression modifier
    pub fn get_aggression_modifier(&self) -> f32 {
        0.5 + (self.personality.aggression * 0.5)
    }

    /// Check if AI should help allies
    pub fn should_help_allies(&self) -> bool {
        self.personality.loyalty > 0.5
    }

    /// Get search behavior based on intelligence
    pub fn get_search_thoroughness(&self) -> f32 {
        self.personality.intelligence
    }

    /// Enable or disable AI
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.change_state(AIBehaviorState::Idle);
        }
    }

    /// Reset AI to initial state
    pub fn reset(&mut self) {
        self.current_state = AIBehaviorState::Idle;
        self.previous_state = AIBehaviorState::Idle;
        self.current_target = None;
        self.state_timer = Duration::from_secs(0);
        self.memory = AIMemory::default();
        self.decision_factors = AIDecisionFactors::default();
    }

    /// Get state priority for decision making
    pub fn get_state_priority(&self, state: &AIBehaviorState) -> u32 {
        match state {
            AIBehaviorState::Dead => 0,
            AIBehaviorState::Flee => 9,
            AIBehaviorState::Attack => 8,
            AIBehaviorState::Hunt => 7,
            AIBehaviorState::Search => 6,
            AIBehaviorState::Guard => 5,
            AIBehaviorState::Follow => 4,
            AIBehaviorState::Patrol => 3,
            AIBehaviorState::Wander => 2,
            AIBehaviorState::Idle => 1,
        }
    }

    /// Check if current state can be interrupted by new state
    pub fn can_interrupt_with(&self, new_state: &AIBehaviorState) -> bool {
        let current_priority = self.get_state_priority(&self.current_state);
        let new_priority = self.get_state_priority(new_state);
        new_priority > current_priority
    }
}

/// AI target selection component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct AITargetSelector {
    pub target_types: Vec<AITargetType>,
    pub selection_strategy: TargetSelectionStrategy,
    pub max_target_distance: f32,
    pub target_memory_duration: Duration,
    pub last_target_update: Instant,
    pub update_frequency: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AITargetType {
    Player,
    Enemy,
    Ally,
    Neutral,
    Item,
    Location,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetSelectionStrategy {
    Nearest,
    Weakest,
    Strongest,
    MostThreatening,
    Random,
    LastSeen,
    HighestPriority,
}

impl Default for AITargetSelector {
    fn default() -> Self {
        AITargetSelector {
            target_types: vec![AITargetType::Player, AITargetType::Enemy],
            selection_strategy: TargetSelectionStrategy::Nearest,
            max_target_distance: 20.0,
            target_memory_duration: Duration::from_secs(10),
            last_target_update: Instant::now(),
            update_frequency: Duration::from_millis(500),
        }
    }
}

impl AITargetSelector {
    pub fn new(target_types: Vec<AITargetType>, strategy: TargetSelectionStrategy) -> Self {
        AITargetSelector {
            target_types,
            selection_strategy: strategy,
            ..Default::default()
        }
    }

    pub fn can_update_target(&self) -> bool {
        Instant::now().duration_since(self.last_target_update) >= self.update_frequency
    }

    pub fn mark_target_updated(&mut self) {
        self.last_target_update = Instant::now();
    }
}

/// AI decision system component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[storage(VecStorage)]
pub struct AIDecisionSystem {
    pub decision_tree: Vec<AIDecisionRule>,
    pub decision_weights: HashMap<AIBehaviorState, f32>,
    pub last_decision: Option<AIBehaviorState>,
    pub decision_confidence: f32,
    pub decision_history: VecDeque<(AIBehaviorState, Instant, f32)>, // state, time, confidence
    pub max_history_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIDecisionRule {
    pub condition: AICondition,
    pub action: AIBehaviorState,
    pub priority: u32,
    pub confidence_modifier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AICondition {
    HealthBelow(f32),
    HealthAbove(f32),
    EnemiesNearby(u32),
    AlliesNearby(u32),
    TargetInRange(f32),
    TargetVisible,
    TargetNotVisible,
    TimeInState(Duration),
    ThreatLevel(f32),
    IsOutnumbered,
    HasTarget,
    NoTarget,
    And(Vec<AICondition>),
    Or(Vec<AICondition>),
    Not(Box<AICondition>),
}

impl Default for AIDecisionSystem {
    fn default() -> Self {
        let mut system = AIDecisionSystem {
            decision_tree: Vec::new(),
            decision_weights: HashMap::new(),
            last_decision: None,
            decision_confidence: 0.0,
            decision_history: VecDeque::new(),
            max_history_size: 10,
        };

        // Initialize default decision weights
        system.decision_weights.insert(AIBehaviorState::Flee, 1.0);
        system.decision_weights.insert(AIBehaviorState::Attack, 0.8);
        system.decision_weights.insert(AIBehaviorState::Hunt, 0.7);
        system.decision_weights.insert(AIBehaviorState::Search, 0.6);
        system.decision_weights.insert(AIBehaviorState::Guard, 0.5);
        system.decision_weights.insert(AIBehaviorState::Patrol, 0.4);
        system.decision_weights.insert(AIBehaviorState::Wander, 0.3);
        system.decision_weights.insert(AIBehaviorState::Idle, 0.1);

        // Add default decision rules
        system.add_default_rules();

        system
    }
}

impl AIDecisionSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_rule(&mut self, rule: AIDecisionRule) {
        self.decision_tree.push(rule);
        // Sort by priority (higher priority first)
        self.decision_tree.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    pub fn add_default_rules(&mut self) {
        // High priority: Flee when health is low
        self.add_rule(AIDecisionRule {
            condition: AICondition::HealthBelow(0.3),
            action: AIBehaviorState::Flee,
            priority: 100,
            confidence_modifier: 1.0,
        });

        // High priority: Attack when target is in range and visible
        self.add_rule(AIDecisionRule {
            condition: AICondition::And(vec![
                AICondition::TargetInRange(2.0),
                AICondition::TargetVisible,
                AICondition::HealthAbove(0.3),
            ]),
            action: AIBehaviorState::Attack,
            priority: 90,
            confidence_modifier: 0.9,
        });

        // Medium priority: Hunt when target is visible but not in range
        self.add_rule(AIDecisionRule {
            condition: AICondition::And(vec![
                AICondition::TargetVisible,
                AICondition::Not(Box::new(AICondition::TargetInRange(2.0))),
            ]),
            action: AIBehaviorState::Hunt,
            priority: 70,
            confidence_modifier: 0.8,
        });

        // Medium priority: Search when we have a target but can't see it
        self.add_rule(AIDecisionRule {
            condition: AICondition::And(vec![
                AICondition::HasTarget,
                AICondition::TargetNotVisible,
            ]),
            action: AIBehaviorState::Search,
            priority: 60,
            confidence_modifier: 0.7,
        });

        // Low priority: Patrol when no target
        self.add_rule(AIDecisionRule {
            condition: AICondition::NoTarget,
            action: AIBehaviorState::Patrol,
            priority: 30,
            confidence_modifier: 0.5,
        });
    }

    pub fn evaluate_condition(&self, condition: &AICondition, factors: &AIDecisionFactors) -> bool {
        match condition {
            AICondition::HealthBelow(threshold) => factors.health_percentage < *threshold,
            AICondition::HealthAbove(threshold) => factors.health_percentage > *threshold,
            AICondition::EnemiesNearby(count) => factors.number_of_enemies >= *count,
            AICondition::AlliesNearby(count) => factors.number_of_allies >= *count,
            AICondition::TargetInRange(range) => factors.distance_to_target <= *range,
            AICondition::TargetVisible => factors.has_line_of_sight,
            AICondition::TargetNotVisible => !factors.has_line_of_sight,
            AICondition::TimeInState(duration) => factors.time_since_last_action >= *duration,
            AICondition::ThreatLevel(level) => factors.current_threat_level >= *level,
            AICondition::IsOutnumbered => factors.is_outnumbered,
            AICondition::HasTarget => factors.distance_to_target < f32::MAX,
            AICondition::NoTarget => factors.distance_to_target >= f32::MAX,
            AICondition::And(conditions) => {
                conditions.iter().all(|c| self.evaluate_condition(c, factors))
            },
            AICondition::Or(conditions) => {
                conditions.iter().any(|c| self.evaluate_condition(c, factors))
            },
            AICondition::Not(condition) => {
                !self.evaluate_condition(condition, factors)
            },
        }
    }

    pub fn make_decision(&mut self, factors: &AIDecisionFactors, personality: &AIPersonality) -> Option<AIBehaviorState> {
        let mut best_action = None;
        let mut best_confidence = 0.0;

        for rule in &self.decision_tree {
            if self.evaluate_condition(&rule.condition, factors) {
                let base_weight = self.decision_weights.get(&rule.action).unwrap_or(&0.5);
                let personality_modifier = self.get_personality_modifier(&rule.action, personality);
                let confidence = base_weight * rule.confidence_modifier * personality_modifier;

                if confidence > best_confidence {
                    best_confidence = confidence;
                    best_action = Some(rule.action.clone());
                }
            }
        }

        if let Some(action) = best_action {
            self.last_decision = Some(action.clone());
            self.decision_confidence = best_confidence;
            
            // Add to history
            self.decision_history.push_back((action.clone(), Instant::now(), best_confidence));
            if self.decision_history.len() > self.max_history_size {
                self.decision_history.pop_front();
            }
            
            Some(action)
        } else {
            None
        }
    }

    fn get_personality_modifier(&self, action: &AIBehaviorState, personality: &AIPersonality) -> f32 {
        match action {
            AIBehaviorState::Attack => 0.5 + (personality.aggression * 0.5),
            AIBehaviorState::Flee => 0.5 + ((1.0 - personality.courage) * 0.5),
            AIBehaviorState::Hunt => 0.5 + (personality.aggression * 0.3) + (personality.alertness * 0.2),
            AIBehaviorState::Search => 0.5 + (personality.intelligence * 0.3) + (personality.curiosity * 0.2),
            AIBehaviorState::Guard => 0.5 + (personality.loyalty * 0.5),
            AIBehaviorState::Patrol => 0.5 + (personality.alertness * 0.3),
            AIBehaviorState::Wander => 0.5 + (personality.curiosity * 0.5),
            _ => 1.0,
        }
    }

    pub fn get_decision_confidence(&self) -> f32 {
        self.decision_confidence
    }

    pub fn get_recent_decisions(&self, count: usize) -> Vec<&(AIBehaviorState, Instant, f32)> {
        self.decision_history.iter().rev().take(count).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ai_component_creation() {
        let ai = AI::default();
        assert_eq!(ai.current_state, AIBehaviorState::Idle);
        assert!(ai.enabled);
        assert!(ai.can_make_decision());
    }

    #[test]
    fn test_ai_state_change() {
        let mut ai = AI::default();
        ai.change_state(AIBehaviorState::Hunt);
        
        assert_eq!(ai.current_state, AIBehaviorState::Hunt);
        assert_eq!(ai.previous_state, AIBehaviorState::Idle);
    }

    #[test]
    fn test_ai_personality_traits() {
        let aggressive = AIPersonality::aggressive();
        assert!(aggressive.aggression > 0.7);
        assert!(aggressive.courage > 0.6);
        
        let cowardly = AIPersonality::cowardly();
        assert!(cowardly.courage < 0.2);
        assert!(cowardly.aggression < 0.3);
    }

    #[test]
    fn test_ai_memory() {
        let mut memory = AIMemory::default();
        let entity = Entity::from_raw_index(0);
        let position = crate::components::Position { x: 10, y: 20, z: 1 };
        
        memory.remember_entity(entity, position);
        assert!(memory.seen_entities.contains_key(&entity));
        
        let remembered_pos = memory.get_last_known_position(entity);
        assert_eq!(remembered_pos, Some(position));
    }

    #[test]
    fn test_decision_system() {
        let mut decision_system = AIDecisionSystem::new();
        let factors = AIDecisionFactors {
            health_percentage: 0.2,
            ..Default::default()
        };
        let personality = AIPersonality::default();
        
        let decision = decision_system.make_decision(&factors, &personality);
        assert_eq!(decision, Some(AIBehaviorState::Flee));
    }

    #[test]
    fn test_condition_evaluation() {
        let decision_system = AIDecisionSystem::new();
        let factors = AIDecisionFactors {
            health_percentage: 0.2,
            number_of_enemies: 3,
            has_line_of_sight: true,
            ..Default::default()
        };
        
        assert!(decision_system.evaluate_condition(&AICondition::HealthBelow(0.3), &factors));
        assert!(decision_system.evaluate_condition(&AICondition::EnemiesNearby(2), &factors));
        assert!(decision_system.evaluate_condition(&AICondition::TargetVisible, &factors));
        assert!(!decision_system.evaluate_condition(&AICondition::HealthAbove(0.5), &factors));
    }

    #[test]
    fn test_target_selector() {
        let selector = AITargetSelector::new(
            vec![AITargetType::Player, AITargetType::Enemy],
            TargetSelectionStrategy::Nearest
        );
        
        assert_eq!(selector.target_types.len(), 2);
        assert_eq!(selector.selection_strategy, TargetSelectionStrategy::Nearest);
    }

    #[test]
    fn test_ai_state_priorities() {
        let ai = AI::default();
        
        assert!(ai.get_state_priority(&AIBehaviorState::Flee) > ai.get_state_priority(&AIBehaviorState::Attack));
        assert!(ai.get_state_priority(&AIBehaviorState::Attack) > ai.get_state_priority(&AIBehaviorState::Idle));
        
        assert!(ai.can_interrupt_with(&AIBehaviorState::Flee));
        assert!(!ai.can_interrupt_with(&AIBehaviorState::Idle));
    }

    #[test]
    fn test_complex_conditions() {
        let decision_system = AIDecisionSystem::new();
        let factors = AIDecisionFactors {
            health_percentage: 0.8,
            has_line_of_sight: true,
            distance_to_target: 1.5,
            ..Default::default()
        };
        
        let complex_condition = AICondition::And(vec![
            AICondition::HealthAbove(0.5),
            AICondition::TargetVisible,
            AICondition::TargetInRange(2.0),
        ]);
        
        assert!(decision_system.evaluate_condition(&complex_condition, &factors));
        
        let or_condition = AICondition::Or(vec![
            AICondition::HealthBelow(0.1),
            AICondition::TargetVisible,
        ]);
        
        assert!(decision_system.evaluate_condition(&or_condition, &factors));
    }
}