use bevy::prelude::*;
use crate::guild::asynchronous_exploration::{
    AsyncExplorationManager, AsyncExplorationState, AsyncExpedition, ExpeditionState
};
use crate::guild::mission_board::MissionBoard;
use crate::components::{Player, Name};
use crate::ui::{UIState, UIAction, UIElement, UIBox, UIText, UIButton, UIPanel};

/// Asynchronous exploration UI state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AsyncExplorationUIState {
    Hidden,
    Overview,
    ActiveExpeditions,
    CompletedExpeditions,
    Settings,
}

/// Asynchronous exploration UI resource
#[derive(Resource)]
pub struct AsyncExplorationUI {
    pub state: AsyncExplorationUIState,
    pub selected_expedition: Option<String>,
    pub show_notifications: bool,
    pub auto_scroll_events: bool,
    pub scroll_offset: usize,
}

impl Default for AsyncExplorationUI {
    fn default() -> Self {
        AsyncExplorationUI {
            state: AsyncExplorationUIState::Hidden,
            selected_expedition: None,
            show_notifications: true,
            auto_scroll_events: true,
            scroll_offset: 0,
        }
    }
}

/// Asynchronous exploration UI action
#[derive(Debug, Clone)]
pub enum AsyncExplorationUIAction {
    ToggleUI,
    SetState(AsyncExplorationUIState),
    SelectExpedition(String),
    StartAsyncExploration,
    PauseAsyncExploration,
    ResumeAsyncExploration,
    StopAsyncExploration,
    CancelExpedition(String),
    ToggleAutoAssign,
    ToggleOfflineProgress,
    SetSimulationSpeed(f64),
    ScrollUp,
    ScrollDown,
    ToggleNotifications,
}

/// System for handling asynchronous exploration UI input
pub fn async_exploration_ui_input_system(
    mut async_ui: ResMut<AsyncExplorationUI>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ui_actions: EventWriter<UIAction>,
) {
    // Toggle UI with 'A' key (for Async)
    if keyboard_input.just_pressed(KeyCode::A) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::ToggleUI)));
    }
    
    // Only process other inputs if UI is visible
    if async_ui.state == AsyncExplorationUIState::Hidden {
        return;
    }
    
    // Navigation between UI states
    if keyboard_input.just_pressed(KeyCode::Key1) || keyboard_input.just_pressed(KeyCode::Numpad1) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::SetState(AsyncExplorationUIState::Overview))));
    }
    if keyboard_input.just_pressed(KeyCode::Key2) || keyboard_input.just_pressed(KeyCode::Numpad2) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::SetState(AsyncExplorationUIState::ActiveExpeditions))));
    }
    if keyboard_input.just_pressed(KeyCode::Key3) || keyboard_input.just_pressed(KeyCode::Numpad3) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::SetState(AsyncExplorationUIState::CompletedExpeditions))));
    }
    if keyboard_input.just_pressed(KeyCode::Key4) || keyboard_input.just_pressed(KeyCode::Numpad4) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::SetState(AsyncExplorationUIState::Settings))));
    }
    
    // Scrolling
    if keyboard_input.just_pressed(KeyCode::Up) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::ScrollUp)));
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::ScrollDown)));
    }
    
    // Close UI with Escape
    if keyboard_input.just_pressed(KeyCode::Escape) {
        ui_actions.send(UIAction::Custom(Box::new(AsyncExplorationUIAction::SetState(AsyncExplorationUIState::Hidden))));
    }
}

