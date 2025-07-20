use bevy::prelude::*;
use crate::guild::guild_ui_types::{GuildUI, GuildUIState, GuildUIAction};
use crate::guild::guild_core::{Guild, GuildMember, GuildManager, GuildResource, GuildFacility};
use crate::guild::mission::Mission;
use crate::guild::mission_board::MissionBoard;
use crate::guild::agent_behavior::AgentBehaviorType;
use crate::components::{Player, Name};
use crate::ui::{UIState, UIAction, UIElement, UIBox, UIText, UIButton, UIPanel};

/// Render the missions screen
pub fn render_missions_screen(
    guild_ui: &GuildUI,
    mission_board: &MissionBoard,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild_id) = selected_guild_id {
        // Mission list
        ui_elements.push(UIElement::Box(UIBox {
            x: 7,
            y: 6,
            width: 66,
            height: 20,
            title: Some("Available Missions".to_string()),
            border: true,
        }));
        
        // Filter options
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 7,
            text: format!("Show: [{}] Available [{}] Completed [{}] Failed",
                if !guild_ui.show_completed_missions && !guild_ui.show_failed_missions { "X" } else { " " },
                if guild_ui.show_completed_missions { "X" } else { " " },
                if guild_ui.show_failed_missions { "X" } else { " " }),
            color: None,
        }));
        
        // Headers
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 8,
            text: format!("{:<30} {:<10} {:<10} {:<10}", "Name", "Difficulty", "Status", "Progress"),
            color: None,
        }));
        
        // Mission rows
        let mut y = 9;
        let mut count = 0;
        
        // Get missions for this guild
        let guild_missions = mission_board.get_missions_by_guild(guild_id);
        
        // Filter missions based on status
        let filtered_missions: Vec<&Mission> = guild_missions.into_iter()
            .filter(|m| {
                match m.status {
                    crate::guild::mission_types::MissionStatus::Available |
                    crate::guild::mission_types::MissionStatus::Assigned |
                    crate::guild::mission_types::MissionStatus::InProgress => true,
                    crate::guild::mission_types::MissionStatus::Completed => guild_ui.show_completed_missions,
                    crate::guild::mission_types::MissionStatus::Failed |
                    crate::guild::mission_types::MissionStatus::Expired => guild_ui.show_failed_missions,
                }
            })
            .collect();
        
        // Apply scroll offset
        let start_idx = guild_ui.scroll_offset.min(filtered_missions.len().saturating_sub(1));
        let visible_missions = filtered_missions.iter().skip(start_idx).take(15);
        
        for mission in visible_missions {
            // Highlight selected mission
            let color = if guild_ui.selected_mission.as_ref() == Some(&mission.id) {
                Some((255, 255, 0))
            } else {
                None
            };
            
            let progress = if mission.status == crate::guild::mission_types::MissionStatus::InProgress {
                format!("{:.0}%", mission.progress_percentage())
            } else {
                "".to_string()
            };
            
            ui_elements.push(UIElement::Button(UIButton {
                x: 9,
                y,
                width: 62,
                height: 1,
                text: format!("{:<30} {:<10} {:<10} {:<10}", 
                    mission.name, 
                    mission.difficulty.name(), 
                    format!("{:?}", mission.status),
                    progress),
                action: UIAction::Custom(Box::new(GuildUIAction::SelectMission(mission.id.clone()))),
                enabled: true,
            }));
            
            y += 1;
            count += 1;
            
            if count >= 15 {
                break;
            }
        }
        
        // Mission details if selected
        if let Some(mission_id) = &guild_ui.selected_mission {
            if let Some(mission) = mission_board.get_mission(mission_id) {
                ui_elements.push(UIElement::Box(UIBox {
                    x: 7,
                    y: 27,
                    width: 66,
                    height: 12,
                    title: Some("Mission Details".to_string()),
                    border: true,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 28,
                    text: format!("Name: {}", mission.name),
                    color: None,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 29,
                    text: format!("Difficulty: {}", mission.difficulty.name()),
                    color: None,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 30,
                    text: format!("Status: {:?}", mission.status),
                    color: None,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 31,
                    text: format!("Description: {}", mission.description),
                    color: None,
                }));
                
                // Objectives
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 32,
                    text: "Objectives:".to_string(),
                    color: None,
                }));
                
                let mut obj_y = 33;
                for (i, objective) in mission.objectives.iter().enumerate().take(3) {
                    let progress = match &objective.status {
                        crate::guild::mission_types::MissionObjectiveStatus::InProgress { current, total } => {
                            format!("{}/{}", current, total)
                        },
                        crate::guild::mission_types::MissionObjectiveStatus::Completed => "Completed".to_string(),
                        crate::guild::mission_types::MissionObjectiveStatus::Failed => "Failed".to_string(),
                        _ => "Not Started".to_string(),
                    };
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: obj_y,
                        text: format!("{}. {} - {}", i + 1, objective.objective_type.description(), progress),
                        color: None,
                    }));
                    
                    obj_y += 1;
                }
                
                // Rewards
                ui_elements.push(UIElement::Text(UIText {
                    x: 40,
                    y: 32,
                    text: "Rewards:".to_string(),
                    color: None,
                }));
                
                let mut reward_y = 33;
                for (i, reward) in mission.rewards.iter().enumerate().take(3) {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 42,
                        y: reward_y,
                        text: format!("{}. {}", i + 1, reward.description()),
                        color: None,
                    }));
                    
                    reward_y += 1;
                }
                
                // Action buttons
                if mission.status == crate::guild::mission_types::MissionStatus::Available {
                    if let Some(selected_member) = guild_ui.selected_member {
                        ui_elements.push(UIElement::Button(UIButton {
                            x: 9,
                            y: 37,
                            width: 30,
                            height: 1,
                            text: "Assign to Selected Member".to_string(),
                            action: UIAction::Custom(Box::new(GuildUIAction::AssignMission(selected_member, mission.id.clone()))),
                            enabled: true,
                        }));
                    } else {
                        ui_elements.push(UIElement::Text(UIText {
                            x: 9,
                            y: 37,
                            text: "Select a member to assign this mission".to_string(),
                            color: None,
                        }));
                    }
                }
            }
        }
    }
}

