# Consumable System Documentation

## Overview

The Consumable System provides comprehensive consumable item functionality for the ASCII Dungeon Explorer game. It includes item usage mechanics, status effects, cooldowns, restrictions, and a variety of consumable types with different effects.

## Core Components

### Consumable Component

The main component that defines consumable item properties:

```rust
pub struct Consumable {
    pub consumable_type: ConsumableType,
    pub effects: Vec<ConsumableEffect>,
    pub use_time: f32,
    pub cooldown: f32,
    pub charges: Option<i32>,
    pub requirements: ConsumableRequirements,
    pub restrictions: Vec<ConsumableRestriction>,
}
```

**Key Features:**
- **Multiple Effects**: Items can have multiple simultaneous effects
- **Usage Time**: Realistic consumption time (potions are fast, food is slow)
- **Cooldowns**: Prevent spam usage of powerful items
- **Limited Charges**: Some items have limited uses
- **Requirements**: Level, attribute, or skill requirements
- **Restrictions**: Situational usage limitations

### Consumable Effects

Comprehensive effect system supporting various types of consumable effects:

```rust
pub enum ConsumableEffect {
    Healing { amount: i32, over_time: bool },
    ManaRestore { amount: i32, over_time: bool },
    StaminaRestore { amount: i32, over_time: bool },
    StatusEffect { effect_type: StatusEffectType, duration: f32, power: i32 },
    AttributeBoost { attribute: String, amount: i32, duration: f32 },
    CureCondition { condition: StatusEffectType },
    SpellCast { spell_id: String },
    Teleport { range: i32, random: bool },
    RevealMap { radius: i32 },
    Identify { count: i32 },
    Custom { effect_id: String, parameters: HashMap<String, String> },
}
```

**Effect Types:**
- **Instant Effects**: Immediate healing, mana restoration
- **Over-Time Effects**: Gradual healing, regeneration
- **Status Effects**: Temporary buffs and debuffs
- **Utility Effects**: Teleportation, map revelation, item identification
- **Spell Effects**: Cast spells from consumables
- **Custom Effects**: Extensible system for unique effects

### Status Effect System

Comprehensive status effect management:

```rust
pub struct StatusEffects {
    pub effects: HashMap<StatusEffectType, StatusEffect>,
}

pub struct StatusEffect {
    pub power: i32,
    pub duration: f32,
    pub tick_interval: f32,
    pub last_tick: f32,
    pub source: Option<Entity>,
}
```

**Status Effect Types:**
- **Beneficial**: Regeneration, Haste, Strength, Protection, Invisibility
- **Harmful**: Poison, Disease, Curse, Weakness, Paralysis
- **Neutral**: Detect, Levitation, Water Walking

**Status Effect Features:**
- **Stacking Rules**: Some effects stack, others replace
- **Duration Tracking**: Automatic expiration
- **Tick System**: Periodic effects (poison, regeneration)
- **Source Tracking**: Track what caused the effect

### Cooldown System

Prevents consumable spam and adds tactical depth:

```rust
pub struct ConsumableCooldowns {
    pub cooldowns: HashMap<String, f32>,
    pub global_cooldown: f32,
}
```

**Cooldown Features:**
- **Individual Cooldowns**: Per-consumable-type cooldowns
- **Global Cooldown**: Prevents rapid consumption of any consumables
- **Automatic Updates**: Time-based cooldown reduction
- **Flexible Configuration**: Different cooldowns for different items

### Restriction System

Situational limitations on consumable usage:

```rust
pub enum ConsumableRestriction {
    NoCombat,
    NoMovement,
    NoStatusEffect(StatusEffectType),
    NoLocation(String),
    DailyLimit(i32),
    HealthThreshold(f32),
    ManaThreshold(f32),
    Custom(String),
}
```

**Restriction Types:**
- **Combat Restrictions**: Cannot use during combat
- **Health/Mana Thresholds**: Only usable below certain percentages
- **Status Restrictions**: Cannot use while certain effects are active
- **Location Restrictions**: Cannot use in certain areas
- **Usage Limits**: Daily or total usage limits

## Consumable Types

### Potions

Fast-acting liquid consumables:

```rust
// Health potions of different potencies
PotionPotency::Minor    // 15 HP, 25 gold
PotionPotency::Lesser   // 25 HP, 50 gold
PotionPotency::Greater  // 50 HP, 100 gold
PotionPotency::Superior // 75 HP, 200 gold
PotionPotency::Ultimate // 100 HP, 500 gold
```

**Potion Types:**
- **Health Potions**: Instant healing
- **Mana Potions**: Instant mana restoration
- **Regeneration Potions**: Healing over time
- **Stat Potions**: Temporary attribute boosts
- **Cure Potions**: Remove specific conditions
- **Emergency Potions**: Powerful but restricted usage

