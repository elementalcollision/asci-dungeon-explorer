use specs::{System, ReadStorage, WriteStorage, Entities, Entity, Join};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use crate::ai::ai_components::{
    AI, AIBehaviorState, AIDecisionSystem, AIDecisionFactors, AICondition, 
    AIDecisionRule, AIPersonality,
};
use crate::components::{Position, Health, Player, Name};

/// Decision making system that evaluates AI behavior choices
pub struct DecisionMakingSystem {
    last_update: Instant,
    update_frequency: Duration,
    decision_history: HashMap<Entity, VecDeque<DecisionRecord>>,
    max_history_per_entity: usize,
}

#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub timestamp: Instant,
    pub decision: AIBehaviorState,
    pub confidence: f32,
    pub factors: AIDecisionFactors,
    pub execution_time: Duration,
}

impl DecisionMakingSystem {
    pub fn new() -> Self {
        DecisionMakingSystem {
            last_update: Instant::now(),
            update_frequency: Duration::from_millis(200), // 5 times per second
            decision_history: HashMap::new(),
            max_history_per_entity: 20,
        }
    }

    pub fn with_update_frequency(mut self, frequency: Duration) -> Self {
        self.update_frequency = frequency;
        self
    }

    pub fn with_max_history(mut self, max_history: usize) -> Self {
        self.max_history_per_entity = max_history;
        self
    }
}

impl<'a> System<'a> for DecisionMakingSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, AI>,
        WriteStorage<'a, AIDecisionSystem>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, (entities, mut ais, mut decision_systems, positions, healths, players, names): Self::SystemData) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_frequency {
            return;
        }

        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Clean up old decision history
        for history in self.decision_history.values_mut() {
            history.retain(|record| now.duration_since(record.timestamp) < Duration::from_secs(60));
        }

        // Process each AI entity with a decision system
        for (entity, ai, decision_system) in (&entities, &mut ais, &mut decision_systems).join() {
            if !ai.enabled || !ai.can_make_decision() {
                continue;
            }

            let decision_start = Instant::now();

            // Make decision
            if let Some(new_state) = decision_system.make_decision(&ai.decision_factors, &ai.personality) {
                let execution_time = decision_start.elapsed();
                let confidence = decision_system.get_decision_confidence();

                // Record the decision
                let record = DecisionRecord {
                    timestamp: now,
                    decision: new_state.clone(),
                    confidence,
                    factors: ai.decision_factors.clone(),
                    execution_time,
                };

                // Add to history
                let history = self.decision_history.entry(entity).or_insert_with(VecDeque::new);
                history.push_back(record);
                if history.len() > self.max_history_per_entity {
                    history.pop_front();
                }

                // Apply decision if it can interrupt current state
                if ai.can_interrupt_with(&new_state) {
                    ai.change_state(new_state);
                }
            }

            // Update decision system with learning from recent decisions
            self.update_decision_learning(entity, decision_system, &ai.personality);
        }
    }
}

impl DecisionMakingSystem {
    /// Update decision system based on recent decision outcomes
    fn update_decision_learning(
        &mut self,
        entity: Entity,
        decision_system: &mut AIDecisionSystem,
        personality: &AIPersonality,
    ) {
        if let Some(history) = self.decision_history.get(&entity) {
            if history.len() < 3 {
                return; // Need some history to learn from
            }

            // Analyze recent decision patterns
            let recent_decisions: Vec<&DecisionRecord> = history.iter().rev().take(5).collect();
            
            // Check for decision oscillation (rapidly changing between states)
            let mut state_changes = 0;
            for i in 1..recent_decisions.len() {
                if recent_decisions[i].decision != recent_decisions[i-1].decision {
                    state_changes += 1;
                }
            }

            // If too much oscillation, increase decision cooldown
            if state_changes > 3 {
                // Increase confidence requirements for state changes
                for rule in &mut decision_system.decision_tree {
                    rule.confidence_modifier *= 0.9; // Reduce confidence slightly
                }
            }

            // Analyze decision effectiveness based on personality
            self.analyze_decision_effectiveness(decision_system, &recent_decisions, personality);
        }
    }

