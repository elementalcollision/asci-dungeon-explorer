use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::guild::mission_types::*;

/// Mission component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Mission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub difficulty: MissionDifficulty,
    pub objectives: Vec<MissionObjective>,
    pub rewards: Vec<MissionReward>,
    pub status: MissionStatus,
    pub assigned_agents: HashSet<Entity>,
    pub location: Option<String>,
    pub time_limit: Option<f64>,
    pub expiration_time: Option<f64>,
    pub creation_time: f64,
    pub completion_time: Option<f64>,
    pub guild_id: String,
    pub tags: HashSet<String>,
    pub required_level: u32,
}

impl Default for Mission {
    fn default() -> Self {
        Mission {
            id: "".to_string(),
            name: "".to_string(),
            description: "".to_string(),
            difficulty: MissionDifficulty::Medium,
            objectives: Vec::new(),
            rewards: Vec::new(),
            status: MissionStatus::Available,
            assigned_agents: HashSet::new(),
            location: None,
            time_limit: None,
            expiration_time: None,
            creation_time: 0.0,
            completion_time: None,
            guild_id: "".to_string(),
            tags: HashSet::new(),
            required_level: 1,
        }
    }
}

impl Mission {
    /// Create a new mission
    pub fn new(id: String, name: String, description: String, difficulty: MissionDifficulty, guild_id: String, creation_time: f64) -> Self {
        let mut mission = Mission {
            id,
            name,
            description,
            difficulty,
            guild_id,
            creation_time,
            required_level: difficulty.recommended_level(),
            ..Default::default()
        };
        
        // Set expiration time (missions expire after 7 days)
        mission.expiration_time = Some(creation_time + 7.0 * 24.0 * 60.0 * 60.0);
        
        mission
    }
    
    /// Add an objective to the mission
    pub fn add_objective(&mut self, objective: MissionObjective) {
        self.objectives.push(objective);
    }
    
    /// Add a reward to the mission
    pub fn add_reward(&mut self, reward: MissionReward) {
        self.rewards.push(reward);
    }
    
    /// Assign an agent to the mission
    pub fn assign_agent(&mut self, agent: Entity) -> bool {
        if self.status == MissionStatus::Available {
            self.status = MissionStatus::Assigned;
        }
        self.assigned_agents.insert(agent)
    }
    
    /// Remove an agent from the mission
    pub fn remove_agent(&mut self, agent: &Entity) -> bool {
        let result = self.assigned_agents.remove(agent);
        
        // If no agents are assigned, set status back to available
        if self.assigned_agents.is_empty() && self.status == MissionStatus::Assigned {
            self.status = MissionStatus::Available;
        }
        
        result
    }
    
    /// Start the mission
    pub fn start(&mut self, current_time: f64) -> bool {
        if self.status != MissionStatus::Assigned || self.assigned_agents.is_empty() {
            return false;
        }
        
        self.status = MissionStatus::InProgress;
        
        // Set time limit if specified
        if let Some(limit) = self.time_limit {
            self.expiration_time = Some(current_time + limit);
        }
        
        true
    }
    
    /// Complete the mission
    pub fn complete(&mut self, current_time: f64) -> bool {
        if self.status != MissionStatus::InProgress {
            return false;
        }
        
        // Check if all objectives are completed
        if !self.objectives.iter().all(|obj| obj.is_completed()) {
            return false;
        }
        
        self.status = MissionStatus::Completed;
        self.completion_time = Some(current_time);
        
        true
    }
    
    /// Fail the mission
    pub fn fail(&mut self) {
        if self.status == MissionStatus::InProgress || self.status == MissionStatus::Assigned {
            self.status = MissionStatus::Failed;
        }
    }
    
    /// Check if mission is expired
    pub fn check_expiration(&mut self, current_time: f64) -> bool {
        if let Some(expiration) = self.expiration_time {
            if current_time > expiration && 
               (self.status == MissionStatus::Available || 
                self.status == MissionStatus::Assigned || 
                self.status == MissionStatus::InProgress) {
                self.status = MissionStatus::Expired;
                return true;
            }
        }
        false
    }
    
    /// Get mission progress percentage
    pub fn progress_percentage(&self) -> f32 {
        if self.objectives.is_empty() {
            return 0.0;
        }
        
        let total_progress: f32 = self.objectives.iter()
            .map(|obj| obj.progress_percentage())
            .sum();
        
        total_progress / self.objectives.len() as f32
    }
    
    /// Check if all objectives are completed
    pub fn all_objectives_completed(&self) -> bool {
        !self.objectives.is_empty() && self.objectives.iter().all(|obj| obj.is_completed())
    }
    
    /// Add a tag to the mission
    pub fn add_tag(&mut self, tag: &str) {
        self.tags.insert(tag.to_string());
    }
    
    /// Check if mission has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

/// Mission tracker component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct MissionTracker {
    pub active_mission: Option<String>,
    pub completed_missions: Vec<String>,
    pub failed_missions: Vec<String>,
    pub mission_progress: HashMap<String, Vec<MissionObjectiveStatus>>,
    pub mission_history: Vec<(String, MissionStatus, f64)>, // mission_id, status, timestamp
}

impl Default for MissionTracker {
    fn default() -> Self {
        MissionTracker {
            active_mission: None,
            completed_missions: Vec::new(),
            failed_missions: Vec::new(),
            mission_progress: HashMap::new(),
            mission_history: Vec::new(),
        }
    }
}

