use crossterm::event::{KeyEvent, KeyCode};
use crate::game_state::{RunState, GameState};
use crate::components::{AttributeType, ClassType, BackgroundType};
use super::CharacterCreationState;

pub fn handle_character_creation_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match game_state.run_state {
        RunState::CharacterName => handle_name_input(key_event, game_state, creation_state),
        RunState::CharacterClass => handle_class_input(key_event, game_state, creation_state),
        RunState::CharacterBackground => handle_background_input(key_event, game_state, creation_state),
        RunState::CharacterAttributes => handle_attributes_input(key_event, game_state, creation_state),
        RunState::CharacterEquipment => handle_equipment_input(key_event, game_state, creation_state),
        RunState::CharacterConfirm => handle_confirm_input(key_event, game_state, creation_state),
        _ => false,
    }
}

fn handle_name_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match key_event.code {
        KeyCode::Char(c) => {
            if creation_state.player_name.len() < 20 {
                creation_state.player_name.push(c);
            }
            true
        },
        KeyCode::Backspace => {
            creation_state.player_name.pop();
            true
        },
        KeyCode::Enter => {
            if !creation_state.player_name.is_empty() {
                game_state.run_state = RunState::CharacterClass;
            }
            true
        },
        KeyCode::Esc => {
            game_state.run_state = RunState::MainMenu;
            true
        },
        _ => false,
    }
}

fn handle_class_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match key_event.code {
        KeyCode::Char('1') | KeyCode::Char('f') => {
            creation_state.selected_class = ClassType::Fighter;
            true
        },
        KeyCode::Char('2') | KeyCode::Char('r') => {
            creation_state.selected_class = ClassType::Rogue;
            true
        },
        KeyCode::Char('3') | KeyCode::Char('m') => {
            creation_state.selected_class = ClassType::Mage;
            true
        },
        KeyCode::Char('4') | KeyCode::Char('c') => {
            creation_state.selected_class = ClassType::Cleric;
            true
        },
        KeyCode::Char('5') | KeyCode::Char('a') => {
            creation_state.selected_class = ClassType::Ranger;
            true
        },
        KeyCode::Enter => {
            game_state.run_state = RunState::CharacterBackground;
            true
        },
        KeyCode::Esc => {
            game_state.run_state = RunState::CharacterName;
            true
        },
        _ => false,
    }
}

fn handle_background_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match key_event.code {
        KeyCode::Char('1') | KeyCode::Char('s') => {
            creation_state.selected_background = BackgroundType::Soldier;
            true
        },
        KeyCode::Char('2') | KeyCode::Char('c') => {
            creation_state.selected_background = BackgroundType::Scholar;
            true
        },
        KeyCode::Char('3') | KeyCode::Char('n') => {
            creation_state.selected_background = BackgroundType::Noble;
            true
        },
        KeyCode::Char('4') | KeyCode::Char('o') => {
            creation_state.selected_background = BackgroundType::Outlaw;
            true
        },
        KeyCode::Char('5') | KeyCode::Char('a') => {
            creation_state.selected_background = BackgroundType::Acolyte;
            true
        },
        KeyCode::Char('6') | KeyCode::Char('m') => {
            creation_state.selected_background = BackgroundType::Merchant;
            true
        },
        KeyCode::Enter => {
            // Apply class and background bonuses before moving to attributes
            creation_state.apply_class_bonuses();
            creation_state.apply_background_bonuses();
            game_state.run_state = RunState::CharacterAttributes;
            true
        },
        KeyCode::Esc => {
            game_state.run_state = RunState::CharacterClass;
            true
        },
        _ => false,
    }
}

fn handle_attributes_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            // Move selection up
            creation_state.selected_attribute = match creation_state.selected_attribute {
                AttributeType::Strength => AttributeType::Charisma,
                AttributeType::Dexterity => AttributeType::Strength,
                AttributeType::Constitution => AttributeType::Dexterity,
                AttributeType::Intelligence => AttributeType::Constitution,
                AttributeType::Wisdom => AttributeType::Intelligence,
                AttributeType::Charisma => AttributeType::Wisdom,
            };
            true
        },
        KeyCode::Down | KeyCode::Char('j') => {
            // Move selection down
            creation_state.selected_attribute = match creation_state.selected_attribute {
                AttributeType::Strength => AttributeType::Dexterity,
                AttributeType::Dexterity => AttributeType::Constitution,
                AttributeType::Constitution => AttributeType::Intelligence,
                AttributeType::Intelligence => AttributeType::Wisdom,
                AttributeType::Wisdom => AttributeType::Charisma,
                AttributeType::Charisma => AttributeType::Strength,
            };
            true
        },
        KeyCode::Left | KeyCode::Char('h') => {
            // Decrease selected attribute
            creation_state.attributes.decrease_attribute(creation_state.selected_attribute);
            true
        },
        KeyCode::Right | KeyCode::Char('l') => {
            // Increase selected attribute
            creation_state.attributes.increase_attribute(creation_state.selected_attribute);
            true
        },
        KeyCode::Enter => {
            if creation_state.attributes.unspent_points == 0 {
                game_state.run_state = RunState::CharacterEquipment;
            }
            true
        },
        KeyCode::Esc => {
            // Reset attributes and go back
            creation_state.attributes = Attributes::new();
            game_state.run_state = RunState::CharacterBackground;
            true
        },
        _ => false,
    }
}

fn handle_equipment_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match key_event.code {
        KeyCode::Up | KeyCode::Char('k') => {
            // Move selection up
            if creation_state.selected_equipment > 0 {
                creation_state.selected_equipment -= 1;
            } else {
                creation_state.selected_equipment = creation_state.available_equipment.len() - 1;
            }
            true
        },
        KeyCode::Down | KeyCode::Char('j') => {
            // Move selection down
            creation_state.selected_equipment = (creation_state.selected_equipment + 1) % creation_state.available_equipment.len();
            true
        },
        KeyCode::Char(' ') => {
            // Toggle selection
            let idx = creation_state.selected_equipment;
            if creation_state.selected_equipment_indices.contains(&idx) {
                creation_state.selected_equipment_indices.retain(|&i| i != idx);
            } else if creation_state.selected_equipment_indices.len() < 3 {
                // Limit to 3 starting equipment items
                creation_state.selected_equipment_indices.push(idx);
            }
            true
        },
        KeyCode::Enter => {
            game_state.run_state = RunState::CharacterConfirm;
            true
        },
        KeyCode::Esc => {
            game_state.run_state = RunState::CharacterAttributes;
            true
        },
        _ => false,
    }
}

fn handle_confirm_input(key_event: KeyEvent, game_state: &mut GameState, creation_state: &mut CharacterCreationState) -> bool {
    match key_event.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            // Create the player and start the game
            let player_x = 40;
            let player_y = 25;
            let player = creation_state.create_player(&mut game_state.world, player_x, player_y);
            game_state.player = Some(player);
            game_state.run_state = RunState::PreRun;
            true
        },
        KeyCode::Char('n') | KeyCode::Esc => {
            game_state.run_state = RunState::CharacterEquipment;
            true
        },
        _ => false,
    }
}