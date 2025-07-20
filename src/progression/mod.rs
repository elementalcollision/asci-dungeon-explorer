pub mod milestone_system;
pub mod unlockable_content;
pub mod world_changes;
pub mod player_history;
pub mod progression_integration;

pub use milestone_system::{
    MilestoneSystem, Milestone, MilestoneType, MilestoneImportance, MilestoneStatus,
    MilestoneCondition, MilestoneProgress, MilestoneReward, CompletedMilestone,
    MilestoneStatistics, MilestoneSaveData,
};

pub use unlockable_content::{
    UnlockableContentSystem, UnlockableContent, ContentType, ContentRarity,
    UnlockCondition, UnlockedContentRecord, ContentUnlockStatistics,
    UnlockableContentSaveData,
};

pub use world_changes::{
    WorldChangesSystem, WorldChange, WorldChangeType, PersistenceLevel, ChangeScope,
    WorldChangeEvent, WorldChangeEventType, WorldChangeStatistics, WorldChangesSaveData,
};

pub use player_history::{
    PlayerHistorySystem, HistoryEvent, HistoryEventType, EventImportance,
    GameSession, HistoryStatistics, PlayerHistorySaveData,
};

pub use progression_integration::{
    ProgressionIntegration, ProgressionStatistics, ProgressionSaveData,
};