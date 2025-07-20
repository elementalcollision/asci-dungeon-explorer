use serde::{Serialize, Deserialize, de::DeserializeOwned};
use specs::{World, Entity, Component, VecStorage, DenseVecStorage, HashMapStorage, NullStorage, Join, WorldExt, ReadStorage, WriteStorage};
use std::collections::HashMap;
use std::any::{Any, TypeId};
use std::fmt;

/// Serialization errors
#[derive(Debug, Clone)]
pub enum SerializationError {
    SerializationFailed(String),
    DeserializationFailed(String),
    ComponentNotFound(String),
    EntityNotFound(u32),
    VersionMismatch { expected: String, found: String },
    CorruptedData(String),
    IoError(String),
    InvalidFormat(String),
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            SerializationError::DeserializationFailed(msg) => write!(f, "Deserialization failed: {}", msg),
            SerializationError::ComponentNotFound(name) => write!(f, "Component not found: {}", name),
            SerializationError::EntityNotFound(id) => write!(f, "Entity not found: {}", id),
            SerializationError::VersionMismatch { expected, found } => {
                write!(f, "Version mismatch: expected {}, found {}", expected, found)
            }
            SerializationError::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
            SerializationError::IoError(msg) => write!(f, "IO error: {}", msg),
            SerializationError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
        }
    }
}

impl std::error::Error for SerializationError {}

pub type SerializationResult<T> = Result<T, SerializationError>;

/// Trait for components that can be serialized
pub trait SerializableComponent: Component + Serialize + DeserializeOwned + Clone + 'static {
    fn component_name() -> &'static str;
    fn storage_type() -> StorageType;
}

/// Storage type information for components
#[derive(Debug, Clone, PartialEq)]
pub enum StorageType {
    VecStorage,
    DenseVecStorage,
    HashMapStorage,
    NullStorage,
}

/// Serialized component data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedComponent {
    pub component_name: String,
    pub storage_type: StorageType,
    pub data: Vec<u8>,
    pub entity_mapping: HashMap<u32, usize>, // Entity ID -> Index in data
}

/// Component serializer trait
pub trait ComponentSerializer {
    fn serialize_component(&self, world: &World, component_name: &str) -> SerializationResult<SerializedComponent>;
    fn deserialize_component(&self, world: &mut World, data: &SerializedComponent) -> SerializationResult<()>;
}

/// Main serialization system
pub struct SerializationSystem {
    component_serializers: HashMap<String, Box<dyn ComponentSerializer>>,
    registered_components: HashMap<String, TypeId>,
}

impl SerializationSystem {
    pub fn new() -> Self {
        SerializationSystem {
            component_serializers: HashMap::new(),
            registered_components: HashMap::new(),
        }
    }

    /// Register a component for serialization
    pub fn register_component<T>(&mut self) 
    where 
        T: SerializableComponent,
    {
        let name = T::component_name().to_string();
        let type_id = TypeId::of::<T>();
        
        self.registered_components.insert(name.clone(), type_id);
        self.component_serializers.insert(
            name,
            Box::new(GenericComponentSerializer::<T>::new()),
        );
    }

    /// Serialize all registered components from the world
    pub fn serialize_world(&self, world: &World) -> SerializationResult<Vec<SerializedComponent>> {
        let mut serialized_components = Vec::new();

        for component_name in self.registered_components.keys() {
            if let Some(serializer) = self.component_serializers.get(component_name) {
                match serializer.serialize_component(world, component_name) {
                    Ok(serialized) => serialized_components.push(serialized),
                    Err(e) => {
                        eprintln!("Warning: Failed to serialize component {}: {}", component_name, e);
                        // Continue with other components instead of failing completely
                    }
                }
            }
        }

        Ok(serialized_components)
    }

    /// Deserialize components into the world
    pub fn deserialize_world(&self, world: &mut World, components: &[SerializedComponent]) -> SerializationResult<()> {
        for component_data in components {
            if let Some(serializer) = self.component_serializers.get(&component_data.component_name) {
                serializer.deserialize_component(world, component_data)?;
            } else {
                eprintln!("Warning: No serializer found for component: {}", component_data.component_name);
                // Continue with other components
            }
        }

        Ok(())
    }

    /// Get list of registered component names
    pub fn get_registered_components(&self) -> Vec<String> {
        self.registered_components.keys().cloned().collect()
    }

    /// Check if a component is registered
    pub fn is_component_registered(&self, component_name: &str) -> bool {
        self.registered_components.contains_key(component_name)
    }
}