/// Render the facilities screen
pub fn render_facilities_screen(
    guild_ui: &GuildUI,
    guild_manager: &GuildManager,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild_id) = selected_guild_id {
        if let Some(guild) = guild_manager.get_guild(guild_id) {
            // Facilities list
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 20,
                title: Some("Guild Facilities".to_string()),
                border: true,
            }));
            
            // Headers
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: format!("{:<20} {:<10} {:<10} {:<20}", "Facility", "Level", "Staff", "Status"),
                color: None,
            }));
            
            // Facility rows
            let mut y = 8;
            let mut count = 0;
            
            // Get all possible facilities
            let all_facilities = GuildFacility::all();
            
            // Apply scroll offset
            let start_idx = guild_ui.scroll_offset.min(all_facilities.len().saturating_sub(1));
            let visible_facilities = all_facilities.iter().skip(start_idx).take(15);
            
            for facility in visible_facilities {
                let (level, staff, status) = if let Some(instance) = guild.facilities.get(facility) {
                    (instance.level, instance.staff.len(), "Built")
                } else {
                    (0, 0, "Not Built")
                };
                
                // Highlight selected facility
                let color = if guild_ui.selected_facility == Some(*facility) {
                    Some((255, 255, 0))
                } else {
                    None
                };
                
                ui_elements.push(UIElement::Button(UIButton {
                    x: 9,
                    y,
                    width: 62,
                    height: 1,
                    text: format!("{:<20} {:<10} {:<10} {:<20}", facility.name(), level, staff, status),
                    action: UIAction::Custom(Box::new(GuildUIAction::SelectFacility(*facility))),
                    enabled: true,
                }));
                
                y += 1;
                count += 1;
                
                if count >= 15 {
                    break;
                }
            }
            
            // Facility details if selected
            if let Some(facility) = guild_ui.selected_facility {
                ui_elements.push(UIElement::Box(UIBox {
                    x: 7,
                    y: 27,
                    width: 66,
                    height: 10,
                    title: Some("Facility Details".to_string()),
                    border: true,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 28,
                    text: format!("Name: {}", facility.name()),
                    color: None,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 29,
                    text: format!("Description: {}", facility.description()),
                    color: None,
                }));
                
                // Show facility instance details if built
                if let Some(instance) = guild.facilities.get(&facility) {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 30,
                        text: format!("Level: {}", instance.level),
                        color: None,
                    }));
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 31,
                        text: format!("Staff: {}", instance.staff.len()),
                        color: None,
                    }));
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 32,
                        text: format!("Effectiveness: {:.1}", instance.effectiveness()),
                        color: None,
                    }));
                    
                    // Upgrade button
                    if instance.level < 5 {
                        ui_elements.push(UIElement::Button(UIButton {
                            x: 9,
                            y: 34,
                            width: 20,
                            height: 1,
                            text: "Upgrade Facility".to_string(),
                            action: UIAction::Custom(Box::new(GuildUIAction::UpgradeFacility(facility))),
                            enabled: true,
                        }));
                        
                        // Show upgrade cost
                        let mut cost_text = "Cost: ".to_string();
                        let base_cost = facility.build_cost();
                        let level_multiplier = instance.level as f32 * 0.5 + 1.0;
                        
                        for (resource, amount) in &base_cost {
                            let upgraded_amount = (*amount as f32 * level_multiplier) as u32;
                            cost_text.push_str(&format!("{} {}, ", upgraded_amount, resource.name()));
                        }
                        
                        // Remove trailing comma and space
                        if cost_text.ends_with(", ") {
                            cost_text.truncate(cost_text.len() - 2);
                        }
                        
                        ui_elements.push(UIElement::Text(UIText {
                            x: 9,
                            y: 35,
                            text: cost_text,
                            color: None,
                        }));
                    }
                } else {
                    // Build button
                    ui_elements.push(UIElement::Button(UIButton {
                        x: 9,
                        y: 34,
                        width: 20,
                        height: 1,
                        text: "Build Facility".to_string(),
                        action: UIAction::Custom(Box::new(GuildUIAction::BuildFacility(facility))),
                        enabled: true,
                    }));
                    
                    // Show build cost
                    let mut cost_text = "Cost: ".to_string();
                    let cost = facility.build_cost();
                    
                    for (resource, amount) in &cost {
                        cost_text.push_str(&format!("{} {}, ", amount, resource.name()));
                    }
                    
                    // Remove trailing comma and space
                    if cost_text.ends_with(", ") {
                        cost_text.truncate(cost_text.len() - 2);
                    }
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 35,
                        text: cost_text,
                        color: None,
                    }));
                }
            }
        }
    }
}

