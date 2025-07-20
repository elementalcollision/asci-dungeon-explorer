use bevy::prelude::*;
use crate::guild::synchronous_exploration::{
    SyncExplorationManager, SyncExplorationMode, ControlSwitchEvent, ControlSwitchType,
    FormationType, SharedAction, SharedActionType, PartyMember
};
use crate::components::{Position, Player};

/// System for handling synchronous exploration input
pub fn sync_exploration_input_system(
    mut sync_manager: ResMut<SyncExplorationManager>,
    mut control_events: EventWriter<ControlSwitchEvent>,
    keyboard_input: Res<Input<KeyCode>>,
    party_query: Query<Entity, With<PartyMember>>,
) {
    // Toggle synchronous exploration mode
    if keyboard_input.just_pressed(KeyCode::F1) {
        let new_mode = match sync_manager.mode {
            SyncExplorationMode::Disabled => SyncExplorationMode::SingleControl,
            SyncExplorationMode::SingleControl => SyncExplorationMode::PartyControl,
            SyncExplorationMode::PartyControl => SyncExplorationMode::CooperativeMode,
            SyncExplorationMode::CooperativeMode => SyncExplorationMode::Disabled,
        };
        sync_manager.enable_mode(new_mode);
        info!("Switched to synchronous exploration mode: {:?}", new_mode);
    }
    
    // Only process other inputs if sync exploration is enabled
    if sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    // Switch control between party members
    if keyboard_input.just_pressed(KeyCode::Tab) {
        if let Some(current_entity) = sync_manager.get_controlled_entity() {
            // Find next party member to control
            let party_members: Vec<Entity> = sync_manager.active_party.iter().copied().collect();
            if let Some(current_index) = party_members.iter().position(|&e| e == current_entity) {
                let next_index = (current_index + 1) % party_members.len();
                let next_entity = party_members[next_index];
                
                control_events.send(ControlSwitchEvent {
                    from_entity: Some(current_entity),
                    to_entity: next_entity,
                    switch_type: ControlSwitchType::Manual,
                });
            }
        }
    }
    
    // Quick switch to specific party members (1-6 keys)
    for (i, key) in [KeyCode::Key1, KeyCode::Key2, KeyCode::Key3, KeyCode::Key4, KeyCode::Key5, KeyCode::Key6].iter().enumerate() {
        if keyboard_input.just_pressed(*key) {
            let party_members: Vec<Entity> = sync_manager.active_party.iter().copied().collect();
            if i < party_members.len() {
                let target_entity = party_members[i];
                
                control_events.send(ControlSwitchEvent {
                    from_entity: sync_manager.get_controlled_entity(),
                    to_entity: target_entity,
                    switch_type: ControlSwitchType::Manual,
                });
            }
        }
    }
    
    // Formation controls
    if keyboard_input.just_pressed(KeyCode::F) {
        // Cycle through formation types
        let new_formation = match sync_manager.formation_type {
            FormationType::Line => FormationType::Column,
            FormationType::Column => FormationType::Wedge,
            FormationType::Wedge => FormationType::Box,
            FormationType::Box => FormationType::Circle,
            FormationType::Circle => FormationType::Line,
            FormationType::Custom => FormationType::Line,
        };
        sync_manager.set_formation(new_formation);
        info!("Changed formation to: {:?}", new_formation);
    }
    
    // Toggle auto-follow
    if keyboard_input.just_pressed(KeyCode::F2) {
        sync_manager.auto_follow = !sync_manager.auto_follow;
        info!("Auto-follow: {}", if sync_manager.auto_follow { "enabled" } else { "disabled" });
    }
    
    // Toggle shared vision
    if keyboard_input.just_pressed(KeyCode::F3) {
        sync_manager.shared_vision = !sync_manager.shared_vision;
        info!("Shared vision: {}", if sync_manager.shared_vision { "enabled" } else { "disabled" });
    }
    
    // Toggle cooperative actions
    if keyboard_input.just_pressed(KeyCode::F4) {
        sync_manager.cooperative_actions = !sync_manager.cooperative_actions;
        info!("Cooperative actions: {}", if sync_manager.cooperative_actions { "enabled" } else { "disabled" });
    }
    
    // Toggle turn-based mode
    if keyboard_input.just_pressed(KeyCode::F5) {
        sync_manager.turn_based_mode = !sync_manager.turn_based_mode;
        info!("Turn-based mode: {}", if sync_manager.turn_based_mode { "enabled" } else { "disabled" });
        
        if sync_manager.turn_based_mode {
            // Initialize turn order
            sync_manager.turn_order = sync_manager.active_party.iter().copied().collect();
            sync_manager.current_turn = 0;
            
            // Initialize action points
            for &entity in &sync_manager.active_party {
                sync_manager.action_points.insert(entity, sync_manager.max_action_points);
            }
        } else {
            sync_manager.turn_order.clear();
            sync_manager.action_points.clear();
        }
    }
    
    // Party management
    if keyboard_input.just_pressed(KeyCode::P) {
        // Add nearby entities to party (simplified for demo)
        for entity in party_query.iter() {
            if sync_manager.active_party.len() < 6 && !sync_manager.active_party.contains(&entity) {
                sync_manager.add_to_party(entity);
                info!("Added entity {:?} to party", entity);
                break;
            }
        }
    }
    
    // Remove current entity from party
    if keyboard_input.just_pressed(KeyCode::R) {
        if let Some(current_entity) = sync_manager.get_controlled_entity() {
            if sync_manager.active_party.len() > 1 {
                sync_manager.remove_from_party(current_entity);
                info!("Removed entity {:?} from party", current_entity);
            }
        }
    }
}

