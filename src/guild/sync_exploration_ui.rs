use bevy::prelude::*;
use crate::guild::synchronous_exploration::{
    SyncExplorationManager, SyncExplorationMode, PartyMember, PartyFormation, PartyRole
};
use crate::components::{Position, Player, Health, Name};
use crate::ui::{UIState, UIAction, UIElement, UIBox, UIText, UIButton, UIPanel};

/// Synchronous exploration UI state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SyncExplorationUIState {
    Hidden,
    PartyStatus,
    FormationConfig,
    TurnOrder,
    CooperativeActions,
}

/// Synchronous exploration UI resource
#[derive(Resource)]
pub struct SyncExplorationUI {
    pub state: SyncExplorationUIState,
    pub show_party_status: bool,
    pub show_turn_indicator: bool,
    pub show_action_points: bool,
    pub show_formation_overlay: bool,
}

impl Default for SyncExplorationUI {
    fn default() -> Self {
        SyncExplorationUI {
            state: SyncExplorationUIState::Hidden,
            show_party_status: true,
            show_turn_indicator: true,
            show_action_points: true,
            show_formation_overlay: false,
        }
    }
}

/// Synchronous exploration UI action
#[derive(Debug, Clone)]
pub enum SyncExplorationUIAction {
    ToggleUI,
    SetState(SyncExplorationUIState),
    TogglePartyStatus,
    ToggleTurnIndicator,
    ToggleActionPoints,
    ToggleFormationOverlay,
    SetFormation(PartyFormation),
    ToggleTurnBasedMode,
    ToggleAutoFollow,
    ToggleSharedVision,
    ToggleSharedInventory,
    ToggleCooperativeActions,
    SwitchToNextMember,
    SwitchToPreviousMember,
    EndTurn,
}

/// System for handling synchronous exploration UI input
pub fn sync_exploration_ui_input_system(
    mut sync_ui: ResMut<SyncExplorationUI>,
    mut sync_manager: ResMut<SyncExplorationManager>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ui_actions: EventWriter<UIAction>,
) {
    // Toggle UI with 'P' key (for Party)
    if keyboard_input.just_pressed(KeyCode::P) {
        ui_actions.send(UIAction::Custom(Box::new(SyncExplorationUIAction::ToggleUI)));
    }
    
    // Quick toggles
    if keyboard_input.just_pressed(KeyCode::F) {
        ui_actions.send(UIAction::Custom(Box::new(SyncExplorationUIAction::ToggleFormationOverlay)));
    }
    
    if keyboard_input.just_pressed(KeyCode::T) {
        ui_actions.send(UIAction::Custom(Box::new(SyncExplorationUIAction::ToggleTurnBasedMode)));
    }
    
    // Party member switching (handled in party_control_system but we can also handle UI updates here)
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if keyboard_input.pressed(KeyCode::LShift) || keyboard_input.pressed(KeyCode::RShift) {
            ui_actions.send(UIAction::Custom(Box::new(SyncExplorationUIAction::SwitchToPreviousMember)));
        } else {
            ui_actions.send(UIAction::Custom(Box::new(SyncExplorationUIAction::SwitchToNextMember)));
        }
    }
    
    // End turn in turn-based mode
    if keyboard_input.just_pressed(KeyCode::Space) && sync_manager.turn_based_mode {
        ui_actions.send(UIAction::Custom(Box::new(SyncExplorationUIAction::EndTurn)));
    }
}

