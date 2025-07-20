use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ai::ai_component::{AIBehaviorState, AIPersonality, AIComponent};
use crate::guild::guild_core::{GuildMember, GuildResource};
use crate::components::{Position, Health};

/// Agent behavior types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentBehaviorType {
    Aggressive,    // Prioritizes combat and hunting enemies
    Cautious,      // Prioritizes safety and avoiding danger
    Balanced,      // Balanced approach to exploration and combat
    Thorough,      // Explores thoroughly, checks every corner
    Speedy,        // Moves quickly through areas, minimal exploration
    ResourceFocused, // Prioritizes gathering resources
    Protective,    // Focuses on protecting other agents or the player
}

impl Default for AgentBehaviorType {
    fn default() -> Self {
        AgentBehaviorType::Balanced
    }
}

/// Agent target priority
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TargetPriority {
    pub target_type: TargetType,
    pub priority: u32,
}

/// Types of targets agents can prioritize
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetType {
    Enemy(String),     // Enemy type name
    Item(String),      // Item type name
    Feature(String),   // Feature type name
    Resource(GuildResource), // Guild resource
    Exit,              // Level exit
    Player,            // The player character
    GuildMember,       // Other guild members
}

/// Agent risk tolerance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskTolerance {
    VeryLow,   // Extremely cautious, avoids all danger
    Low,       // Cautious, avoids most danger
    Medium,    // Balanced approach to risk
    High,      // Takes calculated risks
    VeryHigh,  // Takes significant risks
}

impl Default for RiskTolerance {
    fn default() -> Self {
        RiskTolerance::Medium
    }
}

/// Agent cooperation styles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CooperationStyle {
    Independent, // Acts alone, minimal cooperation
    Supportive,  // Supports others but maintains independence
    Leader,      // Takes leadership role in groups
    Follower,    // Follows others' leads
}

impl Default for CooperationStyle {
    fn default() -> Self {
        CooperationStyle::Supportive
    }
}

/// Agent behavior component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehavior {
    pub behavior_type: AgentBehaviorType,
    pub priority_targets: Vec<TargetPriority>,
    pub item_preferences: HashMap<String, u32>, // Item type -> priority
    pub risk_tolerance: RiskTolerance,
    pub cooperation_style: CooperationStyle,
    pub special_abilities: Vec<String>,
    pub mission_focus: Option<String>, // Current mission focus
    pub exploration_thoroughness: f32, // 0.0 = minimal, 1.0 = complete
    pub combat_aggression: f32,        // 0.0 = avoid, 1.0 = seek out
    pub resource_focus: HashMap<GuildResource, f32>, // Resource -> focus level
}

impl Default for AgentBehavior {
    fn default() -> Self {
        AgentBehavior {
            behavior_type: AgentBehaviorType::Balanced,
            priority_targets: Vec::new(),
            item_preferences: HashMap::new(),
            risk_tolerance: RiskTolerance::Medium,
            cooperation_style: CooperationStyle::Supportive,
            special_abilities: Vec::new(),
            mission_focus: None,
            exploration_thoroughness: 0.5,
            combat_aggression: 0.5,
            resource_focus: HashMap::new(),
        }
    }
}

