pub mod serialization;
pub mod save_system;
pub mod world_serializer;
pub mod version_manager;
pub mod save_load_system;
pub mod autosave_system;
pub mod crash_recovery;
pub mod save_rotation;
pub mod save_cleanup;
pub mod game_persistence_integration;
pub mod usage_example;
pub mod autosave_integration_example;

pub use serialization::{
    SerializationSystem, SerializableComponent, ComponentSerializer, SerializationError,
    SerializationResult, SaveData, LoadData
};
pub use save_system::{
    SaveSystem, SaveSlot, SaveMetadata, SaveFile, SaveError, SaveResult
};
pub use world_serializer::{
    WorldSerializer, WorldState, EntityData, ComponentData, ResourceData
};
pub use version_manager::{
    VersionManager, SaveVersion, VersionCompatibility, MigrationResult
};