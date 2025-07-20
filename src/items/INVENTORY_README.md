# Inventory System Documentation

## Overview

The Inventory System provides comprehensive item management capabilities for the ASCII Dungeon Explorer game. It includes inventory management, item pickup/dropping, UI components, and container interactions.

## Core Components

### AdvancedInventory

The main inventory component that manages a character's items:

```rust
pub struct AdvancedInventory {
    pub items: Vec<InventorySlot>,
    pub capacity: usize,
    pub weight_limit: f32,
    pub current_weight: f32,
    pub gold: i32,
    pub auto_pickup: bool,
    pub sort_mode: InventorySortMode,
}
```

**Key Features:**
- **Capacity Management**: Limited number of item slots
- **Weight System**: Realistic weight-based carrying capacity
- **Gold Tracking**: Built-in currency management
- **Auto-pickup**: Automatic item collection
- **Sorting**: Multiple sorting modes (Name, Type, Value, Weight)
- **Stacking**: Automatic item stacking for compatible items

**Methods:**
- `new(capacity, weight_limit)`: Create new inventory
- `is_full()`: Check if inventory is at capacity
- `is_overweight()`: Check if carrying too much weight
- `can_add_item(weight)`: Check if item can be added
- `add_item(entity, quantity, weight)`: Add item to inventory
- `remove_item(slot_index, quantity, weight)`: Remove item from inventory
- `find_item(entity)`: Find item's slot index
- `get_total_value(world)`: Calculate total inventory value
- `sort_inventory(world)`: Sort items based on current mode

### InventorySlot

Individual inventory slots that hold items:

```rust
pub struct InventorySlot {
    pub entity: Entity,
    pub quantity: i32,
    pub locked: bool,
}
```

**Features:**
- **Entity Reference**: Links to the actual item entity
- **Quantity Tracking**: Supports stackable items
- **Lock Protection**: Prevents accidental dropping of important items

### Container

Component for chests, barrels, corpses, and other item containers:

```rust
pub struct Container {
    pub items: Vec<Entity>,
    pub capacity: usize,
    pub locked: bool,
    pub key_required: Option<Entity>,
    pub container_type: ContainerType,
}
```

**Container Types:**
- **Chest**: Standard treasure containers
- **Barrel**: Large storage containers
- **Crate**: Wooden storage boxes
- **Bag**: Portable containers
- **Corpse**: Lootable dead entities
- **Altar**: Special ritual containers

## Systems

### ItemPickupSystem

Handles item pickup requests and inventory management:

```rust
pub struct ItemPickupSystem;
```

**Functionality:**
- Processes `WantsToPickupItem` components
- Checks inventory capacity and weight limits
- Handles item stacking automatically
- Removes items from world when picked up
- Provides feedback messages
- Handles pickup failures gracefully

### ItemDropSystem

Manages item dropping and world placement:

```rust
pub struct ItemDropSystem;
```

**Functionality:**
- Processes `WantsToDropItem` components
- Finds suitable drop locations near player
- Updates item positions in world
- Handles item stacking when dropping
- Provides feedback messages

### AutoPickupSystem

Automatically picks up valuable or useful items:

```rust
pub struct AutoPickupSystem;
```

**Auto-pickup Rules:**
- Health potions and consumables
- Crafting materials
- Items of Uncommon rarity or higher
- Configurable per-player preferences

### InventoryManagementSystem

Maintains inventory state and organization:

```rust
pub struct InventoryManagementSystem;
```

**Functionality:**
- Updates inventory weight calculations
- Cleans up empty inventory slots
- Applies sorting when enabled
- Manages inventory bonuses

## User Interface

### InventoryUI

Complete inventory interface with navigation and filtering:

```rust
pub struct InventoryUI {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub items_per_page: usize,
    pub show_details: bool,
    pub filter_mode: InventoryFilter,
    pub sort_mode: InventorySortMode,
}
```

