use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use crate::guild::guild_core::{Guild, GuildMember, GuildManager};
use crate::guild::world_instance::{WorldInstance, WorldInstanceManager};
use crate::components::{Position, Player, Health, Name};
use crate::ai::ai_component::AIComponent;

/// Synchronous exploration mode state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncExplorationMode {
    Disabled,
    PlayerControl,
    AgentControl,
    CooperativeControl,
}

impl Default for SyncExplorationMode {
    fn default() -> Self {
        SyncExplorationMode::Disabled
    }
}

/// Synchronous exploration manager resource
#[derive(Resource)]
pub struct SyncExplorationManager {
    pub mode: SyncExplorationMode,
    pub active_party: HashSet<Entity>,
    pub controlled_entity: Option<Entity>,
    pub party_leader: Option<Entity>,
    pub shared_resources: HashMap<String, u32>,
    pub formation: PartyFormation,
    pub auto_follow: bool,
    pub shared_vision: bool,
    pub shared_inventory: bool,
    pub cooperative_actions: bool,
    pub turn_order: Vec<Entity>,
    pub current_turn: usize,
    pub turn_based_mode: bool,
    pub action_points: HashMap<Entity, u32>,
    pub max_action_points: u32,
}

impl Default for SyncExplorationManager {
    fn default() -> Self {
        SyncExplorationManager {
            mode: SyncExplorationMode::default(),
            active_party: HashSet::new(),
            controlled_entity: None,
            party_leader: None,
            shared_resources: HashMap::new(),
            formation: PartyFormation::default(),
            auto_follow: true,
            shared_vision: true,
            shared_inventory: false,
            cooperative_actions: true,
            turn_order: Vec::new(),
            current_turn: 0,
            turn_based_mode: false,
            action_points: HashMap::new(),
            max_action_points: 3,
        }
    }
}

/// Party formation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartyFormation {
    None,
    Line,
    Column,
    Diamond,
    Circle,
    Custom,
}

impl Default for PartyFormation {
    fn default() -> Self {
        PartyFormation::Line
    }
}

/// Party member role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PartyRole {
    Leader,
    Tank,
    DPS,
    Support,
    Scout,
    Healer,
}

/// Party member component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PartyMember {
    pub role: PartyRole,
    pub formation_position: Vec2,
    pub follow_target: Option<Entity>,
    pub is_controlled: bool,
    pub is_active: bool,
    pub shared_resources: bool,
    pub auto_actions: bool,
}

impl Default for PartyMember {
    fn default() -> Self {
        PartyMember {
            role: PartyRole::DPS,
            formation_position: Vec2::ZERO,
            follow_target: None,
            is_controlled: false,
            is_active: true,
            shared_resources: true,
            auto_actions: false,
        }
    }
}

/// Cooperative action component
#[derive(Component, Debug, Clone)]
pub struct CooperativeAction {
    pub action_type: CooperativeActionType,
    pub participants: HashSet<Entity>,
    pub required_participants: u32,
    pub target_position: Option<Vec2>,
    pub target_entity: Option<Entity>,
    pub duration: f32,
    pub progress: f32,
    pub is_active: bool,
}

/// Types of cooperative actions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CooperativeActionType {
    CombinedAttack,
    GroupHeal,
    FormationMove,
    SharedSpell,
    CoordinatedDefense,
    TeamLift,
    GroupPuzzleSolve,
    ChainAction,
}

/// Shared resource component
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SharedResource {
    pub resource_type: String,
    pub amount: u32,
    pub max_amount: u32,
    pub shared_with: HashSet<Entity>,
    pub auto_distribute: bool,
    pub distribution_priority: Vec<Entity>,
}

impl SyncExplorationManager {
    /// Create a new synchronous exploration manager
    pub fn new() -> Self {
        SyncExplorationManager::default()
    }
    
    /// Enable synchronous exploration mode
    pub fn enable_sync_mode(&mut self, mode: SyncExplorationMode) {
        self.mode = mode;
    }
    
    /// Disable synchronous exploration mode
    pub fn disable_sync_mode(&mut self) {
        self.mode = SyncExplorationMode::Disabled;
        self.active_party.clear();
        self.controlled_entity = None;
        self.party_leader = None;
        self.turn_order.clear();
        self.current_turn = 0;
    }
    
