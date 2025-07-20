# Item Generation System Documentation

## Overview

The Item Generation System provides comprehensive procedural item generation for the ASCII Dungeon Explorer game. It includes rarity-based generation, depth scaling, affix systems, loot tables, and intelligent name generation.

## Core Components

### ItemGenerator

The main item generation engine that creates items based on various parameters:

```rust
pub struct ItemGenerator {
    pub loot_tables: HashMap<String, LootTable>,
    pub affix_tables: HashMap<ItemType, AffixTable>,
    pub rarity_weights: RarityWeights,
    pub depth_scaling: DepthScaling,
}
```

**Key Features:**
- **Procedural Generation**: Creates items based on context and parameters
- **Depth Scaling**: Items become more powerful at deeper dungeon levels
- **Rarity System**: Seven-tier rarity system with appropriate generation weights
- **Affix System**: Prefix and suffix modifiers for enhanced items
- **Context Awareness**: Different generation patterns for different situations

**Main Methods:**
- `generate_item()`: Create a single item with specified parameters
- `generate_from_loot_table()`: Generate items from predefined loot tables
- `apply_depth_scaling()`: Scale item stats based on dungeon depth
- `apply_rarity_modifications()`: Apply rarity-specific enhancements
- `apply_affixes()`: Add prefix/suffix modifiers to items

### Generation Contexts

Items are generated differently based on context:

```rust
pub enum GenerationContext {
    Combat,    // Dropped by monsters
    Treasure,  // Found in chests/treasure
    Merchant,  // Sold by merchants
    Random,    // Random generation
}
```

- **Combat**: Emphasizes weapons, armor, and healing items
- **Treasure**: Higher chance of valuable items and materials
- **Merchant**: Balanced selection suitable for trade
- **Random**: Equal distribution across all item types

### Rarity System

Seven-tier rarity system with scaling benefits:

```rust
pub enum ItemRarity {
    Trash,      // 0.1x value multiplier
    Common,     // 1.0x value multiplier
    Uncommon,   // 2.0x value multiplier
    Rare,       // 5.0x value multiplier
    Epic,       // 10.0x value multiplier
    Legendary,  // 25.0x value multiplier
    Artifact,   // 100.0x value multiplier
}
```

**Rarity Effects:**
- **Value Scaling**: Higher rarities have exponentially higher values
- **Stat Bonuses**: Better stats and more powerful effects
- **Magical Properties**: Rare+ items can have enchantments
- **Affix Count**: Higher rarities get more prefix/suffix modifiers
- **Visual Distinction**: Color-coded display based on rarity

### Depth Scaling

Items scale with dungeon depth for progressive difficulty:

```rust
pub struct DepthScaling {
    pub stat_scaling: f32,     // 0.1 = 10% increase per level
    pub value_scaling: f32,    // 0.05 = 5% value increase per level
    pub rarity_scaling: f32,   // 1.0 = 1 point rarity bonus per level
}
```

**Scaling Effects:**
- **Stat Bonuses**: Attack, damage, and defense scale with depth
- **Item Values**: Economic value increases with depth
- **Rarity Chances**: Better rarities become more common at depth
- **Affix Power**: Stronger affixes appear at greater depths

## Affix System

### Affix Types

Items can have prefixes and suffixes that modify their properties:

```rust
pub struct Affix {
    pub name: String,
    pub affix_type: AffixType,
    pub stat_bonuses: HashMap<String, i32>,
    pub value_bonus: i32,
    pub weight: i32,
}

pub enum AffixType {
    Prefix,  // "Sharp Sword"
    Suffix,  // "Sword of Power"
}
```

**Common Prefixes:**
- **Sharp**: +2 Damage
- **Heavy**: +4 Damage, -1 Attack
- **Swift**: +3 Attack
- **Sturdy**: +3 Defense
- **Blessed**: Various bonuses

**Common Suffixes:**
- **of Power**: +2 Strength
- **of Precision**: +5% Critical Chance
- **of Protection**: +4 Defense
- **of Vitality**: +3 Constitution
- **of the Ancients**: Multiple bonuses

### Affix Application Rules

- **Uncommon**: 1 affix maximum
- **Rare**: 1-2 affixes
- **Epic**: 1-3 affixes
- **Legendary**: 2-3 affixes
- **Artifact**: 2-4 affixes

## Loot Table System

### LootTableManager

Manages predefined loot tables for different scenarios:

```rust
pub struct LootTableManager {
    pub tables: HashMap<String, LootTable>,
    pub monster_tables: HashMap<String, String>,
    pub depth_tables: HashMap<i32, String>,
    pub special_tables: HashMap<String, String>,
}
```

**Loot Table Types:**
- **Monster Tables**: Specific drops for different monster types
- **Container Tables**: Chest and container contents
- **Depth Tables**: General loot based on dungeon depth
- **Special Tables**: Location-specific loot (Library, Armory, etc.)

### Predefined Loot Tables

**Monster Loot:**
- **Goblin**: Basic weapons, potions, small amounts of gold
- **Skeleton**: Weapons, armor, bones, scrolls
- **Orc**: Heavy weapons, armor, food, metal materials
- **Dragon**: Legendary weapons, epic armor, gems, large gold hoards

**Container Loot:**
- **Wooden Chest**: Common items, tools, small gold amounts
- **Iron Chest**: Uncommon weapons/armor, materials, medium gold
- **Golden Chest**: Rare items, gems, scrolls, large gold amounts

**Special Location Loot:**
- **Library**: Scrolls, staves, herbs, knowledge items
- **Armory**: Weapons, armor, ammunition, metal materials
- **Treasury**: Gems, jewelry, gold, valuable items

## Name Generation System

### ItemNameGenerator

Procedural name generation based on item properties:

```rust
pub struct ItemNameGenerator {
    pub weapon_bases: HashMap<WeaponType, Vec<String>>,
    pub armor_bases: HashMap<ArmorType, Vec<String>>,
    pub prefixes: Vec<NameAffix>,
    pub suffixes: Vec<NameAffix>,
    pub legendary_names: Vec<String>,
    pub artifact_names: Vec<String>,
}
```

**Name Generation Rules:**
- **Common/Trash**: Simple base names, occasional quality descriptors
- **Uncommon**: Quality descriptors or simple magical affixes
- **Rare/Epic**: Magical prefixes and suffixes
- **Legendary**: Predefined legendary names or epic generation
- **Artifact**: Unique artifact names with "The" prefix

**Example Names:**
- Common: "Iron Sword", "Worn Leather Armor"
- Uncommon: "Fine Steel Blade", "Sturdy Chain Mail"
- Rare: "Flaming Sword of Power", "Blessed Shield of Protection"
- Epic: "Ancient Blade of the Eagle", "Divine Armor of Warding"
- Legendary: "Excalibur", "Mjolnir"
- Artifact: "The Worldrender", "Eternity's Edge"

## Integration Examples

### Basic Item Generation

```rust
let generator = ItemGenerator::new();
let mut rng = RandomNumberGenerator::new();

// Generate a random item
let item = generator.generate_item(
    world,
    Position { x: 10, y: 10 },
    15, // dungeon depth
    GenerationContext::Treasure,
    &mut rng,
);

// Generate appropriate name
let name_generator = ItemNameGenerator::new();
let item_type = /* get from item properties */;
let rarity = /* get from item properties */;
let has_enchantments = /* check for magical properties */;

let name = name_generator.generate_name(&item_type, &rarity, has_enchantments, &mut rng);
```

### Loot Table Usage

```rust
let loot_manager = LootTableManager::new();

// Generate monster loot
let goblin_loot = loot_manager.generate_monster_loot(
    world,
    &generator,
    "Goblin",
    position,
    depth,
    &mut rng,
);

// Generate container loot
let chest_loot = loot_manager.generate_container_loot(
    world,
    &generator,
    "golden_chest",
    position,
    depth,
    &mut rng,
);

// Generate special location loot
let library_loot = loot_manager.generate_special_loot(
    world,
    &generator,
    "Library",
    position,
    depth,
    &mut rng,
);
```

### Dungeon Population

```rust
fn populate_dungeon(world: &mut World, depth: i32) -> Vec<Entity> {
    let integration = ItemGenerationIntegration::new();
    let mut rng = RandomNumberGenerator::new();
    let mut items = Vec::new();

    // Scatter random loot
    for x in 0..map_width {
        for y in 0..map_height {
            if rng.roll_dice(1, 100) <= 3 { // 3% chance
                let item = integration.item_generator.generate_item(
                    world,
                    Position { x, y },
                    depth,
                    GenerationContext::Random,
                    &mut rng,
                );
                items.push(item);
            }
        }
    }

    // Place treasure chests
    for _ in 0..5 {
        let chest_loot = integration.loot_manager.generate_container_loot(
            world,
            &integration.item_generator,
            "iron_chest",
            random_position(),
            depth,
            &mut rng,
        );
        items.extend(chest_loot);
    }

    items
}
```