impl AgentBehavior {
    /// Create a new agent behavior with specific type
    pub fn new(behavior_type: AgentBehaviorType) -> Self {
        let mut behavior = match behavior_type {
            AgentBehaviorType::Aggressive => AgentBehavior {
                behavior_type: AgentBehaviorType::Aggressive,
                risk_tolerance: RiskTolerance::High,
                exploration_thoroughness: 0.3,
                combat_aggression: 0.9,
                ..Default::default()
            },
            AgentBehaviorType::Cautious => AgentBehavior {
                behavior_type: AgentBehaviorType::Cautious,
                risk_tolerance: RiskTolerance::Low,
                exploration_thoroughness: 0.7,
                combat_aggression: 0.2,
                ..Default::default()
            },
            AgentBehaviorType::Balanced => AgentBehavior::default(),
            AgentBehaviorType::Thorough => AgentBehavior {
                behavior_type: AgentBehaviorType::Thorough,
                risk_tolerance: RiskTolerance::Medium,
                exploration_thoroughness: 1.0,
                combat_aggression: 0.4,
                ..Default::default()
            },
            AgentBehaviorType::Speedy => AgentBehavior {
                behavior_type: AgentBehaviorType::Speedy,
                risk_tolerance: RiskTolerance::High,
                exploration_thoroughness: 0.1,
                combat_aggression: 0.5,
                ..Default::default()
            },
            AgentBehaviorType::ResourceFocused => AgentBehavior {
                behavior_type: AgentBehaviorType::ResourceFocused,
                risk_tolerance: RiskTolerance::Medium,
                exploration_thoroughness: 0.8,
                combat_aggression: 0.3,
                ..Default::default()
            },
            AgentBehaviorType::Protective => AgentBehavior {
                behavior_type: AgentBehaviorType::Protective,
                risk_tolerance: RiskTolerance::Medium,
                exploration_thoroughness: 0.5,
                combat_aggression: 0.7,
                cooperation_style: CooperationStyle::Supportive,
                ..Default::default()
            },
        };

        // Set default target priorities based on behavior type
        behavior.set_default_priorities();
        behavior
    }

    /// Set default target priorities based on behavior type
    fn set_default_priorities(&mut self) {
        self.priority_targets.clear();
        
        match self.behavior_type {
            AgentBehaviorType::Aggressive => {
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Enemy("Any".to_string()), 
                    priority: 10 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Item("Weapon".to_string()), 
                    priority: 5 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Exit, 
                    priority: 1 
                });
            },
            AgentBehaviorType::Cautious => {
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Item("HealthPotion".to_string()), 
                    priority: 10 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Exit, 
                    priority: 5 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Enemy("Any".to_string()), 
                    priority: 1 
                });
            },
            AgentBehaviorType::ResourceFocused => {
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Resource(GuildResource::Gold), 
                    priority: 10 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Resource(GuildResource::RareArtifacts), 
                    priority: 8 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Resource(GuildResource::Supplies), 
                    priority: 6 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Enemy("Any".to_string()), 
                    priority: 3 
                });
            },
            AgentBehaviorType::Protective => {
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::GuildMember, 
                    priority: 10 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Player, 
                    priority: 9 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Enemy("Any".to_string()), 
                    priority: 7 
                });
            },
            _ => {
                // Balanced default priorities
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Enemy("Any".to_string()), 
                    priority: 5 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Item("Any".to_string()), 
                    priority: 5 
                });
                self.priority_targets.push(TargetPriority { 
                    target_type: TargetType::Exit, 
                    priority: 5 
                });
            }
        }
    }

    /// Add a target priority
    pub fn add_target_priority(&mut self, target_type: TargetType, priority: u32) {
        // Remove existing priority for this target type if it exists
        self.priority_targets.retain(|p| p.target_type != target_type);
        
        // Add new priority
        self.priority_targets.push(TargetPriority {
            target_type,
            priority,
        });
        
        // Sort by priority (higher first)
        self.priority_targets.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Set item preference
    pub fn set_item_preference(&mut self, item_type: String, priority: u32) {
        self.item_preferences.insert(item_type, priority);
    }

    /// Add special ability
    pub fn add_special_ability(&mut self, ability: String) {
        if !self.special_abilities.contains(&ability) {
            self.special_abilities.push(ability);
        }
    }

    /// Set mission focus
    pub fn set_mission_focus(&mut self, mission: Option<String>) {
        self.mission_focus = mission;
    }

    /// Set resource focus
    pub fn set_resource_focus(&mut self, resource: GuildResource, focus_level: f32) {
        self.resource_focus.insert(resource, focus_level.clamp(0.0, 1.0));
    }

    /// Get AI personality based on agent behavior
    pub fn get_ai_personality(&self) -> AIPersonality {
        match self.behavior_type {
            AgentBehaviorType::Aggressive => AIPersonality {
                aggression: 0.9,
                courage: 0.8,
                intelligence: 0.6,
                curiosity: 0.4,
                loyalty: 0.5,
                alertness: 0.7,
            },
            AgentBehaviorType::Cautious => AIPersonality {
                aggression: 0.2,
                courage: 0.3,
                intelligence: 0.8,
                curiosity: 0.5,
                loyalty: 0.6,
                alertness: 0.9,
            },
            AgentBehaviorType::Balanced => AIPersonality {
                aggression: 0.5,
                courage: 0.5,
                intelligence: 0.6,
                curiosity: 0.6,
                loyalty: 0.6,
                alertness: 0.6,
            },
            AgentBehaviorType::Thorough => AIPersonality {
                aggression: 0.4,
                courage: 0.5,
                intelligence: 0.8,
                curiosity: 0.9,
                loyalty: 0.6,
                alertness: 0.8,
            },
            AgentBehaviorType::Speedy => AIPersonality {
                aggression: 0.6,
                courage: 0.7,
                intelligence: 0.5,
                curiosity: 0.3,
                loyalty: 0.4,
                alertness: 0.6,
            },
            AgentBehaviorType::ResourceFocused => AIPersonality {
                aggression: 0.3,
                courage: 0.4,
                intelligence: 0.7,
                curiosity: 0.8,
                loyalty: 0.5,
                alertness: 0.7,
            },
            AgentBehaviorType::Protective => AIPersonality {
                aggression: 0.7,
                courage: 0.8,
                intelligence: 0.6,
                curiosity: 0.4,
                loyalty: 0.9,
                alertness: 0.8,
            },
        }
    }
}