    /// Analyze how effective recent decisions have been
    fn analyze_decision_effectiveness(
        &self,
        decision_system: &mut AIDecisionSystem,
        recent_decisions: &[&DecisionRecord],
        personality: &AIPersonality,
    ) {
        // Group decisions by state
        let mut state_outcomes: HashMap<AIBehaviorState, Vec<f32>> = HashMap::new();
        
        for decision in recent_decisions {
            let effectiveness = self.calculate_decision_effectiveness(decision, personality);
            state_outcomes.entry(decision.decision.clone())
                .or_insert_with(Vec::new)
                .push(effectiveness);
        }

        // Update decision weights based on effectiveness
        for (state, outcomes) in state_outcomes {
            if outcomes.len() >= 2 {
                let average_effectiveness: f32 = outcomes.iter().sum::<f32>() / outcomes.len() as f32;
                
                // Adjust weight based on effectiveness
                let current_weight = decision_system.decision_weights.get(&state).unwrap_or(&0.5);
                let adjustment = (average_effectiveness - 0.5) * 0.1; // Small adjustments
                let new_weight = (current_weight + adjustment).clamp(0.1, 1.0);
                
                decision_system.decision_weights.insert(state, new_weight);
            }
        }
    }

    /// Calculate how effective a decision was based on outcomes
    fn calculate_decision_effectiveness(&self, decision: &DecisionRecord, personality: &AIPersonality) -> f32 {
        let mut effectiveness = 0.5; // Base effectiveness

        // Factor in decision confidence
        effectiveness += (decision.confidence - 0.5) * 0.2;

        // Factor in health changes (if we had before/after health data)
        // This would require tracking health changes over time
        
        // Factor in personality alignment
        let personality_alignment = match decision.decision {
            AIBehaviorState::Attack => personality.aggression,
            AIBehaviorState::Flee => 1.0 - personality.courage,
            AIBehaviorState::Hunt => (personality.aggression + personality.alertness) / 2.0,
            AIBehaviorState::Search => (personality.intelligence + personality.curiosity) / 2.0,
            AIBehaviorState::Guard => personality.loyalty,
            AIBehaviorState::Patrol => personality.alertness,
            AIBehaviorState::Wander => personality.curiosity,
            _ => 0.5,
        };

        effectiveness += (personality_alignment - 0.5) * 0.3;

        // Factor in execution time (faster decisions might be better in some cases)
        let execution_factor = if decision.execution_time < Duration::from_millis(50) {
            0.1 // Quick decisions get a small bonus
        } else if decision.execution_time > Duration::from_millis(200) {
            -0.1 // Slow decisions get a small penalty
        } else {
            0.0
        };

        effectiveness += execution_factor;

        effectiveness.clamp(0.0, 1.0)
    }

    /// Get decision statistics for an entity
    pub fn get_decision_statistics(&self, entity: Entity) -> Option<DecisionStatistics> {
        if let Some(history) = self.decision_history.get(&entity) {
            if history.is_empty() {
                return None;
            }

            let total_decisions = history.len();
            let average_confidence: f32 = history.iter()
                .map(|record| record.confidence)
                .sum::<f32>() / total_decisions as f32;

            let average_execution_time = Duration::from_nanos(
                history.iter()
                    .map(|record| record.execution_time.as_nanos())
                    .sum::<u128>() / total_decisions as u128
            );

            // Count decisions by state
            let mut state_counts: HashMap<AIBehaviorState, usize> = HashMap::new();
            for record in history {
                *state_counts.entry(record.decision.clone()).or_insert(0) += 1;
            }

            // Find most common decision
            let most_common_decision = state_counts.iter()
                .max_by_key(|(_, count)| *count)
                .map(|(state, _)| state.clone());

            // Calculate decision frequency (decisions per minute)
            let time_span = if history.len() > 1 {
                history.back().unwrap().timestamp.duration_since(history.front().unwrap().timestamp)
            } else {
                Duration::from_secs(1)
            };
            let decisions_per_minute = (total_decisions as f32 * 60.0) / time_span.as_secs() as f32;

            Some(DecisionStatistics {
                total_decisions,
                average_confidence,
                average_execution_time,
                state_distribution: state_counts,
                most_common_decision,
                decisions_per_minute,
                recent_decisions: history.iter().rev().take(5).cloned().collect(),
            })
        } else {
            None
        }
    }