/// System for handling asynchronous exploration UI actions
pub fn async_exploration_ui_action_system(
    mut async_ui: ResMut<AsyncExplorationUI>,
    mut async_manager: ResMut<AsyncExplorationManager>,
    mut ui_actions: EventReader<UIAction>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    
    for action in ui_actions.iter() {
        if let UIAction::Custom(custom_action) = action {
            if let Some(async_action) = custom_action.downcast_ref::<AsyncExplorationUIAction>() {
                match async_action {
                    AsyncExplorationUIAction::ToggleUI => {
                        if async_ui.state == AsyncExplorationUIState::Hidden {
                            async_ui.state = AsyncExplorationUIState::Overview;
                        } else {
                            async_ui.state = AsyncExplorationUIState::Hidden;
                        }
                    },
                    AsyncExplorationUIAction::SetState(state) => {
                        async_ui.state = state.clone();
                    },
                    AsyncExplorationUIAction::SelectExpedition(expedition_id) => {
                        async_ui.selected_expedition = Some(expedition_id.clone());
                    },
                    AsyncExplorationUIAction::StartAsyncExploration => {
                        async_manager.start(current_time);
                    },
                    AsyncExplorationUIAction::PauseAsyncExploration => {
                        async_manager.pause();
                    },
                    AsyncExplorationUIAction::ResumeAsyncExploration => {
                        async_manager.resume(current_time);
                    },
                    AsyncExplorationUIAction::StopAsyncExploration => {
                        async_manager.stop();
                    },
                    AsyncExplorationUIAction::CancelExpedition(expedition_id) => {
                        async_manager.cancel_expedition(expedition_id);
                    },
                    AsyncExplorationUIAction::ToggleAutoAssign => {
                        async_manager.auto_assign_missions = !async_manager.auto_assign_missions;
                    },
                    AsyncExplorationUIAction::ToggleOfflineProgress => {
                        async_manager.offline_progress_enabled = !async_manager.offline_progress_enabled;
                    },
                    AsyncExplorationUIAction::SetSimulationSpeed(speed) => {
                        async_manager.simulation_speed = *speed;
                    },
                    AsyncExplorationUIAction::ScrollUp => {
                        if async_ui.scroll_offset > 0 {
                            async_ui.scroll_offset -= 1;
                        }
                    },
                    AsyncExplorationUIAction::ScrollDown => {
                        async_ui.scroll_offset += 1;
                    },
                    AsyncExplorationUIAction::ToggleNotifications => {
                        async_ui.show_notifications = !async_ui.show_notifications;
                    },
                }
            }
        }
    }
}

/// System for rendering asynchronous exploration UI
pub fn async_exploration_ui_render_system(
    async_ui: Res<AsyncExplorationUI>,
    async_manager: Res<AsyncExplorationManager>,
    mission_board: Res<MissionBoard>,
    mut ui_elements: ResMut<Vec<UIElement>>,
) {
    // Only render if UI is visible
    if async_ui.state == AsyncExplorationUIState::Hidden {
        return;
    }
    
    // Main container
    ui_elements.push(UIElement::Panel(UIPanel {
        x: 5,
        y: 2,
        width: 70,
        height: 40,
        title: "Asynchronous Exploration".to_string(),
        border: true,
    }));
    
    // Navigation tabs
    ui_elements.push(UIElement::Text(UIText {
        x: 7,
        y: 4,
        text: format!("[1] Overview | [2] Active | [3] Completed | [4] Settings"),
        color: None,
    }));
    
    // Status indicator
    let status_text = match async_manager.state {
        AsyncExplorationState::Inactive => "Inactive",
        AsyncExplorationState::Active => "Active",
        AsyncExplorationState::Paused => "Paused",
        AsyncExplorationState::Completed => "Completed",
    };
    
    let status_color = match async_manager.state {
        AsyncExplorationState::Active => Some((0, 255, 0)),
        AsyncExplorationState::Paused => Some((255, 255, 0)),
        AsyncExplorationState::Inactive => Some((255, 0, 0)),
        _ => None,
    };
    
    ui_elements.push(UIElement::Text(UIText {
        x: 60,
        y: 4,
        text: format!("Status: {}", status_text),
        color: status_color,
    }));
    
    // Render appropriate content based on state
    match async_ui.state {
        AsyncExplorationUIState::Overview => render_overview_screen(&async_ui, &async_manager, &mut ui_elements),
        AsyncExplorationUIState::ActiveExpeditions => render_active_expeditions_screen(&async_ui, &async_manager, &mission_board, &mut ui_elements),
        AsyncExplorationUIState::CompletedExpeditions => render_completed_expeditions_screen(&async_ui, &async_manager, &mut ui_elements),
        AsyncExplorationUIState::Settings => render_settings_screen(&async_ui, &async_manager, &mut ui_elements),
        _ => {}
    }
    
    // Footer
    ui_elements.push(UIElement::Text(UIText {
        x: 7,
        y: 41,
        text: format!("[A] Close | [↑/↓] Scroll | Speed: {:.1}x", async_manager.simulation_speed),
        color: None,
    }));
}

