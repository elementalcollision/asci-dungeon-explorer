pub mod achievement_system;
pub mod achievement_ui;
pub mod achievement_notifications;
pub mod achievement_storage;
pub mod achievement_integration;

pub use achievement_system::{
    AchievementSystem, Achievement, AchievementType, AchievementRarity, AchievementDifficulty,
    AchievementProgress, AchievementReward, UnlockedAchievement, AchievementNotification,
    AchievementStatistics, AchievementSaveData, GameEvent,
};

pub use achievement_ui::{
    AchievementUI, AchievementUIState, AchievementSortMode, AchievementFilter,
    AchievementNotificationPopup,
};

pub use achievement_notifications::{
    AchievementNotificationSystem, NotificationConfig, NotificationStyle, 
    NotificationAnimation, NotificationPosition, ActiveNotification, AchievementSoundSystem,
};

pub use achievement_storage::{
    AchievementStorage, AchievementStorageConfig, AchievementStorageError,
    AchievementStorageMetadata, StorageStatistics,
};

pub use achievement_integration::{
    AchievementIntegration, AchievementIntegrationBuilder, AchievementSystemStatus,
};