use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ai::ai_component::{AIComponent, AIBehaviorState, AIAction, DecisionNode, DecisionTest};
use crate::guild::agent_behavior::{AgentBehavior, AgentStatus, AgentBehaviorType, RiskTolerance};
use crate::components::{Position, Health, Viewshed};
use crate::map::DungeonMap;

/// Agent decision context
#[derive(Debug, Clone)]
pub struct AgentDecisionContext {
    pub health_percentage: f32,
    pub nearby_enemies: u32,
    pub nearby_allies: u32,
    pub nearby_items: u32,
    pub nearby_resources: u32,
    pub distance_to_exit: f32,
    pub distance_to_objective: f32,
    pub explored_percentage: f32,
    pub danger_level: f32,
    pub time_in_level: f32,
    pub mission_progress: f32,
    pub inventory_fullness: f32,
}

impl Default for AgentDecisionContext {
    fn default() -> Self {
        AgentDecisionContext {
            health_percentage: 1.0,
            nearby_enemies: 0,
            nearby_allies: 0,
            nearby_items: 0,
            nearby_resources: 0,
            distance_to_exit: f32::MAX,
            distance_to_objective: f32::MAX,
            explored_percentage: 0.0,
            danger_level: 0.0,
            time_in_level: 0.0,
            mission_progress: 0.0,
            inventory_fullness: 0.0,
        }
    }
}

/// Agent decision component
#[derive(Component, Debug)]
pub struct AgentDecisionMaker {
    pub decision_tree: DecisionNode,
    pub context_variables: HashMap<String, f32>,
    pub last_decision: Option<AgentDecision>,
    pub decision_cooldown: f32,
}

/// Agent decisions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AgentDecision {
    ExploreArea,
    EngageEnemy(Entity),
    CollectItem(Entity),
    MoveToLocation(IVec2),
    UseItem(Entity),
    ReturnToSafety,
    CompleteObjective,
    FollowLeader(Entity),
    DefendAlly(Entity),
    Rest,
    Retreat,
    CallForHelp,
}

impl Default for AgentDecisionMaker {
    fn default() -> Self {
        // Create a basic decision tree
        let decision_tree = create_default_decision_tree();
        
        AgentDecisionMaker {
            decision_tree,
            context_variables: HashMap::new(),
            last_decision: None,
            decision_cooldown: 0.0,
        }
    }
}

