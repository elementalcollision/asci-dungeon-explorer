use serde::{Serialize, Deserialize};
use specs::{World, Entity, WorldExt, Builder, Join};
use std::collections::HashMap;
use crate::persistence::serialization::{SerializationSystem, SerializationResult, SerializationError, SerializedComponent};
use crate::map::Map;
use crate::resources::{GameLog, RandomNumberGenerator};

/// Complete world state for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub entities: Vec<EntityData>,
    pub components: Vec<SerializedComponent>,
    pub resources: HashMap<String, ResourceData>,
    pub next_entity_id: u32,
    pub generation: u64,
    pub metadata: HashMap<String, String>,
}

/// Entity data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    pub id: u32,
    pub generation: u32,
    pub component_mask: Vec<String>, // List of component names this entity has
}

/// Component data wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    pub entity_id: u32,
    pub component_name: String,
    pub data: Vec<u8>,
}

/// Resource data for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceData {
    pub resource_name: String,
    pub data: Vec<u8>,
}

/// World serializer that handles complete world state
pub struct WorldSerializer {
    serialization_system: SerializationSystem,
    resource_serializers: HashMap<String, Box<dyn ResourceSerializer>>,
}

/// Trait for serializing resources
pub trait ResourceSerializer: Send + Sync {
    fn serialize_resource(&self, world: &World) -> SerializationResult<Vec<u8>>;
    fn deserialize_resource(&self, world: &mut World, data: &[u8]) -> SerializationResult<()>;
}

impl WorldSerializer {
    pub fn new(serialization_system: SerializationSystem) -> Self {
        let mut serializer = WorldSerializer {
            serialization_system,
            resource_serializers: HashMap::new(),
        };

        // Register default resource serializers
        serializer.register_resource_serializer::<Map>("Map");
        serializer.register_resource_serializer::<GameLog>("GameLog");
        serializer.register_resource_serializer::<RandomNumberGenerator>("RandomNumberGenerator");

        serializer
    }

    /// Register a resource serializer
    pub fn register_resource_serializer<T>(&mut self, name: &str)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
    {
        self.resource_serializers.insert(
            name.to_string(),
            Box::new(GenericResourceSerializer::<T>::new()),
        );
    }

    /// Serialize the entire world state
    pub fn serialize_world(&self, world: &World) -> SerializationResult<WorldState> {
        // Serialize entities
        let entities = self.serialize_entities(world)?;

        // Serialize components
        let components = self.serialization_system.serialize_world(world)?;

        // Serialize resources
        let resources = self.serialize_resources(world)?;

        // Get world metadata
        let (next_entity_id, generation) = self.get_world_metadata(world);

        Ok(WorldState {
            entities,
            components,
            resources,
            next_entity_id,
            generation,
            metadata: HashMap::new(),
        })
    }

    /// Deserialize world state
    pub fn deserialize_world(&self, world: &mut World, world_state: &WorldState) -> SerializationResult<()> {
        // Clear the world first
        self.clear_world(world);

        // Restore world metadata
        self.restore_world_metadata(world, world_state.next_entity_id, world_state.generation);

        // Recreate entities
        self.deserialize_entities(world, &world_state.entities)?;

        // Deserialize components
        self.serialization_system.deserialize_world(world, &world_state.components)?;

        // Deserialize resources
        self.deserialize_resources(world, &world_state.resources)?;

        Ok(())
    }

    /// Serialize only specific entities
    pub fn serialize_entities_selective(&self, world: &World, entity_ids: &[u32]) -> SerializationResult<WorldState> {
        let entities = world.entities();
        let mut selected_entities = Vec::new();

        // Filter entities
        for (entity, _) in (&entities, &world.read_storage::<crate::components::Position>()).join() {
            if entity_ids.contains(&entity.id()) {
                selected_entities.push(EntityData {
                    id: entity.id(),
                    generation: entity.gen().id(),
                    component_mask: self.get_entity_component_mask(world, entity),
                });
            }
        }

        // Serialize only components for selected entities
        let components = self.serialize_components_for_entities(world, entity_ids)?;

        Ok(WorldState {
            entities: selected_entities,
            components,
            resources: HashMap::new(), // Don't include resources in selective serialization
            next_entity_id: 0,
            generation: 0,
            metadata: HashMap::new(),
        })
    }