    /// Add entity to active party
    pub fn add_to_party(&mut self, entity: Entity) -> bool {
        if self.active_party.len() < 6 { // Max party size
            self.active_party.insert(entity);
            
            // Set as leader if first member
            if self.party_leader.is_none() {
                self.party_leader = Some(entity);
            }
            
            // Add to turn order if turn-based mode
            if self.turn_based_mode && !self.turn_order.contains(&entity) {
                self.turn_order.push(entity);
                self.action_points.insert(entity, self.max_action_points);
            }
            
            true
        } else {
            false
        }
    }
    
    /// Remove entity from active party
    pub fn remove_from_party(&mut self, entity: &Entity) -> bool {
        let removed = self.active_party.remove(entity);
        
        if removed {
            // Update leader if necessary
            if self.party_leader == Some(*entity) {
                self.party_leader = self.active_party.iter().next().copied();
            }
            
            // Update controlled entity if necessary
            if self.controlled_entity == Some(*entity) {
                self.controlled_entity = self.party_leader;
            }
            
            // Remove from turn order
            if let Some(index) = self.turn_order.iter().position(|&e| e == *entity) {
                self.turn_order.remove(index);
                
                // Adjust current turn if necessary
                if self.current_turn >= index && self.current_turn > 0 {
                    self.current_turn -= 1;
                }
            }
            
            self.action_points.remove(entity);
        }
        
        removed
    }
    
    /// Set controlled entity
    pub fn set_controlled_entity(&mut self, entity: Entity) -> bool {
        if self.active_party.contains(&entity) {
            self.controlled_entity = Some(entity);
            true
        } else {
            false
        }
    }
    
    /// Get controlled entity
    pub fn get_controlled_entity(&self) -> Option<Entity> {
        self.controlled_entity
    }
    
    /// Set party leader
    pub fn set_party_leader(&mut self, entity: Entity) -> bool {
        if self.active_party.contains(&entity) {
            self.party_leader = Some(entity);
            true
        } else {
            false
        }
    }
    
    /// Get party leader
    pub fn get_party_leader(&self) -> Option<Entity> {
        self.party_leader
    }
    
    /// Switch to next party member
    pub fn switch_to_next_member(&mut self) -> Option<Entity> {
        if let Some(current) = self.controlled_entity {
            let members: Vec<Entity> = self.active_party.iter().copied().collect();
            if let Some(current_index) = members.iter().position(|&e| e == current) {
                let next_index = (current_index + 1) % members.len();
                let next_entity = members[next_index];
                self.controlled_entity = Some(next_entity);
                return Some(next_entity);
            }
        }
        
        // If no current entity or not found, use first party member
        if let Some(&first) = self.active_party.iter().next() {
            self.controlled_entity = Some(first);
            Some(first)
        } else {
            None
        }
    }
    
    /// Switch to previous party member
    pub fn switch_to_previous_member(&mut self) -> Option<Entity> {
        if let Some(current) = self.controlled_entity {
            let members: Vec<Entity> = self.active_party.iter().copied().collect();
            if let Some(current_index) = members.iter().position(|&e| e == current) {
                let prev_index = if current_index == 0 {
                    members.len() - 1
                } else {
                    current_index - 1
                };
                let prev_entity = members[prev_index];
                self.controlled_entity = Some(prev_entity);
                return Some(prev_entity);
            }
        }
        
        // If no current entity or not found, use last party member
        if let Some(&last) = self.active_party.iter().last() {
            self.controlled_entity = Some(last);
            Some(last)
        } else {
            None
        }
    }
    
    /// Enable turn-based mode
    pub fn enable_turn_based_mode(&mut self) {
        self.turn_based_mode = true;
        
        // Initialize turn order and action points
        self.turn_order = self.active_party.iter().copied().collect();
        for &entity in &self.active_party {
            self.action_points.insert(entity, self.max_action_points);
        }
        
        self.current_turn = 0;
    }
    
    /// Disable turn-based mode
    pub fn disable_turn_based_mode(&mut self) {
        self.turn_based_mode = false;
        self.turn_order.clear();
        self.action_points.clear();
        self.current_turn = 0;
    }
    
