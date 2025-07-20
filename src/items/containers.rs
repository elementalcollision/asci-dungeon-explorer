use specs::{Component, VecStorage, System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, ReadExpect};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::components::{Position, Player, Name, Renderable};
use crate::items::{ItemProperties, ItemType, ItemRarity};
use crate::resources::{GameLog, RandomNumberGenerator};

/// Component for containers that can hold items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Container {
    pub contents: Vec<Entity>,
    pub capacity: usize,
    pub is_open: bool,
    pub container_type: ContainerType,
    pub lock_level: Option<i32>,
    pub is_trapped: bool,
    pub trap_type: Option<TrapType>,
    pub loot_table: Option<String>,
}

impl Container {
    pub fn new(container_type: ContainerType, capacity: usize) -> Self {
        Container {
            contents: Vec::new(),
            capacity,
            is_open: false,
            container_type,
            lock_level: None,
            is_trapped: false,
            trap_type: None,
            loot_table: None,
        }
    }

    pub fn with_lock(mut self, lock_level: i32) -> Self {
        self.lock_level = Some(lock_level);
        self
    }

    pub fn with_trap(mut self, trap_type: TrapType) -> Self {
        self.is_trapped = true;
        self.trap_type = Some(trap_type);
        self
    }

    pub fn with_loot_table(mut self, loot_table: String) -> Self {
        self.loot_table = Some(loot_table);
        self
    }

    pub fn is_locked(&self) -> bool {
        self.lock_level.is_some()
    }

    pub fn is_full(&self) -> bool {
        self.contents.len() >= self.capacity
    }

    pub fn add_item(&mut self, item: Entity) -> Result<(), String> {
        if self.is_full() {
            return Err("Container is full".to_string());
        }
        self.contents.push(item);
        Ok(())
    }

    pub fn remove_item(&mut self, item: Entity) -> bool {
        if let Some(pos) = self.contents.iter().position(|&x| x == item) {
            self.contents.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_contents(&self) -> &Vec<Entity> {
        &self.contents
    }

    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}

/// Types of containers
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum ContainerType {
    Chest,
    Barrel,
    Crate,
    Sack,
    Urn,
    Coffin,
    Bookshelf,
    Wardrobe,
    Desk,
    Safe,
}

impl ContainerType {
    pub fn name(&self) -> &'static str {
        match self {
            ContainerType::Chest => "Chest",
            ContainerType::Barrel => "Barrel",
            ContainerType::Crate => "Crate",
            ContainerType::Sack => "Sack",
            ContainerType::Urn => "Urn",
            ContainerType::Coffin => "Coffin",
            ContainerType::Bookshelf => "Bookshelf",
            ContainerType::Wardrobe => "Wardrobe",
            ContainerType::Desk => "Desk",
            ContainerType::Safe => "Safe",
        }
    }

    pub fn glyph(&self) -> char {
        match self {
            ContainerType::Chest => '=',
            ContainerType::Barrel => '8',
            ContainerType::Crate => '#',
            ContainerType::Sack => 'u',
            ContainerType::Urn => 'U',
            ContainerType::Coffin => '&',
            ContainerType::Bookshelf => 'H',
            ContainerType::Wardrobe => 'H',
            ContainerType::Desk => 'T',
            ContainerType::Safe => '■',
        }
    }

    pub fn color(&self) -> crossterm::style::Color {
        match self {
            ContainerType::Chest => crossterm::style::Color::DarkYellow,
            ContainerType::Barrel => crossterm::style::Color::DarkYellow,
            ContainerType::Crate => crossterm::style::Color::DarkYellow,
            ContainerType::Sack => crossterm::style::Color::DarkGrey,
            ContainerType::Urn => crossterm::style::Color::Grey,
            ContainerType::Coffin => crossterm::style::Color::DarkGrey,
            ContainerType::Bookshelf => crossterm::style::Color::DarkYellow,
            ContainerType::Wardrobe => crossterm::style::Color::DarkYellow,
            ContainerType::Desk => crossterm::style::Color::DarkYellow,
            ContainerType::Safe => crossterm::style::Color::DarkGrey,
        }
    }

    pub fn default_capacity(&self) -> usize {
        match self {
            ContainerType::Chest => 20,
            ContainerType::Barrel => 15,
            ContainerType::Crate => 12,
            ContainerType::Sack => 8,
            ContainerType::Urn => 5,
            ContainerType::Coffin => 10,
            ContainerType::Bookshelf => 25,
            ContainerType::Wardrobe => 30,
            ContainerType::Desk => 10,
            ContainerType::Safe => 15,
        }
    }
}

/// Types of traps that can be on containers
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum TrapType {
    Poison,
    Explosion,
    Needle,
    Gas,
    Curse,
    Alarm,
    Teleport,
    Freeze,
}

impl TrapType {
    pub fn name(&self) -> &'static str {
        match self {
            TrapType::Poison => "Poison Trap",
            TrapType::Explosion => "Explosive Trap",
            TrapType::Needle => "Poison Needle",
            TrapType::Gas => "Gas Trap",
            TrapType::Curse => "Cursed Container",
            TrapType::Alarm => "Alarm Trap",
            TrapType::Teleport => "Teleport Trap",
            TrapType::Freeze => "Freeze Trap",
        }
    }

