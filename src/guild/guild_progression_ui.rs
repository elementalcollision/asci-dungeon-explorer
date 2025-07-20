use bevy::prelude::*;
use crate::guild::guild_core::{Guild, GuildManager, GuildFacility};
use crate::guild::guild_progression::{GuildProgression, GuildUpgrade, FacilityUpgrade, GuildSpecialization, GuildPerk};
use crate::ui::{UIState, UIAction, UIElement, UIBox, UIText, UIButton, UIPanel};

/// Guild progression UI state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GuildProgressionUIState {
    Hidden,
    Overview,
    Upgrades,
    Facilities,
    Milestones,
    Specialization,
}

/// Guild progression UI resource
#[derive(Resource)]
pub struct GuildProgressionUI {
    pub state: GuildProgressionUIState,
    pub selected_guild: Option<String>,
    pub selected_upgrade: Option<String>,
    pub selected_facility: Option<GuildFacility>,
    pub scroll_offset: usize,
}

impl Default for GuildProgressionUI {
    fn default() -> Self {
        GuildProgressionUI {
            state: GuildProgressionUIState::Hidden,
            selected_guild: None,
            selected_upgrade: None,
            selected_facility: None,
            scroll_offset: 0,
        }
    }
}

/// Guild progression UI action
#[derive(Debug, Clone)]
pub enum GuildProgressionUIAction {
    ToggleUI,
    SetState(GuildProgressionUIState),
    SelectGuild(String),
    SelectUpgrade(String),
    SelectFacility(GuildFacility),
    ApplyUpgrade(String),
    ApplyFacilityUpgrade(GuildFacility, usize),
    SetSpecialization(GuildSpecialization),
    ScrollUp,
    ScrollDown,
}

