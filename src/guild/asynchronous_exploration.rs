use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, VecDeque};
use rand::{Rng, thread_rng};
use crate::guild::guild_core::{Guild, GuildMember, GuildResource};
use crate::guild::mission::{Mission, MissionTracker};
use crate::guild::agent_progression::AgentStats;

/// Asynchronous exploration state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AsyncExplorationState {
    Inactive,
    Active,
    Paused,
    Completed,
}

impl Default for AsyncExplorationState {
    fn default() -> Self {
        AsyncExplorationState::Inactive
    }
}

/// Time-based simulation manager
#[derive(Resource)]
pub struct AsyncExplorationManager {
    pub state: AsyncExplorationState,
    pub simulation_speed: f64,
    pub last_update_time: f64,
    pub accumulated_time: f64,
    pub active_expeditions: HashMap<String, AsyncExpedition>,
    pub completed_expeditions: Vec<AsyncExpedition>,
    pub event_queue: VecDeque<AsyncEvent>,
    pub auto_assign_missions: bool,
    pub max_concurrent_expeditions: usize,
    pub offline_progress_enabled: bool,
    pub last_offline_time: Option<f64>,
}

impl Default for AsyncExplorationManager {
    fn default() -> Self {
        AsyncExplorationManager {
            state: AsyncExplorationState::default(),
            simulation_speed: 1.0,
            last_update_time: 0.0,
            accumulated_time: 0.0,
            active_expeditions: HashMap::new(),
            completed_expeditions: Vec::new(),
            event_queue: VecDeque::new(),
            auto_assign_missions: true,
            max_concurrent_expeditions: 5,
            offline_progress_enabled: true,
            last_offline_time: None,
        }
    }
}

/// Asynchronous expedition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncExpedition {
    pub id: String,
    pub mission_id: String,
    pub assigned_agents: Vec<Entity>,
    pub start_time: f64,
    pub estimated_duration: f64,
    pub actual_duration: Option<f64>,
    pub progress: f32,
    pub state: ExpeditionState,
    pub events: Vec<ExpeditionEvent>,
    pub rewards: Vec<ExpeditionReward>,
    pub casualties: Vec<Entity>,
    pub success_chance: f32,
    pub difficulty_modifier: f32,
}

/// Expedition state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpeditionState {
    Preparing,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Expedition event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpeditionEvent {
    pub event_type: ExpeditionEventType,
    pub timestamp: f64,
    pub description: String,
    pub participants: Vec<Entity>,
    pub outcome: EventOutcome,
}

/// Types of expedition events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpeditionEventType {
    Combat,
    Discovery,
    Trap,
    Puzzle,
    Rest,
    ResourceGain,
    ResourceLoss,
    InjuryRecovery,
    SkillGain,
    EquipmentFound,
    EquipmentLost,
    TeamworkBonus,
    LeadershipMoment,
    Crisis,
    Breakthrough,
}

/// Event outcome
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventOutcome {
    Success,
    Failure,
    PartialSuccess,
    CriticalSuccess,
    CriticalFailure,
}

/// Expedition reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpeditionReward {
    pub reward_type: ExpeditionRewardType,
    pub amount: u32,
    pub recipient: Option<Entity>,
}

/// Types of expedition rewards
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpeditionRewardType {
    Experience,
    Gold,
    Items,
    Reputation,
    SkillPoints,
    GuildResource(GuildResource),
}

/// Asynchronous event
#[derive(Debug, Clone)]
pub struct AsyncEvent {
    pub event_type: AsyncEventType,
    pub timestamp: f64,
    pub expedition_id: Option<String>,
    pub data: HashMap<String, String>,
}

/// Types of asynchronous events
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsyncEventType {
    ExpeditionStarted,
    ExpeditionCompleted,
    ExpeditionFailed,
    AgentInjured,
    AgentLevelUp,
    ResourceDiscovered,
    EmergencyReturn,
    MissionObjectiveCompleted,
    UnexpectedEncounter,
    EquipmentUpgrade,
}

impl AsyncExplorationManager {
    /// Create a new async exploration manager
    pub fn new() -> Self {
        AsyncExplorationManager::default()
    }
    
    /// Start asynchronous exploration
    pub fn start(&mut self, current_time: f64) {
        self.state = AsyncExplorationState::Active;
        self.last_update_time = current_time;
    }
    
    /// Pause asynchronous exploration
    pub fn pause(&mut self) {
        self.state = AsyncExplorationState::Paused;
    }
    
    /// Resume asynchronous exploration
    pub fn resume(&mut self, current_time: f64) {
        self.state = AsyncExplorationState::Active;
        self.last_update_time = current_time;
    }
    