**Features:**
- **Navigation**: Arrow key navigation with scrolling
- **Filtering**: Filter by item type (Weapons, Armor, Consumables, etc.)
- **Sorting**: Sort by Name, Type, Value, or Weight
- **Item Details**: Detailed item information display
- **Color Coding**: Rarity-based color coding
- **Weight/Capacity Display**: Visual inventory status

**Controls:**
- `↑↓`: Navigate items
- `Enter`: Use/Equip item
- `D`: Drop selected item
- `I`: Toggle item details
- `S`: Cycle sort modes
- `F`: Cycle filter modes
- `A`: Toggle auto-pickup
- `ESC`: Close inventory

### ContainerUI

Interface for interacting with containers:

```rust
pub struct ContainerUI {
    pub selected_container_index: usize,
    pub selected_inventory_index: usize,
    pub active_panel: ContainerPanel,
    pub transfer_mode: bool,
}
```

**Features:**
- **Split View**: Container and inventory side-by-side
- **Transfer Items**: Move items between container and inventory
- **Bulk Operations**: Transfer all items at once
- **Panel Navigation**: Switch between container and inventory
- **Visual Feedback**: Highlight active panel and selected items

**Controls:**
- `↑↓`: Navigate items in active panel
- `Tab`: Switch between container and inventory
- `Enter`: Transfer selected item
- `T`: Transfer all items from active panel
- `ESC`: Close container interface

## Sorting and Filtering

### Sort Modes

```rust
pub enum InventorySortMode {
    None,        // No sorting
    Name,        // Alphabetical by name
    Type,        // Group by item type
    Value,       // Highest value first
    Weight,      // Lightest first
}
```

### Filter Modes

```rust
pub enum InventoryFilter {
    All,         // Show all items
    Weapons,     // Weapons only
    Armor,       // Armor and accessories
    Consumables, // Potions, food, scrolls
    Tools,       // Utility items
    Materials,   // Crafting materials
    Valuable,    // Rare items and above
}
```

## Integration Examples

### Basic Setup

```rust
// Register components
world.register::<AdvancedInventory>();
world.register::<Container>();
world.register::<WantsToPickupItem>();
world.register::<WantsToDropItem>();

// Create player with inventory
let player = world.create_entity()
    .with(Player)
    .with(Position { x: 0, y: 0 })
    .with(AdvancedInventory::new(20, 100.0)) // 20 slots, 100 lbs
    .build();

// Add systems to dispatcher
let mut dispatcher = DispatcherBuilder::new()
    .with(ItemPickupSystem, "pickup", &[])
    .with(ItemDropSystem, "drop", &["pickup"])
    .with(AutoPickupSystem, "auto_pickup", &[])
    .with(InventoryManagementSystem, "inventory_mgmt", &["pickup", "drop"])
    .build();
```

### Item Pickup

```rust
// Player wants to pick up an item
world.write_storage::<WantsToPickupItem>()
    .insert(player_entity, WantsToPickupItem { item: item_entity })
    .expect("Failed to insert pickup intent");

// Run systems to process the pickup
dispatcher.dispatch(&world);
world.maintain();
```

### Inventory UI Integration

```rust
// Create inventory UI
let mut inventory_ui = InventoryUI::new();

// Handle input
let action = inventory_ui.handle_input(key_event, &mut world, player_entity);
match action {
    InventoryAction::UseItem(entity) => {
        // Handle item usage
    },
    InventoryAction::DropItem(entity) => {
        // Create drop intent
        world.write_storage::<WantsToDropItem>()
            .insert(player_entity, WantsToDropItem { item: entity })
            .expect("Failed to insert drop intent");
    },
    InventoryAction::Close => {
        // Close inventory interface
    },
    _ => {},
}

// Render inventory
inventory_ui.render(&world, player_entity, terminal_width, terminal_height)?;
```

### Container Interaction