/// System for handling guild progression UI input
pub fn guild_progression_ui_input_system(
    mut progression_ui: ResMut<GuildProgressionUI>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ui_actions: EventWriter<UIAction>,
) {
    // Toggle UI with 'U' key (for Upgrades)
    if keyboard_input.just_pressed(KeyCode::U) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::ToggleUI)));
    }
    
    // Only process other inputs if UI is visible
    if progression_ui.state == GuildProgressionUIState::Hidden {
        return;
    }
    
    // Navigation between UI states
    if keyboard_input.just_pressed(KeyCode::Key1) || keyboard_input.just_pressed(KeyCode::Numpad1) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::SetState(GuildProgressionUIState::Overview))));
    }
    if keyboard_input.just_pressed(KeyCode::Key2) || keyboard_input.just_pressed(KeyCode::Numpad2) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::SetState(GuildProgressionUIState::Upgrades))));
    }
    if keyboard_input.just_pressed(KeyCode::Key3) || keyboard_input.just_pressed(KeyCode::Numpad3) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::SetState(GuildProgressionUIState::Facilities))));
    }
    if keyboard_input.just_pressed(KeyCode::Key4) || keyboard_input.just_pressed(KeyCode::Numpad4) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::SetState(GuildProgressionUIState::Milestones))));
    }
    if keyboard_input.just_pressed(KeyCode::Key5) || keyboard_input.just_pressed(KeyCode::Numpad5) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::SetState(GuildProgressionUIState::Specialization))));
    }
    
    // Scrolling
    if keyboard_input.just_pressed(KeyCode::Up) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::ScrollUp)));
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::ScrollDown)));
    }
    
    // Close UI with Escape
    if keyboard_input.just_pressed(KeyCode::Escape) {
        ui_actions.send(UIAction::Custom(Box::new(GuildProgressionUIAction::SetState(GuildProgressionUIState::Hidden))));
    }
}/// 
System for handling guild progression UI actions
pub fn guild_progression_ui_action_system(
    mut progression_ui: ResMut<GuildProgressionUI>,
    mut guild_manager: ResMut<GuildManager>,
    mut ui_actions: EventReader<UIAction>,
) {
    for action in ui_actions.iter() {
        if let UIAction::Custom(custom_action) = action {
            if let Some(progression_action) = custom_action.downcast_ref::<GuildProgressionUIAction>() {
                match progression_action {
                    GuildProgressionUIAction::ToggleUI => {
                        if progression_ui.state == GuildProgressionUIState::Hidden {
                            progression_ui.state = GuildProgressionUIState::Overview;
                        } else {
                            progression_ui.state = GuildProgressionUIState::Hidden;
                        }
                    },
                    GuildProgressionUIAction::SetState(state) => {
                        progression_ui.state = state.clone();
                    },
                    GuildProgressionUIAction::SelectGuild(guild_id) => {
                        progression_ui.selected_guild = Some(guild_id.clone());
                    },
                    GuildProgressionUIAction::SelectUpgrade(upgrade_id) => {
                        progression_ui.selected_upgrade = Some(upgrade_id.clone());
                    },
                    GuildProgressionUIAction::SelectFacility(facility) => {
                        progression_ui.selected_facility = Some(*facility);
                    },
                    GuildProgressionUIAction::ApplyUpgrade(upgrade_id) => {
                        if let Some(guild_id) = &progression_ui.selected_guild {
                            if let Some(guild) = guild_manager.get_guild_mut(guild_id) {
                                if let Some(progression) = guild.get_component_mut::<GuildProgression>() {
                                    if let Err(err) = progression.apply_upgrade(upgrade_id, guild) {
                                        error!("Failed to apply upgrade: {}", err);
                                    }
                                }
                            }
                        }
                    },
                    GuildProgressionUIAction::ApplyFacilityUpgrade(facility, upgrade_index) => {
                        if let Some(guild_id) = &progression_ui.selected_guild {
                            if let Some(guild) = guild_manager.get_guild_mut(guild_id) {
                                if let Some(progression) = guild.get_component_mut::<GuildProgression>() {
                                    if let Err(err) = progression.apply_facility_upgrade(*facility, *upgrade_index, guild) {
                                        error!("Failed to apply facility upgrade: {}", err);
                                    }
                                }
                            }
                        }
                    },
                    GuildProgressionUIAction::SetSpecialization(specialization) => {
                        if let Some(guild_id) = &progression_ui.selected_guild {
                            if let Some(guild) = guild_manager.get_guild_mut(guild_id) {
                                if let Some(progression) = guild.get_component_mut::<GuildProgression>() {
                                    progression.set_specialization(*specialization);
                                }
                            }
                        }
                    },
                    GuildProgressionUIAction::ScrollUp => {
                        if progression_ui.scroll_offset > 0 {
                            progression_ui.scroll_offset -= 1;
                        }
                    },
                    GuildProgressionUIAction::ScrollDown => {
                        progression_ui.scroll_offset += 1;
                    },
                }
            }
        }
    }
}

