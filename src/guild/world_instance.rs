use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::map::DungeonMap;
use crate::guild::mission::Mission;

/// World instance type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstanceType {
    MainWorld,
    MissionDungeon,
    SpecialArea,
    GuildHall,
    PersonalQuarters,
    TrainingGround,
    Custom,
}

impl Default for InstanceType {
    fn default() -> Self {
        InstanceType::MainWorld
    }
}

/// World instance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InstanceStatus {
    Initializing,
    Active,
    Paused,
    Completed,
    Failed,
    Archived,
}

impl Default for InstanceStatus {
    fn default() -> Self {
        InstanceStatus::Initializing
    }
}

/// World instance component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct WorldInstance {
    pub id: String,
    pub name: String,
    pub instance_type: InstanceType,
    pub status: InstanceStatus,
    pub owner_guild: Option<String>,
    pub owner_entity: Option<Entity>,
    pub mission_id: Option<String>,
    pub creation_time: f64,
    pub last_active_time: f64,
    pub expiration_time: Option<f64>,
    pub difficulty: u32,
    pub level: u32,
    pub tags: HashSet<String>,
    pub parent_instance: Option<String>,
    pub child_instances: HashSet<String>,
    pub connected_entities: HashSet<Entity>,
    pub seed: u64,
    pub custom_properties: HashMap<String, String>,
}

impl Default for WorldInstance {
    fn default() -> Self {
        WorldInstance {
            id: Uuid::new_v4().to_string(),
            name: "Default Instance".to_string(),
            instance_type: InstanceType::default(),
            status: InstanceStatus::default(),
            owner_guild: None,
            owner_entity: None,
            mission_id: None,
            creation_time: 0.0,
            last_active_time: 0.0,
            expiration_time: None,
            difficulty: 1,
            level: 1,
            tags: HashSet::new(),
            parent_instance: None,
            child_instances: HashSet::new(),
            connected_entities: HashSet::new(),
            seed: rand::random(),
            custom_properties: HashMap::new(),
        }
    }
}

impl WorldInstance {
    /// Create a new world instance
    pub fn new(name: &str, instance_type: InstanceType, current_time: f64) -> Self {
        WorldInstance {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            instance_type,
            status: InstanceStatus::Initializing,
            creation_time: current_time,
            last_active_time: current_time,
            ..Default::default()
        }
    }
    
    /// Create a mission instance
    pub fn new_mission_instance(mission: &Mission, current_time: f64) -> Self {
        let mut instance = WorldInstance::new(
            &format!("Mission: {}", mission.name),
            InstanceType::MissionDungeon,
            current_time,
        );
        
        instance.mission_id = Some(mission.id.clone());
        instance.owner_guild = Some(mission.guild_id.clone());
        instance.difficulty = mission.difficulty.recommended_level();
        instance.level = mission.required_level;
        
        // Add mission tags to instance
        for tag in &mission.tags {
            instance.tags.insert(tag.clone());
        }
        
        instance
    }
    
    /// Activate the instance
    pub fn activate(&mut self, current_time: f64) {
        self.status = InstanceStatus::Active;
        self.last_active_time = current_time;
    }
    
    /// Pause the instance
    pub fn pause(&mut self, current_time: f64) {
        self.status = InstanceStatus::Paused;
        self.last_active_time = current_time;
    }
    
    /// Complete the instance
    pub fn complete(&mut self, current_time: f64) {
        self.status = InstanceStatus::Completed;
        self.last_active_time = current_time;
    }
    
    /// Fail the instance
    pub fn fail(&mut self, current_time: f64) {
        self.status = InstanceStatus::Failed;
        self.last_active_time = current_time;
    }
    
    /// Archive the instance
    pub fn archive(&mut self, current_time: f64) {
        self.status = InstanceStatus::Archived;
        self.last_active_time = current_time;
    }
    
    /// Add a connected entity
    pub fn add_entity(&mut self, entity: Entity) {
        self.connected_entities.insert(entity);
    }
    
