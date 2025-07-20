use specs::{System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, ReadExpect};
use crate::components::{Position, Player, Name, Item, Inventory, WantsToPickupItem, WantsToDropItem};
use crate::items::{ItemProperties, ItemStack, get_item_display_name};
use crate::resources::{GameLog, RandomNumberGenerator};
use crate::map::Map;

// Enhanced Inventory component with more features
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct AdvancedInventory {
    pub items: Vec<InventorySlot>,
    pub capacity: usize,
    pub weight_limit: f32,
    pub current_weight: f32,
    pub gold: i32,
    pub auto_pickup: bool,
    pub sort_mode: InventorySortMode,
}

impl AdvancedInventory {
    pub fn new(capacity: usize, weight_limit: f32) -> Self {
        AdvancedInventory {
            items: Vec::new(),
            capacity,
            weight_limit,
            current_weight: 0.0,
            gold: 0,
            auto_pickup: false,
            sort_mode: InventorySortMode::None,
        }
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn is_overweight(&self) -> bool {
        self.current_weight > self.weight_limit
    }

    pub fn can_add_item(&self, weight: f32) -> bool {
        !self.is_full() && (self.current_weight + weight) <= self.weight_limit
    }

    pub fn add_item(&mut self, entity: Entity, quantity: i32, weight: f32) -> bool {
        if !self.can_add_item(weight * quantity as f32) {
            return false;
        }

        // Try to stack with existing items first
        for slot in &mut self.items {
            if slot.can_stack_with(entity) {
                let added = slot.add_quantity(quantity);
                if added > 0 {
                    self.current_weight += weight * added as f32;
                    return true;
                }
            }
        }

        // Create new slot if we can't stack
        if !self.is_full() {
            self.items.push(InventorySlot::new(entity, quantity));
            self.current_weight += weight * quantity as f32;
            return true;
        }

        false
    }

    pub fn remove_item(&mut self, slot_index: usize, quantity: i32, weight: f32) -> Option<(Entity, i32)> {
        if slot_index >= self.items.len() {
            return None;
        }

        let slot = &mut self.items[slot_index];
        let removed = slot.remove_quantity(quantity);
        
        if removed > 0 {
            self.current_weight -= weight * removed as f32;
            self.current_weight = self.current_weight.max(0.0);
        }

        if slot.quantity <= 0 {
            let entity = slot.entity;
            self.items.remove(slot_index);
            Some((entity, removed))
        } else {
            Some((slot.entity, removed))
        }
    }

    pub fn find_item(&self, entity: Entity) -> Option<usize> {
        self.items.iter().position(|slot| slot.entity == entity)
    }

    pub fn get_total_value(&self, world: &specs::World) -> i32 {
        let mut total = self.gold;
        
        for slot in &self.items {
            if let Some(value) = crate::items::get_item_current_value(world, slot.entity) {
                total += value * slot.quantity;
            }
        }
        
        total
    }

    pub fn sort_inventory(&mut self, world: &specs::World) {
        match self.sort_mode {
            InventorySortMode::None => {},
            InventorySortMode::Name => {
                self.items.sort_by(|a, b| {
                    let name_a = get_item_display_name(world, a.entity).unwrap_or_default();
                    let name_b = get_item_display_name(world, b.entity).unwrap_or_default();
                    name_a.cmp(&name_b)
                });
            },
            InventorySortMode::Type => {
                let properties = world.read_storage::<ItemProperties>();
                self.items.sort_by(|a, b| {
                    let type_a = properties.get(a.entity).map(|p| format!("{:?}", p.item_type)).unwrap_or_default();
                    let type_b = properties.get(b.entity).map(|p| format!("{:?}", p.item_type)).unwrap_or_default();
                    type_a.cmp(&type_b)
                });
            },
            InventorySortMode::Value => {
                self.items.sort_by(|a, b| {
                    let value_a = crate::items::get_item_current_value(world, a.entity);
                    let value_b = crate::items::get_item_current_value(world, b.entity);
                    value_b.cmp(&value_a) // Descending order
                });
            },
            InventorySortMode::Weight => {
                let properties = world.read_storage::<ItemProperties>();
                self.items.sort_by(|a, b| {
                    let weight_a = properties.get(a.entity).map(|p| p.weight).unwrap_or(0.0);
                    let weight_b = properties.get(b.entity).map(|p| p.weight).unwrap_or(0.0);
                    weight_a.partial_cmp(&weight_b).unwrap_or(std::cmp::Ordering::Equal)
                });
            },
        }
    }

    pub fn get_items_by_type(&self, world: &specs::World, item_type: &crate::items::ItemType) -> Vec<usize> {
        let properties = world.read_storage::<ItemProperties>();
        self.items.iter().enumerate()
            .filter_map(|(index, slot)| {
                if let Some(props) = properties.get(slot.entity) {
                    if std::mem::discriminant(&props.item_type) == std::mem::discriminant(item_type) {
                        Some(index)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InventorySlot {
    pub entity: Entity,
    pub quantity: i32,
    pub locked: bool, // Prevents accidental dropping
}

impl InventorySlot {
    pub fn new(entity: Entity, quantity: i32) -> Self {
        InventorySlot {
            entity,
            quantity,
            locked: false,
        }
    }

    pub fn can_stack_with(&self, other_entity: Entity) -> bool {
        // This would need to check if items are stackable and identical
        // For now, just check if it's the same entity
        self.entity == other_entity
    }

    pub fn add_quantity(&mut self, amount: i32) -> i32 {
        // This would need to check stack limits
        // For now, just add the amount
        self.quantity += amount;
        amount
    }

    pub fn remove_quantity(&mut self, amount: i32) -> i32 {
        let removed = self.quantity.min(amount);
        self.quantity -= removed;
        removed
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum InventorySortMode {
    None,
    Name,
    Type,
    Value,
    Weight,
}

// System for handling item pickup
pub struct ItemPickupSystem;

impl<'a> System<'a> for ItemPickupSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, AdvancedInventory>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ItemProperties>,
        WriteStorage<'a, ItemStack>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_pickup,
            mut inventories,
            mut positions,
            names,
            properties,
            mut stacks,
            mut gamelog,
        ) = data;

        let mut to_remove = Vec::new();

        for (entity, pickup, inventory) in (&entities, &wants_pickup, &mut inventories).join() {
            let item_entity = pickup.item;
            
            // Get item properties
            if let Some(props) = properties.get(item_entity) {
                let item_name = names.get(item_entity)
                    .map(|n| n.name.clone())
                    .unwrap_or("Unknown Item".to_string());

                // Check if item has a stack component
                let quantity = if let Some(stack) = stacks.get(item_entity) {
                    stack.quantity
                } else {
                    1
                };

                let total_weight = props.weight * quantity as f32;

                // Try to add to inventory
                if inventory.add_item(item_entity, quantity, props.weight) {
                    // Remove item from world position
                    positions.remove(item_entity);
                    
                    // Log the pickup
                    if quantity > 1 {
                        gamelog.entries.push(format!("You pick up {} {}s.", quantity, item_name));
                    } else {
                        gamelog.entries.push(format!("You pick up the {}.", item_name));
                    }
                } else {
                    // Inventory full or overweight
                    if inventory.is_full() {
                        gamelog.entries.push("Your inventory is full!".to_string());
                    } else if inventory.current_weight + total_weight > inventory.weight_limit {
                        gamelog.entries.push("That would be too heavy to carry!".to_string());
                    }
                }
            }

            to_remove.push(entity);
        }

        // Clean up pickup intents
        for entity in to_remove {
            wants_pickup.remove(entity);
        }
    }
}

// System for handling item dropping
pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        WriteStorage<'a, AdvancedInventory>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ItemProperties>,
        WriteStorage<'a, ItemStack>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
        ReadExpect<'a, Map>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_drop,
            mut inventories,
            mut positions,
            names,
            properties,
            mut stacks,
            players,
            mut gamelog,
            map,
        ) = data;

        let mut to_remove = Vec::new();

        for (entity, drop_intent, inventory) in (&entities, &wants_drop, &mut inventories).join() {
            // Find the slot to drop
            if let Some(slot_index) = inventory.find_item(drop_intent.item) {
                let item_entity = drop_intent.item;
                
                if let Some(props) = properties.get(item_entity) {
                    let item_name = names.get(item_entity)
                        .map(|n| n.name.clone())
                        .unwrap_or("Unknown Item".to_string());

                    // Get player position for dropping
                    if let Some(player_pos) = positions.get(entity) {
                        let drop_pos = self.find_drop_position(player_pos, &map);
                        
                        // Remove from inventory (drop 1 item by default)
                        if let Some((dropped_entity, quantity)) = inventory.remove_item(slot_index, 1, props.weight) {
                            // Place item in world
                            positions.insert(dropped_entity, drop_pos)
                                .expect("Failed to set dropped item position");

                            // Update stack if necessary
                            if let Some(stack) = stacks.get_mut(dropped_entity) {
                                stack.quantity = quantity;
                            }

                            // Log the drop
                            gamelog.entries.push(format!("You drop the {}.", item_name));
                        }
                    }
                }
            }

            to_remove.push(entity);
        }

        // Clean up drop intents
        for entity in to_remove {
            wants_drop.remove(entity);
        }
    }
}

impl ItemDropSystem {
    fn find_drop_position(&self, player_pos: &Position, map: &Map) -> Position {
        // Try to find an empty adjacent position
        let offsets = [
            (0, 0),   // Same position
            (0, -1),  // North
            (1, 0),   // East
            (0, 1),   // South
            (-1, 0),  // West
            (1, -1),  // Northeast
            (1, 1),   // Southeast
            (-1, 1),  // Southwest
            (-1, -1), // Northwest
        ];

        for (dx, dy) in offsets.iter() {
            let x = player_pos.x + dx;
            let y = player_pos.y + dy;

            if x >= 0 && x < map.width && y >= 0 && y < map.height {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx].walkable {
                    return Position { x, y };
                }
            }
        }

        // If no valid position found, drop at player position
        player_pos.clone()
    }
}

// System for automatic item pickup
pub struct AutoPickupSystem;

impl<'a> System<'a> for AutoPickupSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, AdvancedInventory>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ItemProperties>,
        WriteStorage<'a, WantsToPickupItem>,
        Write<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            players,
            positions,
            mut inventories,
            items,
            names,
            properties,
            mut wants_pickup,
            mut gamelog,
        ) = data;