    /// Get current turn entity
    pub fn get_current_turn_entity(&self) -> Option<Entity> {
        if self.turn_based_mode && !self.turn_order.is_empty() {
            Some(self.turn_order[self.current_turn % self.turn_order.len()])
        } else {
            None
        }
    }
    
    /// End current turn
    pub fn end_turn(&mut self) -> Option<Entity> {
        if self.turn_based_mode && !self.turn_order.is_empty() {
            self.current_turn = (self.current_turn + 1) % self.turn_order.len();
            
            // Reset action points for new turn entity
            let current_entity = self.turn_order[self.current_turn];
            self.action_points.insert(current_entity, self.max_action_points);
            
            Some(current_entity)
        } else {
            None
        }
    }
    
    /// Use action points
    pub fn use_action_points(&mut self, entity: Entity, points: u32) -> bool {
        if let Some(current_points) = self.action_points.get_mut(&entity) {
            if *current_points >= points {
                *current_points -= points;
                return true;
            }
        }
        false
    }
    
    /// Get action points for entity
    pub fn get_action_points(&self, entity: &Entity) -> u32 {
        self.action_points.get(entity).copied().unwrap_or(0)
    }
    
    /// Check if entity can act
    pub fn can_act(&self, entity: &Entity) -> bool {
        if self.turn_based_mode {
            self.get_current_turn_entity() == Some(*entity) && self.get_action_points(entity) > 0
        } else {
            true
        }
    }
    
    /// Set formation
    pub fn set_formation(&mut self, formation: PartyFormation) {
        self.formation = formation;
    }
    
    /// Calculate formation positions
    pub fn calculate_formation_positions(&self, leader_position: Vec2) -> HashMap<Entity, Vec2> {
        let mut positions = HashMap::new();
        let members: Vec<Entity> = self.active_party.iter().copied().collect();
        
        if members.is_empty() {
            return positions;
        }
        
        match self.formation {
            PartyFormation::None => {
                // No formation, everyone at leader position
                for entity in members {
                    positions.insert(entity, leader_position);
                }
            },
            PartyFormation::Line => {
                // Horizontal line formation
                let spacing = 1.5;
                let start_offset = -(members.len() as f32 - 1.0) * spacing / 2.0;
                
                for (i, entity) in members.iter().enumerate() {
                    let offset = Vec2::new(start_offset + i as f32 * spacing, 0.0);
                    positions.insert(*entity, leader_position + offset);
                }
            },
            PartyFormation::Column => {
                // Vertical column formation
                let spacing = 1.5;
                
                for (i, entity) in members.iter().enumerate() {
                    let offset = Vec2::new(0.0, -(i as f32) * spacing);
                    positions.insert(*entity, leader_position + offset);
                }
            },
            PartyFormation::Diamond => {
                // Diamond formation
                match members.len() {
                    1 => positions.insert(members[0], leader_position),
                    2 => {
                        positions.insert(members[0], leader_position);
                        positions.insert(members[1], leader_position + Vec2::new(0.0, -1.5));
                    },
                    3 => {
                        positions.insert(members[0], leader_position);
                        positions.insert(members[1], leader_position + Vec2::new(-1.5, -1.5));
                        positions.insert(members[2], leader_position + Vec2::new(1.5, -1.5));
                    },
                    _ => {
                        positions.insert(members[0], leader_position);
                        positions.insert(members[1], leader_position + Vec2::new(-1.5, -1.5));
                        positions.insert(members[2], leader_position + Vec2::new(1.5, -1.5));
                        positions.insert(members[3], leader_position + Vec2::new(0.0, -3.0));
                        
                        // Additional members in a second row
                        for (i, entity) in members.iter().skip(4).enumerate() {
                            let offset = Vec2::new((i as f32 - 1.0) * 1.5, -4.5);
                            positions.insert(*entity, leader_position + offset);
                        }
                    }
                };
            },
            PartyFormation::Circle => {
                // Circular formation
                let radius = 2.0;
                let angle_step = std::f32::consts::TAU / members.len() as f32;
                
                for (i, entity) in members.iter().enumerate() {
                    let angle = i as f32 * angle_step;
                    let offset = Vec2::new(angle.cos() * radius, angle.sin() * radius);
                    positions.insert(*entity, leader_position + offset);
                }
            },
            PartyFormation::Custom => {
                // Custom formation would be defined elsewhere
                // For now, default to line formation
                let spacing = 1.5;
                let start_offset = -(members.len() as f32 - 1.0) * spacing / 2.0;
                
                for (i, entity) in members.iter().enumerate() {
                    let offset = Vec2::new(start_offset + i as f32 * spacing, 0.0);
                    positions.insert(*entity, leader_position + offset);
                }
            },
        }
        
        positions
    }
    