/// Render the resources screen
pub fn render_resources_screen(
    guild_ui: &GuildUI,
    guild_manager: &GuildManager,
    selected_guild_id: &Option<String>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild_id) = selected_guild_id {
        if let Some(guild) = guild_manager.get_guild(guild_id) {
            // Resources list
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 15,
                title: Some("Guild Resources".to_string()),
                border: true,
            }));
            
            // Headers
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: format!("{:<20} {:<10} {:<30}", "Resource", "Amount", "Description"),
                color: None,
            }));
            
            // Resource rows
            let mut y = 8;
            for resource in GuildResource::all() {
                let amount = guild.resources.get(&resource).unwrap_or(&0);
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y,
                    text: format!("{:<20} {:<10} {:<30}", resource.name(), amount, ""),
                    color: None,
                }));
                
                y += 1;
            }
            
            // Resource income/expenses
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 22,
                width: 66,
                height: 10,
                title: Some("Resource Flow".to_string()),
                border: true,
            }));
            
            // In a real implementation, you would show actual resource flow
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 24,
                text: "No resource flow data available.".to_string(),
                color: None,
            }));
        }
    }
}

/// Render the agent configuration screen
pub fn render_agent_config_screen(
    guild_ui: &GuildUI,
    agent_query: &Query<(Entity, &GuildMember, Option<&crate::guild::agent_behavior::AgentBehavior>, Option<&crate::guild::agent_progression::AgentStats>, Option<&crate::guild::agent_progression::AgentProgression>, Option<&crate::guild::mission::MissionTracker>, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(entity) = guild_ui.selected_member {
        if let Ok((_, member, behavior, stats, progression, tracker, name)) = agent_query.get(entity) {
            let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
            
            // Agent info
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 6,
                text: format!("Agent: {}", name_str),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 7,
                text: format!("Specialization: {}", member.specialization),
                color: None,
            }));
            
            // Behavior configuration
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 9,
                width: 66,
                height: 10,
                title: Some("Behavior Configuration".to_string()),
                border: true,
            }));
            
            let current_behavior = behavior.map_or(AgentBehaviorType::Balanced, |b| b.behavior_type);
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 10,
                text: format!("Current Behavior: {:?}", current_behavior),
                color: None,
            }));
            
            // Behavior options
            let behaviors = [
                AgentBehaviorType::Aggressive,
                AgentBehaviorType::Cautious,
                AgentBehaviorType::Balanced,
                AgentBehaviorType::Thorough,
                AgentBehaviorType::Speedy,
                AgentBehaviorType::ResourceFocused,
                AgentBehaviorType::Protective,
            ];
            
            let mut y = 12;
            for behavior_type in behaviors {
                let selected = if behavior_type == current_behavior {
                    "[X] "
                } else {
                    "[ ] "
                };
                
                ui_elements.push(UIElement::Button(UIButton {
                    x: 9,
                    y,
                    width: 30,
                    height: 1,
                    text: format!("{}{:?}", selected, behavior_type),
                    action: UIAction::Custom(Box::new(GuildUIAction::SetAgentBehavior(entity, behavior_type))),
                    enabled: true,
                }));
                
                y += 1;
            }
            
            // Stats configuration
            if let Some(stats) = stats {
                ui_elements.push(UIElement::Box(UIBox {
                    x: 7,
                    y: 20,
                    width: 66,
                    height: 15,
                    title: Some("Stats Configuration".to_string()),
                    border: true,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 21,
                    text: format!("Level: {} | Experience: {}/{}", stats.level, stats.experience, stats.experience_to_next_level),
                    color: None,
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 22,
                    text: format!("Available Stat Points: {}", stats.available_stat_points),
                    color: None,
                }));
                
                // Attribute list
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 24,
                    text: "Attributes:".to_string(),
                    color: None,
                }));
                
                let mut attr_y = 25;
                for (attr, value) in &stats.attributes {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: attr_y,
                        text: format!("{}: {}", attr, value),
                        color: None,
                    }));
                    
                    // Add upgrade button if points available
                    if stats.available_stat_points > 0 {
                        ui_elements.push(UIElement::Button(UIButton {
                            x: 30,
                            y: attr_y,
                            width: 10,
                            height: 1,
                            text: "Upgrade".to_string(),
                            action: UIAction::Custom(Box::new(GuildUIAction::UpgradeAgentStat(entity, attr.clone()))),
                            enabled: true,
                        }));
                    }
                    
                    attr_y += 1;
                }
                
                // Skills list
                ui_elements.push(UIElement::Text(UIText {
                    x: 45,
                    y: 24,
                    text: "Skills:".to_string(),
                    color: None,
                }));
                
                let mut skill_y = 25;
                for (skill, value) in &stats.skills {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 47,
                        y: skill_y,
                        text: format!("{}: {}", skill, value),
                        color: None,
                    }));
                    
                    skill_y += 1;
                }
            }
        }
    }
}