    pub fn damage(&self) -> i32 {
        match self {
            TrapType::Poison => 5,
            TrapType::Explosion => 15,
            TrapType::Needle => 3,
            TrapType::Gas => 8,
            TrapType::Curse => 0, // No direct damage
            TrapType::Alarm => 0, // No damage
            TrapType::Teleport => 0, // No damage
            TrapType::Freeze => 0, // No damage
        }
    }
}

/// Intent component for opening containers
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToOpenContainer {
    pub container: Entity,
    pub force_open: bool, // Ignore locks
}

/// Intent component for closing containers
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToCloseContainer {
    pub container: Entity,
}

/// Intent component for taking items from containers
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToTakeFromContainer {
    pub container: Entity,
    pub item: Entity,
}

/// Intent component for putting items into containers
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToPutInContainer {
    pub container: Entity,
    pub item: Entity,
}

/// System for handling container interactions
pub struct ContainerSystem;

impl<'a> System<'a> for ContainerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToOpenContainer>,
        WriteStorage<'a, WantsToCloseContainer>,
        WriteStorage<'a, WantsToTakeFromContainer>,
        WriteStorage<'a, WantsToPutInContainer>,
        WriteStorage<'a, Container>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_open,
            mut wants_close,
            mut wants_take,
            mut wants_put,
            mut containers,
            names,
            players,
            mut gamelog,
            mut rng,
        ) = data;

        // Process open container requests
        let mut to_remove_open = Vec::new();

        for (entity, open_intent) in (&entities, &wants_open).join() {
            let container_entity = open_intent.container;
            
            if let Some(container) = containers.get_mut(container_entity) {
                let container_name = names.get(container_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Container".to_string());

                // Check if already open
                if container.is_open {
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("The {} is already open.", container_name));
                    }
                    to_remove_open.push(entity);
                    continue;
                }

                // Check if locked
                if let Some(lock_level) = container.lock_level {
                    if !open_intent.force_open {
                        // TODO: Check player's lockpicking skill
                        let lockpick_skill = 0; // Placeholder
                        let difficulty = lock_level * 5;
                        let roll = rng.roll_dice(1, 20) + lockpick_skill;
                        
                        if roll < difficulty {
                            if players.get(entity).is_some() {
                                gamelog.entries.push(format!("The {} is locked and you cannot open it.", container_name));
                            }
                            to_remove_open.push(entity);
                            continue;
                        } else {
                            if players.get(entity).is_some() {
                                gamelog.entries.push(format!("You successfully pick the lock on the {}.", container_name));
                            }
                        }
                    }
                }

                // Check for traps
                if container.is_trapped {
                    if let Some(trap_type) = container.trap_type {
                        // TODO: Check player's trap detection skill
                        let detect_skill = 0; // Placeholder
                        let detect_roll = rng.roll_dice(1, 20) + detect_skill;
                        
                        if detect_roll < 15 {
                            // Trap triggers
                            self.trigger_trap(entity, trap_type, &mut gamelog, &mut rng, &players);
                        } else {
                            if players.get(entity).is_some() {
                                gamelog.entries.push(format!("You notice a {} on the {} and avoid it.", trap_type.name(), container_name));
                            }
                        }
                        
                        // Disarm the trap after triggering or detecting
                        container.is_trapped = false;
                        container.trap_type = None;
                    }
                }

                // Open the container
                container.is_open = true;
                
                if players.get(entity).is_some() {
                    if container.is_empty() {
                        gamelog.entries.push(format!("You open the {}. It is empty.", container_name));
                    } else {
                        gamelog.entries.push(format!("You open the {}. It contains {} items.", 
                            container_name, container.contents.len()));
                    }
                }
            }
            
            to_remove_open.push(entity);
        }

        // Clean up open intents
        for entity in to_remove_open {
            wants_open.remove(entity);
        }

        // Process close container requests
        let mut to_remove_close = Vec::new();

        for (entity, close_intent) in (&entities, &wants_close).join() {
            let container_entity = close_intent.container;
            
            if let Some(container) = containers.get_mut(container_entity) {
                let container_name = names.get(container_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Container".to_string());

                if !container.is_open {
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("The {} is already closed.", container_name));
                    }
                } else {
                    container.is_open = false;
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("You close the {}.", container_name));
                    }
                }
            }
            
            to_remove_close.push(entity);
        }

        // Clean up close intents
        for entity in to_remove_close {
            wants_close.remove(entity);
        }

        // Process take from container requests
        let mut to_remove_take = Vec::new();

        for (entity, take_intent) in (&entities, &wants_take).join() {
            let container_entity = take_intent.container;
            let item_entity = take_intent.item;
            
            if let Some(container) = containers.get_mut(container_entity) {
                let container_name = names.get(container_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Container".to_string());
                
                let item_name = names.get(item_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Item".to_string());

                if !container.is_open {
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("The {} is closed.", container_name));
                    }
                } else if container.remove_item(item_entity) {
                    // TODO: Add item to player inventory
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("You take the {} from the {}.", item_name, container_name));
                    }
                } else {
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("The {} is not in the {}.", item_name, container_name));
                    }
                }
            }
            
            to_remove_take.push(entity);
        }

        // Clean up take intents
        for entity in to_remove_take {
            wants_take.remove(entity);
        }

        // Process put in container requests
        let mut to_remove_put = Vec::new();

        for (entity, put_intent) in (&entities, &wants_put).join() {
            let container_entity = put_intent.container;
            let item_entity = put_intent.item;
            
            if let Some(container) = containers.get_mut(container_entity) {
                let container_name = names.get(container_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Container".to_string());
                
                let item_name = names.get(item_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Item".to_string());

                if !container.is_open {
                    if players.get(entity).is_some() {
                        gamelog.entries.push(format!("The {} is closed.", container_name));
                    }
                } else {
                    match container.add_item(item_entity) {
                        Ok(()) => {
                            // TODO: Remove item from player inventory
                            if players.get(entity).is_some() {
                                gamelog.entries.push(format!("You put the {} in the {}.", item_name, container_name));
                            }
                        },
                        Err(msg) => {
                            if players.get(entity).is_some() {
                                gamelog.entries.push(msg);
                            }
                        }
                    }
                }
            }
            
            to_remove_put.push(entity);
        }

        // Clean up put intents
        for entity in to_remove_put {
            wants_put.remove(entity);
        }
    }
}

