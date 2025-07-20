# Item and Inventory System

## Overview

The Item and Inventory System provides a comprehensive framework for managing items in the ASCII Dungeon Explorer game. It includes item creation, properties, serialization, and various utility functions for item management.

## Core Components

### ItemProperties
The main component that defines an item's characteristics:
- **Name and Description**: Basic identification
- **Item Type**: Categorizes items (Weapon, Armor, Consumable, etc.)
- **Rarity**: From Trash to Artifact, affects value and appearance
- **Value and Weight**: Economic and physical properties
- **Durability**: For items that can break or wear out
- **Requirements**: Level, attribute, and skill requirements
- **Tags**: Flexible categorization system

### Item Types
- **Weapons**: Swords, axes, bows, staves, etc.
- **Armor**: Helmets, chest pieces, shields, rings, etc.
- **Consumables**: Potions, food, scrolls, ammunition
- **Tools**: Lockpicks, torches, keys, containers
- **Materials**: Crafting components like metals, gems, herbs
- **Quest Items**: Special items for quests
- **Miscellaneous**: Everything else

### Item Rarity System
Items have different rarity levels that affect their properties:
- **Trash** (Gray): Broken or worthless items
- **Common** (White): Standard items
- **Uncommon** (Green): Slightly better than average
- **Rare** (Blue): Significantly enhanced items
- **Epic** (Purple): Powerful items with special properties
- **Legendary** (Orange): Extremely rare and powerful
- **Artifact** (Gold): Unique items with extraordinary abilities

### Stackable Items
Items that can be grouped together:
- **ItemStack Component**: Manages quantity and stack limits
- **Automatic Stacking**: Similar items combine when possible
- **Stack Limits**: Different items have different maximum stack sizes

### Magical Items
Items with supernatural properties:
- **Enchantments**: Positive magical effects
- **Curses**: Negative magical effects
- **Magic Level**: Determines the power of magical properties
- **Identification**: Some magical items need to be identified

### Item Bonuses
Items can provide various bonuses to characters:
- **Attribute Bonuses**: Strength, Dexterity, etc.
- **Skill Bonuses**: Combat skills, utility skills
- **Combat Bonuses**: Attack, damage, defense, critical chance
- **Special Bonuses**: Unique effects like resistances or regeneration

## Item Factory

The `ItemFactory` provides methods to create different types of items:

```rust
let factory = ItemFactory::new();
let mut rng = RandomNumberGenerator::new();

// Create specific items
let sword = factory.create_weapon(world, WeaponType::Sword, position, &mut rng);
let potion = factory.create_consumable(world, ConsumableType::Potion, position, &mut rng);
let armor = factory.create_armor(world, ArmorType::Chest, position, &mut rng);

// Create random items
let random_weapon = factory.create_random_weapon(world, position, &mut rng);
let magical_item = factory.create_magical_item(world, base_type, position, magic_level, &mut rng);
```

## Item Database and Templates

The system includes a template-based approach for defining items:

```rust
// Create or load item database
let db = ItemDatabase::create_default_database();
let db = ItemDatabase::load_from_file("items.json")?;

// Create items from templates
if let Some(template) = db.get_item_template("iron_sword") {
    let entity = template.create_item(world, position);
}
```

## Serialization System

Items can be serialized for save games and data persistence:

```rust
// Serialize individual items
let serializable = SerializableItem::from_entity(world, entity);

// Serialize all items in the world
let collection = ItemCollection::from_world(world);
collection.save_to_file("save_game_items.json")?;

// Load items back
let collection = ItemCollection::load_from_file("save_game_items.json")?;
let entities = collection.to_world(world);
```

## Utility Functions

The system provides many utility functions for common operations:

```rust
// Find items at a location
let items = find_items_at_position(world, x, y);

// Get item information
let name = get_item_display_name(world, entity);
let description = get_item_display_description(world, entity);
let value = get_item_current_value(world, entity);
let info = get_item_info_string(world, entity);

// Check item properties
let can_stack = can_stack_items(world, item1, item2);
let meets_reqs = meets_requirements(world, item, character);

// Calculate weights and counts
let weight = get_total_weight_at_position(world, x, y);
let counts = count_items_by_type(world);
```

## Integration with ECS

The item system is fully integrated with the Specs ECS framework:

```rust
// Register components in your world
world.register::<Item>();
world.register::<ItemProperties>();
world.register::<ItemStack>();
world.register::<ItemIdentification>();
world.register::<MagicalItem>();
world.register::<ItemBonuses>();

// Query items in systems
let entities = world.entities();
let items = world.read_storage::<Item>();
let properties = world.read_storage::<ItemProperties>();

for (entity, _item, props) in (&entities, &items, &properties).join() {
    // Process items
}
```

## Examples

### Creating a Basic Item
```rust
let properties = ItemProperties::new("Magic Sword".to_string(), ItemType::Weapon(WeaponType::Sword))
    .with_description("A sword imbued with magical power".to_string())
    .with_rarity(ItemRarity::Rare)
    .with_value(500)
    .with_durability(120);

let entity = world.create_entity()
    .with(Item)
    .with(Name { name: "Magic Sword".to_string() })
    .with(properties)
    .with(position)
    .with(renderable)
    .build();
```

### Creating a Magical Item
```rust
let mut magical_item = MagicalItem::new(3);

let enchantment = Enchantment {
    name: "Flame Weapon".to_string(),
    description: "Weapon deals fire damage".to_string(),
    enchantment_type: EnchantmentType::Fire,
    power: 5,
    duration: None,
};

magical_item.add_enchantment(enchantment);
```

### Working with Stacks
```rust
let mut stack = ItemStack::new(5, 20); // 5 items, max 20
let overflow = stack.add(10); // Try to add 10 more
let removed = stack.remove(3); // Remove 3 items
```

## Testing

The system includes comprehensive tests covering:
- Component creation and manipulation
- Item factory functionality
- Serialization and deserialization
- Utility function behavior
- Integration scenarios

Run tests with:
```bash
cargo test items::tests
```

## Future Enhancements

Planned improvements include:
- **Crafting System**: Combine materials to create new items
- **Item Sets**: Bonuses for wearing complete sets
- **Dynamic Properties**: Items that change based on usage
- **Item Modification**: Upgrade and customize existing items
- **Advanced Identification**: More complex identification mechanics
- **Item Decay**: Items that degrade over time
- **Unique Items**: One-of-a-kind artifacts with special properties

## Performance Considerations

- Use entity queries efficiently in systems
- Cache frequently accessed item data
- Batch item operations when possible
- Consider using component storage hints for better performance
- Serialize only necessary data for save games

## Best Practices

1. **Use the Factory**: Always use `ItemFactory` for creating items
2. **Template-Based**: Define items in templates for consistency
3. **Component Composition**: Use multiple components for complex items
4. **Proper Cleanup**: Remove items properly when destroyed
5. **Validation**: Validate item data when loading from files
6. **Error Handling**: Handle serialization errors gracefully
7. **Testing**: Test item interactions thoroughly