```rust
// Create a chest with items
let chest = world.create_entity()
    .with(Position { x: 10, y: 10 })
    .with(Name { name: "Treasure Chest".to_string() })
    .with(Container::new(15, ContainerType::Chest))
    .build();

// Add items to container
let mut containers = world.write_storage::<Container>();
if let Some(container) = containers.get_mut(chest) {
    container.add_item(sword_entity);
    container.add_item(potion_entity);
}

// Use container UI
let mut container_ui = ContainerUI::new();
let action = container_ui.handle_input(key_event);
match action {
    ContainerAction::TakeFromContainer(index) => {
        // Transfer item from container to inventory
    },
    ContainerAction::PutInContainer(index) => {
        // Transfer item from inventory to container
    },
    _ => {},
}
```

## Advanced Features

### Inventory Bonuses

Items can provide inventory bonuses:

```rust
#[derive(Component)]
pub struct InventoryBonus {
    pub capacity_bonus: i32,
    pub weight_bonus: f32,
    pub source: String,
}

// Bag of Holding provides +10 capacity
world.create_entity()
    .with(InventoryBonus {
        capacity_bonus: 10,
        weight_bonus: 0.0,
        source: "Bag of Holding".to_string(),
    })
    .build();
```

### Custom Pickup Rules

```rust
impl AutoPickupSystem {
    fn should_auto_pickup(&self, props: &ItemProperties) -> bool {
        match props.item_type {
            ItemType::Consumable(ConsumableType::Potion) => true,
            ItemType::Material(_) => true,
            _ => props.rarity >= ItemRarity::Uncommon,
        }
    }
}
```

### Weight Management

```rust
// Check if player is overweight
let inventories = world.read_storage::<AdvancedInventory>();
if let Some(inventory) = inventories.get(player_entity) {
    if inventory.is_overweight() {
        // Apply movement penalties
        // Reduce movement speed
        // Increase stamina consumption
    }
}
```

## Performance Considerations

1. **Efficient Queries**: Use specific component combinations in systems
2. **Batch Operations**: Process multiple pickup/drop requests together
3. **UI Optimization**: Only render visible inventory items
4. **Memory Management**: Clean up empty inventory slots regularly
5. **Caching**: Cache frequently accessed item properties

## Testing

The inventory system includes comprehensive tests:

```bash
# Run inventory system tests
cargo test inventory_system::tests

# Run inventory UI tests
cargo test inventory_ui::tests

# Run integration tests
cargo test inventory_integration::tests
```

## Future Enhancements

1. **Quick Actions**: Hotkey slots for frequently used items
2. **Item Sets**: Equipment sets with bonuses
3. **Inventory Tabs**: Organize items into categories
4. **Search Function**: Find items by name or property
5. **Drag and Drop**: Mouse-based item management
6. **Item Comparison**: Side-by-side equipment comparison
7. **Inventory Sharing**: Trade items between players
8. **Smart Sorting**: AI-powered inventory organization

## Troubleshooting

### Common Issues

1. **Items Not Stacking**: Ensure items have identical properties
2. **Pickup Failures**: Check inventory capacity and weight limits
3. **UI Not Updating**: Ensure systems run after inventory changes
4. **Performance Issues**: Optimize system queries and UI rendering

### Debug Commands

```rust
// Print inventory contents
fn debug_inventory(world: &World, entity: Entity) {
    let inventories = world.read_storage::<AdvancedInventory>();
    if let Some(inventory) = inventories.get(entity) {
        println!("Inventory: {}/{} items, {:.1}/{:.1} lbs",
            inventory.items.len(), inventory.capacity,
            inventory.current_weight, inventory.weight_limit);
    }
}

// Validate inventory state
fn validate_inventory(world: &World, entity: Entity) -> bool {
    let inventories = world.read_storage::<AdvancedInventory>();
    if let Some(inventory) = inventories.get(entity) {
        // Check for negative quantities
        for slot in &inventory.items {
            if slot.quantity <= 0 {
                return false;
            }
        }
        // Check weight calculation
        // ... additional validation
    }
    true
}
```

This inventory system provides a solid foundation for item management in the ASCII Dungeon Explorer game, with room for future expansion and customization.