    /// Remove a connected entity
    pub fn remove_entity(&mut self, entity: &Entity) {
        self.connected_entities.remove(entity);
    }
    
    /// Add a child instance
    pub fn add_child_instance(&mut self, child_id: &str) {
        self.child_instances.insert(child_id.to_string());
    }
    
    /// Remove a child instance
    pub fn remove_child_instance(&mut self, child_id: &str) {
        self.child_instances.remove(child_id);
    }
    
    /// Set a custom property
    pub fn set_property(&mut self, key: &str, value: &str) {
        self.custom_properties.insert(key.to_string(), value.to_string());
    }
    
    /// Get a custom property
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.custom_properties.get(key)
    }
    
    /// Check if instance is active
    pub fn is_active(&self) -> bool {
        self.status == InstanceStatus::Active
    }
    
    /// Check if instance is completed
    pub fn is_completed(&self) -> bool {
        self.status == InstanceStatus::Completed
    }
    
    /// Check if instance is failed
    pub fn is_failed(&self) -> bool {
        self.status == InstanceStatus::Failed
    }
    
    /// Check if instance is archived
    pub fn is_archived(&self) -> bool {
        self.status == InstanceStatus::Archived
    }
    
    /// Check if instance is expired
    pub fn check_expiration(&mut self, current_time: f64) -> bool {
        if let Some(expiration) = self.expiration_time {
            if current_time > expiration && self.status != InstanceStatus::Archived {
                self.status = InstanceStatus::Archived;
                self.last_active_time = current_time;
                return true;
            }
        }
        false
    }
    
    /// Add a tag to the instance
    pub fn add_tag(&mut self, tag: &str) {
        self.tags.insert(tag.to_string());
    }
    
    /// Check if instance has a specific tag
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }
}

/// Instance transition marker component
#[derive(Component, Debug)]
pub struct InstanceTransition {
    pub target_instance: String,
    pub entry_point: Option<Vec2>,
    pub transition_type: TransitionType,
}

/// Instance transition types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionType {
    Portal,
    Door,
    Teleport,
    Mission,
    Return,
}

/// Instance map component
#[derive(Component, Debug)]
pub struct InstanceMap {
    pub instance_id: String,
    pub map: DungeonMap,
    pub entry_points: HashMap<String, Vec2>,
    pub exit_points: HashMap<String, Vec2>,
    pub special_locations: HashMap<String, Vec2>,
}

/// World instance manager resource
#[derive(Resource, Default)]
pub struct WorldInstanceManager {
    pub instances: HashMap<String, WorldInstance>,
    pub active_instance: Option<String>,
    pub player_instance: Option<String>,
    pub instance_maps: HashMap<String, Entity>,
}

impl WorldInstanceManager {
    /// Create a new world instance manager
    pub fn new() -> Self {
        WorldInstanceManager {
            instances: HashMap::new(),
            active_instance: None,
            player_instance: None,
            instance_maps: HashMap::new(),
        }
    }
    
    /// Add an instance
    pub fn add_instance(&mut self, instance: WorldInstance) {
        let id = instance.id.clone();
        self.instances.insert(id.clone(), instance);
        
        // If this is the first instance, make it active
        if self.active_instance.is_none() {
            self.active_instance = Some(id);
        }
    }
    
    /// Get an instance by ID
    pub fn get_instance(&self, id: &str) -> Option<&WorldInstance> {
        self.instances.get(id)
    }
    
    /// Get a mutable reference to an instance
    pub fn get_instance_mut(&mut self, id: &str) -> Option<&mut WorldInstance> {
        self.instances.get_mut(id)
    }
    
    /// Remove an instance
    pub fn remove_instance(&mut self, id: &str) -> Option<WorldInstance> {
        // If removing active instance, clear active instance
        if self.active_instance.as_deref() == Some(id) {
            self.active_instance = None;
        }
        
        // If removing player instance, clear player instance
        if self.player_instance.as_deref() == Some(id) {
            self.player_instance = None;
        }
        
        // Remove from instance maps
        self.instance_maps.remove(id);
        
        self.instances.remove(id)
    }
    