/// System for handling synchronous exploration UI actions
pub fn sync_exploration_ui_action_system(
    mut sync_ui: ResMut<SyncExplorationUI>,
    mut sync_manager: ResMut<SyncExplorationManager>,
    mut ui_actions: EventReader<UIAction>,
) {
    for action in ui_actions.iter() {
        if let UIAction::Custom(custom_action) = action {
            if let Some(sync_action) = custom_action.downcast_ref::<SyncExplorationUIAction>() {
                match sync_action {
                    SyncExplorationUIAction::ToggleUI => {
                        if sync_ui.state == SyncExplorationUIState::Hidden {
                            sync_ui.state = SyncExplorationUIState::PartyStatus;
                        } else {
                            sync_ui.state = SyncExplorationUIState::Hidden;
                        }
                    },
                    SyncExplorationUIAction::SetState(state) => {
                        sync_ui.state = state.clone();
                    },
                    SyncExplorationUIAction::TogglePartyStatus => {
                        sync_ui.show_party_status = !sync_ui.show_party_status;
                    },
                    SyncExplorationUIAction::ToggleTurnIndicator => {
                        sync_ui.show_turn_indicator = !sync_ui.show_turn_indicator;
                    },
                    SyncExplorationUIAction::ToggleActionPoints => {
                        sync_ui.show_action_points = !sync_ui.show_action_points;
                    },
                    SyncExplorationUIAction::ToggleFormationOverlay => {
                        sync_ui.show_formation_overlay = !sync_ui.show_formation_overlay;
                    },
                    SyncExplorationUIAction::SetFormation(formation) => {
                        sync_manager.set_formation(*formation);
                    },
                    SyncExplorationUIAction::ToggleTurnBasedMode => {
                        if sync_manager.turn_based_mode {
                            sync_manager.disable_turn_based_mode();
                        } else {
                            sync_manager.enable_turn_based_mode();
                        }
                    },
                    SyncExplorationUIAction::ToggleAutoFollow => {
                        sync_manager.auto_follow = !sync_manager.auto_follow;
                    },
                    SyncExplorationUIAction::ToggleSharedVision => {
                        sync_manager.shared_vision = !sync_manager.shared_vision;
                    },
                    SyncExplorationUIAction::ToggleSharedInventory => {
                        sync_manager.shared_inventory = !sync_manager.shared_inventory;
                    },
                    SyncExplorationUIAction::ToggleCooperativeActions => {
                        sync_manager.cooperative_actions = !sync_manager.cooperative_actions;
                    },
                    SyncExplorationUIAction::SwitchToNextMember => {
                        sync_manager.switch_to_next_member();
                    },
                    SyncExplorationUIAction::SwitchToPreviousMember => {
                        sync_manager.switch_to_previous_member();
                    },
                    SyncExplorationUIAction::EndTurn => {
                        sync_manager.end_turn();
                    },
                }
            }
        }
    }
}

/// System for rendering synchronous exploration UI
pub fn sync_exploration_ui_render_system(
    sync_ui: Res<SyncExplorationUI>,
    sync_manager: Res<SyncExplorationManager>,
    party_query: Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    mut ui_elements: ResMut<Vec<UIElement>>,
) {
    // Always show party status if enabled and in sync mode
    if sync_manager.mode != SyncExplorationMode::Disabled && sync_ui.show_party_status {
        render_party_status_hud(&sync_manager, &party_query, &mut ui_elements);
    }
    
    // Show turn indicator if in turn-based mode
    if sync_manager.turn_based_mode && sync_ui.show_turn_indicator {
        render_turn_indicator(&sync_manager, &party_query, &mut ui_elements);
    }
    
    // Show action points if enabled
    if sync_manager.turn_based_mode && sync_ui.show_action_points {
        render_action_points(&sync_manager, &party_query, &mut ui_elements);
    }
    
    // Show formation overlay if enabled
    if sync_ui.show_formation_overlay {
        render_formation_overlay(&sync_manager, &party_query, &mut ui_elements);
    }
    
    // Render main UI if visible
    if sync_ui.state != SyncExplorationUIState::Hidden {
        render_main_ui(&sync_ui, &sync_manager, &party_query, &mut ui_elements);
    }
}