/// System for handling party movement input
pub fn party_movement_input_system(
    sync_manager: Res<SyncExplorationManager>,
    keyboard_input: Res<Input<KeyCode>>,
    mut position_query: Query<&mut Position>,
    mut commands: Commands,
) {
    if sync_manager.mode != SyncExplorationMode::PartyControl {
        return;
    }
    
    let mut movement = Vec2::ZERO;
    
    // Get movement input
    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        movement.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        movement.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
        movement.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
        movement.x += 1.0;
    }
    
    if movement != Vec2::ZERO {
        movement = movement.normalize();
        
        // Move all party members in formation
        if let Some(leader_entity) = sync_manager.get_party_leader() {
            if let Ok(mut leader_pos) = position_query.get_mut(leader_entity) {
                leader_pos.0 += movement * 0.1; // Adjust speed as needed
                
                // Create formation move action for other party members
                if sync_manager.cooperative_actions {
                    let mut participants = sync_manager.active_party.clone();
                    participants.remove(&leader_entity);
                    
                    if !participants.is_empty() {
                        let shared_action = SharedAction {
                            action_type: SharedActionType::FormationMove { 
                                target_position: leader_pos.0 
                            },
                            initiator: leader_entity,
                            participants,
                            required_participants: 1,
                            completion_time: 0.0,
                            auto_execute: true,
                        };
                        
                        commands.spawn(shared_action);
                    }
                }
            }
        }
    }
}

/// System for handling shared action input
pub fn shared_action_input_system(
    sync_manager: Res<SyncExplorationManager>,
    keyboard_input: Res<Input<KeyCode>>,
    mut commands: Commands,
    position_query: Query<&Position>,
    time: Res<Time>,
) {
    if !sync_manager.cooperative_actions || sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    let current_time = time.elapsed_seconds_f64();
    
    // Group heal action (H key)
    if keyboard_input.just_pressed(KeyCode::H) {
        if let Some(initiator) = sync_manager.get_controlled_entity() {
            let shared_action = SharedAction {
                action_type: SharedActionType::GroupHeal { heal_amount: 20 },
                initiator,
                participants: sync_manager.active_party.clone(),
                required_participants: 2,
                completion_time: current_time + 2.0, // 2 second cast time
                auto_execute: false,
            };
            
            commands.spawn(shared_action);
            info!("Initiated group heal action");
        }
    }
    
    // Group defense action (G key)
    if keyboard_input.just_pressed(KeyCode::G) {
        if let Some(initiator) = sync_manager.get_controlled_entity() {
            let shared_action = SharedAction {
                action_type: SharedActionType::GroupDefense { defense_bonus: 5 },
                initiator,
                participants: sync_manager.active_party.clone(),
                required_participants: 3,
                completion_time: current_time + 1.0,
                auto_execute: true,
            };
            
            commands.spawn(shared_action);
            info!("Initiated group defense action");
        }
    }
    
    // Coordinated search action (Shift + S)
    if keyboard_input.pressed(KeyCode::LShift) && keyboard_input.just_pressed(KeyCode::S) {
        if let Some(initiator) = sync_manager.get_controlled_entity() {
            if let Ok(position) = position_query.get(initiator) {
                let shared_action = SharedAction {
                    action_type: SharedActionType::CoordinatedSearch { 
                        search_area: position.0 
                    },
                    initiator,
                    participants: sync_manager.active_party.clone(),
                    required_participants: 2,
                    completion_time: current_time + 3.0,
                    auto_execute: true,
                };
                
                commands.spawn(shared_action);
                info!("Initiated coordinated search action");
            }
        }
    }
}

