use bevy::prelude::*;
use crate::guild::guild_ui_types::{GuildUI, GuildUIState, GuildUIAction};
use crate::guild::guild_core::{Guild, GuildMember, GuildManager, GuildResource, GuildFacility};
use crate::guild::mission::Mission;
use crate::guild::mission_board::MissionBoard;
use crate::guild::agent_behavior::AgentBehaviorType;
use crate::components::{Player, Name};
use crate::ui::{UIState, UIAction, UIElement, UIBox, UIText, UIButton, UIPanel};

/// System for rendering the guild UI
pub fn guild_ui_render_system(
    guild_ui: Res<GuildUI>,
    guild_manager: Res<GuildManager>,
    mission_board: Res<MissionBoard>,
    agent_query: Query<(Entity, &GuildMember, Option<&crate::guild::agent_behavior::AgentBehavior>, Option<&crate::guild::agent_progression::AgentStats>, Option<&crate::guild::agent_progression::AgentProgression>, Option<&crate::guild::mission::MissionTracker>, Option<&Name>)>,
    player_query: Query<Entity, With<Player>>,
    mut ui_elements: ResMut<Vec<UIElement>>,
) {
    // Clear existing UI elements
    ui_elements.clear();
    
    // Only render if UI is visible
    if guild_ui.state == GuildUIState::Hidden {
        return;
    }
    
    // Get player's guild
    let player_guild_id = if let Some(player_entity) = player_query.iter().next() {
        if let Ok((_, guild_member, _, _, _, _, _)) = agent_query.get(player_entity) {
            Some(guild_member.guild_id.clone())
        } else {
            None
        }
    } else {
        None
    };
    
    // Get selected guild
    let selected_guild_id = guild_ui.selected_guild.clone().or(player_guild_id.clone());
    
    // Main container
    ui_elements.push(UIElement::Panel(UIPanel {
        x: 5,
        y: 2,
        width: 70,
        height: 40,
        title: "Guild Management".to_string(),
        border: true,
    }));
    
    // Navigation tabs
    ui_elements.push(UIElement::Text(UIText {
        x: 7,
        y: 4,
        text: format!("[1] Main | [2] Members | [3] Missions | [4] Facilities | [5] Resources"),
        color: None,
    }));
    
    // Render appropriate content based on state
    match guild_ui.state {
        GuildUIState::Main => render_main_screen(&guild_ui, &guild_manager, &selected_guild_id, &mut ui_elements),
        GuildUIState::Members => render_members_screen(&guild_ui, &guild_manager, &selected_guild_id, &agent_query, &mut ui_elements),
        GuildUIState::Missions => render_missions_screen(&guild_ui, &mission_board, &selected_guild_id, &mut ui_elements),
        GuildUIState::Facilities => render_facilities_screen(&guild_ui, &guild_manager, &selected_guild_id, &mut ui_elements),
        GuildUIState::Resources => render_resources_screen(&guild_ui, &guild_manager, &selected_guild_id, &mut ui_elements),
        GuildUIState::AgentConfig => render_agent_config_screen(&guild_ui, &agent_query, &mut ui_elements),
        _ => {}
    }
    
    // Footer
    ui_elements.push(UIElement::Text(UIText {
        x: 7,
        y: 41,
        text: format!("[ESC] Close | [↑/↓] Scroll"),
        color: None,
    }));
}