/// Render party status HUD
fn render_party_status_hud(
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    // Party status panel
    ui_elements.push(UIElement::Panel(UIPanel {
        x: 1,
        y: 1,
        width: 30,
        height: 8,
        title: "Party Status".to_string(),
        border: true,
    }));
    
    // Mode indicator
    let mode_text = match sync_manager.mode {
        SyncExplorationMode::Disabled => "Solo",
        SyncExplorationMode::PlayerControl => "Player Control",
        SyncExplorationMode::AgentControl => "Agent Control",
        SyncExplorationMode::CooperativeControl => "Cooperative",
    };
    
    ui_elements.push(UIElement::Text(UIText {
        x: 3,
        y: 2,
        text: format!("Mode: {}", mode_text),
        color: None,
    }));
    
    // Party size
    ui_elements.push(UIElement::Text(UIText {
        x: 3,
        y: 3,
        text: format!("Party Size: {}", sync_manager.active_party.len()),
        color: None,
    }));
    
    // Formation
    let formation_text = match sync_manager.formation {
        PartyFormation::None => "None",
        PartyFormation::Line => "Line",
        PartyFormation::Column => "Column",
        PartyFormation::Diamond => "Diamond",
        PartyFormation::Circle => "Circle",
        PartyFormation::Custom => "Custom",
    };
    
    ui_elements.push(UIElement::Text(UIText {
        x: 3,
        y: 4,
        text: format!("Formation: {}", formation_text),
        color: None,
    }));
    
    // Controlled entity
    if let Some(controlled) = sync_manager.get_controlled_entity() {
        if let Ok((_, _, _, _, name)) = party_query.get(controlled) {
            let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
            ui_elements.push(UIElement::Text(UIText {
                x: 3,
                y: 5,
                text: format!("Controlling: {}", name_str),
                color: Some((255, 255, 0)),
            }));
        }
    }
    
    // Settings indicators
    let mut settings_y = 6;
    if sync_manager.auto_follow {
        ui_elements.push(UIElement::Text(UIText {
            x: 3,
            y: settings_y,
            text: "[Auto-Follow]".to_string(),
            color: Some((0, 255, 0)),
        }));
        settings_y += 1;
    }
    
    if sync_manager.shared_vision {
        ui_elements.push(UIElement::Text(UIText {
            x: 15,
            y: 6,
            text: "[Shared Vision]".to_string(),
            color: Some((0, 255, 0)),
        }));
    }
}

/// Render turn indicator
fn render_turn_indicator(
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(current_entity) = sync_manager.get_current_turn_entity() {
        if let Ok((_, _, _, _, name)) = party_query.get(current_entity) {
            let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
            
            ui_elements.push(UIElement::Panel(UIPanel {
                x: 35,
                y: 1,
                width: 20,
                height: 3,
                title: "Current Turn".to_string(),
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 37,
                y: 2,
                text: name_str,
                color: Some((255, 255, 0)),
            }));
        }
    }
}

/// Render action points
fn render_action_points(
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    if let Some(current_entity) = sync_manager.get_current_turn_entity() {
        let action_points = sync_manager.get_action_points(&current_entity);
        
        ui_elements.push(UIElement::Text(UIText {
            x: 37,
            y: 3,
            text: format!("AP: {}/{}", action_points, sync_manager.max_action_points),
            color: if action_points > 0 { Some((0, 255, 0)) } else { Some((255, 0, 0)) },
        }));
    }
}

/// Render formation overlay
fn render_formation_overlay(
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    // Get leader position
    let leader_position = if let Some(leader_entity) = sync_manager.get_party_leader() {
        if let Ok((_, _, pos, _, _)) = party_query.get(leader_entity) {
            Some(pos.0)
        } else {
            None
        }
    } else {
        None
    };
    
    if let Some(leader_pos) = leader_position {
        let formation_positions = sync_manager.calculate_formation_positions(leader_pos);
        
        // Render formation positions as overlay markers
        for (entity, target_pos) in formation_positions {
            if let Ok((_, party_member, current_pos, _, name)) = party_query.get(entity) {
                let name_str = name.map_or("?".to_string(), |n| n.name.chars().next().unwrap_or('?').to_string());
                
                // Convert world position to screen position (simplified)
                let screen_x = (target_pos.x + 40.0) as i32;
                let screen_y = (target_pos.y + 20.0) as i32;
                
                // Only render if on screen
                if screen_x >= 0 && screen_x < 80 && screen_y >= 0 && screen_y < 50 {
                    ui_elements.push(UIElement::Text(UIText {
                        x: screen_x,
                        y: screen_y,
                        text: name_str,
                        color: if party_member.is_controlled { 
                            Some((255, 255, 0)) 
                        } else { 
                            Some((0, 255, 255)) 
                        },
                    }));
                }
            }
        }
    }
}

/// Render main UI
fn render_main_ui(
    sync_ui: &SyncExplorationUI,
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    // Main container
    ui_elements.push(UIElement::Panel(UIPanel {
        x: 10,
        y: 5,
        width: 60,
        height: 35,
        title: "Synchronous Exploration".to_string(),
        border: true,
    }));
    
    // Navigation tabs
    ui_elements.push(UIElement::Text(UIText {
        x: 12,
        y: 7,
        text: "[1] Party | [2] Formation | [3] Turn Order | [4] Cooperative".to_string(),
        color: None,
    }));
    
    match sync_ui.state {
        SyncExplorationUIState::PartyStatus => render_party_status_screen(sync_manager, party_query, ui_elements),
        SyncExplorationUIState::FormationConfig => render_formation_config_screen(sync_manager, ui_elements),
        SyncExplorationUIState::TurnOrder => render_turn_order_screen(sync_manager, party_query, ui_elements),
        SyncExplorationUIState::CooperativeActions => render_cooperative_actions_screen(sync_manager, ui_elements),
        _ => {}
    }
    
    // Footer
    ui_elements.push(UIElement::Text(UIText {
        x: 12,
        y: 38,
        text: "[P] Close | [Tab] Switch Member | [T] Toggle Turn-Based | [F] Formation Overlay".to_string(),
        color: None,
    }));
}

