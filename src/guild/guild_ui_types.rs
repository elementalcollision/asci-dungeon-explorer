use bevy::prelude::*;
use crate::ui::{UIAction};

/// Guild UI state
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GuildUIState {
    Hidden,
    Main,
    Members,
    Missions,
    Facilities,
    Resources,
    AgentConfig,
}

/// Guild UI resource
#[derive(Resource)]
pub struct GuildUI {
    pub state: GuildUIState,
    pub selected_guild: Option<String>,
    pub selected_member: Option<Entity>,
    pub selected_mission: Option<String>,
    pub selected_facility: Option<crate::guild::guild_core::GuildFacility>,
    pub scroll_offset: usize,
    pub filter: String,
    pub show_completed_missions: bool,
    pub show_failed_missions: bool,
}

impl Default for GuildUI {
    fn default() -> Self {
        GuildUI {
            state: GuildUIState::Hidden,
            selected_guild: None,
            selected_member: None,
            selected_mission: None,
            selected_facility: None,
            scroll_offset: 0,
            filter: String::new(),
            show_completed_missions: false,
            show_failed_missions: false,
        }
    }
}

/// Guild UI action
#[derive(Debug, Clone)]
pub enum GuildUIAction {
    ToggleUI,
    SetState(GuildUIState),
    SelectGuild(String),
    SelectMember(Entity),
    SelectMission(String),
    SelectFacility(crate::guild::guild_core::GuildFacility),
    AssignMission(Entity, String),
    ConfigureAgent(Entity),
    SetAgentBehavior(Entity, crate::guild::agent_behavior::AgentBehaviorType),
    UpgradeAgentStat(Entity, String),
    UpgradeFacility(crate::guild::guild_core::GuildFacility),
    BuildFacility(crate::guild::guild_core::GuildFacility),
    ScrollUp,
    ScrollDown,
    SetFilter(String),
    ToggleShowCompletedMissions,
    ToggleShowFailedMissions,
}