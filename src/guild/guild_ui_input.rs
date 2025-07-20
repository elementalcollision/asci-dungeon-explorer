use bevy::prelude::*;
use crate::guild::guild_ui_types::{GuildUI, GuildUIState, GuildUIAction};
use crate::ui::{UIState, UIAction};

/// System for handling guild UI input
pub fn guild_ui_input_system(
    mut ui_state: ResMut<UIState>,
    mut guild_ui: ResMut<GuildUI>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ui_actions: EventWriter<UIAction>,
) {
    // Toggle guild UI with 'G' key
    if keyboard_input.just_pressed(KeyCode::G) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::ToggleUI)));
    }
    
    // Only process other inputs if UI is visible
    if guild_ui.state == GuildUIState::Hidden {
        return;
    }
    
    // Navigation between UI states
    if keyboard_input.just_pressed(KeyCode::Key1) || keyboard_input.just_pressed(KeyCode::Numpad1) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Main))));
    }
    if keyboard_input.just_pressed(KeyCode::Key2) || keyboard_input.just_pressed(KeyCode::Numpad2) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Members))));
    }
    if keyboard_input.just_pressed(KeyCode::Key3) || keyboard_input.just_pressed(KeyCode::Numpad3) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Missions))));
    }
    if keyboard_input.just_pressed(KeyCode::Key4) || keyboard_input.just_pressed(KeyCode::Numpad4) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Facilities))));
    }
    if keyboard_input.just_pressed(KeyCode::Key5) || keyboard_input.just_pressed(KeyCode::Numpad5) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Resources))));
    }
    
    // Scrolling
    if keyboard_input.just_pressed(KeyCode::Up) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::ScrollUp)));
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::ScrollDown)));
    }
    
    // Close UI with Escape
    if keyboard_input.just_pressed(KeyCode::Escape) {
        ui_actions.send(UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Hidden))));
    }
}

/// System for handling guild UI actions
pub fn guild_ui_action_system(
    mut guild_ui: ResMut<GuildUI>,
    mut ui_state: ResMut<UIState>,
    mut ui_actions: EventReader<UIAction>,
    mut guild_manager: ResMut<crate::guild::guild_core::GuildManager>,
    mut mission_board: ResMut<crate::guild::mission_board::MissionBoard>,
    mut agent_query: Query<(Entity, &mut crate::guild::agent_behavior::AgentBehavior, &mut crate::guild::agent_progression::AgentStats, Option<&mut crate::guild::mission::MissionTracker>)>,
    player_query: Query<Entity, With<crate::components::Player>>,
) {
    for action in ui_actions.iter() {
        if let UIAction::Custom(custom_action) = action {
            if let Some(guild_action) = custom_action.downcast_ref::<GuildUIAction>() {
                match guild_action {
                    GuildUIAction::ToggleUI => {
                        if guild_ui.state == GuildUIState::Hidden {
                            guild_ui.state = GuildUIState::Main;
                            ui_state.active = true;
                        } else {
                            guild_ui.state = GuildUIState::Hidden;
                            ui_state.active = false;
                        }
                    },
                    GuildUIAction::SetState(state) => {
                        guild_ui.state = state.clone();
                        if *state == GuildUIState::Hidden {
                            ui_state.active = false;
                        } else {
                            ui_state.active = true;
                        }
                    },
                    GuildUIAction::SelectGuild(guild_id) => {
                        guild_ui.selected_guild = Some(guild_id.clone());
                        guild_ui.selected_member = None;
                        guild_ui.selected_mission = None;
                        guild_ui.selected_facility = None;
                    },
                    GuildUIAction::SelectMember(entity) => {
                        guild_ui.selected_member = Some(*entity);
                    },
                    GuildUIAction::SelectMission(mission_id) => {
                        guild_ui.selected_mission = Some(mission_id.clone());
                    },
                    GuildUIAction::SelectFacility(facility) => {
                        guild_ui.selected_facility = Some(*facility);
                    },
                    GuildUIAction::AssignMission(entity, mission_id) => {
                        // Assign mission to agent
                        if let Some(mission) = mission_board.get_mission_mut(mission_id) {
                            mission.assign_agent(*entity);
                            
                            // Create or update mission tracker
                            if let Ok((_, _, _, Some(mut tracker))) = agent_query.get_mut(*entity) {
                                tracker.start_mission(mission_id, 0.0); // Current time would be used in real implementation
                            }
                        }
                    },
                    GuildUIAction::ConfigureAgent(entity) => {
                        guild_ui.selected_member = Some(*entity);
                        guild_ui.state = GuildUIState::AgentConfig;
                    },
                    GuildUIAction::SetAgentBehavior(entity, behavior_type) => {
                        // Update agent behavior
                        if let Ok((_, mut behavior, _, _)) = agent_query.get_mut(*entity) {
                            *behavior = crate::guild::agent_behavior::AgentBehavior::new(*behavior_type);
                        }
                    },
                    GuildUIAction::UpgradeAgentStat(entity, stat_name) => {
                        // Upgrade agent stat
                        if let Ok((_, _, mut stats, _)) = agent_query.get_mut(*entity) {
                            stats.increase_attribute(stat_name, 1);
                        }
                    },
                    GuildUIAction::UpgradeFacility(facility) => {
                        // Upgrade facility
                        if let Some(guild_id) = &guild_ui.selected_guild {
                            if let Some(guild) = guild_manager.get_guild_mut(guild_id) {
                                guild.upgrade_facility(*facility);
                            }
                        }
                    },
                    GuildUIAction::BuildFacility(facility) => {
                        // Build facility
                        if let Some(guild_id) = &guild_ui.selected_guild {
                            if let Some(guild) = guild_manager.get_guild_mut(guild_id) {
                                guild.build_facility(*facility);
                            }
                        }
                    },
                    GuildUIAction::ScrollUp => {
                        if guild_ui.scroll_offset > 0 {
                            guild_ui.scroll_offset -= 1;
                        }
                    },
                    GuildUIAction::ScrollDown => {
                        guild_ui.scroll_offset += 1;
                    },
                    GuildUIAction::SetFilter(filter) => {
                        guild_ui.filter = filter.clone();
                    },
                    GuildUIAction::ToggleShowCompletedMissions => {
                        guild_ui.show_completed_missions = !guild_ui.show_completed_missions;
                    },
                    GuildUIAction::ToggleShowFailedMissions => {
                        guild_ui.show_failed_missions = !guild_ui.show_failed_missions;
                    },
                }
            }
        }
    }
}