    /// Stop asynchronous exploration
    pub fn stop(&mut self) {
        self.state = AsyncExplorationState::Inactive;
        self.active_expeditions.clear();
        self.event_queue.clear();
    }
}    
/// Create a new expedition
    pub fn create_expedition(&mut self, mission: &Mission, agents: Vec<Entity>, current_time: f64) -> Result<String, String> {
        if self.active_expeditions.len() >= self.max_concurrent_expeditions {
            return Err("Maximum concurrent expeditions reached".to_string());
        }
        
        let expedition_id = format!("expedition_{}_{}", mission.id, current_time as u64);
        
        // Calculate estimated duration based on mission difficulty and agent capabilities
        let base_duration = match mission.difficulty {
            crate::guild::mission_types::MissionDifficulty::Trivial => 300.0,  // 5 minutes
            crate::guild::mission_types::MissionDifficulty::Easy => 600.0,     // 10 minutes
            crate::guild::mission_types::MissionDifficulty::Medium => 1200.0,  // 20 minutes
            crate::guild::mission_types::MissionDifficulty::Hard => 2400.0,    // 40 minutes
            crate::guild::mission_types::MissionDifficulty::VeryHard => 3600.0, // 1 hour
            crate::guild::mission_types::MissionDifficulty::Extreme => 7200.0,  // 2 hours
        };
        
        // Adjust duration based on number of agents (more agents = faster completion)
        let agent_modifier = 1.0 - (agents.len() as f64 - 1.0) * 0.1;
        let estimated_duration = base_duration * agent_modifier.max(0.5);
        
        // Calculate success chance based on agent levels vs mission difficulty
        let success_chance = 0.7; // Base success chance, would be calculated from agent stats
        
        let expedition = AsyncExpedition {
            id: expedition_id.clone(),
            mission_id: mission.id.clone(),
            assigned_agents: agents,
            start_time: current_time,
            estimated_duration,
            actual_duration: None,
            progress: 0.0,
            state: ExpeditionState::Preparing,
            events: Vec::new(),
            rewards: Vec::new(),
            casualties: Vec::new(),
            success_chance,
            difficulty_modifier: mission.difficulty.reward_multiplier(),
        };
        
        self.active_expeditions.insert(expedition_id.clone(), expedition);
        
        // Add start event
        self.event_queue.push_back(AsyncEvent {
            event_type: AsyncEventType::ExpeditionStarted,
            timestamp: current_time,
            expedition_id: Some(expedition_id.clone()),
            data: HashMap::new(),
        });
        
        Ok(expedition_id)
    }
    
    /// Update asynchronous exploration simulation
    pub fn update(&mut self, current_time: f64, delta_time: f64) {
        if self.state != AsyncExplorationState::Active {
            return;
        }
        
        self.accumulated_time += delta_time * self.simulation_speed;
        
        // Update active expeditions
        let mut completed_expeditions = Vec::new();
        
        for (expedition_id, expedition) in &mut self.active_expeditions {
            let elapsed_time = current_time - expedition.start_time;
            expedition.progress = (elapsed_time / expedition.estimated_duration).min(1.0) as f32;
            
            // Change state from preparing to in progress
            if expedition.state == ExpeditionState::Preparing && elapsed_time > 10.0 {
                expedition.state = ExpeditionState::InProgress;
            }
            
            // Generate random events during expedition
            if expedition.state == ExpeditionState::InProgress {
                self.generate_expedition_events(expedition, current_time);
            }
            
            // Check if expedition is completed
            if expedition.progress >= 1.0 && expedition.state == ExpeditionState::InProgress {
                let success = thread_rng().gen::<f32>() < expedition.success_chance;
                
                if success {
                    expedition.state = ExpeditionState::Completed;
                    self.generate_expedition_rewards(expedition);
                    
                    self.event_queue.push_back(AsyncEvent {
                        event_type: AsyncEventType::ExpeditionCompleted,
                        timestamp: current_time,
                        expedition_id: Some(expedition_id.clone()),
                        data: HashMap::new(),
                    });
                } else {
                    expedition.state = ExpeditionState::Failed;
                    
                    self.event_queue.push_back(AsyncEvent {
                        event_type: AsyncEventType::ExpeditionFailed,
                        timestamp: current_time,
                        expedition_id: Some(expedition_id.clone()),
                        data: HashMap::new(),
                    });
                }
                
                expedition.actual_duration = Some(elapsed_time);
                completed_expeditions.push(expedition_id.clone());
            }
        }
        
        // Move completed expeditions
        for expedition_id in completed_expeditions {
            if let Some(expedition) = self.active_expeditions.remove(&expedition_id) {
                self.completed_expeditions.push(expedition);
            }
        }
        
        self.last_update_time = current_time;
    }
    
    /// Generate random events during expedition
    fn generate_expedition_events(&mut self, expedition: &mut AsyncExpedition, current_time: f64) {
        let mut rng = thread_rng();
        
        // Only generate events occasionally
        if rng.gen::<f32>() > 0.1 {
            return;
        }
        
        let event_types = [
            ExpeditionEventType::Combat,
            ExpeditionEventType::Discovery,
            ExpeditionEventType::Trap,
            ExpeditionEventType::Puzzle,
            ExpeditionEventType::Rest,
            ExpeditionEventType::ResourceGain,
        ];
        
        let event_type = event_types[rng.gen_range(0..event_types.len())].clone();
        let outcome = match rng.gen_range(0..100) {
            0..=10 => EventOutcome::CriticalFailure,
            11..=25 => EventOutcome::Failure,
            26..=40 => EventOutcome::PartialSuccess,
            41..=85 => EventOutcome::Success,
            _ => EventOutcome::CriticalSuccess,
        };
        
        let description = self.generate_event_description(&event_type, &outcome);
        
        let event = ExpeditionEvent {
            event_type,
            timestamp: current_time,
            description,
            participants: expedition.assigned_agents.clone(),
            outcome,
        };
        
        expedition.events.push(event);
    }
    
    /// Generate description for expedition event
    fn generate_event_description(&self, event_type: &ExpeditionEventType, outcome: &EventOutcome) -> String {
        match (event_type, outcome) {
            (ExpeditionEventType::Combat, EventOutcome::Success) => "The party successfully defeated a group of enemies.".to_string(),
            (ExpeditionEventType::Combat, EventOutcome::Failure) => "The party was forced to retreat from combat.".to_string(),
            (ExpeditionEventType::Discovery, EventOutcome::Success) => "The party discovered a hidden treasure cache.".to_string(),
            (ExpeditionEventType::Trap, EventOutcome::Failure) => "A party member triggered a trap and was injured.".to_string(),
            (ExpeditionEventType::Puzzle, EventOutcome::Success) => "The party solved an ancient puzzle mechanism.".to_string(),
            (ExpeditionEventType::Rest, EventOutcome::Success) => "The party found a safe place to rest and recover.".to_string(),
            (ExpeditionEventType::ResourceGain, EventOutcome::Success) => "The party found valuable resources.".to_string(),
            _ => "Something happened during the expedition.".to_string(),
        }
    }
    
    /// Generate rewards for completed expedition
    fn generate_expedition_rewards(&mut self, expedition: &mut AsyncExpedition) {
        let mut rng = thread_rng();
        
        // Base rewards
        let base_exp = (100.0 * expedition.difficulty_modifier) as u32;
        let base_gold = (50.0 * expedition.difficulty_modifier) as u32;
        
        // Experience for all participants
        for &agent in &expedition.assigned_agents {
            expedition.rewards.push(ExpeditionReward {
                reward_type: ExpeditionRewardType::Experience,
                amount: base_exp + rng.gen_range(0..20),
                recipient: Some(agent),
            });
        }
        
        // Gold reward
        expedition.rewards.push(ExpeditionReward {
            reward_type: ExpeditionRewardType::Gold,
            amount: base_gold + rng.gen_range(0..50),
            recipient: None,
        });
        
        // Random additional rewards
        if rng.gen::<f32>() < 0.3 {
            expedition.rewards.push(ExpeditionReward {
                reward_type: ExpeditionRewardType::GuildResource(GuildResource::Reputation),
                amount: (10.0 * expedition.difficulty_modifier) as u32,
                recipient: None,
            });
        }
        
        if rng.gen::<f32>() < 0.2 {
            expedition.rewards.push(ExpeditionReward {
                reward_type: ExpeditionRewardType::Items,
                amount: 1,
                recipient: Some(expedition.assigned_agents[rng.gen_range(0..expedition.assigned_agents.len())]),
            });
        }
    }
    
    /// Calculate offline progress
    pub fn calculate_offline_progress(&mut self, offline_duration: f64, current_time: f64) -> Vec<AsyncEvent> {
        if !self.offline_progress_enabled {
            return Vec::new();
        }
        
        let mut offline_events = Vec::new();
        
        // Simulate time passage for active expeditions
        let simulation_time = offline_duration * self.simulation_speed;
        let mut simulation_current_time = current_time - offline_duration;
        
        while simulation_current_time < current_time {
            let step_size = 60.0; // 1 minute steps
            self.update(simulation_current_time, step_size);
            simulation_current_time += step_size;
            
            // Collect events generated during offline simulation
            while let Some(event) = self.event_queue.pop_front() {
                offline_events.push(event);
            }
        }
        
        self.last_offline_time = Some(current_time);
        offline_events
    }
    
    /// Get expedition by ID
    pub fn get_expedition(&self, expedition_id: &str) -> Option<&AsyncExpedition> {
        self.active_expeditions.get(expedition_id)
    }
    
    /// Get all active expeditions
    pub fn get_active_expeditions(&self) -> Vec<&AsyncExpedition> {
        self.active_expeditions.values().collect()
    }
    
    /// Get completed expeditions
    pub fn get_completed_expeditions(&self) -> &Vec<AsyncExpedition> {
        &self.completed_expeditions
    }
    
    /// Get pending events
    pub fn get_pending_events(&self) -> &VecDeque<AsyncEvent> {
        &self.event_queue
    }
    
    /// Pop next event
    pub fn pop_event(&mut self) -> Option<AsyncEvent> {
        self.event_queue.pop_front()
    }
    
    /// Cancel expedition
    pub fn cancel_expedition(&mut self, expedition_id: &str) -> bool {
        if let Some(mut expedition) = self.active_expeditions.remove(expedition_id) {
            expedition.state = ExpeditionState::Cancelled;
            self.completed_expeditions.push(expedition);
            true
        } else {
            false
        }
    }

