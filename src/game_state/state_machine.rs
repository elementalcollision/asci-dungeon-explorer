use serde::{Serialize, Deserialize};

#[derive(PartialEq, Eq, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum StateType {
    MainMenu,
    Playing,
    Inventory,
    CharacterSheet,
    GameOver,
    LevelUp,
    Targeting,
    SaveGame,
    LoadGame,
    Options,
    Help,
    Pause,
    GuildManagement,
    MissionAssignment,
    AgentConfiguration,
}