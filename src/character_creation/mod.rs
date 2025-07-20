use specs::{World, WorldExt, Entity, Builder};
use crossterm::event::{KeyEvent, KeyCode};
use crossterm::style::Color;

use crate::components::*;
use crate::game_state::{RunState, GameState};
use crate::rendering::terminal::with_terminal;
use crate::resources::GameLog;

mod state;
mod input_handler;
mod renderer;

pub use state::CharacterCreationState;
pub use input_handler::handle_character_creation_input;
pub use renderer::render_character_creation;