/// Render overview screen
fn render_overview_screen(
    async_ui: &AsyncExplorationUI,
    async_manager: &AsyncExplorationManager,
    ui_elements: &mut Vec<UIElement>,
) {
    // Statistics
    ui_elements.push(UIElement::Box(UIBox {
        x: 7,
        y: 6,
        width: 66,
        height: 8,
        title: Some("Statistics".to_string()),
        border: true,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y: 7,
        text: format!("Active Expeditions: {}", async_manager.active_expeditions.len()),
        color: None,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y: 8,
        text: format!("Completed Expeditions: {}", async_manager.completed_expeditions.len()),
        color: None,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y: 9,
        text: format!("Pending Events: {}", async_manager.event_queue.len()),
        color: None,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y: 10,
        text: format!("Simulation Speed: {:.1}x", async_manager.simulation_speed),
        color: None,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y: 11,
        text: format!("Auto-Assign: {}", if async_manager.auto_assign_missions { "Enabled" } else { "Disabled" }),
        color: None,
    }));
    
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y: 12,
        text: format!("Offline Progress: {}", if async_manager.offline_progress_enabled { "Enabled" } else { "Disabled" }),
        color: None,
    }));
    
    // Control buttons
    ui_elements.push(UIElement::Box(UIBox {
        x: 7,
        y: 15,
        width: 66,
        height: 6,
        title: Some("Controls".to_string()),
        border: true,
    }));
    
    match async_manager.state {
        AsyncExplorationState::Inactive => {
            ui_elements.push(UIElement::Button(UIButton {
                x: 9,
                y: 17,
                width: 20,
                height: 1,
                text: "Start Async Exploration".to_string(),
                action: UIAction::Custom(Box::new(AsyncExplorationUIAction::StartAsyncExploration)),
                enabled: true,
            }));
        },
        AsyncExplorationState::Active => {
            ui_elements.push(UIElement::Button(UIButton {
                x: 9,
                y: 17,
                width: 15,
                height: 1,
                text: "Pause".to_string(),
                action: UIAction::Custom(Box::new(AsyncExplorationUIAction::PauseAsyncExploration)),
                enabled: true,
            }));
            
            ui_elements.push(UIElement::Button(UIButton {
                x: 26,
                y: 17,
                width: 15,
                height: 1,
                text: "Stop".to_string(),
                action: UIAction::Custom(Box::new(AsyncExplorationUIAction::StopAsyncExploration)),
                enabled: true,
            }));
        },
        AsyncExplorationState::Paused => {
            ui_elements.push(UIElement::Button(UIButton {
                x: 9,
                y: 17,
                width: 15,
                height: 1,
                text: "Resume".to_string(),
                action: UIAction::Custom(Box::new(AsyncExplorationUIAction::ResumeAsyncExploration)),
                enabled: true,
            }));
            
            ui_elements.push(UIElement::Button(UIButton {
                x: 26,
                y: 17,
                width: 15,
                height: 1,
                text: "Stop".to_string(),
                action: UIAction::Custom(Box::new(AsyncExplorationUIAction::StopAsyncExploration)),
                enabled: true,
            }));
        },
        _ => {}
    }
    
    // Recent events
    ui_elements.push(UIElement::Box(UIBox {
        x: 7,
        y: 22,
        width: 66,
        height: 15,
        title: Some("Recent Events".to_string()),
        border: true,
    }));
    
    if async_manager.event_queue.is_empty() {
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 24,
            text: "No recent events".to_string(),
            color: None,
        }));
    } else {
        let mut y = 24;
        let events: Vec<_> = async_manager.event_queue.iter().take(10).collect();
        
        for event in events {
            let event_text = format!("{:?}: {:?}", event.event_type, event.expedition_id.as_deref().unwrap_or("N/A"));
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y,
                text: event_text,
                color: None,
            }));
            y += 1;
            
            if y >= 35 {
                break;
            }
        }
    }
}