    /// Add shared resource
    pub fn add_shared_resource(&mut self, resource_type: &str, amount: u32) {
        *self.shared_resources.entry(resource_type.to_string()).or_insert(0) += amount;
    }
    
    /// Remove shared resource
    pub fn remove_shared_resource(&mut self, resource_type: &str, amount: u32) -> bool {
        if let Some(current) = self.shared_resources.get_mut(resource_type) {
            if *current >= amount {
                *current -= amount;
                return true;
            }
        }
        false
    }
    
    /// Get shared resource amount
    pub fn get_shared_resource(&self, resource_type: &str) -> u32 {
        self.shared_resources.get(resource_type).copied().unwrap_or(0)
    }
}

impl PartyMember {
    /// Create a new party member
    pub fn new(role: PartyRole) -> Self {
        PartyMember {
            role,
            ..Default::default()
        }
    }
    
    /// Set formation position
    pub fn set_formation_position(&mut self, position: Vec2) {
        self.formation_position = position;
    }
    
    /// Set follow target
    pub fn set_follow_target(&mut self, target: Option<Entity>) {
        self.follow_target = target;
    }
    
    /// Enable/disable control
    pub fn set_controlled(&mut self, controlled: bool) {
        self.is_controlled = controlled;
    }
    
    /// Enable/disable activity
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }
}

impl CooperativeAction {
    /// Create a new cooperative action
    pub fn new(action_type: CooperativeActionType, required_participants: u32) -> Self {
        CooperativeAction {
            action_type,
            participants: HashSet::new(),
            required_participants,
            target_position: None,
            target_entity: None,
            duration: 1.0,
            progress: 0.0,
            is_active: false,
        }
    }
    
    /// Add participant
    pub fn add_participant(&mut self, entity: Entity) -> bool {
        if self.participants.len() < self.required_participants as usize {
            self.participants.insert(entity);
            
            // Start action if we have enough participants
            if self.participants.len() >= self.required_participants as usize {
                self.is_active = true;
            }
            
            true
        } else {
            false
        }
    }
    
    /// Remove participant
    pub fn remove_participant(&mut self, entity: &Entity) -> bool {
        let removed = self.participants.remove(entity);
        
        if removed && self.participants.len() < self.required_participants as usize {
            self.is_active = false;
            self.progress = 0.0;
        }
        
        removed
    }
    
    /// Update action progress
    pub fn update(&mut self, delta_time: f32) -> bool {
        if self.is_active {
            self.progress += delta_time / self.duration;
            
            if self.progress >= 1.0 {
                self.progress = 1.0;
                return true; // Action completed
            }
        }
        
        false
    }
    
    /// Check if action is ready
    pub fn is_ready(&self) -> bool {
        self.participants.len() >= self.required_participants as usize
    }
    
    /// Check if action is completed
    pub fn is_completed(&self) -> bool {
        self.progress >= 1.0
    }
}

impl SharedResource {
    /// Create a new shared resource
    pub fn new(resource_type: &str, amount: u32, max_amount: u32) -> Self {
        SharedResource {
            resource_type: resource_type.to_string(),
            amount,
            max_amount,
            shared_with: HashSet::new(),
            auto_distribute: false,
            distribution_priority: Vec::new(),
        }
    }
    
    /// Add entity to sharing list
    pub fn add_sharer(&mut self, entity: Entity) {
        self.shared_with.insert(entity);
    }
    
    /// Remove entity from sharing list
    pub fn remove_sharer(&mut self, entity: &Entity) {
        self.shared_with.remove(entity);
    }
    
    /// Set distribution priority
    pub fn set_distribution_priority(&mut self, priority: Vec<Entity>) {
        self.distribution_priority = priority;
    }
    