/// Agent status component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    pub current_task: Option<String>,
    pub task_progress: f32,
    pub health_percentage: f32,
    pub resource_levels: HashMap<String, f32>,
    pub discovered_areas: Vec<String>,
    pub enemies_defeated: u32,
    pub items_collected: u32,
    pub distance_traveled: f32,
    pub time_active: f32,
    pub status_effects: Vec<String>,
    pub current_location: Option<String>,
    pub is_in_danger: bool,
    pub is_returning: bool,
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus {
            current_task: None,
            task_progress: 0.0,
            health_percentage: 1.0,
            resource_levels: HashMap::new(),
            discovered_areas: Vec::new(),
            enemies_defeated: 0,
            items_collected: 0,
            distance_traveled: 0.0,
            time_active: 0.0,
            status_effects: Vec::new(),
            current_location: None,
            is_in_danger: false,
            is_returning: false,
        }
    }
}

/// System for integrating agent behavior with AI components
pub fn agent_behavior_ai_integration_system(
    mut commands: Commands,
    mut agent_query: Query<(Entity, &AgentBehavior, &mut AgentStatus), Without<AIComponent>>,
    health_query: Query<&Health>,
) {
    for (entity, agent_behavior, mut agent_status) in agent_query.iter_mut() {
        // Create AI component based on agent behavior
        let ai_personality = agent_behavior.get_ai_personality();
        let ai_component = AIComponent::new(ai_personality);
        
        // Update agent status
        if let Ok(health) = health_query.get(entity) {
            agent_status.health_percentage = health.current as f32 / health.max as f32;
        }
        
        // Add AI component to entity
        commands.entity(entity).insert(ai_component);
    }
}