        // Find players with auto-pickup enabled
        for (player_entity, _player, player_pos, inventory) in 
            (&entities, &players, &positions, &mut inventories).join() {
            
            if !inventory.auto_pickup {
                continue;
            }

            // Find items at player position
            for (item_entity, _item, item_pos, props) in 
                (&entities, &items, &positions, &properties).join() {
                
                if item_pos.x == player_pos.x && item_pos.y == player_pos.y {
                    // Check if item should be auto-picked up
                    if self.should_auto_pickup(props) {
                        // Add pickup intent
                        wants_pickup.insert(player_entity, WantsToPickupItem { item: item_entity })
                            .expect("Failed to insert pickup intent");
                    }
                }
            }
        }
    }
}

impl AutoPickupSystem {
    fn should_auto_pickup(&self, props: &ItemProperties) -> bool {
        // Auto-pickup rules - customize as needed
        match props.item_type {
            crate::items::ItemType::Consumable(crate::items::ConsumableType::Potion) => true,
            crate::items::ItemType::Material(_) => true,
            _ => props.rarity >= crate::items::ItemRarity::Uncommon,
        }
    }
}

// System for inventory management and organization
pub struct InventoryManagementSystem;

impl<'a> System<'a> for InventoryManagementSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, AdvancedInventory>,
        ReadStorage<'a, ItemProperties>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut inventories, properties, names) = data;

        for (_entity, inventory) in (&entities, &mut inventories).join() {
            // Update current weight
            self.update_inventory_weight(inventory, &properties);
            
            // Auto-sort if enabled
            if inventory.sort_mode != InventorySortMode::None {
                // Note: We can't pass world here, so sorting would need to be done elsewhere
                // or we'd need to restructure this system
            }
            
            // Clean up empty slots
            inventory.items.retain(|slot| slot.quantity > 0);
        }
    }
}