/// Render the main guild screen
pub fn render_main_screen(
    guild_ui: &GuildUI,
    guild_manager: &GuildManager,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild_id) = selected_guild_id {
        if let Some(guild) = guild_manager.get_guild(guild_id) {
            // Guild info
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 6,
                text: format!("Guild: {}", guild.name),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 7,
                text: format!("Level: {}", guild.level),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 8,
                text: format!("Reputation: {}", guild.reputation),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 9,
                text: format!("Members: {}", guild.members.len()),
                color: None,
            }));
            
            // Description
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 11,
                width: 66,
                height: 4,
                title: Some("Description".to_string()),
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 12,
                text: guild.description.clone(),
                color: None,
            }));
            
            // Resources summary
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 16,
                width: 30,
                height: 7,
                title: Some("Resources".to_string()),
                border: true,
            }));
            
            let mut y = 17;
            for resource in GuildResource::all() {
                let amount = guild.resources.get(&resource).unwrap_or(&0);
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y,
                    text: format!("{}: {}", resource.name(), amount),
                    color: None,
                }));
                y += 1;
            }
            
            // Facilities summary
            ui_elements.push(UIElement::Box(UIBox {
                x: 40,
                y: 16,
                width: 33,
                height: 7,
                title: Some("Facilities".to_string()),
                border: true,
            }));
            
            y = 17;
            let facilities = guild.facilities.iter().take(5);
            for (facility, instance) in facilities {
                ui_elements.push(UIElement::Text(UIText {
                    x: 42,
                    y,
                    text: format!("{} (Level {})", facility.name(), instance.level),
                    color: None,
                }));
                y += 1;
            }
            
            // Recent activity
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 24,
                width: 66,
                height: 7,
                title: Some("Recent Activity".to_string()),
                border: true,
            }));
            
            // In a real implementation, you would show actual recent activity
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 26,
                text: "No recent activity to display.".to_string(),
                color: None,
            }));
            
            // Navigation buttons
            ui_elements.push(UIElement::Button(UIButton {
                x: 7,
                y: 32,
                width: 20,
                height: 1,
                text: "View Members".to_string(),
                action: UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Members))),
                enabled: true,
            }));
            
            ui_elements.push(UIElement::Button(UIButton {
                x: 30,
                y: 32,
                width: 20,
                height: 1,
                text: "View Missions".to_string(),
                action: UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Missions))),
                enabled: true,
            }));
            
            ui_elements.push(UIElement::Button(UIButton {
                x: 53,
                y: 32,
                width: 20,
                height: 1,
                text: "View Facilities".to_string(),
                action: UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Facilities))),
                enabled: true,
            }));
        } else {
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 6,
                text: "No guild selected.".to_string(),
                color: None,
            }));
        }
    } else {
        ui_elements.push(UIElement::Text(UIText {
            x: 7,
            y: 6,
            text: "You are not a member of any guild.".to_string(),
            color: None,
        }));
        
        // List available guilds
        ui_elements.push(UIElement::Box(UIBox {
            x: 7,
            y: 8,
            width: 66,
            height: 10,
            title: Some("Available Guilds".to_string()),
            border: true,
        }));
        
        let mut y = 9;
        for (id, guild) in &guild_manager.guilds {
            ui_elements.push(UIElement::Button(UIButton {
                x: 9,
                y,
                width: 62,
                height: 1,
                text: format!("{} (Level {})", guild.name, guild.level),
                action: UIAction::Custom(Box::new(GuildUIAction::SelectGuild(id.clone()))),
                enabled: true,
            }));
            y += 1;
        }
    }
}