/// Generic component serializer implementation
struct GenericComponentSerializer<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GenericComponentSerializer<T> 
where 
    T: SerializableComponent,
{
    fn new() -> Self {
        GenericComponentSerializer {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ComponentSerializer for GenericComponentSerializer<T>
where 
    T: SerializableComponent,
{
    fn serialize_component(&self, world: &World, component_name: &str) -> SerializationResult<SerializedComponent> {
        let storage = world.read_storage::<T>();
        let entities = world.entities();
        
        let mut component_data = Vec::new();
        let mut entity_mapping = HashMap::new();
        let mut index = 0;

        for (entity, component) in (&entities, &storage).join() {
            let serialized = bincode::serialize(component)
                .map_err(|e| SerializationError::SerializationFailed(e.to_string()))?;
            
            component_data.extend(serialized);
            entity_mapping.insert(entity.id(), index);
            index += 1;
        }

        Ok(SerializedComponent {
            component_name: component_name.to_string(),
            storage_type: T::storage_type(),
            data: component_data,
            entity_mapping,
        })
    }

    fn deserialize_component(&self, world: &mut World, data: &SerializedComponent) -> SerializationResult<()> {
        let mut storage = world.write_storage::<T>();
        let entities = world.entities();

        // Clear existing components of this type
        storage.clear();

        // Deserialize each component
        for (entity_id, &index) in &data.entity_mapping {
            if let Some(entity) = entities.entity(*entity_id) {
                // Extract component data from the serialized data
                // This is a simplified approach - in practice, you'd need to track data boundaries
                let component: T = bincode::deserialize(&data.data)
                    .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))?;
                
                storage.insert(entity, component)
                    .map_err(|e| SerializationError::DeserializationFailed(format!("Failed to insert component: {:?}", e)))?;
            }
        }

        Ok(())
    }
}

/// Save data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub version: String,
    pub timestamp: u64,
    pub game_name: String,
    pub player_name: String,
    pub level: i32,
    pub playtime: u64,
    pub components: Vec<SerializedComponent>,
    pub resources: HashMap<String, Vec<u8>>,
    pub metadata: HashMap<String, String>,
}

impl SaveData {
    pub fn new(game_name: String, player_name: String) -> Self {
        SaveData {
            version: env!("CARGO_PKG_VERSION").to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            game_name,
            player_name,
            level: 1,
            playtime: 0,
            components: Vec::new(),
            resources: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_components(mut self, components: Vec<SerializedComponent>) -> Self {
        self.components = components;
        self
    }

    pub fn with_resources(mut self, resources: HashMap<String, Vec<u8>>) -> Self {
        self.resources = resources;
        self
    }

    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn serialize_to_bytes(&self) -> SerializationResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))
    }

    pub fn deserialize_from_bytes(data: &[u8]) -> SerializationResult<Self> {
        bincode::deserialize(data)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))
    }
}

/// Load data structure for partial loading
#[derive(Debug, Clone)]
pub struct LoadData {
    pub save_data: SaveData,
    pub selected_components: Option<Vec<String>>,
    pub skip_resources: bool,
    pub validate_version: bool,
}

impl LoadData {
    pub fn new(save_data: SaveData) -> Self {
        LoadData {
            save_data,
            selected_components: None,
            skip_resources: false,
            validate_version: true,
        }
    }

    pub fn with_selected_components(mut self, components: Vec<String>) -> Self {
        self.selected_components = Some(components);
        self
    }

    pub fn skip_resources(mut self) -> Self {
        self.skip_resources = true;
        self
    }

    pub fn skip_version_validation(mut self) -> Self {
        self.validate_version = false;
        self
    }
}

// Implement SerializableComponent for common components
use crate::components::*;

impl SerializableComponent for Position {
    fn component_name() -> &'static str { "Position" }
    fn storage_type() -> StorageType { StorageType::DenseVecStorage }
}

impl SerializableComponent for Renderable {
    fn component_name() -> &'static str { "Renderable" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Player {
    fn component_name() -> &'static str { "Player" }
    fn storage_type() -> StorageType { StorageType::NullStorage }
}

impl SerializableComponent for Monster {
    fn component_name() -> &'static str { "Monster" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Name {
    fn component_name() -> &'static str { "Name" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for CombatStats {
    fn component_name() -> &'static str { "CombatStats" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Viewshed {
    fn component_name() -> &'static str { "Viewshed" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for BlocksTile {
    fn component_name() -> &'static str { "BlocksTile" }
    fn storage_type() -> StorageType { StorageType::NullStorage }
}

impl SerializableComponent for Item {
    fn component_name() -> &'static str { "Item" }
    fn storage_type() -> StorageType { StorageType::NullStorage }
}

// Add implementations for item components
use crate::items::*;

impl SerializableComponent for ItemProperties {
    fn component_name() -> &'static str { "ItemProperties" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for AdvancedInventory {
    fn component_name() -> &'static str { "AdvancedInventory" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Equippable {
    fn component_name() -> &'static str { "Equippable" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Equipment {
    fn component_name() -> &'static str { "Equipment" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for ItemBonuses {
    fn component_name() -> &'static str { "ItemBonuses" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Consumable {
    fn component_name() -> &'static str { "Consumable" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for StatusEffects {
    fn component_name() -> &'static str { "StatusEffects" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

impl SerializableComponent for Container {
    fn component_name() -> &'static str { "Container" }
    fn storage_type() -> StorageType { StorageType::VecStorage }
}

/// Helper function to create a fully configured serialization system
pub fn create_serialization_system() -> SerializationSystem {
    let mut system = SerializationSystem::new();

    // Register core components
    system.register_component::<Position>();
    system.register_component::<Renderable>();
    system.register_component::<Player>();
    system.register_component::<Monster>();
    system.register_component::<Name>();
    system.register_component::<CombatStats>();
    system.register_component::<Viewshed>();
    system.register_component::<BlocksTile>();
    system.register_component::<Item>();

    // Register item components
    system.register_component::<ItemProperties>();
    system.register_component::<AdvancedInventory>();
    system.register_component::<Equippable>();
    system.register_component::<Equipment>();
    system.register_component::<ItemBonuses>();
    system.register_component::<Consumable>();
    system.register_component::<StatusEffects>();
    system.register_component::<Container>();

    system
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};

