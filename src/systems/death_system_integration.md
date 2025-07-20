# Death System Integration Guide

## Overview

The death system handles entity death, animations, corpse creation, and cleanup in the ASCII Dungeon Explorer game.

## Components

### Death-Related Components

1. **Dead** - Marks entities that have died
   - `cause`: The cause of death (combat, environment, etc.)
   - `time_of_death`: When the entity died

2. **Corpse** - Represents corpses left behind
   - `original_entity`: Reference to the original entity
   - `decay_timer`: How long until the corpse disappears
   - `loot_generated`: Whether loot has been generated from this corpse

3. **DeathAnimation** - Handles death animations
   - `animation_type`: Type of death animation (fade, dissolve, explosion, collapse)
   - `duration`: How long the animation lasts
   - `elapsed`: How much time has passed
   - `original_glyph`: Original character representation
   - `original_color`: Original color

## Systems

### DeathSystem

The main death system that:
- Detects entities with HP <= 0
- Marks them as dead
- Starts death animations
- Drops inventory items
- Creates corpses after animation completes
- Handles corpse decay

### DeadEntityCleanupSystem

Cleans up dead entities after their death animations complete.

## Integration Steps

1. **Register Components** in your World:
```rust
world.register::<Dead>();
world.register::<Corpse>();
world.register::<DeathAnimation>();
```

2. **Add Systems** to your system runner:
```rust
// Add to your system dispatcher
.with(DeathSystem {}, "death_system", &[])
.with(DeadEntityCleanupSystem {}, "cleanup_system", &["death_system"])
```

3. **Handle Player Death** in your game state:
```rust
// Check if player is dead
let dead_storage = world.read_storage::<Dead>();
let player_entities = world.read_storage::<Player>();

for (entity, _player, _dead) in (&entities, &player_entities, &dead_storage).join() {
    // Handle game over, respawn, etc.
    game_state = GameState::GameOver;
}
```

## Death Animation Types

- **Fade**: Entity gradually fades to black
- **Dissolve**: Entity changes to dissolution characters (▓▒░)
- **Explosion**: Bright flash followed by explosion effects
- **Collapse**: Entity collapses into a flat representation

## Corpse System

- Corpses are created after death animations complete
- They have a decay timer (default: 100 turns)
- Corpses use the '%' character with dark red color
- They can potentially contain loot (not yet implemented)

## Death Triggers

The system can trigger various events on death:
- Player death (game over)
- Quest updates
- Achievement unlocks
- Environmental changes
- NPC reactions

## Future Enhancements

- Loot generation from corpses
- Different corpse types based on creature
- Resurrection mechanics
- Death-triggered environmental effects
- Sound effects for death animations