    fn serialize_entities(&self, world: &World) -> SerializationResult<Vec<EntityData>> {
        let entities = world.entities();
        let mut entity_data = Vec::new();

        for entity in entities.join() {
            entity_data.push(EntityData {
                id: entity.id(),
                generation: entity.gen().id(),
                component_mask: self.get_entity_component_mask(world, entity),
            });
        }

        Ok(entity_data)
    }

    fn deserialize_entities(&self, world: &mut World, entities: &[EntityData]) -> SerializationResult<()> {
        for entity_data in entities {
            // Create entity with specific ID (this is a simplified approach)
            let entity = world.create_entity().build();
            
            // In a real implementation, you'd need to ensure the entity ID matches
            // This might require custom entity creation or ID mapping
        }

        Ok(())
    }

    fn get_entity_component_mask(&self, world: &World, entity: Entity) -> Vec<String> {
        let mut components = Vec::new();

        // Check each registered component type
        for component_name in self.serialization_system.get_registered_components() {
            if self.entity_has_component(world, entity, &component_name) {
                components.push(component_name);
            }
        }

        components
    }

    fn entity_has_component(&self, world: &World, entity: Entity, component_name: &str) -> bool {
        // This is a simplified check - in practice, you'd need to check each component type
        match component_name {
            "Position" => world.read_storage::<crate::components::Position>().get(entity).is_some(),
            "Name" => world.read_storage::<crate::components::Name>().get(entity).is_some(),
            "Player" => world.read_storage::<crate::components::Player>().get(entity).is_some(),
            "Monster" => world.read_storage::<crate::components::Monster>().get(entity).is_some(),
            "CombatStats" => world.read_storage::<crate::components::CombatStats>().get(entity).is_some(),
            "Renderable" => world.read_storage::<crate::components::Renderable>().get(entity).is_some(),
            "Viewshed" => world.read_storage::<crate::components::Viewshed>().get(entity).is_some(),
            "BlocksTile" => world.read_storage::<crate::components::BlocksTile>().get(entity).is_some(),
            "Item" => world.read_storage::<crate::components::Item>().get(entity).is_some(),
            _ => false,
        }
    }

    fn serialize_components_for_entities(&self, world: &World, entity_ids: &[u32]) -> SerializationResult<Vec<SerializedComponent>> {
        // This would be a more complex implementation that only serializes components
        // for specific entities. For now, we'll use the full serialization
        self.serialization_system.serialize_world(world)
    }

    fn serialize_resources(&self, world: &World) -> SerializationResult<HashMap<String, ResourceData>> {
        let mut resources = HashMap::new();

        for (name, serializer) in &self.resource_serializers {
            match serializer.serialize_resource(world) {
                Ok(data) => {
                    resources.insert(name.clone(), ResourceData {
                        resource_name: name.clone(),
                        data,
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Failed to serialize resource {}: {}", name, e);
                    // Continue with other resources
                }
            }
        }

        Ok(resources)
    }

    fn deserialize_resources(&self, world: &mut World, resources: &HashMap<String, ResourceData>) -> SerializationResult<()> {
        for (name, resource_data) in resources {
            if let Some(serializer) = self.resource_serializers.get(name) {
                serializer.deserialize_resource(world, &resource_data.data)?;
            } else {
                eprintln!("Warning: No deserializer found for resource: {}", name);
            }
        }

        Ok(())
    }

    fn get_world_metadata(&self, world: &World) -> (u32, u64) {
        // In a real implementation, you'd extract this from the world
        // For now, return placeholder values
        (1000, 1)
    }

    fn restore_world_metadata(&self, world: &mut World, next_entity_id: u32, generation: u64) {
        // In a real implementation, you'd restore the world's internal state
        // This might involve accessing internal ECS structures
    }

    fn clear_world(&self, world: &mut World) {
        // Clear all entities and components
        world.delete_all();
        
        // Clear resources would need to be done manually for each resource type
        // This is a limitation of the current approach
    }

    /// Create a snapshot of the current world state
    pub fn create_snapshot(&self, world: &World) -> SerializationResult<Vec<u8>> {
        let world_state = self.serialize_world(world)?;
        bincode::serialize(&world_state)
            .map_err(|e| SerializationError::SerializationFailed(e.to_string()))
    }

    /// Restore world from a snapshot
    pub fn restore_from_snapshot(&self, world: &mut World, snapshot: &[u8]) -> SerializationResult<()> {
        let world_state: WorldState = bincode::deserialize(snapshot)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))?;
        
        self.deserialize_world(world, &world_state)
    }