/// Create a default decision tree for agents
fn create_default_decision_tree() -> DecisionNode {
    // This is a simplified decision tree - in a real implementation,
    // you would create a more complex tree based on agent behavior
    DecisionNode::Condition {
        test: DecisionTest::HealthBelow(0.3),
        true_branch: Box::new(DecisionNode::Action {
            action: AIAction::SetState(AIBehaviorState::Flee),
        }),
        false_branch: Box::new(DecisionNode::Condition {
            test: DecisionTest::ContextVariable("danger_level".to_string(), 0.7),
            true_branch: Box::new(DecisionNode::Condition {
                test: DecisionTest::ContextVariable("risk_tolerance".to_string(), 0.6),
                true_branch: Box::new(DecisionNode::Action {
                    action: AIAction::SetState(AIBehaviorState::Attack),
                }),
                false_branch: Box::new(DecisionNode::Action {
                    action: AIAction::SetState(AIBehaviorState::Flee),
                }),
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
        }),
    }
}

impl AgentDecisionMaker {
    /// Create a new decision maker based on agent behavior
    pub fn new(agent_behavior: &AgentBehavior) -> Self {
        let mut decision_maker = AgentDecisionMaker::default();
        
        // Set context variables based on agent behavior
        decision_maker.context_variables.insert("exploration_thoroughness".to_string(), agent_behavior.exploration_thoroughness);
        decision_maker.context_variables.insert("combat_aggression".to_string(), agent_behavior.combat_aggression);
        
        // Set risk tolerance
        let risk_value = match agent_behavior.risk_tolerance {
            RiskTolerance::VeryLow => 0.1,
            RiskTolerance::Low => 0.3,
            RiskTolerance::Medium => 0.5,
            RiskTolerance::High => 0.7,
            RiskTolerance::VeryHigh => 0.9,
        };
        decision_maker.context_variables.insert("risk_tolerance".to_string(), risk_value);
        
        // Customize decision tree based on behavior type
        decision_maker.decision_tree = match agent_behavior.behavior_type {
            AgentBehaviorType::Aggressive => create_aggressive_decision_tree(),
            AgentBehaviorType::Cautious => create_cautious_decision_tree(),
            AgentBehaviorType::ResourceFocused => create_resource_focused_decision_tree(),
            AgentBehaviorType::Protective => create_protective_decision_tree(),
            _ => create_default_decision_tree(),
        };
        
        decision_maker
    }
    
    /// Make a decision based on current context
    pub fn make_decision(&mut self, context: &AgentDecisionContext) -> AgentDecision {
        // Update context variables
        self.context_variables.insert("health_percentage".to_string(), context.health_percentage);
        self.context_variables.insert("nearby_enemies".to_string(), context.nearby_enemies as f32);
        self.context_variables.insert("nearby_allies".to_string(), context.nearby_allies as f32);
        self.context_variables.insert("nearby_items".to_string(), context.nearby_items as f32);
        self.context_variables.insert("danger_level".to_string(), context.danger_level);
        self.context_variables.insert("mission_progress".to_string(), context.mission_progress);
        
        // In a real implementation, you would evaluate the decision tree here
        // For now, we'll use a simplified approach
        
        if context.health_percentage < 0.3 {
            // Low health, retreat or use healing item
            if context.nearby_enemies > 0 {
                self.last_decision = Some(AgentDecision::Retreat);
            } else {
                self.last_decision = Some(AgentDecision::UseItem(Entity::from_raw(0))); // Placeholder entity
            }
        } else if context.danger_level > 0.7 && self.context_variables.get("risk_tolerance").unwrap_or(&0.5) < &0.6 {
            // High danger and low risk tolerance, retreat
            self.last_decision = Some(AgentDecision::Retreat);
        } else if context.nearby_enemies > 0 && self.context_variables.get("combat_aggression").unwrap_or(&0.5) > &0.6 {
            // Enemies nearby and high combat aggression, engage
            self.last_decision = Some(AgentDecision::EngageEnemy(Entity::from_raw(0))); // Placeholder entity
        } else if context.nearby_items > 0 && context.inventory_fullness < 0.9 {
            // Items nearby and inventory not full, collect
            self.last_decision = Some(AgentDecision::CollectItem(Entity::from_raw(0))); // Placeholder entity
        } else if context.explored_percentage < 0.8 && self.context_variables.get("exploration_thoroughness").unwrap_or(&0.5) > &0.6 {
            // Area not fully explored and high thoroughness, explore
            self.last_decision = Some(AgentDecision::ExploreArea);
        } else if context.mission_progress > 0.9 {
            // Mission almost complete, finish objective
            self.last_decision = Some(AgentDecision::CompleteObjective);
        } else {
            // Default: explore
            self.last_decision = Some(AgentDecision::ExploreArea);
        }
        
        self.last_decision.clone().unwrap()
    }
    
    /// Update decision cooldown
    pub fn update(&mut self, delta_time: f32) {
        if self.decision_cooldown > 0.0 {
            self.decision_cooldown -= delta_time;
        }
    }
    
    /// Check if can make a new decision
    pub fn can_decide(&self) -> bool {
        self.decision_cooldown <= 0.0
    }
    
    /// Set a context variable
    pub fn set_context_variable(&mut self, name: &str, value: f32) {
        self.context_variables.insert(name.to_string(), value);
    }
}

/// Create an aggressive decision tree
fn create_aggressive_decision_tree() -> DecisionNode {
    // In a real implementation, this would be a more complex tree
    // For now, we'll use a simplified version
    DecisionNode::Condition {
        test: DecisionTest::HealthBelow(0.2),
        true_branch: Box::new(DecisionNode::Action {
            action: AIAction::SetState(AIBehaviorState::Flee),
        }),
        false_branch: Box::new(DecisionNode::Condition {
            test: DecisionTest::HasTarget,
            true_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Attack),
            }),
            false_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Hunt),
            }),
        }),
    }
}