/// System for rendering guild progression UI
pub fn guild_progression_ui_render_system(
    progression_ui: Res<GuildProgressionUI>,
    guild_manager: Res<GuildManager>,
    mut ui_elements: ResMut<Vec<UIElement>>,
) {
    // Only render if UI is visible
    if progression_ui.state == GuildProgressionUIState::Hidden {
        return;
    }
    
    // Main container
    ui_elements.push(UIElement::Panel(UIPanel {
        x: 5,
        y: 2,
        width: 70,
        height: 40,
        title: "Guild Progression".to_string(),
        border: true,
    }));
    
    // Navigation tabs
    ui_elements.push(UIElement::Text(UIText {
        x: 7,
        y: 4,
        text: format!("[1] Overview | [2] Upgrades | [3] Facilities | [4] Milestones | [5] Specialization"),
        color: None,
    }));
    
    // Get selected guild
    let selected_guild = if let Some(guild_id) = &progression_ui.selected_guild {
        guild_manager.get_guild(guild_id)
    } else if !guild_manager.guilds.is_empty() {
        let first_guild_id = guild_manager.guilds.keys().next().unwrap();
        guild_manager.get_guild(first_guild_id)
    } else {
        None
    };
    
    // Render appropriate content based on state
    match progression_ui.state {
        GuildProgressionUIState::Overview => render_overview_screen(&progression_ui, selected_guild, &mut ui_elements),
        GuildProgressionUIState::Upgrades => render_upgrades_screen(&progression_ui, selected_guild, &mut ui_elements),
        GuildProgressionUIState::Facilities => render_facilities_screen(&progression_ui, selected_guild, &mut ui_elements),
        GuildProgressionUIState::Milestones => render_milestones_screen(&progression_ui, selected_guild, &mut ui_elements),
        GuildProgressionUIState::Specialization => render_specialization_screen(&progression_ui, selected_guild, &mut ui_elements),
        _ => {}
    }
    
    // Footer
    ui_elements.push(UIElement::Text(UIText {
        x: 7,
        y: 41,
        text: format!("[U] Close | [↑/↓] Scroll"),
        color: None,
    }));
}/
// Render overview screen
fn render_overview_screen(
    progression_ui: &GuildProgressionUI,
    selected_guild: Option<&Guild>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild) = selected_guild {
        // Get progression component
        if let Some(progression) = guild.get_component::<GuildProgression>() {
            // Guild info
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 3,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: format!("Guild: {} (Level {})", guild.name, progression.level),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 8,
                text: format!("Experience: {}/{} | Reputation Level: {}", 
                    progression.experience, progression.experience_to_next_level, progression.reputation_level),
                color: None,
            }));
            
            // Resources
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 10,
                width: 66,
                height: 5,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 11,
                text: "Resources:".to_string(),
                color: None,
            }));
            
            let gold = guild.resources.get(&crate::guild::guild_core::GuildResource::Gold).copied().unwrap_or(0);
            let supplies = guild.resources.get(&crate::guild::guild_core::GuildResource::Supplies).copied().unwrap_or(0);
            let magic_essence = guild.resources.get(&crate::guild::guild_core::GuildResource::MagicEssence).copied().unwrap_or(0);
            let rare_artifacts = guild.resources.get(&crate::guild::guild_core::GuildResource::RareArtifacts).copied().unwrap_or(0);
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 12,
                text: format!("Gold: {} | Supplies: {}", gold, supplies),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 13,
                text: format!("Magic Essence: {} | Rare Artifacts: {}", magic_essence, rare_artifacts),
                color: None,
            }));
            
            // Specialization
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 16,
                width: 66,
                height: 3,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 17,
                text: format!("Specialization: {:?}", progression.specialization),
                color: None,
            }));
            
            // Active perks
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 20,
                width: 66,
                height: 10,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 21,
                text: "Active Perks:".to_string(),
                color: None,
            }));
            
            let mut y = 22;
            for perk in &progression.perks {
                if y < 29 {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y,
                        text: format!("- {:?}", perk),
                        color: None,
                    }));
                    y += 1;
                }
            }
            
            if progression.perks.is_empty() {
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 22,
                    text: "No active perks".to_string(),
                    color: None,
                }));
            }
        } else {
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 6,
                text: "Guild progression data not available".to_string(),
                color: None,
            }));
        }
    } else {
        ui_elements.push(UIElement::Text(UIText {
            x: 7,
            y: 6,
            text: "No guild selected".to_string(),
            color: None,
        }));
    }
}

