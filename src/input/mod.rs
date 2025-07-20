use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveUpLeft,
    MoveUpRight,
    MoveDownLeft,
    MoveDownRight,
    Wait,
    PickupItem,
    ShowInventory,
    ShowCharacterSheet,
    UseStairs,
    SaveGame,
    Quit,
    NoAction,
}

pub fn handle_player_input(key_event: KeyEvent) -> PlayerAction {
    match key_event.code {
        // Movement keys
        KeyCode::Left | KeyCode::Char('h') => PlayerAction::MoveLeft,
        KeyCode::Right | KeyCode::Char('l') => PlayerAction::MoveRight,
        KeyCode::Up | KeyCode::Char('k') => PlayerAction::MoveUp,
        KeyCode::Down | KeyCode::Char('j') => PlayerAction::MoveDown,
        KeyCode::Char('y') => PlayerAction::MoveUpLeft,
        KeyCode::Char('u') => PlayerAction::MoveUpRight,
        KeyCode::Char('b') => PlayerAction::MoveDownLeft,
        KeyCode::Char('n') => PlayerAction::MoveDownRight,
        
        // Action keys
        KeyCode::Char('.') | KeyCode::Char(' ') => PlayerAction::Wait,
        KeyCode::Char('g') => PlayerAction::PickupItem,
        KeyCode::Char('i') => PlayerAction::ShowInventory,
        KeyCode::Char('c') => PlayerAction::ShowCharacterSheet,
        KeyCode::Char('>') => PlayerAction::UseStairs,
        
        // System keys
        KeyCode::Char('s') if key_event.modifiers.contains(KeyModifiers::CONTROL) => PlayerAction::SaveGame,
        KeyCode::Char('q') => PlayerAction::Quit,
        
        _ => PlayerAction::NoAction,
    }
}