    /// Set active instance
    pub fn set_active_instance(&mut self, id: &str) -> bool {
        if self.instances.contains_key(id) {
            self.active_instance = Some(id.to_string());
            true
        } else {
            false
        }
    }
    
    /// Set player instance
    pub fn set_player_instance(&mut self, id: &str) -> bool {
        if self.instances.contains_key(id) {
            self.player_instance = Some(id.to_string());
            true
        } else {
            false
        }
    }
    
    /// Get active instance
    pub fn get_active_instance(&self) -> Option<&WorldInstance> {
        self.active_instance.as_ref().and_then(|id| self.instances.get(id))
    }
    
    /// Get player instance
    pub fn get_player_instance(&self) -> Option<&WorldInstance> {
        self.player_instance.as_ref().and_then(|id| self.instances.get(id))
    }
    
    /// Get instances by type
    pub fn get_instances_by_type(&self, instance_type: InstanceType) -> Vec<&WorldInstance> {
        self.instances.values()
            .filter(|instance| instance.instance_type == instance_type)
            .collect()
    }
    
    /// Get instances by status
    pub fn get_instances_by_status(&self, status: InstanceStatus) -> Vec<&WorldInstance> {
        self.instances.values()
            .filter(|instance| instance.status == status)
            .collect()
    }
    
    /// Get instances by tag
    pub fn get_instances_by_tag(&self, tag: &str) -> Vec<&WorldInstance> {
        self.instances.values()
            .filter(|instance| instance.has_tag(tag))
            .collect()
    }
    
    /// Get instances by guild
    pub fn get_instances_by_guild(&self, guild_id: &str) -> Vec<&WorldInstance> {
        self.instances.values()
            .filter(|instance| instance.owner_guild.as_deref() == Some(guild_id))
            .collect()
    }
    
    /// Get instances by mission
    pub fn get_instances_by_mission(&self, mission_id: &str) -> Vec<&WorldInstance> {
        self.instances.values()
            .filter(|instance| instance.mission_id.as_deref() == Some(mission_id))
            .collect()
    }
    
    /// Register an instance map
    pub fn register_instance_map(&mut self, instance_id: &str, map_entity: Entity) {
        self.instance_maps.insert(instance_id.to_string(), map_entity);
    }
    
    /// Get an instance map entity
    pub fn get_instance_map_entity(&self, instance_id: &str) -> Option<Entity> {
        self.instance_maps.get(instance_id).copied()
    }
    
    /// Update instance statuses
    pub fn update_instances(&mut self, current_time: f64) {
        for instance in self.instances.values_mut() {
            instance.check_expiration(current_time);
        }
    }
    
    /// Clean up archived instances
    pub fn clean_up_archived_instances(&mut self, days_to_keep: u32, current_time: f64) {
        let seconds_to_keep = days_to_keep as f64 * 24.0 * 60.0 * 60.0;
        let cutoff_time = current_time - seconds_to_keep;
        
        let archived_instances: Vec<String> = self.instances.iter()
            .filter(|(_, instance)| {
                instance.is_archived() && instance.last_active_time < cutoff_time
            })
            .map(|(id, _)| id.clone())
            .collect();
        
        for id in archived_instances {
            self.remove_instance(&id);
        }
    }
    
    /// Create a new mission instance
    pub fn create_mission_instance(&mut self, mission: &Mission, current_time: f64) -> String {
        let instance = WorldInstance::new_mission_instance(mission, current_time);
        let id = instance.id.clone();
        self.add_instance(instance);
        id
    }
    