    /// Get serialization statistics
    pub fn get_serialization_stats(&self, world: &World) -> SerializationStats {
        let entities = world.entities();
        let entity_count = entities.join().count();
        
        let mut component_counts = HashMap::new();
        for component_name in self.serialization_system.get_registered_components() {
            let count = self.count_components(world, &component_name);
            component_counts.insert(component_name, count);
        }

        SerializationStats {
            entity_count,
            component_counts,
            resource_count: self.resource_serializers.len(),
        }
    }

    fn count_components(&self, world: &World, component_name: &str) -> usize {
        // This would need to be implemented for each component type
        match component_name {
            "Position" => world.read_storage::<crate::components::Position>().join().count(),
            "Name" => world.read_storage::<crate::components::Name>().join().count(),
            "Player" => world.read_storage::<crate::components::Player>().join().count(),
            "Monster" => world.read_storage::<crate::components::Monster>().join().count(),
            "CombatStats" => world.read_storage::<crate::components::CombatStats>().join().count(),
            _ => 0,
        }
    }
}

/// Generic resource serializer
struct GenericResourceSerializer<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> GenericResourceSerializer<T> {
    fn new() -> Self {
        GenericResourceSerializer {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ResourceSerializer for GenericResourceSerializer<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    fn serialize_resource(&self, world: &World) -> SerializationResult<Vec<u8>> {
        if let Some(resource) = world.try_fetch::<T>() {
            bincode::serialize(&*resource)
                .map_err(|e| SerializationError::SerializationFailed(e.to_string()))
        } else {
            Err(SerializationError::ComponentNotFound("Resource not found".to_string()))
        }
    }

    fn deserialize_resource(&self, world: &mut World, data: &[u8]) -> SerializationResult<()> {
        let resource: T = bincode::deserialize(data)
            .map_err(|e| SerializationError::DeserializationFailed(e.to_string()))?;
        
        world.insert(resource);
        Ok(())
    }
}

/// Serialization statistics
#[derive(Debug, Clone)]
pub struct SerializationStats {
    pub entity_count: usize,
    pub component_counts: HashMap<String, usize>,
    pub resource_count: usize,
}

impl SerializationStats {
    pub fn total_components(&self) -> usize {
        self.component_counts.values().sum()
    }

    pub fn print_stats(&self) {
        println!("Serialization Statistics:");
        println!("  Entities: {}", self.entity_count);
        println!("  Total Components: {}", self.total_components());
        println!("  Resources: {}", self.resource_count);
        println!("  Component Breakdown:");
        
        for (name, count) in &self.component_counts {
            println!("    {}: {}", name, count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::components::*;
    use crate::persistence::serialization::create_serialization_system;

    fn setup_test_world() -> World {
        let mut world = World::new();
        world.register::<Position>();
        world.register::<Name>();
        world.register::<Player>();
        world.register::<CombatStats>();
        world.register::<Renderable>();
        
        // Add some test entities
        world.create_entity()
            .with(Position { x: 10, y: 20 })
            .with(Name { name: "Player".to_string() })
            .with(Player)
            .with(CombatStats { max_hp: 100, hp: 100, defense: 10, power: 15 })
            .build();
        
        world.create_entity()
            .with(Position { x: 5, y: 8 })
            .with(Name { name: "Orc".to_string() })
            .with(Monster)
            .with(CombatStats { max_hp: 30, hp: 30, defense: 5, power: 8 })
            .build();

        world.insert(GameLog::new());
        world.insert(Map::new(80, 50, 1));
        
        world
    }

    #[test]
    fn test_world_serializer_creation() {
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        assert!(!world_serializer.resource_serializers.is_empty());
    }

    #[test]
    fn test_entity_serialization() {
        let world = setup_test_world();
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        let entities = world_serializer.serialize_entities(&world).unwrap();
        
        assert_eq!(entities.len(), 2); // Player and Orc
        
        // Check that entities have component masks
        for entity_data in &entities {
            assert!(!entity_data.component_mask.is_empty());
        }
    }

    #[test]
    fn test_world_state_serialization() {
        let world = setup_test_world();
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        let world_state = world_serializer.serialize_world(&world).unwrap();
        
        assert_eq!(world_state.entities.len(), 2);
        assert!(!world_state.components.is_empty());
        assert!(!world_state.resources.is_empty());
    }

    #[test]
    fn test_snapshot_creation_and_restoration() {
        let world = setup_test_world();
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        // Create snapshot
        let snapshot = world_serializer.create_snapshot(&world).unwrap();
        assert!(!snapshot.is_empty());
        
        // Create new world and restore from snapshot
        let mut new_world = World::new();
        new_world.register::<Position>();
        new_world.register::<Name>();
        new_world.register::<Player>();
        new_world.register::<Monster>();
        new_world.register::<CombatStats>();
        new_world.register::<Renderable>();
        
        world_serializer.restore_from_snapshot(&mut new_world, &snapshot).unwrap();
        
        // Verify restoration worked
        let entities = new_world.entities();
        let positions = new_world.read_storage::<Position>();
        let names = new_world.read_storage::<Name>();
        
        let entity_count = (&entities, &positions, &names).join().count();
        assert_eq!(entity_count, 2);
    }

    #[test]
    fn test_serialization_stats() {
        let world = setup_test_world();
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        let stats = world_serializer.get_serialization_stats(&world);
        
        assert_eq!(stats.entity_count, 2);
        assert!(stats.total_components() > 0);
        assert!(stats.resource_count > 0);
        
        // Check specific component counts
        assert_eq!(stats.component_counts.get("Position"), Some(&2));
        assert_eq!(stats.component_counts.get("Name"), Some(&2));
        assert_eq!(stats.component_counts.get("Player"), Some(&1));
        assert_eq!(stats.component_counts.get("Monster"), Some(&1));
    }

    #[test]
    fn test_selective_entity_serialization() {
        let world = setup_test_world();
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        // Get entity IDs
        let entities = world.entities();
        let entity_ids: Vec<u32> = entities.join().take(1).map(|e| e.id()).collect();
        
        let world_state = world_serializer.serialize_entities_selective(&world, &entity_ids).unwrap();
        
        assert_eq!(world_state.entities.len(), 1);
        assert!(world_state.resources.is_empty()); // Resources not included in selective serialization
    }

    #[test]
    fn test_component_mask_generation() {
        let world = setup_test_world();
        let serialization_system = create_serialization_system();
        let world_serializer = WorldSerializer::new(serialization_system);
        
        let entities = world.entities();
        let players = world.read_storage::<Player>();
        
        for (entity, _) in (&entities, &players).join() {
            let mask = world_serializer.get_entity_component_mask(&world, entity);
            
            // Player entity should have these components
            assert!(mask.contains(&"Position".to_string()));
            assert!(mask.contains(&"Name".to_string()));
            assert!(mask.contains(&"Player".to_string()));
            assert!(mask.contains(&"CombatStats".to_string()));
        }
    }
}