impl ContainerSystem {
    fn trigger_trap(
        &self,
        target: Entity,
        trap_type: TrapType,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
        players: &ReadStorage<Player>,
    ) {
        let damage = trap_type.damage();
        
        if players.get(target).is_some() {
            match trap_type {
                TrapType::Poison => {
                    gamelog.entries.push(format!("A poison dart hits you for {} damage! You feel sick.", damage));
                    // TODO: Apply poison status effect
                },
                TrapType::Explosion => {
                    gamelog.entries.push(format!("The container explodes for {} damage!", damage));
                },
                TrapType::Needle => {
                    gamelog.entries.push(format!("A poisoned needle pricks you for {} damage!", damage));
                    // TODO: Apply poison status effect
                },
                TrapType::Gas => {
                    gamelog.entries.push(format!("Poisonous gas escapes, dealing {} damage!", damage));
                    // TODO: Apply poison status effect
                },
                TrapType::Curse => {
                    gamelog.entries.push("You feel a dark curse settle upon you!".to_string());
                    // TODO: Apply curse status effect
                },
                TrapType::Alarm => {
                    gamelog.entries.push("A loud alarm sounds! Nearby enemies are alerted!".to_string());
                    // TODO: Alert nearby enemies
                },
                TrapType::Teleport => {
                    gamelog.entries.push("You are suddenly teleported to a random location!".to_string());
                    // TODO: Teleport player
                },
                TrapType::Freeze => {
                    gamelog.entries.push("You are frozen in place by magical ice!".to_string());
                    // TODO: Apply freeze status effect
                },
            }
        }
        
        // TODO: Apply actual damage and effects to the target entity
    }
}

/// Loot table for generating container contents
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LootTable {
    pub id: String,
    pub entries: Vec<LootEntry>,
    pub min_items: usize,
    pub max_items: usize,
}