## Advanced Features

### Custom Loot Tables

```rust
let custom_table = LootTable {
    entries: vec![
        LootEntry {
            item_type: Some(ItemType::Weapon(WeaponType::Sword)),
            table_reference: None,
            weight: 30,
            quantity_range: (1, 1),
            rarity_override: Some(ItemRarity::Rare),
        },
        // ... more entries
    ],
    guaranteed_drops: 2,
    max_drops: 4,
};

loot_manager.add_table("custom_boss", custom_table);
```

### Custom Affixes

```rust
let custom_affix = Affix {
    name: "Vampiric".to_string(),
    affix_type: AffixType::Prefix,
    stat_bonuses: vec![
        ("damage".to_string(), 3),
        ("life_steal".to_string(), 5),
    ].into_iter().collect(),
    value_bonus: 75,
    weight: 10,
};

// Add to weapon affix table
if let Some(weapon_table) = generator.affix_tables.get_mut(&ItemType::Weapon(WeaponType::Sword)) {
    weapon_table.prefixes.push(custom_affix);
}
```

### Rarity Weight Customization

```rust
let mut custom_weights = HashMap::new();
custom_weights.insert(ItemRarity::Common, 40);
custom_weights.insert(ItemRarity::Uncommon, 30);
custom_weights.insert(ItemRarity::Rare, 20);
custom_weights.insert(ItemRarity::Epic, 8);
custom_weights.insert(ItemRarity::Legendary, 2);

generator.rarity_weights.base_weights = custom_weights;
```

### Depth Scaling Adjustment

```rust
generator.depth_scaling = DepthScaling {
    stat_scaling: 0.15,    // 15% increase per level
    value_scaling: 0.08,   // 8% value increase per level
    rarity_scaling: 1.5,   // 1.5 point rarity bonus per level
};
```

## Performance Considerations

1. **Efficient Generation**: Use appropriate contexts to limit item type selection
2. **Caching**: Cache frequently used loot tables and affix combinations
3. **Batch Generation**: Generate multiple items at once when possible
4. **Memory Management**: Clean up unused generated items regularly
5. **Randomization**: Use efficient random number generation

## Configuration and Persistence

### Save/Load Loot Tables

```rust
// Save loot tables to file
loot_manager.save_to_file("loot_tables.json")?;

// Load loot tables from file
let loaded_manager = LootTableManager::load_from_file("loot_tables.json")?;
```

### Save/Load Name Generator

```rust
// Save name generator data
name_generator.save_to_file("name_generator.json")?;

// Load name generator data
let loaded_generator = ItemNameGenerator::load_from_file("name_generator.json")?;
```

## Testing and Validation

The system includes comprehensive tests for:

```bash
# Run generation system tests
cargo test item_generation::tests
cargo test loot_tables::tests
cargo test name_generator::tests
cargo test generation_integration::tests
```

**Test Coverage:**
- Item generation with different parameters
- Depth scaling verification
- Rarity distribution testing
- Affix application validation
- Loot table functionality
- Name generation quality
- Integration scenarios

## Future Enhancements

1. **Set Items**: Items that provide bonuses when worn together
2. **Cursed Items**: Negative effects with powerful benefits
3. **Evolving Items**: Items that change based on usage
4. **Crafting Integration**: Use generated materials for crafting
5. **Dynamic Loot Tables**: Tables that change based on game state
6. **AI-Generated Names**: Machine learning for more creative names
7. **Seasonal Items**: Time-based special item generation
8. **Player Influence**: Generation based on player actions/preferences

## Troubleshooting

### Common Issues

1. **No Items Generated**: Check loot table weights and generation context
2. **Unbalanced Rarities**: Adjust rarity weights and depth scaling
3. **Boring Names**: Add more base names and affixes to generators
4. **Performance Issues**: Optimize generation frequency and caching

### Debug Tools

```rust
// Show generation statistics
let stats = loot_manager.get_statistics();
println!("Loot tables: {}, Entries: {}", stats.total_tables, stats.total_entries);

// Test item generation
for _ in 0..100 {
    let item = generator.generate_item(world, pos, depth, context, &mut rng);
    // Analyze generated items
}

// Validate loot table weights
for (name, table) in &loot_manager.tables {
    let total_weight: i32 = table.entries.iter().map(|e| e.weight).sum();
    println!("Table '{}' total weight: {}", name, total_weight);
}
```

This item generation system provides a robust foundation for creating diverse, interesting, and appropriately scaled items throughout the game world.