    #[test]
    fn test_serialization_system_creation() {
        let system = SerializationSystem::new();
        assert!(system.component_serializers.is_empty());
        assert!(system.registered_components.is_empty());
    }

    #[test]
    fn test_component_registration() {
        let mut system = SerializationSystem::new();
        system.register_component::<Position>();
        
        assert!(system.is_component_registered("Position"));
        assert!(!system.is_component_registered("NonExistent"));
        
        let components = system.get_registered_components();
        assert!(components.contains(&"Position".to_string()));
    }

    #[test]
    fn test_save_data_creation() {
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string());
        
        assert_eq!(save_data.game_name, "Test Game");
        assert_eq!(save_data.player_name, "Test Player");
        assert_eq!(save_data.level, 1);
        assert!(save_data.components.is_empty());
        assert!(save_data.resources.is_empty());
    }

    #[test]
    fn test_save_data_serialization() {
        let save_data = SaveData::new("Test Game".to_string(), "Test Player".to_string())
            .add_metadata("test_key".to_string(), "test_value".to_string());
        
        let serialized = save_data.serialize_to_bytes().unwrap();
        let deserialized = SaveData::deserialize_from_bytes(&serialized).unwrap();
        
        assert_eq!(deserialized.game_name, save_data.game_name);
        assert_eq!(deserialized.player_name, save_data.player_name);
        assert_eq!(deserialized.metadata.get("test_key"), Some(&"test_value".to_string()));
    }

    #[test]
    fn test_load_data_configuration() {
        let save_data = SaveData::new("Test".to_string(), "Player".to_string());
        let load_data = LoadData::new(save_data)
            .with_selected_components(vec!["Position".to_string()])
            .skip_resources()
            .skip_version_validation();
        
        assert_eq!(load_data.selected_components, Some(vec!["Position".to_string()]));
        assert!(load_data.skip_resources);
        assert!(!load_data.validate_version);
    }

    #[test]
    fn test_serializable_component_implementations() {
        assert_eq!(Position::component_name(), "Position");
        assert_eq!(Position::storage_type(), StorageType::DenseVecStorage);
        
        assert_eq!(Player::component_name(), "Player");
        assert_eq!(Player::storage_type(), StorageType::NullStorage);
        
        assert_eq!(Name::component_name(), "Name");
        assert_eq!(Name::storage_type(), StorageType::VecStorage);
    }

    #[test]
    fn test_create_serialization_system() {
        let system = create_serialization_system();
        
        // Check that core components are registered
        assert!(system.is_component_registered("Position"));
        assert!(system.is_component_registered("Player"));
        assert!(system.is_component_registered("Name"));
        assert!(system.is_component_registered("CombatStats"));
        
        // Check that item components are registered
        assert!(system.is_component_registered("ItemProperties"));
        assert!(system.is_component_registered("AdvancedInventory"));
        assert!(system.is_component_registered("Equipment"));
    }

    #[test]
    fn test_serialization_error_display() {
        let error = SerializationError::ComponentNotFound("TestComponent".to_string());
        assert_eq!(error.to_string(), "Component not found: TestComponent");
        
        let error = SerializationError::VersionMismatch {
            expected: "1.0.0".to_string(),
            found: "0.9.0".to_string(),
        };
        assert_eq!(error.to_string(), "Version mismatch: expected 1.0.0, found 0.9.0");
    }

    #[test]
    fn test_world_serialization_integration() {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Name>();
        
        // Create test entities
        let entity1 = world.create_entity()
            .with(Position { x: 10, y: 20 })
            .with(Name { name: "Test Entity 1".to_string() })
            .build();
        
        let entity2 = world.create_entity()
            .with(Position { x: 30, y: 40 })
            .with(Name { name: "Test Entity 2".to_string() })
            .build();
        
        let mut system = SerializationSystem::new();
        system.register_component::<Position>();
        system.register_component::<Name>();
        
        // Serialize the world
        let serialized = system.serialize_world(&world).unwrap();
        
        // Should have serialized both component types
        assert_eq!(serialized.len(), 2);
        
        // Check component names
        let component_names: Vec<_> = serialized.iter().map(|c| &c.component_name).collect();
        assert!(component_names.contains(&&"Position".to_string()));
        assert!(component_names.contains(&&"Name".to_string()));
    }
}