/// System for updating agent behavior based on mission and status
pub fn agent_behavior_update_system(
    time: Res<Time>,
    mut agent_query: Query<(&mut AgentBehavior, &mut AgentStatus, &GuildMember)>,
) {
    let delta_time = time.delta_seconds();
    
    for (mut agent_behavior, mut agent_status, guild_member) in agent_query.iter_mut() {
        // Update time active
        agent_status.time_active += delta_time;
        
        // Adjust behavior based on health
        if agent_status.health_percentage < 0.3 {
            // When health is low, become more cautious
            agent_behavior.risk_tolerance = RiskTolerance::Low;
            agent_behavior.combat_aggression = 0.2;
            
            // Prioritize health items
            agent_behavior.add_target_priority(
                TargetType::Item("HealthPotion".to_string()), 
                10
            );
            
            // Consider returning to safety
            if agent_status.health_percentage < 0.2 {
                agent_status.is_returning = true;
            }
        } else if agent_status.health_percentage > 0.7 {
            // Reset to default behavior when health is good
            match agent_behavior.behavior_type {
                AgentBehaviorType::Aggressive => agent_behavior.risk_tolerance = RiskTolerance::High,
                AgentBehaviorType::Cautious => agent_behavior.risk_tolerance = RiskTolerance::Low,
                _ => agent_behavior.risk_tolerance = RiskTolerance::Medium,
            }
            
            // Reset returning status if health recovered
            if agent_status.is_returning && agent_status.health_percentage > 0.8 {
                agent_status.is_returning = false;
            }
        }
        
        // Adjust behavior based on mission focus
        if let Some(mission_focus) = &agent_behavior.mission_focus {
            if mission_focus.contains("explore") {
                agent_behavior.exploration_thoroughness = 0.9;
            } else if mission_focus.contains("combat") {
                agent_behavior.combat_aggression = 0.8;
            } else if mission_focus.contains("resource") {
                // Focus on resources
                for resource in [GuildResource::Gold, GuildResource::Supplies, GuildResource::RareArtifacts] {
                    agent_behavior.set_resource_focus(resource, 0.8);
                }
            }
        }
        
        // Adjust behavior based on specialization
        if guild_member.specialization.contains("Fighter") {
            agent_behavior.combat_aggression = (agent_behavior.combat_aggression + 0.8) / 2.0;
        } else if guild_member.specialization.contains("Scout") {
            agent_behavior.exploration_thoroughness = (agent_behavior.exploration_thoroughness + 0.9) / 2.0;
        } else if guild_member.specialization.contains("Collector") {
            // Increase focus on items and resources
            for resource in [GuildResource::Gold, GuildResource::Supplies, GuildResource::RareArtifacts] {
                let current = agent_behavior.resource_focus.get(&resource).copied().unwrap_or(0.5);
                agent_behavior.set_resource_focus(resource, (current + 0.9) / 2.0);
            }
        }
    }
}

/// Plugin for agent behavior systems
pub struct AgentBehaviorPlugin;

impl Plugin for AgentBehaviorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            agent_behavior_ai_integration_system,
            agent_behavior_update_system,
        ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_behavior_creation() {
        let aggressive = AgentBehavior::new(AgentBehaviorType::Aggressive);
        assert_eq!(aggressive.behavior_type, AgentBehaviorType::Aggressive);
        assert_eq!(aggressive.risk_tolerance, RiskTolerance::High);
        assert!(aggressive.combat_aggression > 0.8);
        
        let cautious = AgentBehavior::new(AgentBehaviorType::Cautious);
        assert_eq!(cautious.behavior_type, AgentBehaviorType::Cautious);
        assert_eq!(cautious.risk_tolerance, RiskTolerance::Low);
        assert!(cautious.combat_aggression < 0.3);
    }

    #[test]
    fn test_target_priorities() {
        let mut behavior = AgentBehavior::default();
        assert!(!behavior.priority_targets.is_empty()); // Default priorities should be set
        
        // Add new priority
        behavior.add_target_priority(TargetType::Item("RareItem".to_string()), 20);
        
        // Check if it's the highest priority now
        assert_eq!(behavior.priority_targets[0].priority, 20);
        match &behavior.priority_targets[0].target_type {
            TargetType::Item(name) => assert_eq!(name, "RareItem"),
            _ => panic!("Wrong target type"),
        }
    }

    #[test]
    fn test_ai_personality_mapping() {
        let aggressive = AgentBehavior::new(AgentBehaviorType::Aggressive);
        let personality = aggressive.get_ai_personality();
        assert!(personality.aggression > 0.8);
        assert!(personality.courage > 0.7);
        
        let cautious = AgentBehavior::new(AgentBehaviorType::Cautious);
        let personality = cautious.get_ai_personality();
        assert!(personality.aggression < 0.3);
        assert!(personality.alertness > 0.8);
    }
}