impl InventoryManagementSystem {
    fn update_inventory_weight(&self, inventory: &mut AdvancedInventory, properties: &ReadStorage<ItemProperties>) {
        let mut total_weight = 0.0;
        
        for slot in &inventory.items {
            if let Some(props) = properties.get(slot.entity) {
                total_weight += props.weight * slot.quantity as f32;
            }
        }
        
        inventory.current_weight = total_weight;
    }
}

use specs::Component;
use serde::{Serialize, Deserialize};

// Component for items that can be picked up
#[derive(Component, Debug, Default)]
#[storage(NullStorage)]
pub struct Pickupable;

// Component for containers that can hold items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Container {
    pub items: Vec<Entity>,
    pub capacity: usize,
    pub locked: bool,
    pub key_required: Option<Entity>,
    pub container_type: ContainerType,
}

impl Container {
    pub fn new(capacity: usize, container_type: ContainerType) -> Self {
        Container {
            items: Vec::new(),
            capacity,
            locked: false,
            key_required: None,
            container_type,
        }
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    pub fn add_item(&mut self, item: Entity) -> bool {
        if !self.is_full() {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    pub fn remove_item(&mut self, index: usize) -> Option<Entity> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum ContainerType {
    Chest,
    Barrel,
    Crate,
    Bag,
    Corpse,
    Altar,
}

// Component for tracking inventory capacity bonuses
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct InventoryBonus {
    pub capacity_bonus: i32,
    pub weight_bonus: f32,
    pub source: String, // What provides this bonus
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};

    #[test]
    fn test_advanced_inventory() {
        let mut inventory = AdvancedInventory::new(10, 50.0);
        
        assert_eq!(inventory.capacity, 10);
        assert_eq!(inventory.weight_limit, 50.0);
        assert!(!inventory.is_full());
        assert!(!inventory.is_overweight());
        
        // Test adding items (would need actual entities in real test)
        assert_eq!(inventory.items.len(), 0);
        assert_eq!(inventory.current_weight, 0.0);
    }

    #[test]
    fn test_inventory_slot() {
        let entity = specs::Entity::from_raw(1);
        let mut slot = InventorySlot::new(entity, 5);
        
        assert_eq!(slot.quantity, 5);
        assert!(!slot.locked);
        
        let added = slot.add_quantity(3);
        assert_eq!(added, 3);
        assert_eq!(slot.quantity, 8);
        
        let removed = slot.remove_quantity(2);
        assert_eq!(removed, 2);
        assert_eq!(slot.quantity, 6);
    }

    #[test]
    fn test_container() {
        let mut container = Container::new(5, ContainerType::Chest);
        let entity = specs::Entity::from_raw(1);
        
        assert!(!container.is_full());
        assert!(container.add_item(entity));
        assert_eq!(container.items.len(), 1);
        
        let removed = container.remove_item(0);
        assert_eq!(removed, Some(entity));
        assert_eq!(container.items.len(), 0);
    }
}