impl LootTable {
    pub fn new(id: String) -> Self {
        LootTable {
            id,
            entries: Vec::new(),
            min_items: 1,
            max_items: 3,
        }
    }

    pub fn with_range(mut self, min: usize, max: usize) -> Self {
        self.min_items = min;
        self.max_items = max;
        self
    }

    pub fn add_entry(mut self, entry: LootEntry) -> Self {
        self.entries.push(entry);
        self
    }

    pub fn generate_loot(&self, rng: &mut RandomNumberGenerator) -> Vec<LootResult> {
        let mut results = Vec::new();
        let num_items = rng.range(self.min_items, self.max_items + 1);

        for _ in 0..num_items {
            let total_weight: i32 = self.entries.iter().map(|e| e.weight).sum();
            let mut roll = rng.roll_dice(1, total_weight);

            for entry in &self.entries {
                roll -= entry.weight;
                if roll <= 0 {
                    if rng.roll_dice(1, 100) <= entry.chance {
                        results.push(LootResult {
                            item_type: entry.item_type.clone(),
                            quantity: rng.range(entry.min_quantity, entry.max_quantity + 1),
                            rarity: entry.rarity,
                        });
                    }
                    break;
                }
            }
        }

        results
    }
}

/// Entry in a loot table
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LootEntry {
    pub item_type: String, // Item type identifier
    pub weight: i32,       // Weight for random selection
    pub chance: i32,       // Percentage chance (1-100)
    pub min_quantity: usize,
    pub max_quantity: usize,
    pub rarity: ItemRarity,
}

impl LootEntry {
    pub fn new(item_type: String, weight: i32, chance: i32) -> Self {
        LootEntry {
            item_type,
            weight,
            chance,
            min_quantity: 1,
            max_quantity: 1,
            rarity: ItemRarity::Common,
        }
    }

    pub fn with_quantity(mut self, min: usize, max: usize) -> Self {
        self.min_quantity = min;
        self.max_quantity = max;
        self
    }

    pub fn with_rarity(mut self, rarity: ItemRarity) -> Self {
        self.rarity = rarity;
        self
    }
}

/// Result from loot table generation
#[derive(Debug, Clone)]
pub struct LootResult {
    pub item_type: String,
    pub quantity: usize,
    pub rarity: ItemRarity,
}

/// Container factory for creating different types of containers
pub struct ContainerFactory {
    loot_tables: HashMap<String, LootTable>,
}

impl ContainerFactory {
    pub fn new() -> Self {
        let mut factory = ContainerFactory {
            loot_tables: HashMap::new(),
        };
        factory.initialize_loot_tables();
        factory
    }

    fn initialize_loot_tables(&mut self) {
        // Basic treasure chest loot table
        let treasure_chest = LootTable::new("treasure_chest".to_string())
            .with_range(2, 5)
            .add_entry(LootEntry::new("gold".to_string(), 30, 80).with_quantity(10, 50))
            .add_entry(LootEntry::new("potion_healing".to_string(), 20, 60))
            .add_entry(LootEntry::new("weapon_common".to_string(), 15, 40).with_rarity(ItemRarity::Common))
            .add_entry(LootEntry::new("armor_common".to_string(), 15, 40).with_rarity(ItemRarity::Common))
            .add_entry(LootEntry::new("weapon_uncommon".to_string(), 10, 20).with_rarity(ItemRarity::Uncommon))
            .add_entry(LootEntry::new("armor_uncommon".to_string(), 10, 20).with_rarity(ItemRarity::Uncommon));

        self.loot_tables.insert("treasure_chest".to_string(), treasure_chest);

        // Barrel loot table (consumables and materials)
        let barrel = LootTable::new("barrel".to_string())
            .with_range(1, 3)
            .add_entry(LootEntry::new("food".to_string(), 40, 70).with_quantity(1, 3))
            .add_entry(LootEntry::new("potion_healing".to_string(), 25, 50))
            .add_entry(LootEntry::new("materials".to_string(), 20, 40).with_quantity(2, 5))
            .add_entry(LootEntry::new("gold".to_string(), 15, 30).with_quantity(5, 20));

        self.loot_tables.insert("barrel".to_string(), barrel);

        // Coffin loot table (undead themed)
        let coffin = LootTable::new("coffin".to_string())
            .with_range(1, 4)
            .add_entry(LootEntry::new("gold".to_string(), 25, 60).with_quantity(20, 100))
            .add_entry(LootEntry::new("jewelry".to_string(), 20, 40))
            .add_entry(LootEntry::new("weapon_rare".to_string(), 15, 25).with_rarity(ItemRarity::Rare))
            .add_entry(LootEntry::new("armor_rare".to_string(), 15, 25).with_rarity(ItemRarity::Rare))
            .add_entry(LootEntry::new("cursed_item".to_string(), 10, 15))
            .add_entry(LootEntry::new("bones".to_string(), 15, 30).with_quantity(1, 3));

        self.loot_tables.insert("coffin".to_string(), coffin);

        // Safe loot table (valuable items)
        let safe = LootTable::new("safe".to_string())
            .with_range(1, 3)
            .add_entry(LootEntry::new("gold".to_string(), 30, 90).with_quantity(50, 200))
            .add_entry(LootEntry::new("jewelry".to_string(), 25, 70))
            .add_entry(LootEntry::new("weapon_rare".to_string(), 20, 40).with_rarity(ItemRarity::Rare))
            .add_entry(LootEntry::new("armor_rare".to_string(), 20, 40).with_rarity(ItemRarity::Rare))
            .add_entry(LootEntry::new("weapon_epic".to_string(), 5, 10).with_rarity(ItemRarity::Epic));

        self.loot_tables.insert("safe".to_string(), safe);
    }