/// Render upgrades screen
fn render_upgrades_screen(
    progression_ui: &GuildProgressionUI,
    selected_guild: Option<&Guild>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild) = selected_guild {
        // Get progression component
        if let Some(progression) = guild.get_component::<GuildProgression>() {
            // Available upgrades
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 15,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: "Available Upgrades:".to_string(),
                color: None,
            }));
            
            let available_upgrades = progression.get_available_guild_upgrades();
            
            if available_upgrades.is_empty() {
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 9,
                    text: "No upgrades available".to_string(),
                    color: None,
                }));
            } else {
                let start_idx = progression_ui.scroll_offset.min(available_upgrades.len().saturating_sub(1));
                let end_idx = (start_idx + 5).min(available_upgrades.len());
                
                for (i, upgrade) in available_upgrades[start_idx..end_idx].iter().enumerate() {
                    let y_pos = 9 + i * 3;
                    
                    // Highlight selected upgrade
                    let is_selected = progression_ui.selected_upgrade.as_ref().map_or(false, |id| id == &upgrade.id);
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: y_pos,
                        text: format!("{}. {} (Level: {}, Rep: {})", 
                            i + 1, 
                            upgrade.name, 
                            upgrade.level_requirement,
                            upgrade.reputation_requirement),
                        color: if is_selected { Some(Color::YELLOW) } else { None },
                    }));
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: y_pos + 1,
                        text: upgrade.description.clone(),
                        color: None,
                    }));
                    
                    // Cost display
                    let mut cost_text = "Cost: ".to_string();
                    for (resource, amount) in &upgrade.cost {
                        cost_text.push_str(&format!("{}: {} ", resource.name(), amount));
                    }
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: y_pos + 2,
                        text: cost_text,
                        color: None,
                    }));
                }
            }
            
            // Applied upgrades
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 22,
                width: 66,
                height: 10,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 23,
                text: "Applied Upgrades:".to_string(),
                color: None,
            }));
            
            if progression.applied_upgrades.is_empty() {
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 25,
                    text: "No upgrades applied yet".to_string(),
                    color: None,
                }));
            } else {
                for (i, upgrade) in progression.applied_upgrades.iter().enumerate().take(7) {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: 25 + i,
                        text: format!("- {}", upgrade.name),
                        color: None,
                    }));
                }
            }
            
            // Controls
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 33,
                text: "[1-5] Select upgrade | [Enter] Apply upgrade".to_string(),
                color: None,
            }));
        }
    }
}//
/ Render facilities screen
fn render_facilities_screen(
    progression_ui: &GuildProgressionUI,
    selected_guild: Option<&Guild>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild) = selected_guild {
        // Get progression component
        if let Some(progression) = guild.get_component::<GuildProgression>() {
            // Unlocked facilities
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 15,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: "Unlocked Facilities:".to_string(),
                color: None,
            }));
            
            let mut y = 9;
            for facility in &progression.unlocked_facilities {
                let is_built = guild.facilities.contains_key(facility);
                let is_selected = progression_ui.selected_facility == Some(*facility);
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y,
                    text: format!("{} - {}{}", 
                        facility.name(),
                        if is_built { 
                            format!("Level {}", guild.facilities.get(facility).map_or(0, |f| f.level)) 
                        } else { 
                            "Not Built".to_string() 
                        },
                        if is_selected { " (Selected)" } else { "" }
                    ),
                    color: if is_selected { Some(Color::YELLOW) } else { None },
                }));
                
                y += 1;
                
                // Show available upgrades for selected facility
                if is_selected && is_built {
                    let available_upgrades = progression.get_available_facility_upgrades(*facility);
                    
                    if !available_upgrades.is_empty() {
                        ui_elements.push(UIElement::Text(UIText {
                            x: 11,
                            y,
                            text: "Available Upgrades:".to_string(),
                            color: None,
                        }));
                        
                        y += 1;
                        
                        for (i, upgrade) in available_upgrades.iter().enumerate().take(3) {
                            ui_elements.push(UIElement::Text(UIText {
                                x: 13,
                                y,
                                text: format!("{}. {} (Level {})", i + 1, upgrade.name, upgrade.level_requirement),
                                color: None,
                            }));
                            
                            y += 1;
                        }
                    } else {
                        ui_elements.push(UIElement::Text(UIText {
                            x: 11,
                            y,
                            text: "No upgrades available".to_string(),
                            color: None,
                        }));
                        
                        y += 1;
                    }
                }
                
                y += 1;
            }
            
            // Locked facilities
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 22,
                width: 66,
                height: 10,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 23,
                text: "Locked Facilities:".to_string(),
                color: None,
            }));
            
            y = 25;
            for facility in GuildFacility::all() {
                if !progression.unlocked_facilities.contains(&facility) {
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y,
                        text: format!("{} - Locked", facility.name()),
                        color: None,
                    }));
                    
                    y += 1;
                }
            }
            
            // Controls
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 33,
                text: "[1-9] Select facility | [B] Build facility | [U] Upgrade facility".to_string(),
                color: None,
            }));
        }
    }
}