    /// Distribute resource
    pub fn distribute(&mut self, amount: u32) -> HashMap<Entity, u32> {
        let mut distribution = HashMap::new();
        
        if self.shared_with.is_empty() || amount == 0 {
            return distribution;
        }
        
        let per_entity = amount / self.shared_with.len() as u32;
        let remainder = amount % self.shared_with.len() as u32;
        
        // Distribute evenly
        for entity in &self.shared_with {
            distribution.insert(*entity, per_entity);
        }
        
        // Distribute remainder based on priority
        let mut remainder_left = remainder;
        for entity in &self.distribution_priority {
            if remainder_left == 0 {
                break;
            }
            
            if self.shared_with.contains(entity) {
                *distribution.entry(*entity).or_insert(0) += 1;
                remainder_left -= 1;
            }
        }
        
        // If still remainder, distribute to any entity
        if remainder_left > 0 {
            for entity in &self.shared_with {
                if remainder_left == 0 {
                    break;
                }
                
                *distribution.entry(*entity).or_insert(0) += 1;
                remainder_left -= 1;
            }
        }
        
        distribution
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_exploration_manager() {
        let mut manager = SyncExplorationManager::new();
        
        // Test adding party members
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        
        assert!(manager.add_to_party(entity1));
        assert!(manager.add_to_party(entity2));
        
        assert_eq!(manager.active_party.len(), 2);
        assert_eq!(manager.get_party_leader(), Some(entity1));
        
        // Test switching controlled entity
        assert!(manager.set_controlled_entity(entity2));
        assert_eq!(manager.get_controlled_entity(), Some(entity2));
        
        // Test switching to next member
        let next = manager.switch_to_next_member();
        assert_eq!(next, Some(entity1));
        
        // Test turn-based mode
        manager.enable_turn_based_mode();
        assert!(manager.turn_based_mode);
        assert_eq!(manager.turn_order.len(), 2);
        assert_eq!(manager.get_current_turn_entity(), Some(entity1));
        
        // Test action points
        assert!(manager.use_action_points(entity1, 1));
        assert_eq!(manager.get_action_points(&entity1), 2);
        
        // Test ending turn
        let next_turn = manager.end_turn();
        assert_eq!(next_turn, Some(entity2));
        assert_eq!(manager.get_action_points(&entity2), 3);
    }

    #[test]
    fn test_formation_positions() {
        let mut manager = SyncExplorationManager::new();
        
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);
        
        manager.add_to_party(entity1);
        manager.add_to_party(entity2);
        manager.add_to_party(entity3);
        
        manager.set_formation(PartyFormation::Line);
        
        let leader_pos = Vec2::new(10.0, 10.0);
        let positions = manager.calculate_formation_positions(leader_pos);
        
        assert_eq!(positions.len(), 3);
        assert!(positions.contains_key(&entity1));
        assert!(positions.contains_key(&entity2));
        assert!(positions.contains_key(&entity3));
    }

    #[test]
    fn test_cooperative_action() {
        let mut action = CooperativeAction::new(CooperativeActionType::CombinedAttack, 2);
        
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        
        assert!(!action.is_ready());
        assert!(!action.is_active);
        
        assert!(action.add_participant(entity1));
        assert!(!action.is_ready());
        
        assert!(action.add_participant(entity2));
        assert!(action.is_ready());
        assert!(action.is_active);
        
        // Test progress
        let completed = action.update(0.5);
        assert!(!completed);
        assert_eq!(action.progress, 0.5);
        
        let completed = action.update(0.5);
        assert!(completed);
        assert!(action.is_completed());
    }

    #[test]
    fn test_shared_resource() {
        let mut resource = SharedResource::new("Gold", 100, 1000);
        
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);
        let entity3 = Entity::from_raw(3);
        
        resource.add_sharer(entity1);
        resource.add_sharer(entity2);
        resource.add_sharer(entity3);
        
        let distribution = resource.distribute(10);
        
        assert_eq!(distribution.len(), 3);
        assert_eq!(distribution.get(&entity1), Some(&3));
        assert_eq!(distribution.get(&entity2), Some(&3));
        assert_eq!(distribution.get(&entity3), Some(&3));
        
        // Test with remainder
        let distribution = resource.distribute(11);
        let total: u32 = distribution.values().sum();
        assert_eq!(total, 11);
    }
}