/// Render active expeditions screen
fn render_active_expeditions_screen(
    async_ui: &AsyncExplorationUI,
    async_manager: &AsyncExplorationManager,
    mission_board: &MissionBoard,
    ui_elements: &mut Vec<UIElement>,
) {
    // Active expeditions list
    ui_elements.push(UIElement::Box(UIBox {
        x: 7,
        y: 6,
        width: 66,
        height: 20,
        title: Some("Active Expeditions".to_string()),
        border: true,
    }));
    
    if async_manager.active_expeditions.is_empty() {
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 8,
            text: "No active expeditions".to_string(),
            color: None,
        }));
    } else {
        // Headers
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 7,
            text: format!("{:<20} {:<15} {:<10} {:<15}", "Mission", "State", "Progress", "Time Left"),
            color: None,
        }));
        
        let mut y = 8;
        let expeditions: Vec<_> = async_manager.active_expeditions.values().collect();
        
        for expedition in expeditions.iter().skip(async_ui.scroll_offset).take(15) {
            let mission_name = if let Some(mission) = mission_board.get_mission(&expedition.mission_id) {
                mission.name.clone()
            } else {
                expedition.mission_id.clone()
            };
            
            let state_text = format!("{:?}", expedition.state);
            let progress_text = format!("{:.0}%", expedition.progress * 100.0);
            let time_left = expedition.get_time_remaining(0.0); // Current time would be passed
            let time_text = if time_left > 0.0 {
                format!("{:.0}s", time_left)
            } else {
                "Complete".to_string()
            };
            
            let color = if async_ui.selected_expedition.as_ref() == Some(&expedition.id) {
                Some((255, 255, 0))
            } else {
                None
            };
            
            ui_elements.push(UIElement::Button(UIButton {
                x: 9,
                y,
                width: 62,
                height: 1,
                text: format!("{:<20} {:<15} {:<10} {:<15}", 
                    mission_name.chars().take(20).collect::<String>(),
                    state_text,
                    progress_text,
                    time_text),
                action: UIAction::Custom(Box::new(AsyncExplorationUIAction::SelectExpedition(expedition.id.clone()))),
                enabled: true,
            }));
            
            y += 1;
        }
    }
    
    // Expedition details if selected
    if let Some(expedition_id) = &async_ui.selected_expedition {
        if let Some(expedition) = async_manager.get_expedition(expedition_id) {
            ui_elements.push(UIElement::Box(UIBox {
                x: 7,
                y: 27,
                width: 66,
                height: 12,
                title: Some("Expedition Details".to_string()),
                border: true,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 28,
                text: format!("ID: {}", expedition.id),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 29,
                text: format!("Mission: {}", expedition.mission_id),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 30,
                text: format!("Agents: {}", expedition.assigned_agents.len()),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 31,
                text: format!("Progress: {:.1}%", expedition.progress * 100.0),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 32,
                text: format!("Success Chance: {:.0}%", expedition.success_chance * 100.0),
                color: None,
            }));
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y: 33,
                text: format!("Events: {}", expedition.events.len()),
                color: None,
            }));
            
            // Cancel button
            if expedition.state == ExpeditionState::InProgress || expedition.state == ExpeditionState::Preparing {
                ui_elements.push(UIElement::Button(UIButton {
                    x: 9,
                    y: 35,
                    width: 20,
                    height: 1,
                    text: "Cancel Expedition".to_string(),
                    action: UIAction::Custom(Box::new(AsyncExplorationUIAction::CancelExpedition(expedition.id.clone()))),
                    enabled: true,
                }));
            }
        }
    }
}

/// Render completed expeditions screen
fn render_completed_expeditions_screen(
    async_ui: &AsyncExplorationUI,
    async_manager: &AsyncExplorationManager,
    ui_elements: &mut Vec<UIElement>,
) {
    // Completed expeditions list
    ui_elements.push(UIElement::Box(UIBox {
        x: 7,
        y: 6,
        width: 66,
        height: 30,
        title: Some("Completed Expeditions".to_string()),
        border: true,
    }));
    
    if async_manager.completed_expeditions.is_empty() {
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 8,
            text: "No completed expeditions".to_string(),
            color: None,
        }));
    } else {
        // Headers
        ui_elements.push(UIElement::Text(UIText {
            x: 9,
            y: 7,
            text: format!("{:<20} {:<15} {:<10} {:<15}", "Mission", "Result", "Duration", "Rewards"),
            color: None,
        }));
        
        let mut y = 8;
        let expeditions = &async_manager.completed_expeditions;
        
        for expedition in expeditions.iter().skip(async_ui.scroll_offset).take(25) {
            let mission_name = expedition.mission_id.chars().take(20).collect::<String>();
            let result_text = match expedition.state {
                ExpeditionState::Completed => "Success",
                ExpeditionState::Failed => "Failed",
                ExpeditionState::Cancelled => "Cancelled",
                _ => "Unknown",
            };
            
            let duration_text = if let Some(duration) = expedition.actual_duration {
                format!("{:.0}s", duration)
            } else {
                "N/A".to_string()
            };
            
            let rewards_text = format!("{}", expedition.rewards.len());
            
            let color = match expedition.state {
                ExpeditionState::Completed => Some((0, 255, 0)),
                ExpeditionState::Failed => Some((255, 0, 0)),
                ExpeditionState::Cancelled => Some((255, 255, 0)),
                _ => None,
            };
            
            ui_elements.push(UIElement::Text(UIText {
                x: 9,
                y,
                text: format!("{:<20} {:<15} {:<10} {:<15}", mission_name, result_text, duration_text, rewards_text),
                color,
            }));
            
            y += 1;
        }
    }
}

