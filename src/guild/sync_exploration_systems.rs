use bevy::prelude::*;
use crate::guild::synchronous_exploration::{
    SyncExplorationManager, SyncExplorationMode, PartyMember, CooperativeAction, 
    SharedResource, PartyFormation, PartyRole
};
use crate::components::{Position, Player, Health, Name, Viewshed};
use crate::ai::ai_component::AIComponent;
use crate::input::InputAction;

/// System for managing party formation
pub fn party_formation_system(
    mut sync_manager: ResMut<SyncExplorationManager>,
    mut party_query: Query<(Entity, &mut PartyMember, &mut Position)>,
    leader_query: Query<&Position, (With<Player>, Without<PartyMember>)>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled || !sync_manager.auto_follow {
        return;
    }
    
    // Get leader position
    let leader_position = if let Some(leader_entity) = sync_manager.get_party_leader() {
        if let Ok(pos) = party_query.get(leader_entity) {
            pos.2.0
        } else if let Ok(pos) = leader_query.get_single() {
            pos.0
        } else {
            return;
        }
    } else {
        return;
    };
    
    // Calculate formation positions
    let formation_positions = sync_manager.calculate_formation_positions(leader_position);
    
    // Update party member positions
    for (entity, mut party_member, mut position) in party_query.iter_mut() {
        if sync_manager.active_party.contains(&entity) {
            if let Some(&target_pos) = formation_positions.get(&entity) {
                party_member.set_formation_position(target_pos);
                
                // Move towards formation position if not controlled
                if !party_member.is_controlled && sync_manager.auto_follow {
                    let distance = position.0.distance(target_pos);
                    if distance > 0.5 {
                        let direction = (target_pos - position.0).normalize();
                        position.0 += direction * 2.0 * 0.016; // Assuming 60 FPS
                    }
                }
            }
        }
    }
}

/// System for handling party member control switching
pub fn party_control_system(
    mut sync_manager: ResMut<SyncExplorationManager>,
    mut party_query: Query<(Entity, &mut PartyMember, &mut AIComponent)>,
    mut player_query: Query<Entity, With<Player>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    // Handle control switching
    let mut switch_next = false;
    let mut switch_prev = false;
    
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if keyboard_input.pressed(KeyCode::LShift) || keyboard_input.pressed(KeyCode::RShift) {
            switch_prev = true;
        } else {
            switch_next = true;
        }
    }
    
    if switch_next || switch_prev {
        let new_controlled = if switch_next {
            sync_manager.switch_to_next_member()
        } else {
            sync_manager.switch_to_previous_member()
        };
        
        if let Some(new_entity) = new_controlled {
            // Update party member components
            for (entity, mut party_member, mut ai) in party_query.iter_mut() {
                if sync_manager.active_party.contains(&entity) {
                    if entity == new_entity {
                        party_member.set_controlled(true);
                        ai.enabled = false; // Disable AI for controlled entity
                        
                        // Add Player component if not present
                        if !player_query.contains(entity) {
                            commands.entity(entity).insert(Player);
                        }
                    } else {
                        party_member.set_controlled(false);
                        ai.enabled = true; // Enable AI for non-controlled entities
                        
                        // Remove Player component if present
                        if player_query.contains(entity) {
                            commands.entity(entity).remove::<Player>();
                        }
                    }
                }
            }
        }
    }
}

/// System for managing turn-based mode
pub fn turn_based_system(
    mut sync_manager: ResMut<SyncExplorationManager>,
    mut party_query: Query<(Entity, &mut PartyMember)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled || !sync_manager.turn_based_mode {
        return;
    }
    
    // Handle turn ending
    if keyboard_input.just_pressed(KeyCode::Space) {
        if let Some(current_entity) = sync_manager.get_current_turn_entity() {
            // End current turn
            if let Some(next_entity) = sync_manager.end_turn() {
                // Update party member states
                for (entity, mut party_member) in party_query.iter_mut() {
                    if sync_manager.active_party.contains(&entity) {
                        party_member.set_controlled(entity == next_entity);
                    }
                }
            }
        }
    }
    
    // Auto-end turn if no action points left
    if let Some(current_entity) = sync_manager.get_current_turn_entity() {
        if sync_manager.get_action_points(&current_entity) == 0 {
            // Auto-end turn after a short delay
            // In a real implementation, you might want to add a timer component
            if let Some(next_entity) = sync_manager.end_turn() {
                for (entity, mut party_member) in party_query.iter_mut() {
                    if sync_manager.active_party.contains(&entity) {
                        party_member.set_controlled(entity == next_entity);
                    }
                }
            }
        }
    }
}