/// Create a cautious decision tree
fn create_cautious_decision_tree() -> DecisionNode {
    DecisionNode::Condition {
        test: DecisionTest::HealthBelow(0.5),
        true_branch: Box::new(DecisionNode::Action {
            action: AIAction::SetState(AIBehaviorState::Flee),
        }),
        false_branch: Box::new(DecisionNode::Condition {
            test: DecisionTest::EnemiesNearby(1),
            true_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Flee),
            }),
            false_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Patrol),
            }),
        }),
    }
}

/// Create a resource-focused decision tree
fn create_resource_focused_decision_tree() -> DecisionNode {
    // Simplified version
    DecisionNode::Condition {
        test: DecisionTest::HealthBelow(0.3),
        true_branch: Box::new(DecisionNode::Action {
            action: AIAction::SetState(AIBehaviorState::Flee),
        }),
        false_branch: Box::new(DecisionNode::Condition {
            test: DecisionTest::ContextVariable("nearby_resources".to_string(), 0.5),
            true_branch: Box::new(DecisionNode::Action {
                action: AIAction::Custom("collect_resource".to_string()),
            }),
            false_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Patrol),
            }),
        }),
    }
}

/// Create a protective decision tree
fn create_protective_decision_tree() -> DecisionNode {
    // Simplified version
    DecisionNode::Condition {
        test: DecisionTest::HealthBelow(0.3),
        true_branch: Box::new(DecisionNode::Action {
            action: AIAction::SetState(AIBehaviorState::Flee),
        }),
        false_branch: Box::new(DecisionNode::Condition {
            test: DecisionTest::ContextVariable("ally_in_danger".to_string(), 0.5),
            true_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Attack),
            }),
            false_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Follow),
            }),
        }),
    }
}

/// System for agent decision making
pub fn agent_decision_system(
    time: Res<Time>,
    mut agent_query: Query<(Entity, &AgentBehavior, &AgentStatus, &mut AgentDecisionMaker, &mut AIComponent)>,
    position_query: Query<&Position>,
    health_query: Query<&Health>,
    viewshed_query: Query<&Viewshed>,
    map: Option<Res<DungeonMap>>,
) {
    let delta_time = time.delta_seconds();
    
    for (entity, agent_behavior, agent_status, mut decision_maker, mut ai_component) in agent_query.iter_mut() {
        // Update decision cooldown
        decision_maker.update(delta_time);
        
        // Only make new decisions when cooldown is complete
        if !decision_maker.can_decide() {
            continue;
        }
        
        // Build decision context
        let mut context = AgentDecisionContext::default();
        
        // Update context from agent status
        context.health_percentage = agent_status.health_percentage;
        context.mission_progress = agent_status.task_progress;
        
        // Get position information
        if let Ok(position) = position_query.get(entity) {
            // In a real implementation, you would gather information about
            // nearby entities, items, and map features here
            
            // For now, we'll use placeholder values
            context.nearby_enemies = 0;
            context.nearby_allies = 0;
            context.nearby_items = 0;
            context.nearby_resources = 0;
            
            // Calculate danger level based on health and enemies
            context.danger_level = (1.0 - context.health_percentage) * 0.5 + 
                                  (context.nearby_enemies as f32 * 0.2);
        }
        
        // Make a decision
        let decision = decision_maker.make_decision(&context);
        
        // Apply decision to AI component
        apply_decision_to_ai(&decision, &mut ai_component);
        
        // Set decision cooldown
        decision_maker.decision_cooldown = 0.5; // Make decisions every 0.5 seconds
    }
}