### Food Items

Slower consumption with sustained effects:

```rust
pub enum FoodType {
    Bread,    // Basic sustenance
    Cheese,   // Moderate nutrition
    Meat,     // High nutrition
    Apple,    // Light snack
    Rations,  // Long-lasting sustenance
}
```

**Food Features:**
- **Slow Consumption**: Takes 3 seconds to eat
- **Healing Over Time**: Gradual health restoration
- **Well-Fed Status**: Temporary benefits from nutrition
- **Stackable**: Can carry multiple portions

### Scrolls

Single-use magical items:

```rust
pub enum ScrollType {
    Healing,      // Magical healing
    Fireball,     // Offensive spell
    Teleport,     // Movement spell
    Identify,     // Utility spell
    MagicMapping, // Exploration spell
}
```

**Scroll Features:**
- **Single Use**: Consumed when used
- **Spell Effects**: Cast spells without mana cost
- **Varied Rarity**: From common to rare
- **No Cooldown**: Can be used immediately

## Systems

### ConsumableUsageSystem

Handles consumable usage requests:

```rust
pub struct ConsumableUsageSystem;
```

**Functionality:**
- Processes `WantsToUseConsumable` components
- Validates usage requirements and restrictions
- Applies consumable effects
- Manages charges and depletion
- Sets cooldowns
- Provides usage feedback

### ConsumableUpdateSystem

Manages ongoing effects and cooldowns:

```rust
pub struct ConsumableUpdateSystem;
```

**Functionality:**
- Updates cooldown timers
- Processes status effect durations
- Applies periodic effects (poison, regeneration)
- Removes expired effects
- Provides expiration notifications

## Integration Examples

### Basic Setup

```rust
// Register components
world.register::<Consumable>();
world.register::<WantsToUseConsumable>();
world.register::<ConsumableCooldowns>();
world.register::<StatusEffects>();

// Add systems to dispatcher
let mut dispatcher = DispatcherBuilder::new()
    .with(ConsumableUsageSystem, "consumable_usage", &[])
    .with(ConsumableUpdateSystem, "consumable_update", &["consumable_usage"])
    .build();

// Add delta time resource
world.insert(0.016f32); // 60 FPS
```

### Creating Consumables

```rust
let factory = ConsumableFactory::new();

// Create different types of consumables
let health_potion = factory.create_health_potion(world, position, PotionPotency::Lesser);
let mana_potion = factory.create_mana_potion(world, position, PotionPotency::Lesser);
let bread = factory.create_food(world, position, FoodType::Bread);
let healing_scroll = factory.create_scroll(world, position, ScrollType::Healing);

// Create random consumables by context
let combat_item = factory.create_random_consumable(world, position, ConsumableContext::Combat, &mut rng);
```

### Using Consumables

```rust
// Player wants to use a consumable
world.write_storage::<WantsToUseConsumable>()
    .insert(player_entity, WantsToUseConsumable { 
        item: consumable_entity, 
        target: None // Use on self
    })
    .expect("Failed to insert usage intent");

// Run systems to process usage
dispatcher.dispatch(&world);
world.maintain();
```

### Managing Status Effects

```rust
// Check for status effects
let status_effects = world.read_storage::<StatusEffects>();
if let Some(effects) = status_effects.get(entity) {
    if effects.has_effect(&StatusEffectType::Poison) {
        // Handle poison effect
    }
    
    if effects.has_effect(&StatusEffectType::Regeneration) {
        // Handle regeneration effect
    }
}

// Apply custom status effect
let mut effects = world.write_storage::<StatusEffects>();
if let Some(entity_effects) = effects.get_mut(entity) {
    let effect = StatusEffect::new(5, 30.0) // 5 power, 30 seconds
        .with_tick_interval(1.0)
        .with_source(source_entity);
    
    entity_effects.add_effect(StatusEffectType::Strength, effect);
}
```

### Checking Cooldowns

```rust
let cooldowns = world.read_storage::<ConsumableCooldowns>();
if let Some(cd) = cooldowns.get(player_entity) {
    if cd.is_on_cooldown("Potion") {
        let remaining = cd.get_cooldown("Potion");
        println!("Must wait {:.1} seconds", remaining);
    }
}
```

## Advanced Features

### Custom Effects

```rust
// Create consumable with custom effect
let custom_effect = ConsumableEffect::Custom {
    effect_id: "teleport_to_town".to_string(),
    parameters: vec![
        ("destination".to_string(), "town_square".to_string()),
        ("cost".to_string(), "100".to_string()),
    ].into_iter().collect(),
};

let consumable = Consumable::new(ConsumableType::Scroll)
    .with_effects(vec![custom_effect]);
```

### Complex Restrictions