/// Render the members screen
pub fn render_members_screen(
    guild_ui: &GuildUI,
    guild_manager: &GuildManager,
    selected_guild_id: &Option<String>,
    agent_query: &Query<(Entity, &GuildMember, Option<&crate::guild::agent_behavior::AgentBehavior>, Option<&crate::guild::agent_progression::AgentStats>, Option<&crate::guild::agent_progression::AgentProgression>, Option<&crate::guild::mission::MissionTracker>, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild_id) = selected_guild_id {
        if let Some(guild) = guild_manager.get_guild(guild_id) {
            // Guild name
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 6,
                text: format!("Guild: {}", guild.name),
                color: None,
            }));
            
            // Members list
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 8,
                width: 66,
                height: 20,
                title: Some("Members".to_string()),
                border: true,
            }));
            
            // Filter
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 9,
                text: format!("Filter: {}", guild_ui.filter),
                color: None,
            }));
            
            // Headers
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 10,
                text: format!("{:<20} {:<10} {:<10} {:<10}", "Name", "Rank", "Missions", "Status"),
                color: None,
            }));
            
            // Member rows
            let mut y = 11;
            let mut count = 0;
            
            // Collect members for this guild
            let mut members: Vec<(Entity, &GuildMember, Option<&Name>, Option<&crate::guild::mission::MissionTracker>)> = Vec::new();
            for (entity, member, _, _, _, tracker, name) in agent_query.iter() {
                if member.guild_id == *guild_id {
                    members.push((entity, member, name, tracker));
                }
            }
            
            // Apply scroll offset
            let start_idx = guild_ui.scroll_offset.min(members.len().saturating_sub(1));
            let visible_members = members.iter().skip(start_idx).take(15);
            
            for (entity, member, name, tracker) in visible_members {
                let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
                let missions = tracker.map_or(0, |t| t.completed_missions.len());
                let status = if tracker.and_then(|t| t.active_mission.clone()).is_some() {
                    "On Mission"
                } else {
                    "Available"
                };
                
                // Highlight selected member
                let color = if guild_ui.selected_member == Some(*entity) {
                    Some((255, 255, 0))
                } else {
                    None
                };
                
                ui_elements.push(UIElement::Button(UIButton {
                    x: 9,
                    y,
                    width: 62,
                    height: 1,
                    text: format!("{:<20} {:<10} {:<10} {:<10}", name_str, member.rank.name(), missions, status),
                    action: UIAction::Custom(Box::new(GuildUIAction::SelectMember(*entity))),
                    enabled: true,
                }));
                
                y += 1;
                count += 1;
                
                if count >= 15 {
                    break;
                }
            }
            
            // Member details if selected
            if let Some(selected_entity) = guild_ui.selected_member {
                if let Ok((_, member, behavior, stats, progression, tracker, name)) = agent_query.get(selected_entity) {
                    ui_elements.push(UIElement::Box(UIBox {
                        x: 7,
                        y: 29,
                        width: 66,
                        height: 10,
                        title: Some("Member Details".to_string()),
                        border: true,
                    }));
                    
                    let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 30,
                        text: format!("Name: {}", name_str),
                        color: None,
                    }));
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 31,
                        text: format!("Rank: {}", member.rank.name()),
                        color: None,
                    }));
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 32,
                        text: format!("Specialization: {}", member.specialization),
                        color: None,
                    }));
                    
                    if let Some(behavior) = behavior {
                        ui_elements.push(UIElement::Text(UIText {
                            x: 9,
                            y: 33,
                            text: format!("Behavior: {:?}", behavior.behavior_type),
                            color: None,
                        }));
                    }
                    
                    if let Some(stats) = stats {
                        ui_elements.push(UIElement::Text(UIText {
                            x: 40,
                            y: 30,
                            text: format!("Level: {}", stats.level),
                            color: None,
                        }));
                        
                        ui_elements.push(UIElement::Text(UIText {
                            x: 40,
                            y: 31,
                            text: format!("Experience: {}/{}", stats.experience, stats.experience_to_next_level),
                            color: None,
                        }));
                    }
                    
                    if let Some(tracker) = tracker {
                        let mission_status = if let Some(mission_id) = &tracker.active_mission {
                            format!("On mission: {}", mission_id)
                        } else {
                            "Available".to_string()
                        };
                        
                        ui_elements.push(UIElement::Text(UIText {
                            x: 40,
                            y: 32,
                            text: mission_status,
                            color: None,
                        }));
                        
                        ui_elements.push(UIElement::Text(UIText {
                            x: 40,
                            y: 33,
                            text: format!("Completed missions: {}", tracker.completed_missions.len()),
                            color: None,
                        }));
                    }
                    
                    // Action buttons
                    ui_elements.push(UIElement::Button(UIButton {
                        x: 9,
                        y: 35,
                        width: 20,
                        height: 1,
                        text: "Assign Mission".to_string(),
                        action: UIAction::Custom(Box::new(GuildUIAction::SetState(GuildUIState::Missions))),
                        enabled: true,
                    }));
                    
                    ui_elements.push(UIElement::Button(UIButton {
                        x: 32,
                        y: 35,
                        width: 20,
                        height: 1,
                        text: "Configure Agent".to_string(),
                        action: UIAction::Custom(Box::new(GuildUIAction::ConfigureAgent(selected_entity))),
                        enabled: true,
                    }));
                }
            }
        }
    }
}

/// Render the missions screen
pub fn render_missions_screen(
    guild_ui: &GuildUI,
    mission_board: &MissionBoard,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
);

/// Render the facilities screen
pub fn render_facilities_screen(
    guild_ui: &GuildUI,
    guild_manager: &GuildManager,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
);

/// Render the resources screen
pub fn render_resources_screen(
    guild_ui: &GuildUI,
    guild_manager: &GuildManager,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
);

/// Render the agent configuration screen
pub fn render_agent_config_screen(
    guild_ui: &GuildUI,
    agent_query: &Query<(Entity, &GuildMember, Option<&crate::guild::agent_behavior::AgentBehavior>, Option<&crate::guild::agent_progression::AgentStats>, Option<&crate::guild::agent_progression::AgentProgression>, Option<&crate::guild::mission::MissionTracker>, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
);