impl AsyncExpedition {
    /// Get expedition status summary
    pub fn get_status_summary(&self) -> String {
        match self.state {
            ExpeditionState::Preparing => "Preparing for departure".to_string(),
            ExpeditionState::InProgress => format!("In progress ({:.0}% complete)", self.progress * 100.0),
            ExpeditionState::Completed => "Successfully completed".to_string(),
            ExpeditionState::Failed => "Failed".to_string(),
            ExpeditionState::Cancelled => "Cancelled".to_string(),
        }
    }
    
    /// Get estimated time remaining
    pub fn get_time_remaining(&self, current_time: f64) -> f64 {
        if self.state != ExpeditionState::InProgress {
            return 0.0;
        }
        
        let elapsed = current_time - self.start_time;
        (self.estimated_duration - elapsed).max(0.0)
    }
    
    /// Get total rewards value
    pub fn get_total_reward_value(&self) -> u32 {
        self.rewards.iter().map(|r| r.amount).sum()
    }
    
    /// Get event count by type
    pub fn get_event_count(&self, event_type: &ExpeditionEventType) -> usize {
        self.events.iter().filter(|e| &e.event_type == event_type).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guild::mission_types::MissionDifficulty;

    #[test]
    fn test_async_exploration_manager() {
        let mut manager = AsyncExplorationManager::new();
        assert_eq!(manager.state, AsyncExplorationState::Inactive);
        
        manager.start(100.0);
        assert_eq!(manager.state, AsyncExplorationState::Active);
        
        manager.pause();
        assert_eq!(manager.state, AsyncExplorationState::Paused);
        
        manager.resume(200.0);
        assert_eq!(manager.state, AsyncExplorationState::Active);
    }

    #[test]
    fn test_expedition_creation() {
        let mut manager = AsyncExplorationManager::new();
        
        // Create a mock mission
        let mut mission = crate::guild::mission::Mission::default();
        mission.id = "test_mission".to_string();
        mission.difficulty = MissionDifficulty::Medium;
        
        let agents = vec![Entity::from_raw(1), Entity::from_raw(2)];
        
        let result = manager.create_expedition(&mission, agents, 100.0);
        assert!(result.is_ok());
        
        let expedition_id = result.unwrap();
        assert!(manager.active_expeditions.contains_key(&expedition_id));
        
        let expedition = manager.get_expedition(&expedition_id).unwrap();
        assert_eq!(expedition.mission_id, "test_mission");
        assert_eq!(expedition.assigned_agents.len(), 2);
        assert_eq!(expedition.state, ExpeditionState::Preparing);
    }

    #[test]
    fn test_expedition_progress() {
        let mut manager = AsyncExplorationManager::new();
        manager.start(100.0);
        
        // Create a mock mission
        let mut mission = crate::guild::mission::Mission::default();
        mission.id = "test_mission".to_string();
        mission.difficulty = MissionDifficulty::Trivial; // Short duration for testing
        
        let agents = vec![Entity::from_raw(1)];
        let expedition_id = manager.create_expedition(&mission, agents, 100.0).unwrap();
        
        // Update simulation
        manager.update(200.0, 100.0); // Simulate 100 seconds
        
        let expedition = manager.get_expedition(&expedition_id).unwrap();
        assert!(expedition.progress > 0.0);
        
        // Update until completion
        manager.update(500.0, 300.0); // Simulate more time
        
        // Expedition should be completed and moved to completed list
        assert!(!manager.active_expeditions.contains_key(&expedition_id));
        assert!(!manager.completed_expeditions.is_empty());
    }
}