    pub fn create_container(
        &self,
        world: &mut specs::World,
        position: Position,
        container_type: ContainerType,
        lock_level: Option<i32>,
        trap_type: Option<TrapType>,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        let capacity = container_type.default_capacity();
        let mut container = Container::new(container_type, capacity);

        // Add lock if specified
        if let Some(level) = lock_level {
            container = container.with_lock(level);
        }

        // Add trap if specified
        if let Some(trap) = trap_type {
            container = container.with_trap(trap);
        }

        // Set loot table based on container type
        let loot_table_id = match container_type {
            ContainerType::Chest => "treasure_chest",
            ContainerType::Barrel => "barrel",
            ContainerType::Coffin => "coffin",
            ContainerType::Safe => "safe",
            _ => "treasure_chest", // Default
        };
        container = container.with_loot_table(loot_table_id.to_string());

        // Generate initial contents if loot table exists
        if let Some(loot_table) = self.loot_tables.get(loot_table_id) {
            let loot_results = loot_table.generate_loot(rng);
            
            // TODO: Create actual item entities from loot results
            // For now, we'll just note that the container should have these items
        }

        // Create the container entity
        let entity = world.create_entity()
            .with(Name { name: container_type.name().to_string() })
            .with(container)
            .with(position)
            .with(Renderable {
                glyph: container_type.glyph(),
                fg: container_type.color(),
                bg: crossterm::style::Color::Black,
                render_order: 2,
            })
            .build();

        entity
    }

    pub fn create_random_container(
        &self,
        world: &mut specs::World,
        position: Position,
        dungeon_level: i32,
        rng: &mut RandomNumberGenerator,
    ) -> Entity {
        // Choose random container type
        let container_types = [
            ContainerType::Chest,
            ContainerType::Barrel,
            ContainerType::Crate,
            ContainerType::Urn,
        ];
        let container_type = container_types[rng.roll_dice(1, container_types.len()) - 1];

        // Determine lock level based on dungeon level
        let lock_level = if rng.roll_dice(1, 100) <= (dungeon_level * 10) {
            Some(rng.range(1, dungeon_level + 1))
        } else {
            None
        };

        // Determine trap based on dungeon level
        let trap_type = if rng.roll_dice(1, 100) <= (dungeon_level * 5) {
            let trap_types = [
                TrapType::Poison,
                TrapType::Needle,
                TrapType::Gas,
                TrapType::Explosion,
            ];
            Some(trap_types[rng.roll_dice(1, trap_types.len()) - 1])
        } else {
            None
        };

        self.create_container(world, position, container_type, lock_level, trap_type, rng)
    }

    pub fn get_loot_table(&self, id: &str) -> Option<&LootTable> {
        self.loot_tables.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Container>();
        world.register::<WantsToOpenContainer>();
        world.register::<WantsToCloseContainer>();
        world.register::<WantsToTakeFromContainer>();
        world.register::<WantsToPutInContainer>();
        world.register::<Name>();
        world.register::<Player>();
        world.register::<Position>();
        world.register::<Renderable>();
        world.insert(GameLog::new());
        world.insert(RandomNumberGenerator::new());
        world
    }