    /// Create a child instance
    pub fn create_child_instance(&mut self, parent_id: &str, name: &str, instance_type: InstanceType, current_time: f64) -> Option<String> {
        if !self.instances.contains_key(parent_id) {
            return None;
        }
        
        let mut instance = WorldInstance::new(name, instance_type, current_time);
        instance.parent_instance = Some(parent_id.to_string());
        
        // Copy some properties from parent
        if let Some(parent) = self.instances.get(parent_id) {
            instance.owner_guild = parent.owner_guild.clone();
            instance.difficulty = parent.difficulty;
            instance.level = parent.level;
        }
        
        let id = instance.id.clone();
        self.add_instance(instance);
        
        // Update parent's child instances
        if let Some(parent) = self.instances.get_mut(parent_id) {
            parent.add_child_instance(&id);
        }
        
        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guild::mission_types::MissionDifficulty;

    #[test]
    fn test_world_instance_creation() {
        let instance = WorldInstance::new("Test Instance", InstanceType::MainWorld, 100.0);
        assert_eq!(instance.name, "Test Instance");
        assert_eq!(instance.instance_type, InstanceType::MainWorld);
        assert_eq!(instance.status, InstanceStatus::Initializing);
    }

    #[test]
    fn test_world_instance_lifecycle() {
        let mut instance = WorldInstance::new("Test Instance", InstanceType::MainWorld, 100.0);
        
        // Activate
        instance.activate(200.0);
        assert_eq!(instance.status, InstanceStatus::Active);
        assert_eq!(instance.last_active_time, 200.0);
        
        // Pause
        instance.pause(300.0);
        assert_eq!(instance.status, InstanceStatus::Paused);
        
        // Complete
        instance.complete(400.0);
        assert_eq!(instance.status, InstanceStatus::Completed);
        
        // Archive
        instance.archive(500.0);
        assert_eq!(instance.status, InstanceStatus::Archived);
    }

    #[test]
    fn test_world_instance_manager() {
        let mut manager = WorldInstanceManager::new();
        
        // Create instance
        let instance = WorldInstance::new("Test Instance", InstanceType::MainWorld, 100.0);
        let id = instance.id.clone();
        manager.add_instance(instance);
        
        // Retrieve instance
        let retrieved = manager.get_instance(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Instance");
        
        // Set active instance
        assert!(manager.set_active_instance(&id));
        assert_eq!(manager.active_instance, Some(id.clone()));
        
        // Get active instance
        let active = manager.get_active_instance();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, id);
        
        // Remove instance
        let removed = manager.remove_instance(&id);
        assert!(removed.is_some());
        assert!(manager.active_instance.is_none());
        assert!(manager.get_instance(&id).is_none());
    }

    #[test]
    fn test_mission_instance() {
        // Create a mock mission
        let mut mission = Mission::default();
        mission.id = "test_mission".to_string();
        mission.name = "Test Mission".to_string();
        mission.guild_id = "test_guild".to_string();
        mission.difficulty = MissionDifficulty::Medium;
        mission.required_level = 5;
        mission.add_tag("combat");
        
        // Create mission instance
        let instance = WorldInstance::new_mission_instance(&mission, 100.0);
        
        assert_eq!(instance.instance_type, InstanceType::MissionDungeon);
        assert_eq!(instance.mission_id, Some("test_mission".to_string()));
        assert_eq!(instance.owner_guild, Some("test_guild".to_string()));
        assert_eq!(instance.level, 5);
        assert!(instance.has_tag("combat"));
    }

    #[test]
    fn test_child_instances() {
        let mut manager = WorldInstanceManager::new();
        
        // Create parent instance
        let parent = WorldInstance::new("Parent Instance", InstanceType::MainWorld, 100.0);
        let parent_id = parent.id.clone();
        manager.add_instance(parent);
        
        // Create child instance
        let child_id = manager.create_child_instance(&parent_id, "Child Instance", InstanceType::SpecialArea, 200.0);
        assert!(child_id.is_some());
        
        // Check parent-child relationship
        let parent = manager.get_instance(&parent_id).unwrap();
        assert!(parent.child_instances.contains(&child_id.unwrap()));
        
        let child = manager.get_instance(&child_id.unwrap()).unwrap();
        assert_eq!(child.parent_instance, Some(parent_id));
    }
}