/// System for handling cooperative actions
pub fn cooperative_action_system(
    mut commands: Commands,
    mut cooperative_query: Query<(Entity, &mut CooperativeAction)>,
    party_query: Query<(Entity, &PartyMember, &Position)>,
    sync_manager: Res<SyncExplorationManager>,
    time: Res<Time>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled || !sync_manager.cooperative_actions {
        return;
    }
    
    let delta_time = time.delta_seconds();
    
    for (action_entity, mut action) in cooperative_query.iter_mut() {
        // Update action progress
        let completed = action.update(delta_time);
        
        if completed {
            // Execute cooperative action
            execute_cooperative_action(&action, &party_query, &mut commands);
            
            // Remove completed action
            commands.entity(action_entity).despawn();
        } else if !action.is_ready() {
            // Remove action if not enough participants
            commands.entity(action_entity).despawn();
        }
    }
}

/// Execute a cooperative action
fn execute_cooperative_action(
    action: &CooperativeAction,
    party_query: &Query<(Entity, &PartyMember, &Position)>,
    commands: &mut Commands,
) {
    match action.action_type {
        crate::guild::synchronous_exploration::CooperativeActionType::CombinedAttack => {
            // Execute combined attack
            // In a real implementation, you would calculate damage bonuses
            // and apply effects to the target
        },
        crate::guild::synchronous_exploration::CooperativeActionType::GroupHeal => {
            // Execute group heal
            // In a real implementation, you would heal all participants
        },
        crate::guild::synchronous_exploration::CooperativeActionType::FormationMove => {
            // Execute formation move
            // In a real implementation, you would move all participants
            // to their formation positions simultaneously
        },
        crate::guild::synchronous_exploration::CooperativeActionType::SharedSpell => {
            // Execute shared spell
            // In a real implementation, you would combine mana from participants
            // and cast a more powerful spell
        },
        crate::guild::synchronous_exploration::CooperativeActionType::CoordinatedDefense => {
            // Execute coordinated defense
            // In a real implementation, you would apply defense bonuses
            // to all participants
        },
        crate::guild::synchronous_exploration::CooperativeActionType::TeamLift => {
            // Execute team lift
            // In a real implementation, you would allow lifting heavy objects
            // that require multiple characters
        },
        crate::guild::synchronous_exploration::CooperativeActionType::GroupPuzzleSolve => {
            // Execute group puzzle solve
            // In a real implementation, you would solve puzzles that require
            // multiple characters to activate simultaneously
        },
        crate::guild::synchronous_exploration::CooperativeActionType::ChainAction => {
            // Execute chain action
            // In a real implementation, you would execute a sequence of actions
            // performed by different characters in order
        },
    }
}

/// System for managing shared resources
pub fn shared_resource_system(
    mut sync_manager: ResMut<SyncExplorationManager>,
    mut shared_resource_query: Query<(Entity, &mut SharedResource)>,
    party_query: Query<(Entity, &PartyMember)>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled || !sync_manager.shared_inventory {
        return;
    }
    
    // Update shared resources
    for (resource_entity, mut shared_resource) in shared_resource_query.iter_mut() {
        if shared_resource.auto_distribute && shared_resource.amount > 0 {
            // Distribute resources to party members
            let distribution = shared_resource.distribute(shared_resource.amount);
            
            for (entity, amount) in distribution {
                if sync_manager.active_party.contains(&entity) {
                    // In a real implementation, you would add the resource to the entity's inventory
                    sync_manager.add_shared_resource(&shared_resource.resource_type, amount);
                }
            }
            
            // Clear the shared resource after distribution
            shared_resource.amount = 0;
        }
    }
}

/// System for managing shared vision
pub fn shared_vision_system(
    sync_manager: Res<SyncExplorationManager>,
    mut party_query: Query<(Entity, &PartyMember, &mut Viewshed)>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled || !sync_manager.shared_vision {
        return;
    }
    
    // Collect all visible tiles from party members
    let mut all_visible_tiles = std::collections::HashSet::new();
    
    for (entity, party_member, viewshed) in party_query.iter() {
        if sync_manager.active_party.contains(&entity) && party_member.is_active {
            for &tile in &viewshed.visible_tiles {
                all_visible_tiles.insert(tile);
            }
        }
    }
    
    // Share vision with all party members
    for (entity, party_member, mut viewshed) in party_query.iter_mut() {
        if sync_manager.active_party.contains(&entity) && party_member.is_active {
            // Add all shared visible tiles
            for &tile in &all_visible_tiles {
                viewshed.visible_tiles.insert(tile);
            }
        }
    }
}