    #[test]
    fn test_container_creation() {
        let container = Container::new(ContainerType::Chest, 10);
        
        assert_eq!(container.capacity, 10);
        assert!(!container.is_open);
        assert!(container.is_empty());
        assert!(!container.is_locked());
        assert!(!container.is_trapped);
    }

    #[test]
    fn test_container_with_lock_and_trap() {
        let container = Container::new(ContainerType::Safe, 5)
            .with_lock(3)
            .with_trap(TrapType::Explosion);
        
        assert!(container.is_locked());
        assert_eq!(container.lock_level, Some(3));
        assert!(container.is_trapped);
        assert_eq!(container.trap_type, Some(TrapType::Explosion));
    }

    #[test]
    fn test_container_item_management() {
        let mut world = setup_world();
        
        // Create a container
        let mut container = Container::new(ContainerType::Chest, 2);
        
        // Create some items
        let item1 = world.create_entity().build();
        let item2 = world.create_entity().build();
        let item3 = world.create_entity().build();
        
        // Add items
        assert!(container.add_item(item1).is_ok());
        assert!(container.add_item(item2).is_ok());
        assert!(container.add_item(item3).is_err()); // Should fail - container full
        
        assert_eq!(container.contents.len(), 2);
        assert!(container.is_full());
        
        // Remove item
        assert!(container.remove_item(item1));
        assert!(!container.is_full());
        assert_eq!(container.contents.len(), 1);
        
        // Try to remove non-existent item
        assert!(!container.remove_item(item3));
    }

    #[test]
    fn test_loot_table() {
        let mut rng = RandomNumberGenerator::new();
        
        let loot_table = LootTable::new("test".to_string())
            .with_range(1, 3)
            .add_entry(LootEntry::new("gold".to_string(), 50, 100).with_quantity(10, 20))
            .add_entry(LootEntry::new("potion".to_string(), 30, 80))
            .add_entry(LootEntry::new("weapon".to_string(), 20, 50));
        
        let results = loot_table.generate_loot(&mut rng);
        
        // Should generate 1-3 items
        assert!(results.len() >= 1 && results.len() <= 3);
        
        // Check that results contain valid item types
        for result in results {
            assert!(["gold", "potion", "weapon"].contains(&result.item_type.as_str()));
        }
    }

    #[test]
    fn test_container_factory() {
        let mut world = setup_world();
        let mut rng = RandomNumberGenerator::new();
        let factory = ContainerFactory::new();
        
        let position = Position { x: 5, y: 5 };
        
        // Create a locked, trapped chest
        let chest = factory.create_container(
            &mut world,
            position,
            ContainerType::Chest,
            Some(2),
            Some(TrapType::Poison),
            &mut rng,
        );
        
        // Verify the container was created correctly
        let containers = world.read_storage::<Container>();
        let names = world.read_storage::<Name>();
        
        if let Some(container) = containers.get(chest) {
            assert_eq!(container.container_type, ContainerType::Chest);
            assert!(container.is_locked());
            assert_eq!(container.lock_level, Some(2));
            assert!(container.is_trapped);
            assert_eq!(container.trap_type, Some(TrapType::Poison));
        } else {
            panic!("Container component not found");
        }
        
        if let Some(name) = names.get(chest) {
            assert_eq!(name.name, "Chest");
        } else {
            panic!("Name component not found");
        }
    }

    #[test]
    fn test_container_system() {
        let mut world = setup_world();
        
        // Create a player
        let player = world.create_entity()
            .with(Player)
            .with(Name { name: "Player".to_string() })
            .with(Position { x: 0, y: 0 })
            .build();
        
        // Create a container
        let container = world.create_entity()
            .with(Name { name: "Test Chest".to_string() })
            .with(Container::new(ContainerType::Chest, 10))
            .with(Position { x: 1, y: 1 })
            .build();
        
        // Add open intent
        world.write_storage::<WantsToOpenContainer>()
            .insert(player, WantsToOpenContainer { container, force_open: false })
            .expect("Failed to insert open intent");
        
        // Run the container system
        let mut container_system = ContainerSystem;
        container_system.run_now(&world);
        world.maintain();
        
        // Check that the container is now open
        let containers = world.read_storage::<Container>();
        
        if let Some(container_comp) = containers.get(container) {
            assert!(container_comp.is_open);
        } else {
            panic!("Container should exist");
        }
        
        // Check that a log message was generated
        let gamelog = world.fetch::<GameLog>();
        assert!(!gamelog.entries.is_empty());
        assert!(gamelog.entries[0].contains("open"));
    }
}