    /// Get overall system statistics
    pub fn get_system_statistics(&self) -> SystemDecisionStatistics {
        let total_entities = self.decision_history.len();
        let total_decisions: usize = self.decision_history.values()
            .map(|history| history.len())
            .sum();

        let average_decisions_per_entity = if total_entities > 0 {
            total_decisions as f32 / total_entities as f32
        } else {
            0.0
        };

        let mut all_execution_times = Vec::new();
        let mut all_confidences = Vec::new();
        let mut global_state_counts: HashMap<AIBehaviorState, usize> = HashMap::new();

        for history in self.decision_history.values() {
            for record in history {
                all_execution_times.push(record.execution_time);
                all_confidences.push(record.confidence);
                *global_state_counts.entry(record.decision.clone()).or_insert(0) += 1;
            }
        }

        let average_execution_time = if !all_execution_times.is_empty() {
            Duration::from_nanos(
                all_execution_times.iter()
                    .map(|d| d.as_nanos())
                    .sum::<u128>() / all_execution_times.len() as u128
            )
        } else {
            Duration::from_secs(0)
        };

        let average_confidence = if !all_confidences.is_empty() {
            all_confidences.iter().sum::<f32>() / all_confidences.len() as f32
        } else {
            0.0
        };

        SystemDecisionStatistics {
            total_entities_with_decisions: total_entities,
            total_decisions,
            average_decisions_per_entity,
            average_execution_time,
            average_confidence,
            global_state_distribution: global_state_counts,
        }
    }

    /// Clear decision history for an entity
    pub fn clear_entity_history(&mut self, entity: Entity) {
        self.decision_history.remove(&entity);
    }

    /// Clear all decision history
    pub fn clear_all_history(&mut self) {
        self.decision_history.clear();
    }
}

/// Decision statistics for a single entity
#[derive(Debug, Clone)]
pub struct DecisionStatistics {
    pub total_decisions: usize,
    pub average_confidence: f32,
    pub average_execution_time: Duration,
    pub state_distribution: HashMap<AIBehaviorState, usize>,
    pub most_common_decision: Option<AIBehaviorState>,
    pub decisions_per_minute: f32,
    pub recent_decisions: Vec<DecisionRecord>,
}

/// System-wide decision statistics
#[derive(Debug, Clone)]
pub struct SystemDecisionStatistics {
    pub total_entities_with_decisions: usize,
    pub total_decisions: usize,
    pub average_decisions_per_entity: f32,
    pub average_execution_time: Duration,
    pub average_confidence: f32,
    pub global_state_distribution: HashMap<AIBehaviorState, usize>,
}

/// Advanced decision evaluator with fuzzy logic
pub struct FuzzyDecisionEvaluator {
    pub fuzzy_rules: Vec<FuzzyRule>,
    pub membership_functions: HashMap<String, MembershipFunction>,
}

#[derive(Debug, Clone)]
pub struct FuzzyRule {
    pub conditions: Vec<FuzzyCondition>,
    pub conclusion: FuzzyConclusion,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct FuzzyCondition {
    pub variable: String,
    pub membership_function: String,
    pub negated: bool,
}

#[derive(Debug, Clone)]
pub struct FuzzyConclusion {
    pub action: AIBehaviorState,
    pub confidence: f32,
}

#[derive(Debug, Clone)]
pub enum MembershipFunction {
    Triangular { low: f32, peak: f32, high: f32 },
    Trapezoidal { low: f32, low_peak: f32, high_peak: f32, high: f32 },
    Gaussian { center: f32, width: f32 },
}

impl FuzzyDecisionEvaluator {
    pub fn new() -> Self {
        let mut evaluator = FuzzyDecisionEvaluator {
            fuzzy_rules: Vec::new(),
            membership_functions: HashMap::new(),
        };

        evaluator.initialize_default_functions();
        evaluator.initialize_default_rules();

        evaluator
    }

