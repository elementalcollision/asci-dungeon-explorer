use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use specs::{World, WorldExt, Join};
use crate::components::{PlayerInput, Player};
use crate::game_state::{RunState, GameState};

pub fn handle_input(world: &mut World, event: Event, game_state: &mut GameState) -> bool {
    match event {
        Event::Key(key_event) => handle_key_event(world, key_event, game_state),
        _ => false,
    }
}

fn handle_key_event(world: &mut World, key_event: KeyEvent, game_state: &mut GameState) -> bool {
    // If we're not in the player's turn, ignore input
    if game_state.run_state != RunState::PlayerTurn {
        return false;
    }
    
    // Get player input component
    let mut player_input = world.write_storage::<PlayerInput>();
    let players = world.read_storage::<Player>();
    
    for (_player, input) in (&players, &mut player_input).join() {
        match key_event.code {
            // Movement keys
            KeyCode::Up | KeyCode::Char('k') => {
                input.move_intent = Some((0, -1));
                return true;
            },
            KeyCode::Down | KeyCode::Char('j') => {
                input.move_intent = Some((0, 1));
                return true;
            },
            KeyCode::Left | KeyCode::Char('h') => {
                input.move_intent = Some((-1, 0));
                return true;
            },
            KeyCode::Right | KeyCode::Char('l') => {
                input.move_intent = Some((1, 0));
                return true;
            },
            // Diagonal movement
            KeyCode::Char('y') => {
                input.move_intent = Some((-1, -1));
                return true;
            },
            KeyCode::Char('u') => {
                input.move_intent = Some((1, -1));
                return true;
            },
            KeyCode::Char('b') => {
                input.move_intent = Some((-1, 1));
                return true;
            },
            KeyCode::Char('n') => {
                input.move_intent = Some((1, 1));
                return true;
            },
            // Wait
            KeyCode::Char('.') => {
                input.wait_intent = true;
                return true;
            },
            // Pickup item
            KeyCode::Char('g') | KeyCode::Char(',') => {
                input.pickup_intent = true;
                return true;
            },
            // Inventory
            KeyCode::Char('i') => {
                game_state.run_state = RunState::ShowInventory;
                return true;
            },
            // Drop item
            KeyCode::Char('d') => {
                game_state.run_state = RunState::ShowDropItem;
                return true;
            },
            // Examine
            KeyCode::Char('x') => {
                game_state.run_state = RunState::Examine;
                return true;
            },
            // Quit
            KeyCode::Char('q') => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    game_state.run_state = RunState::QuitGame;
                    return true;
                }
                return false;
            },
            // Save
            KeyCode::Char('s') => {
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    game_state.run_state = RunState::SaveGame;
                    return true;
                }
                return false;
            },
            _ => return false,
        }
    }
    
    false
}