impl MissionTracker {
    /// Start a mission
    pub fn start_mission(&mut self, mission_id: &str, current_time: f64) -> bool {
        if self.active_mission.is_some() {
            return false;
        }
        
        self.active_mission = Some(mission_id.to_string());
        self.mission_history.push((mission_id.to_string(), MissionStatus::InProgress, current_time));
        true
    }
    
    /// Complete the active mission
    pub fn complete_mission(&mut self, current_time: f64) -> Option<String> {
        if let Some(mission_id) = self.active_mission.take() {
            self.completed_missions.push(mission_id.clone());
            self.mission_history.push((mission_id.clone(), MissionStatus::Completed, current_time));
            return Some(mission_id);
        }
        None
    }
    
    /// Fail the active mission
    pub fn fail_mission(&mut self, current_time: f64) -> Option<String> {
        if let Some(mission_id) = self.active_mission.take() {
            self.failed_missions.push(mission_id.clone());
            self.mission_history.push((mission_id.clone(), MissionStatus::Failed, current_time));
            return Some(mission_id);
        }
        None
    }
    
    /// Update mission progress
    pub fn update_mission_progress(&mut self, mission_id: &str, objective_index: usize, status: MissionObjectiveStatus) {
        let progress = self.mission_progress.entry(mission_id.to_string())
            .or_insert_with(Vec::new);
        
        if objective_index >= progress.len() {
            progress.resize(objective_index + 1, MissionObjectiveStatus::NotStarted);
        }
        
        progress[objective_index] = status;
    }
    
    /// Check if a mission is completed
    pub fn is_mission_completed(&self, mission_id: &str) -> bool {
        self.completed_missions.contains(&mission_id.to_string())
    }
    
    /// Check if a mission is failed
    pub fn is_mission_failed(&self, mission_id: &str) -> bool {
        self.failed_missions.contains(&mission_id.to_string())
    }
    
    /// Get mission completion count
    pub fn get_completion_count(&self) -> usize {
        self.completed_missions.len()
    }
    
    /// Get mission failure count
    pub fn get_failure_count(&self) -> usize {
        self.failed_missions.len()
    }
    
    /// Get mission success rate
    pub fn get_success_rate(&self) -> f32 {
        let total = self.completed_missions.len() + self.failed_missions.len();
        if total == 0 {
            return 0.0;
        }
        self.completed_missions.len() as f32 / total as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_creation() {
        let mission = Mission::new(
            "test_mission".to_string(),
            "Test Mission".to_string(),
            "A test mission".to_string(),
            MissionDifficulty::Medium,
            "test_guild".to_string(),
            100.0,
        );
        
        assert_eq!(mission.id, "test_mission");
        assert_eq!(mission.name, "Test Mission");
        assert_eq!(mission.difficulty, MissionDifficulty::Medium);
        assert_eq!(mission.status, MissionStatus::Available);
        assert!(mission.expiration_time.is_some());
    }

    #[test]
    fn test_mission_lifecycle() {
        let mut mission = Mission::new(
            "test_mission".to_string(),
            "Test Mission".to_string(),
            "A test mission".to_string(),
            MissionDifficulty::Medium,
            "test_guild".to_string(),
            100.0,
        );
        
        // Add objective
        let objective_type = MissionObjectiveType::KillEnemies {
            enemy_type: "Goblins".to_string(),
            count: 5,
        };
        let objective = MissionObjective::new(objective_type);
        mission.add_objective(objective);
        
        // Assign agent
        let agent = Entity::from_raw(1);
        assert!(mission.assign_agent(agent));
        assert_eq!(mission.status, MissionStatus::Assigned);
        
        // Start mission
        assert!(mission.start(200.0));
        assert_eq!(mission.status, MissionStatus::InProgress);
        
        // Try to complete (should fail because objective not completed)
        assert!(!mission.complete(300.0));
        assert_eq!(mission.status, MissionStatus::InProgress);
        
        // Complete objective
        mission.objectives[0].complete();
        
        // Complete mission
        assert!(mission.complete(300.0));
        assert_eq!(mission.status, MissionStatus::Completed);
        assert_eq!(mission.completion_time, Some(300.0));
    }

    #[test]
    fn test_mission_tracker() {
        let mut tracker = MissionTracker::default();
        
        // Start mission
        assert!(tracker.start_mission("test_mission", 100.0));
        assert_eq!(tracker.active_mission, Some("test_mission".to_string()));
        
        // Complete mission
        let completed = tracker.complete_mission(200.0);
        assert_eq!(completed, Some("test_mission".to_string()));
        assert!(tracker.active_mission.is_none());
        assert!(tracker.completed_missions.contains(&"test_mission".to_string()));
        
        // Start another mission
        assert!(tracker.start_mission("test_mission2", 300.0));
        
        // Fail mission
        let failed = tracker.fail_mission(400.0);
        assert_eq!(failed, Some("test_mission2".to_string()));
        assert!(tracker.failed_missions.contains(&"test_mission2".to_string()));
        
        // Check stats
        assert_eq!(tracker.get_completion_count(), 1);
        assert_eq!(tracker.get_failure_count(), 1);
        assert_eq!(tracker.get_success_rate(), 0.5);
    }
}