    fn initialize_default_functions(&mut self) {
        // Health membership functions
        self.membership_functions.insert(
            "health_low".to_string(),
            MembershipFunction::Triangular { low: 0.0, peak: 0.0, high: 0.4 }
        );
        self.membership_functions.insert(
            "health_medium".to_string(),
            MembershipFunction::Triangular { low: 0.2, peak: 0.5, high: 0.8 }
        );
        self.membership_functions.insert(
            "health_high".to_string(),
            MembershipFunction::Triangular { low: 0.6, peak: 1.0, high: 1.0 }
        );

        // Distance membership functions
        self.membership_functions.insert(
            "distance_close".to_string(),
            MembershipFunction::Triangular { low: 0.0, peak: 0.0, high: 3.0 }
        );
        self.membership_functions.insert(
            "distance_medium".to_string(),
            MembershipFunction::Triangular { low: 2.0, peak: 5.0, high: 8.0 }
        );
        self.membership_functions.insert(
            "distance_far".to_string(),
            MembershipFunction::Triangular { low: 6.0, peak: 10.0, high: 20.0 }
        );

        // Threat level membership functions
        self.membership_functions.insert(
            "threat_low".to_string(),
            MembershipFunction::Triangular { low: 0.0, peak: 0.0, high: 0.3 }
        );
        self.membership_functions.insert(
            "threat_high".to_string(),
            MembershipFunction::Triangular { low: 0.5, peak: 1.0, high: 1.0 }
        );
    }

    fn initialize_default_rules(&mut self) {
        // Rule: If health is low, then flee with high confidence
        self.fuzzy_rules.push(FuzzyRule {
            conditions: vec![
                FuzzyCondition {
                    variable: "health".to_string(),
                    membership_function: "health_low".to_string(),
                    negated: false,
                }
            ],
            conclusion: FuzzyConclusion {
                action: AIBehaviorState::Flee,
                confidence: 0.9,
            },
            weight: 1.0,
        });

        // Rule: If health is high and distance is close, then attack
        self.fuzzy_rules.push(FuzzyRule {
            conditions: vec![
                FuzzyCondition {
                    variable: "health".to_string(),
                    membership_function: "health_high".to_string(),
                    negated: false,
                },
                FuzzyCondition {
                    variable: "distance".to_string(),
                    membership_function: "distance_close".to_string(),
                    negated: false,
                }
            ],
            conclusion: FuzzyConclusion {
                action: AIBehaviorState::Attack,
                confidence: 0.8,
            },
            weight: 0.9,
        });

        // Rule: If distance is medium and threat is high, then hunt
        self.fuzzy_rules.push(FuzzyRule {
            conditions: vec![
                FuzzyCondition {
                    variable: "distance".to_string(),
                    membership_function: "distance_medium".to_string(),
                    negated: false,
                },
                FuzzyCondition {
                    variable: "threat".to_string(),
                    membership_function: "threat_high".to_string(),
                    negated: false,
                }
            ],
            conclusion: FuzzyConclusion {
                action: AIBehaviorState::Hunt,
                confidence: 0.7,
            },
            weight: 0.8,
        });
    }

    pub fn evaluate_membership(&self, function: &MembershipFunction, value: f32) -> f32 {
        match function {
            MembershipFunction::Triangular { low, peak, high } => {
                if value <= *low || value >= *high {
                    0.0
                } else if value <= *peak {
                    (value - low) / (peak - low)
                } else {
                    (high - value) / (high - peak)
                }
            },
            MembershipFunction::Trapezoidal { low, low_peak, high_peak, high } => {
                if value <= *low || value >= *high {
                    0.0
                } else if value <= *low_peak {
                    (value - low) / (low_peak - low)
                } else if value <= *high_peak {
                    1.0
                } else {
                    (high - value) / (high - high_peak)
                }
            },
            MembershipFunction::Gaussian { center, width } => {
                let diff = value - center;
                (-0.5 * (diff / width).powi(2)).exp()
            },
        }
    }