/// Render settings screen
fn render_settings_screen(
    async_ui: &AsyncExplorationUI,
    async_manager: &AsyncExplorationManager,
    ui_elements: &mut Vec<UIElement>,
) {
    // Settings
    ui_elements.push(UIElement::Box(UIBox {
        x: 7,
        y: 6,
        width: 66,
        height: 30,
        title: Some("Settings".to_string()),
        border: true,
    }));
    
    let mut y = 8;
    
    // Auto-assign missions
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y,
        text: format!("[{}] Auto-assign missions to available agents", 
            if async_manager.auto_assign_missions { "X" } else { " " }),
        color: None,
    }));
    y += 2;
    
    // Offline progress
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y,
        text: format!("[{}] Calculate progress while offline", 
            if async_manager.offline_progress_enabled { "X" } else { " " }),
        color: None,
    }));
    y += 2;
    
    // Simulation speed
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y,
        text: format!("Simulation Speed: {:.1}x", async_manager.simulation_speed),
        color: None,
    }));
    y += 1;
    
    // Speed buttons
    ui_elements.push(UIElement::Button(UIButton {
        x: 9,
        y,
        width: 10,
        height: 1,
        text: "0.5x".to_string(),
        action: UIAction::Custom(Box::new(AsyncExplorationUIAction::SetSimulationSpeed(0.5))),
        enabled: true,
    }));
    
    ui_elements.push(UIElement::Button(UIButton {
        x: 21,
        y,
        width: 10,
        height: 1,
        text: "1.0x".to_string(),
        action: UIAction::Custom(Box::new(AsyncExplorationUIAction::SetSimulationSpeed(1.0))),
        enabled: true,
    }));
    
    ui_elements.push(UIElement::Button(UIButton {
        x: 33,
        y,
        width: 10,
        height: 1,
        text: "2.0x".to_string(),
        action: UIAction::Custom(Box::new(AsyncExplorationUIAction::SetSimulationSpeed(2.0))),
        enabled: true,
    }));
    
    ui_elements.push(UIElement::Button(UIButton {
        x: 45,
        y,
        width: 10,
        height: 1,
        text: "5.0x".to_string(),
        action: UIAction::Custom(Box::new(AsyncExplorationUIAction::SetSimulationSpeed(5.0))),
        enabled: true,
    }));
    
    y += 3;
    
    // Max concurrent expeditions
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y,
        text: format!("Max Concurrent Expeditions: {}", async_manager.max_concurrent_expeditions),
        color: None,
    }));
    y += 2;
    
    // Notifications
    ui_elements.push(UIElement::Text(UIText {
        x: 9,
        y,
        text: format!("[{}] Show notifications", 
            if async_ui.show_notifications { "X" } else { " " }),
        color: None,
    }));
    y += 2;
    
    // Toggle buttons
    ui_elements.push(UIElement::Button(UIButton {
        x: 9,
        y,
        width: 25,
        height: 1,
        text: "Toggle Auto-Assign".to_string(),
        action: UIAction::Custom(Box::new(AsyncExplorationUIAction::ToggleAutoAssign)),
        enabled: true,
    }));
    
    ui_elements.push(UIElement::Button(UIButton {
        x: 36,
        y,
        width: 25,
        height: 1,
        text: "Toggle Offline Progress".to_string(),
        action: UIAction::Custom(Box::new(AsyncExplorationUIAction::ToggleOfflineProgress)),
        enabled: true,
    }));
}

/// Plugin for asynchronous exploration UI
pub struct AsyncExplorationUIPlugin;

impl Plugin for AsyncExplorationUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AsyncExplorationUI>()
           .add_systems(Update, (
               async_exploration_ui_input_system,
               async_exploration_ui_action_system,
               async_exploration_ui_render_system,
           ).chain());
    }
}