```rust
// Emergency potion with multiple restrictions
let emergency_potion = Consumable::new(ConsumableType::Potion)
    .with_effects(vec![
        ConsumableEffect::Healing { amount: 100, over_time: false },
        ConsumableEffect::StatusEffect {
            effect_type: StatusEffectType::Haste,
            duration: 30.0,
            power: 2,
        }
    ])
    .with_cooldown(300.0) // 5 minute cooldown
    .add_restriction(ConsumableRestriction::HealthThreshold(0.25)) // Only below 25% health
    .add_restriction(ConsumableRestriction::DailyLimit(1)); // Once per day
```

### Status Effect Stacking

```rust
// Configure stacking behavior
impl StatusEffects {
    pub fn add_effect(&mut self, effect_type: StatusEffectType, effect: StatusEffect) {
        if let Some(existing) = self.effects.get_mut(&effect_type) {
            match effect_type {
                // Stackable effects
                StatusEffectType::Poison | StatusEffectType::Regeneration => {
                    existing.power += effect.power;
                    existing.duration = existing.duration.max(effect.duration);
                },
                // Non-stackable effects (replace with longer duration)
                _ => {
                    if effect.duration > existing.duration {
                        *existing = effect;
                    }
                }
            }
        } else {
            self.effects.insert(effect_type, effect);
        }
    }
}
```

## Configuration

### Consumable Factory Customization

```rust
impl ConsumableFactory {
    pub fn create_custom_potion(
        &self,
        world: &mut World,
        name: String,
        effects: Vec<ConsumableEffect>,
        value: i32,
        cooldown: f32,
        restrictions: Vec<ConsumableRestriction>,
    ) -> Entity {
        let consumable = Consumable::new(ConsumableType::Potion)
            .with_effects(effects)
            .with_cooldown(cooldown);
        
        // Add restrictions
        let consumable = restrictions.into_iter()
            .fold(consumable, |c, r| c.add_restriction(r));
        
        // Create entity with custom properties
        // ... entity creation code
    }
}
```

### Global Cooldown Configuration

```rust
// Set global cooldown for all consumables
let mut cooldowns = world.write_storage::<ConsumableCooldowns>();
if let Some(cd) = cooldowns.get_mut(player_entity) {
    cd.set_global_cooldown(1.0); // 1 second global cooldown
}
```

## Performance Considerations

1. **Efficient Updates**: Status effects and cooldowns update only when necessary
2. **Batch Processing**: Process multiple consumable uses in single system run
3. **Memory Management**: Expired effects are automatically cleaned up
4. **Tick Optimization**: Status effects only tick when their interval is reached
5. **Component Queries**: Use specific component combinations for better performance

## Testing

The system includes comprehensive tests:

```bash
# Run consumable system tests
cargo test consumable_system::tests
cargo test consumable_factory::tests
cargo test consumable_integration::tests
```

**Test Coverage:**
- Consumable creation and configuration
- Effect application and duration
- Cooldown management
- Restriction validation
- Status effect stacking and expiration
- Integration scenarios

## Future Enhancements

1. **Alchemy System**: Combine consumables to create new ones
2. **Consumable Crafting**: Create consumables from materials
3. **Advanced Status Effects**: More complex effect interactions
4. **Consumable Sets**: Bonuses for using related consumables
5. **Dynamic Effects**: Effects that change based on character stats
6. **Consumable Mastery**: Improved effects with usage experience
7. **Environmental Effects**: Location-based consumable modifications
8. **Social Consumables**: Items that affect multiple players

## Troubleshooting

### Common Issues

1. **Effects Not Applying**: Check consumable requirements and restrictions
2. **Cooldowns Not Working**: Ensure ConsumableUpdateSystem is running
3. **Status Effects Not Expiring**: Verify delta time resource is updated
4. **Charges Not Depleting**: Check consumable usage system processing

### Debug Tools

```rust
// Debug consumable state
fn debug_consumable(world: &World, entity: Entity) {
    let consumables = world.read_storage::<Consumable>();
    if let Some(consumable) = consumables.get(entity) {
        println!("Consumable: {:?}", consumable.consumable_type);
        println!("Effects: {}", consumable.effects.len());
        println!("Charges: {:?}", consumable.charges);
        println!("Cooldown: {}", consumable.cooldown);
    }
}

// Debug status effects
fn debug_status_effects(world: &World, entity: Entity) {
    let effects = world.read_storage::<StatusEffects>();
    if let Some(status_effects) = effects.get(entity) {
        for (effect_type, effect) in &status_effects.effects {
            println!("{:?}: power={}, duration={:.1}s", 
                effect_type, effect.power, effect.duration);
        }
    }
}
```

This consumable system provides a comprehensive foundation for item usage mechanics in the ASCII Dungeon Explorer game, supporting everything from simple healing potions to complex magical effects with sophisticated restriction and cooldown systems.