/// System for handling resource sharing input
pub fn resource_sharing_input_system(
    mut sync_manager: ResMut<SyncExplorationManager>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    // Share gold (Shift + G)
    if keyboard_input.pressed(KeyCode::LShift) && keyboard_input.just_pressed(KeyCode::G) {
        sync_manager.add_shared_resource("gold", 10);
        info!("Added 10 gold to shared resources. Total: {}", sync_manager.get_shared_resource("gold"));
    }
    
    // Share supplies (Shift + U)
    if keyboard_input.pressed(KeyCode::LShift) && keyboard_input.just_pressed(KeyCode::U) {
        sync_manager.add_shared_resource("supplies", 5);
        info!("Added 5 supplies to shared resources. Total: {}", sync_manager.get_shared_resource("supplies"));
    }
    
    // Use shared gold (Ctrl + G)
    if keyboard_input.pressed(KeyCode::LControl) && keyboard_input.just_pressed(KeyCode::G) {
        if sync_manager.remove_shared_resource("gold", 5) {
            info!("Used 5 shared gold. Remaining: {}", sync_manager.get_shared_resource("gold"));
        } else {
            info!("Not enough shared gold available");
        }
    }
    
    // Use shared supplies (Ctrl + U)
    if keyboard_input.pressed(KeyCode::LControl) && keyboard_input.just_pressed(KeyCode::U) {
        if sync_manager.remove_shared_resource("supplies", 1) {
            info!("Used 1 shared supply. Remaining: {}", sync_manager.get_shared_resource("supplies"));
        } else {
            info!("Not enough shared supplies available");
        }
    }
}

/// System for displaying synchronous exploration UI
pub fn sync_exploration_ui_system(
    sync_manager: Res<SyncExplorationManager>,
    mut ui_elements: ResMut<Vec<crate::ui::UIElement>>,
    name_query: Query<&crate::components::Name>,
) {
    if sync_manager.mode == SyncExplorationMode::Disabled {
        return;
    }
    
    // Display current mode
    ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
        x: 2,
        y: 2,
        text: format!("Sync Mode: {:?}", sync_manager.mode),
        color: Some((255, 255, 0)),
    }));
    
    // Display controlled entity
    if let Some(controlled) = sync_manager.get_controlled_entity() {
        let name = name_query.get(controlled)
            .map(|n| n.name.clone())
            .unwrap_or_else(|_| format!("Entity {:?}", controlled));
        
        ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
            x: 2,
            y: 3,
            text: format!("Controlling: {}", name),
            color: Some((0, 255, 0)),
        }));
    }
    
    // Display party members
    ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
        x: 2,
        y: 5,
        text: format!("Party ({}):", sync_manager.active_party.len()),
        color: Some((255, 255, 255)),
    }));
    
    let mut y = 6;
    for (i, &entity) in sync_manager.active_party.iter().enumerate() {
        let name = name_query.get(entity)
            .map(|n| n.name.clone())
            .unwrap_or_else(|_| format!("Entity {:?}", entity));
        
        let is_controlled = sync_manager.get_controlled_entity() == Some(entity);
        let is_leader = sync_manager.get_party_leader() == Some(entity);
        
        let mut status = String::new();
        if is_leader {
            status.push_str(" [L]");
        }
        if is_controlled {
            status.push_str(" [C]");
        }
        
        let color = if is_controlled {
            Some((0, 255, 0))
        } else if is_leader {
            Some((255, 255, 0))
        } else {
            Some((200, 200, 200))
        };
        
        ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
            x: 4,
            y,
            text: format!("{}. {}{}", i + 1, name, status),
            color,
        }));
        
        // Display action points in turn-based mode
        if sync_manager.turn_based_mode {
            let action_points = sync_manager.get_action_points(entity);
            ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
                x: 30,
                y,
                text: format!("AP: {}", action_points),
                color: Some((100, 150, 255)),
            }));
        }
        
        y += 1;
    }
    
    // Display formation type
    ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
        x: 2,
        y: y + 1,
        text: format!("Formation: {:?}", sync_manager.formation_type),
        color: Some((255, 200, 100)),
    }));
    
    // Display settings
    y += 3;
    ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
        x: 2,
        y,
        text: format!("Auto-follow: {} | Shared vision: {} | Cooperative: {} | Turn-based: {}",
            if sync_manager.auto_follow { "ON" } else { "OFF" },
            if sync_manager.shared_vision { "ON" } else { "OFF" },
            if sync_manager.cooperative_actions { "ON" } else { "OFF" },
            if sync_manager.turn_based_mode { "ON" } else { "OFF" }),
        color: Some((150, 150, 150)),
    }));
    
    // Display shared resources
    if !sync_manager.shared_resources.is_empty() {
        y += 2;
        ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
            x: 2,
            y,
            text: "Shared Resources:".to_string(),
            color: Some((255, 255, 255)),
        }));
        
        y += 1;
        for (resource, amount) in &sync_manager.shared_resources {
            ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
                x: 4,
                y,
                text: format!("{}: {}", resource, amount),
                color: Some((200, 255, 200)),
            }));
            y += 1;
        }
    }
    
    // Display controls
    y += 2;
    ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
        x: 2,
        y,
        text: "Controls: [F1] Mode | [Tab] Switch | [1-6] Select | [F] Formation | [P] Add | [R] Remove".to_string(),
        color: Some((100, 100, 100)),
    }));
    
    y += 1;
    ui_elements.push(crate::ui::UIElement::Text(crate::ui::UIText {
        x: 2,
        y,
        text: "[F2] Auto-follow | [F3] Shared vision | [F4] Cooperative | [F5] Turn-based".to_string(),
        color: Some((100, 100, 100)),
    }));
}