/// Render party status screen
fn render_party_status_screen(
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    // Party members list
    ui_elements.push(UIElement::Box(UIBox {
        x: 12,
        y: 9,
        width: 56,
        height: 15,
        title: Some("Party Members".to_string()),
        border: true,
    }));
    
    // Headers
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: 10,
        text: format!("{:<15} {:<10} {:<10} {:<10} {:<10}", "Name", "Role", "Health", "Status", "AP"),
        color: None,
    }));
    
    let mut y = 11;
    for entity in &sync_manager.active_party {
        if let Ok((_, party_member, _, health, name)) = party_query.get(*entity) {
            let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
            let role_str = format!("{:?}", party_member.role);
            let health_str = format!("{}/{}", health.current, health.max);
            let status_str = if party_member.is_controlled {
                "Controlled"
            } else if party_member.is_active {
                "Active"
            } else {
                "Inactive"
            };
            let ap_str = if sync_manager.turn_based_mode {
                sync_manager.get_action_points(entity).to_string()
            } else {
                "-".to_string()
            };
            
            let color = if party_member.is_controlled {
                Some((255, 255, 0))
            } else if sync_manager.get_current_turn_entity() == Some(*entity) {
                Some((0, 255, 0))
            } else {
                None
            };
            
            ui_elements.push(UIElement::Text(UIText {
                x: 14,
                y,
                text: format!("{:<15} {:<10} {:<10} {:<10} {:<10}", 
                    name_str, role_str, health_str, status_str, ap_str),
                color,
            }));
            
            y += 1;
        }
    }
    
    // Settings
    ui_elements.push(UIElement::Box(UIBox {
        x: 12,
        y: 25,
        width: 56,
        height: 10,
        title: Some("Settings".to_string()),
        border: true,
    }));
    
    let mut settings_y = 26;
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: settings_y,
        text: format!("[{}] Auto-Follow", if sync_manager.auto_follow { "X" } else { " " }),
        color: None,
    }));
    settings_y += 1;
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: settings_y,
        text: format!("[{}] Shared Vision", if sync_manager.shared_vision { "X" } else { " " }),
        color: None,
    }));
    settings_y += 1;
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: settings_y,
        text: format!("[{}] Shared Inventory", if sync_manager.shared_inventory { "X" } else { " " }),
        color: None,
    }));
    settings_y += 1;
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: settings_y,
        text: format!("[{}] Cooperative Actions", if sync_manager.cooperative_actions { "X" } else { " " }),
        color: None,
    }));
    settings_y += 1;
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: settings_y,
        text: format!("[{}] Turn-Based Mode", if sync_manager.turn_based_mode { "X" } else { " " }),
        color: None,
    }));
}

/// Render formation config screen
fn render_formation_config_screen(
    sync_manager: &SyncExplorationManager,
    ui_elements: &mut Vec<UIElement>,
) {
    ui_elements.push(UIElement::Box(UIBox {
        x: 12,
        y: 9,
        width: 56,
        height: 25,
        title: Some("Formation Configuration".to_string()),
        border: true,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: 11,
        text: format!("Current Formation: {:?}", sync_manager.formation),
        color: None,
    }));
    
    // Formation options
    let formations = [
        PartyFormation::None,
        PartyFormation::Line,
        PartyFormation::Column,
        PartyFormation::Diamond,
        PartyFormation::Circle,
    ];
    
    let mut y = 13;
    for formation in formations {
        let selected = if formation == sync_manager.formation { "[X]" } else { "[ ]" };
        
        ui_elements.push(UIElement::Button(UIButton {
            x: 14,
            y,
            width: 30,
            height: 1,
            text: format!("{} {:?}", selected, formation),
            action: UIAction::Custom(Box::new(SyncExplorationUIAction::SetFormation(formation))),
            enabled: true,
        }));
        
        y += 2;
    }
}