/// Render milestones screen
fn render_milestones_screen(
    progression_ui: &GuildProgressionUI,
    selected_guild: Option<&Guild>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild) = selected_guild {
        // Get progression component
        if let Some(progression) = guild.get_component::<GuildProgression>() {
            // Milestones
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 25,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: "Guild Milestones:".to_string(),
                color: None,
            }));
            
            if progression.milestones.is_empty() {
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 9,
                    text: "No milestones available".to_string(),
                    color: None,
                }));
            } else {
                let start_idx = progression_ui.scroll_offset.min(progression.milestones.len().saturating_sub(1));
                let end_idx = (start_idx + 3).min(progression.milestones.len());
                
                for (i, milestone) in progression.milestones[start_idx..end_idx].iter().enumerate() {
                    let y_pos = 9 + i * 7;
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 9,
                        y: y_pos,
                        text: format!("{}. {} {}", 
                            i + 1, 
                            milestone.name,
                            if milestone.is_completed { "[COMPLETED]" } else { "" }),
                        color: if milestone.is_completed { Some(Color::GREEN) } else { None },
                    }));
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: y_pos + 1,
                        text: milestone.description.clone(),
                        color: None,
                    }));
                    
                    // Requirements
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: y_pos + 2,
                        text: "Requirements:".to_string(),
                        color: None,
                    }));
                    
                    for (j, req) in milestone.requirements.iter().enumerate().take(2) {
                        let req_text = match req {
                            crate::guild::guild_progression::MilestoneRequirement::GuildLevel(level) => 
                                format!("Guild Level {}", level),
                            crate::guild::guild_progression::MilestoneRequirement::MembersCount(count) => 
                                format!("{} Guild Members", count),
                            crate::guild::guild_progression::MilestoneRequirement::CompletedMissions(count) => 
                                format!("{} Completed Missions", count),
                            crate::guild::guild_progression::MilestoneRequirement::ReputationLevel(level) => 
                                format!("Reputation Level {}", level),
                            crate::guild::guild_progression::MilestoneRequirement::FacilityLevel(facility, level) => 
                                format!("{} Level {}", facility.name(), level),
                            crate::guild::guild_progression::MilestoneRequirement::ResourceAmount(resource, amount) => 
                                format!("{} {}", amount, resource.name()),
                            crate::guild::guild_progression::MilestoneRequirement::SpecificAchievement(id) => 
                                format!("Achievement: {}", id),
                        };
                        
                        ui_elements.push(UIElement::Text(UIText {
                            x: 13,
                            y: y_pos + 3 + j,
                            text: format!("- {}", req_text),
                            color: None,
                        }));
                    }
                    
                    // Rewards
                    ui_elements.push(UIElement::Text(UIText {
                        x: 11,
                        y: y_pos + 5,
                        text: "Rewards: ".to_string(),
                        color: None,
                    }));
                    
                    let mut rewards_text = String::new();
                    for reward in &milestone.rewards {
                        match reward {
                            crate::guild::guild_progression::MilestoneReward::Experience(amount) => {
                                rewards_text.push_str(&format!("{} XP, ", amount));
                            },
                            crate::guild::guild_progression::MilestoneReward::Reputation(amount) => {
                                rewards_text.push_str(&format!("{} Rep, ", amount));
                            },
                            crate::guild::guild_progression::MilestoneReward::Resources(resource, amount) => {
                                rewards_text.push_str(&format!("{} {}, ", amount, resource.name()));
                            },
                            crate::guild::guild_progression::MilestoneReward::UnlockFacility(facility) => {
                                rewards_text.push_str(&format!("Unlock {}, ", facility.name()));
                            },
                            _ => {}
                        }
                    }
                    
                    if !rewards_text.is_empty() {
                        rewards_text.truncate(rewards_text.len() - 2); // Remove trailing comma and space
                    }
                    
                    ui_elements.push(UIElement::Text(UIText {
                        x: 13,
                        y: y_pos + 6,
                        text: rewards_text,
                        color: None,
                    }));
                }
            }
        }
    }
}/// Re
nder specialization screen
fn render_specialization_screen(
    progression_ui: &GuildProgressionUI,
    selected_guild: Option<&Guild>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(guild) = selected_guild {
        // Get progression component
        if let Some(progression) = guild.get_component::<GuildProgression>() {
            // Current specialization
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 6,
                width: 66,
                height: 3,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 7,
                text: format!("Current Specialization: {:?}", progression.specialization),
                color: None,
            }));
            
            // Available specializations
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 10,
                width: 66,
                height: 20,
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 11,
                text: "Available Specializations:".to_string(),
                color: None,
            }));
            
            // Check if specialization is available (requires level 5)
            if progression.level < 5 {
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 13,
                    text: format!("Guild specialization unlocks at level 5 (current: {})", progression.level),
                    color: None,
                }));
            } else {
                // Combat specialization
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 13,
                    text: "1. Combat".to_string(),
                    color: if progression.specialization == GuildSpecialization::Combat { Some(Color::YELLOW) } else { None },
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 11,
                    y: 14,
                    text: "Focuses on combat effectiveness. +15% to combat stats, unlocks special combat training.".to_string(),
                    color: None,
                }));
                
                // Exploration specialization
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 16,
                    text: "2. Exploration".to_string(),
                    color: if progression.specialization == GuildSpecialization::Exploration { Some(Color::YELLOW) } else { None },
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 11,
                    y: 17,
                    text: "Focuses on exploration. Fast travel between discovered locations, better loot finding.".to_string(),
                    color: None,
                }));
                
                // Crafting specialization
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 19,
                    text: "3. Crafting".to_string(),
                    color: if progression.specialization == GuildSpecialization::Crafting { Some(Color::YELLOW) } else { None },
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 11,
                    y: 20,
                    text: "Focuses on crafting. +25% crafting success rate, unlocks special recipes.".to_string(),
                    color: None,
                }));
                
                // Trading specialization
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 22,
                    text: "4. Trading".to_string(),
                    color: if progression.specialization == GuildSpecialization::Trading { Some(Color::YELLOW) } else { None },
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 11,
                    y: 23,
                    text: "Focuses on trading. Better prices at merchants, access to special markets.".to_string(),
                    color: None,
                }));
                
                // Research specialization
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 25,
                    text: "5. Research".to_string(),
                    color: if progression.specialization == GuildSpecialization::Research { Some(Color::YELLOW) } else { None },
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 11,
                    y: 26,
                    text: "Focuses on research. +20% experience gain, faster skill learning.".to_string(),
                    color: None,
                }));
                
                // Balanced specialization
                ui_elements.push(UIElement::Text(UIText {
                    x: 9,
                    y: 28,
                    text: "6. Balanced".to_string(),
                    color: if progression.specialization == GuildSpecialization::Balanced { Some(Color::YELLOW) } else { None },
                }));
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 11,
                    y: 29,
                    text: "No specific focus. +10% to all resource production, minor bonuses to all activities.".to_string(),
                    color: None,
                }));
            }
            
            // Controls
            ui_elements.push(UIElement::Text(UIText {
                x: 7,
                y: 31,
                text: "[1-6] Select specialization | [Enter] Confirm selection".to_string(),
                color: None,
            }));
        }
    }
}

/// Plugin for guild progression UI
pub struct GuildProgressionUIPlugin;

impl Plugin for GuildProgressionUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuildProgressionUI>()
           .add_systems(Update, (
               guild_progression_ui_input_system,
               guild_progression_ui_action_system,
               guild_progression_ui_render_system,
           ).chain());
    }
}