/// System for initializing party members
pub fn party_initialization_system(
    mut commands: Commands,
    sync_manager: Res<SyncExplorationManager>,
    guild_member_query: Query<(Entity, &crate::guild::guild_core::GuildMember), Without<PartyMember>>,
    player_query: Query<Entity, With<Player>>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    // Add PartyMember component to entities in the active party
    for entity in &sync_manager.active_party {
        if let Ok((_, guild_member)) = guild_member_query.get(*entity) {
            // Determine role based on specialization
            let role = match guild_member.specialization.as_str() {
                "Fighter" => PartyRole::Tank,
                "Rogue" => PartyRole::Scout,
                "Mage" => PartyRole::DPS,
                "Cleric" => PartyRole::Healer,
                "Ranger" => PartyRole::Scout,
                _ => PartyRole::DPS,
            };
            
            let mut party_member = PartyMember::new(role);
            
            // Set as controlled if this is the controlled entity
            if sync_manager.get_controlled_entity() == Some(*entity) {
                party_member.set_controlled(true);
            }
            
            // Set follow target to leader if not the leader
            if sync_manager.get_party_leader() != Some(*entity) {
                party_member.set_follow_target(sync_manager.get_party_leader());
            }
            
            commands.entity(*entity).insert(party_member);
        }
    }
}

/// System for handling party member AI
pub fn party_ai_system(
    sync_manager: Res<SyncExplorationManager>,
    mut party_query: Query<(Entity, &PartyMember, &mut AIComponent, &Position)>,
    leader_query: Query<&Position, With<Player>>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    // Get leader position for following
    let leader_position = if let Ok(pos) = leader_query.get_single() {
        Some(pos.0)
    } else if let Some(leader_entity) = sync_manager.get_party_leader() {
        if let Ok((_, _, _, pos)) = party_query.get(leader_entity) {
            Some(pos.0)
        } else {
            None
        }
    } else {
        None
    };
    
    for (entity, party_member, mut ai, position) in party_query.iter_mut() {
        if sync_manager.active_party.contains(&entity) && !party_member.is_controlled {
            // Set AI behavior based on party role and formation
            if sync_manager.auto_follow {
                if let Some(leader_pos) = leader_position {
                    // Set target position for following
                    let target_pos = party_member.formation_position;
                    if position.0.distance(target_pos) > 1.0 {
                        ai.memory.last_known_target_position = Some(target_pos);
                        ai.transition_to_state(crate::ai::ai_component::AIBehaviorState::Follow);
                    }
                }
            }
            
            // Adjust AI behavior based on party role
            match party_member.role {
                PartyRole::Tank => {
                    // Tank should be more aggressive and protective
                    ai.personality.aggression = 0.8;
                    ai.personality.courage = 0.9;
                    ai.personality.loyalty = 0.9;
                },
                PartyRole::DPS => {
                    // DPS should focus on damage
                    ai.personality.aggression = 0.9;
                    ai.personality.courage = 0.7;
                },
                PartyRole::Support | PartyRole::Healer => {
                    // Support should be cautious and stay back
                    ai.personality.aggression = 0.3;
                    ai.personality.courage = 0.5;
                    ai.personality.loyalty = 0.9;
                },
                PartyRole::Scout => {
                    // Scout should be alert and mobile
                    ai.personality.alertness = 0.9;
                    ai.personality.curiosity = 0.8;
                    ai.personality.courage = 0.6;
                },
                PartyRole::Leader => {
                    // Leader should be balanced
                    ai.personality.intelligence = 0.8;
                    ai.personality.loyalty = 0.8;
                    ai.personality.courage = 0.7;
                },
            }
        }
    }
}

/// Plugin for synchronous exploration systems
pub struct SyncExplorationPlugin;

impl Plugin for SyncExplorationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SyncExplorationManager>()
           .add_systems(Update, (
               party_initialization_system,
               party_formation_system,
               party_control_system,
               turn_based_system,
               cooperative_action_system,
               shared_resource_system,
               shared_vision_system,
               party_ai_system,
           ).chain());
    }
}