/// Render turn order screen
fn render_turn_order_screen(
    sync_manager: &SyncExplorationManager,
    party_query: &Query<(Entity, &PartyMember, &Position, &Health, Option<&Name>)>,
    ui_elements: &mut Vec<UIElement>,
) {
    ui_elements.push(UIElement::Box(UIBox {
        x: 12,
        y: 9,
        width: 56,
        height: 25,
        title: Some("Turn Order".to_string()),
        border: true,
    }));
    
    if sync_manager.turn_based_mode {
        ui_elements.push(UIElement::Text(UIText {
            x: 14,
            y: 11,
            text: "Turn-Based Mode: Enabled".to_string(),
            color: Some((0, 255, 0)),
        }));
        
        ui_elements.push(UIElement::Text(UIText {
            x: 14,
            y: 13,
            text: "Turn Order:".to_string(),
            color: None,
        }));
        
        let mut y = 14;
        for (i, entity) in sync_manager.turn_order.iter().enumerate() {
            if let Ok((_, _, _, _, name)) = party_query.get(*entity) {
                let name_str = name.map_or("Unknown".to_string(), |n| n.name.clone());
                let is_current = i == sync_manager.current_turn;
                let ap = sync_manager.get_action_points(entity);
                
                let color = if is_current { Some((255, 255, 0)) } else { None };
                let marker = if is_current { ">" } else { " " };
                
                ui_elements.push(UIElement::Text(UIText {
                    x: 14,
                    y,
                    text: format!("{} {}. {} (AP: {})", marker, i + 1, name_str, ap),
                    color,
                }));
                
                y += 1;
            }
        }
    } else {
        ui_elements.push(UIElement::Text(UIText {
            x: 14,
            y: 11,
            text: "Turn-Based Mode: Disabled".to_string(),
            color: Some((255, 0, 0)),
        }));
        
        ui_elements.push(UIElement::Button(UIButton {
            x: 14,
            y: 13,
            width: 20,
            height: 1,
            text: "Enable Turn-Based Mode".to_string(),
            action: UIAction::Custom(Box::new(SyncExplorationUIAction::ToggleTurnBasedMode)),
            enabled: true,
        }));
    }
}

/// Render cooperative actions screen
fn render_cooperative_actions_screen(
    sync_manager: &SyncExplorationManager,
    ui_elements: &mut Vec<UIElement>,
) {
    ui_elements.push(UIElement::Box(UIBox {
        x: 12,
        y: 9,
        width: 56,
        height: 25,
        title: Some("Cooperative Actions".to_string()),
        border: true,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 14,
        y: 11,
        text: format!("Cooperative Actions: {}", 
            if sync_manager.cooperative_actions { "Enabled" } else { "Disabled" }),
        color: if sync_manager.cooperative_actions { Some((0, 255, 0)) } else { Some((255, 0, 0)) },
    }));
    
    if sync_manager.cooperative_actions {
        ui_elements.push(UIElement::Text(UIText {
            x: 14,
            y: 13,
            text: "Available Cooperative Actions:".to_string(),
            color: None,
        }));
        
        let actions = [
            "Combined Attack - Coordinate attacks for bonus damage",
            "Group Heal - Share healing effects among party",
            "Formation Move - Move as a coordinated unit",
            "Shared Spell - Combine mana for powerful spells",
            "Coordinated Defense - Boost defense when grouped",
            "Team Lift - Lift heavy objects together",
            "Group Puzzle Solve - Solve multi-person puzzles",
            "Chain Action - Execute sequential actions",
        ];
        
        let mut y = 15;
        for action in actions {
            ui_elements.push(UIElement::Text(UIText {
                x: 16,
                y,
                text: format!("• {}", action),
                color: None,
            }));
            y += 1;
        }
    } else {
        ui_elements.push(UIElement::Button(UIButton {
            x: 14,
            y: 13,
            width: 25,
            height: 1,
            text: "Enable Cooperative Actions".to_string(),
            action: UIAction::Custom(Box::new(SyncExplorationUIAction::ToggleCooperativeActions)),
            enabled: true,
        }));
    }
}

/// Plugin for synchronous exploration UI
pub struct SyncExplorationUIPlugin;

impl Plugin for SyncExplorationUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SyncExplorationUI>()
           .add_systems(Update, (
               sync_exploration_ui_input_system,
               sync_exploration_ui_action_system,
               sync_exploration_ui_render_system,
           ).chain());
    }
}