/// Apply agent decision to AI component
fn apply_decision_to_ai(decision: &AgentDecision, ai_component: &mut AIComponent) {
    match decision {
        AgentDecision::ExploreArea => {
            ai_component.transition_to_state(AIBehaviorState::Patrol);
        },
        AgentDecision::EngageEnemy(target) => {
            ai_component.set_target(Some(*target));
            ai_component.transition_to_state(AIBehaviorState::Hunt);
        },
        AgentDecision::CollectItem(_) => {
            ai_component.transition_to_state(AIBehaviorState::Patrol);
        },
        AgentDecision::MoveToLocation(location) => {
            ai_component.memory.last_known_target_position = Some(Vec2::new(location.x as f32, location.y as f32));
            ai_component.transition_to_state(AIBehaviorState::Patrol);
        },
        AgentDecision::UseItem(_) => {
            // This would be handled by inventory systems
        },
        AgentDecision::ReturnToSafety => {
            ai_component.transition_to_state(AIBehaviorState::Flee);
        },
        AgentDecision::CompleteObjective => {
            // Set target to objective location
            ai_component.transition_to_state(AIBehaviorState::Patrol);
        },
        AgentDecision::FollowLeader(leader) => {
            ai_component.set_target(Some(*leader));
            ai_component.transition_to_state(AIBehaviorState::Follow);
        },
        AgentDecision::DefendAlly(ally) => {
            ai_component.set_target(Some(*ally));
            ai_component.transition_to_state(AIBehaviorState::Guard);
        },
        AgentDecision::Rest => {
            ai_component.transition_to_state(AIBehaviorState::Idle);
        },
        AgentDecision::Retreat => {
            ai_component.transition_to_state(AIBehaviorState::Flee);
        },
        AgentDecision::CallForHelp => {
            // This would be handled by communication systems
        },
    }
}

/// Plugin for agent decision systems
pub struct AgentDecisionPlugin;

impl Plugin for AgentDecisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, agent_decision_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guild::agent_behavior::AgentBehavior;

    #[test]
    fn test_decision_maker_creation() {
        let agent_behavior = AgentBehavior::new(AgentBehaviorType::Aggressive);
        let decision_maker = AgentDecisionMaker::new(&agent_behavior);
        
        // Check that context variables were set correctly
        assert!(decision_maker.context_variables.contains_key("combat_aggression"));
        assert!(decision_maker.context_variables.contains_key("risk_tolerance"));
        
        // Check that combat aggression is high for aggressive behavior
        assert!(*decision_maker.context_variables.get("combat_aggression").unwrap() > 0.8);
    }

    #[test]
    fn test_decision_making() {
        let mut decision_maker = AgentDecisionMaker::default();
        
        // Test low health scenario
        let mut context = AgentDecisionContext::default();
        context.health_percentage = 0.2;
        context.nearby_enemies = 2;
        
        let decision = decision_maker.make_decision(&context);
        assert!(matches!(decision, AgentDecision::Retreat | AgentDecision::UseItem(_)));
        
        // Test combat scenario
        context.health_percentage = 0.8;
        decision_maker.set_context_variable("combat_aggression", 0.9);
        
        let decision = decision_maker.make_decision(&context);
        assert!(matches!(decision, AgentDecision::EngageEnemy(_)));
    }

    #[test]
    fn test_decision_cooldown() {
        let mut decision_maker = AgentDecisionMaker::default();
        decision_maker.decision_cooldown = 1.0;
        
        assert!(!decision_maker.can_decide());
        
        decision_maker.update(0.5);
        assert!(!decision_maker.can_decide());
        
        decision_maker.update(0.5);
        assert!(decision_maker.can_decide());
    }
}