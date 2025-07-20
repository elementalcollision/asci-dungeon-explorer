use bevy::prelude::*;
use crate::guild::guild_ui_types::GuildUI;
use crate::guild::guild_ui_input::{guild_ui_input_system, guild_ui_action_system};
use crate::guild::guild_ui_render::guild_ui_render_system;

/// Plugin for guild UI
pub struct GuildUIPlugin;

impl Plugin for GuildUIPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GuildUI>()
           .add_systems(Update, (
               guild_ui_input_system,
               guild_ui_action_system,
               guild_ui_render_system,
           ).chain());
    }
}