    pub fn evaluate_fuzzy_decision(&self, factors: &AIDecisionFactors) -> Option<(AIBehaviorState, f32)> {
        let mut best_action = None;
        let mut best_confidence = 0.0;

        // Create variable map
        let mut variables = HashMap::new();
        variables.insert("health".to_string(), factors.health_percentage);
        variables.insert("distance".to_string(), factors.distance_to_target);
        variables.insert("threat".to_string(), factors.current_threat_level);

        for rule in &self.fuzzy_rules {
            let mut rule_strength = 1.0;

            // Evaluate all conditions
            for condition in &rule.conditions {
                if let (Some(value), Some(function)) = (
                    variables.get(&condition.variable),
                    self.membership_functions.get(&condition.membership_function)
                ) {
                    let membership = self.evaluate_membership(function, *value);
                    let condition_strength = if condition.negated {
                        1.0 - membership
                    } else {
                        membership
                    };

                    rule_strength = rule_strength.min(condition_strength);
                } else {
                    rule_strength = 0.0;
                    break;
                }
            }

            // Apply rule weight and conclusion confidence
            let final_confidence = rule_strength * rule.weight * rule.conclusion.confidence;

            if final_confidence > best_confidence {
                best_confidence = final_confidence;
                best_action = Some(rule.conclusion.action.clone());
            }
        }

        best_action.map(|action| (action, best_confidence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::ai::ai_components::{AI, AIPersonality};

    #[test]
    fn test_decision_making_system() {
        let system = DecisionMakingSystem::new();
        assert_eq!(system.update_frequency, Duration::from_millis(200));
        assert_eq!(system.max_history_per_entity, 20);
    }

    #[test]
    fn test_decision_record() {
        let record = DecisionRecord {
            timestamp: Instant::now(),
            decision: AIBehaviorState::Hunt,
            confidence: 0.8,
            factors: AIDecisionFactors::default(),
            execution_time: Duration::from_millis(50),
        };

        assert_eq!(record.decision, AIBehaviorState::Hunt);
        assert_eq!(record.confidence, 0.8);
    }

    #[test]
    fn test_decision_effectiveness_calculation() {
        let system = DecisionMakingSystem::new();
        let personality = AIPersonality::aggressive();
        
        let decision = DecisionRecord {
            timestamp: Instant::now(),
            decision: AIBehaviorState::Attack,
            confidence: 0.9,
            factors: AIDecisionFactors::default(),
            execution_time: Duration::from_millis(30),
        };

        let effectiveness = system.calculate_decision_effectiveness(&decision, &personality);
        assert!(effectiveness > 0.5); // Should be effective for aggressive personality
    }

    #[test]
    fn test_fuzzy_decision_evaluator() {
        let evaluator = FuzzyDecisionEvaluator::new();
        
        // Test membership function evaluation
        let triangular = MembershipFunction::Triangular { low: 0.0, peak: 0.5, high: 1.0 };
        assert_eq!(evaluator.evaluate_membership(&triangular, 0.5), 1.0);
        assert_eq!(evaluator.evaluate_membership(&triangular, 0.25), 0.5);
        assert_eq!(evaluator.evaluate_membership(&triangular, 0.0), 0.0);
    }

    #[test]
    fn test_fuzzy_decision_evaluation() {
        let evaluator = FuzzyDecisionEvaluator::new();
        
        let factors = AIDecisionFactors {
            health_percentage: 0.2, // Low health
            distance_to_target: 5.0,
            current_threat_level: 0.8,
            ..Default::default()
        };

        let decision = evaluator.evaluate_fuzzy_decision(&factors);
        assert!(decision.is_some());
        
        let (action, confidence) = decision.unwrap();
        assert_eq!(action, AIBehaviorState::Flee); // Should flee with low health
        assert!(confidence > 0.0);
    }

    #[test]
    fn test_system_statistics() {
        let mut system = DecisionMakingSystem::new();
        
        // Add some mock decision history
        let entity = Entity::from_raw_index(0);
        let mut history = VecDeque::new();
        history.push_back(DecisionRecord {
            timestamp: Instant::now(),
            decision: AIBehaviorState::Hunt,
            confidence: 0.8,
            factors: AIDecisionFactors::default(),
            execution_time: Duration::from_millis(50),
        });
        
        system.decision_history.insert(entity, history);
        
        let stats = system.get_system_statistics();
        assert_eq!(stats.total_entities_with_decisions, 1);
        assert_eq!(stats.total_decisions, 1);
    }

    #[test]
    fn test_entity_decision_statistics() {
        let mut system = DecisionMakingSystem::new();
        let entity = Entity::from_raw_index(0);
        
        // Add decision history
        let mut history = VecDeque::new();
        for i in 0..5 {
            history.push_back(DecisionRecord {
                timestamp: Instant::now(),
                decision: if i % 2 == 0 { AIBehaviorState::Hunt } else { AIBehaviorState::Attack },
                confidence: 0.7 + (i as f32 * 0.05),
                factors: AIDecisionFactors::default(),
                execution_time: Duration::from_millis(40 + i * 10),
            });
        }
        
        system.decision_history.insert(entity, history);
        
        let stats = system.get_decision_statistics(entity).unwrap();
        assert_eq!(stats.total_decisions, 5);
        assert!(stats.average_confidence > 0.7);
        assert!(stats.most_common_decision.is_some());
    }
}