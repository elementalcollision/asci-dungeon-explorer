use bevy::prelude::*;
use crate::guild::asynchronous_exploration::AsyncExplorationManager;
use crate::guild::async_exploration_systems::{
    async_exploration_update_system, async_event_processing_system,
    mission_report_processing_system, auto_mission_assignment_system,
    offline_progress_system, expedition_monitoring_system,
    expedition_input_system, periodic_report_system
};

/// Plugin for asynchronous exploration systems
pub struct AsyncExplorationPlugin;

impl Plugin for AsyncExplorationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AsyncExplorationManager>()
           .add_systems(Update, (
               // Core systems
               async_exploration_update_system,
               async_event_processing_system,
               mission_report_processing_system,
               
               // Management systems
               auto_mission_assignment_system,
               offline_progress_system,
               
               // UI and input systems
               expedition_monitoring_system,
               expedition_input_system,
               periodic_report_system,
           ).chain());
    }
}

/// System for initializing asynchronous exploration
pub fn initialize_async_exploration_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds_f64();
    async_manager.last_update_time = current_time;
    
    info!("Initialized asynchronous exploration system");
}

/// System for demonstrating asynchronous exploration
pub fn demo_async_exploration_system(
    mut async_manager: ResMut<AsyncExplorationManager>,
    mission_board: Res<crate::guild::mission_board::MissionBoard>,
    guild_manager: Res<crate::guild::guild_core::GuildManager>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut demo_run: Local<bool>,
) {
    // Run demo once when F7 is pressed
    if keyboard_input.just_pressed(KeyCode::F7) && !*demo_run {
        *demo_run = true;
        let current_time = time.elapsed_seconds_f64();
        
        info!("Starting asynchronous exploration demo");
        
        // Create some demo missions and expeditions
        for guild in guild_manager.guilds.values().take(2) {
            // Create a demo mission
            let mut demo_mission = crate::guild::mission::Mission::new(
                format!("demo_mission_{}", guild.id),
                "Demo Exploration Mission".to_string(),
                "A demonstration of asynchronous exploration capabilities.".to_string(),
                crate::guild::mission_types::MissionDifficulty::Medium,
                guild.id.clone(),
                current_time,
            );
            
            // Add some objectives
            demo_mission.add_objective(crate::guild::mission_types::MissionObjective::new(
                crate::guild::mission_types::MissionObjectiveType::ExploreArea {
                    area_name: "Ancient Ruins".to_string(),
                    percentage: 75,
                }
            ));
            
            demo_mission.add_objective(crate::guild::mission_types::MissionObjective::new(
                crate::guild::mission_types::MissionObjectiveType::CollectItems {
                    item_type: "Ancient Artifacts".to_string(),
                    count: 3,
                }
            ));
            
            // Add rewards
            demo_mission.add_reward(crate::guild::mission_types::MissionReward::Resources {
                resource_type: crate::guild::guild_core::GuildResource::Gold,
                amount: 200,
            });
            
            demo_mission.add_reward(crate::guild::mission_types::MissionReward::Experience {
                amount: 150,
            });
            
            // Create demo agents for the expedition
            let demo_agents = vec![
                Entity::from_raw(100 + guild.members.len() as u32),
                Entity::from_raw(101 + guild.members.len() as u32),
            ];
            
            // Start the expedition
            let expedition_id = async_manager.start_expedition(&demo_mission, demo_agents, current_time);
            
            info!("Started demo expedition {} for guild {}", expedition_id, guild.id);
        }
        
        // Enable auto-assignment for demonstration
        async_manager.auto_assign_missions = true;
        
        // Set faster simulation speed for demo
        async_manager.set_simulation_speed(5.0);
        
        info!("Demo expeditions started with 5x simulation speed");
    }
    
    // Reset demo when F8 is pressed
    if keyboard_input.just_pressed(KeyCode::F8) {
        *demo_run = false;
        
        // Cancel all active expeditions
        let expedition_ids: Vec<String> = async_manager.active_expeditions.keys().cloned().collect();
        for expedition_id in expedition_ids {
            async_manager.cancel_expedition(&expedition_id);
        }
        
        // Reset simulation speed
        async_manager.set_simulation_speed(1.0);
        
        // Clear completed expeditions
        async_manager.completed_expeditions.clear();
        
        info!("Demo reset - all expeditions cancelled");
    }
}

/// System for displaying asynchronous exploration help
pub fn async_exploration_help_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut show_help: Local<bool>,
    mut ui_elements: ResMut<Vec<crate::ui::UIElement>>,
) {
    // Toggle help with F9
    if keyboard_input.just_pressed(KeyCode::F9) {
        *show_help = !*show_help;
    }
    
    if *show_help {
        ui_elements.push(crate::ui::UIElement::Panel(crate::ui::UIPanel {
            x: 10,
            y: 10,
            width: 60,
            height: 20,
            title: "Asynchronous Exploration Help".to_string(),
            border: true,
        }));
        
        let help_text = vec![
            "Asynchronous Exploration allows agents to go on missions",
            "while you continue playing. Expeditions run in the background",
            "and generate events and reports automatically.",
            "",
            "Controls:",
            "[F6] Toggle auto-assign missions",
            "[F7] Start demo expeditions",
            "[F8] Reset demo",
            "[F9] Toggle this help",
            "[+/-] Adjust simulation speed",
            "[0] Reset simulation speed to 1x",
            "",
            "Features:",
            "- Time-based simulation system",
            "- Offline progress calculation",
            "- Automatic event generation",
            "- Mission reports with rewards",
            "- Agent progression during expeditions",
            "",
            "Expeditions will continue even when offline!",
        ];
        
        for (i, line) in help_text.iter().enumerate() {
            ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
                x: 12,
                y: 12 + i as i32,
                text: line.to_string(),
                color: Some((255, 255, 255)),
            }));
        }
    }
}