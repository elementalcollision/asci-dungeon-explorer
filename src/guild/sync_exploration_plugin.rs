use bevy::prelude::*;
use crate::guild::synchronous_exploration::{
    SyncExplorationManager, ControlSwitchEvent, control_switching_system,
    party_formation_system, shared_vision_system, cooperative_actions_system,
    turn_based_system
};
use crate::guild::sync_exploration_input::{
    sync_exploration_input_system, party_movement_input_system,
    shared_action_input_system, resource_sharing_input_system,
    sync_exploration_ui_system
};

/// Plugin for synchronous exploration systems
pub struct SyncExplorationPlugin;

impl Plugin for SyncExplorationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SyncExplorationManager>()
           .add_event::<ControlSwitchEvent>()
           .add_systems(Update, (
               // Input systems
               sync_exploration_input_system,
               party_movement_input_system,
               shared_action_input_system,
               resource_sharing_input_system,
               
               // Core systems
               control_switching_system,
               party_formation_system,
               shared_vision_system,
               cooperative_actions_system,
               turn_based_system,
               
               // UI system
               sync_exploration_ui_system,
           ).chain());
    }
}

/// System for initializing synchronous exploration
pub fn initialize_sync_exploration_system(
    mut commands: Commands,
    mut sync_manager: ResMut<SyncExplorationManager>,
    guild_member_query: Query<(Entity, &crate::guild::guild_core::GuildMember), With<crate::components::Name>>,
    player_query: Query<Entity, With<crate::components::Player>>,
) {
    // Only run once
    if !sync_manager.active_party.is_empty() {
        return;
    }
    
    // Add player to party if exists
    if let Ok(player_entity) = player_query.get_single() {
        sync_manager.add_to_party(player_entity);
        info!("Added player to synchronous exploration party");
    }
    
    // Add some guild members to demonstrate party functionality
    let mut added_count = 0;
    for (entity, guild_member) in guild_member_query.iter() {
        if added_count < 3 && !sync_manager.active_party.contains(&entity) {
            // Add party member component
            commands.entity(entity).insert(crate::guild::synchronous_exploration::PartyMember {
                party_id: "main_party".to_string(),
                role: match guild_member.specialization.as_str() {
                    "Fighter" => crate::guild::synchronous_exploration::PartyRole::Tank,
                    "Rogue" => crate::guild::synchronous_exploration::PartyRole::Scout,
                    "Mage" => crate::guild::synchronous_exploration::PartyRole::DPS,
                    "Cleric" => crate::guild::synchronous_exploration::PartyRole::Support,
                    _ => crate::guild::synchronous_exploration::PartyRole::Utility,
                },
                formation_position: added_count + 1,
                follow_target: sync_manager.get_party_leader(),
                auto_actions: true,
                shared_inventory: true,
                last_action_time: 0.0,
            });
            
            sync_manager.add_to_party(entity);
            added_count += 1;
            
            info!("Added guild member {:?} to synchronous exploration party", entity);
        }
    }
    
    // Set default formation
    sync_manager.set_formation(crate::guild::synchronous_exploration::FormationType::Line);
    
    info!("Initialized synchronous exploration with {} party members", sync_manager.active_party.len());
}