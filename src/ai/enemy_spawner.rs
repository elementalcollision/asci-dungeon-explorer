use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use crate::ai::{AIComponent, BehaviorPatternComponent, BehaviorPatternConfig, BehaviorPattern};
use crate::components::{Position, Health, Name};
use crate::entity_factory::EntityType;

/// Enemy archetype definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyArchetype {
    pub name: String,
    pub entity_type: EntityType,
    pub health: i32,
    pub behavior_pattern: BehaviorPatternConfig,
    pub ai_personality: crate::ai::AIPersonality,
    pub spawn_weight: f32,
    pub min_level: u32,
    pub max_level: u32,
    pub group_spawn_chance: f32,
    pub group_size_range: (u32, u32),
}

impl EnemyArchetype {
    /// Create a goblin scout archetype
    pub fn goblin_scout() -> Self {
        EnemyArchetype {
            name: "Goblin Scout".to_string(),
            entity_type: EntityType::Enemy,
            health: 25,
            behavior_pattern: BehaviorPatternConfig::simple_patrol(6.0),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.4,
                courage: 0.3,
                intelligence: 0.5,
                curiosity: 0.7,
                loyalty: 0.4,
                alertness: 0.8,
            },
            spawn_weight: 1.0,
            min_level: 1,
            max_level: 3,
            group_spawn_chance: 0.3,
            group_size_range: (2, 4),
        }
    }

    /// Create a goblin warrior archetype
    pub fn goblin_warrior() -> Self {
        EnemyArchetype {
            name: "Goblin Warrior".to_string(),
            entity_type: EntityType::Enemy,
            health: 40,
            behavior_pattern: BehaviorPatternConfig::aggressive_hunter(),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.7,
                courage: 0.6,
                intelligence: 0.4,
                curiosity: 0.3,
                loyalty: 0.6,
                alertness: 0.6,
            },
            spawn_weight: 0.8,
            min_level: 2,
            max_level: 5,
            group_spawn_chance: 0.5,
            group_size_range: (2, 3),
        }
    }

    /// Create a goblin shaman archetype (support)
    pub fn goblin_shaman() -> Self {
        EnemyArchetype {
            name: "Goblin Shaman".to_string(),
            entity_type: EntityType::Enemy,
            health: 30,
            behavior_pattern: BehaviorPatternConfig::support("goblin_tribe".to_string()),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.3,
                courage: 0.4,
                intelligence: 0.8,
                curiosity: 0.6,
                loyalty: 0.9,
                alertness: 0.7,
            },
            spawn_weight: 0.3,
            min_level: 3,
            max_level: 6,
            group_spawn_chance: 0.8,
            group_size_range: (3, 5),
        }
    }

    /// Create an orc berserker archetype
    pub fn orc_berserker() -> Self {
        EnemyArchetype {
            name: "Orc Berserker".to_string(),
            entity_type: EntityType::Enemy,
            health: 80,
            behavior_pattern: BehaviorPatternConfig::berserker(),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.9,
                courage: 0.8,
                intelligence: 0.3,
                curiosity: 0.2,
                loyalty: 0.5,
                alertness: 0.5,
            },
            spawn_weight: 0.4,
            min_level: 4,
            max_level: 8,
            group_spawn_chance: 0.2,
            group_size_range: (1, 2),
        }
    }

    /// Create a skeleton guard archetype
    pub fn skeleton_guard() -> Self {
        EnemyArchetype {
            name: "Skeleton Guard".to_string(),
            entity_type: EntityType::Enemy,
            health: 35,
            behavior_pattern: BehaviorPatternConfig::defensive_guard(8.0),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.5,
                courage: 1.0, // Undead don't fear
                intelligence: 0.3,
                curiosity: 0.1,
                loyalty: 1.0, // Bound to duty
                alertness: 0.9,
            },
            spawn_weight: 0.6,
            min_level: 2,
            max_level: 6,
            group_spawn_chance: 0.4,
            group_size_range: (2, 4),
        }
    }

    /// Create a wolf pack hunter archetype
    pub fn wolf_pack_hunter() -> Self {
        EnemyArchetype {
            name: "Wolf".to_string(),
            entity_type: EntityType::Enemy,
            health: 45,
            behavior_pattern: BehaviorPatternConfig::pack_hunter("wolf_pack".to_string()),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.6,
                courage: 0.7,
                intelligence: 0.6,
                curiosity: 0.5,
                loyalty: 0.8,
                alertness: 0.9,
            },
            spawn_weight: 0.5,
            min_level: 3,
            max_level: 7,
            group_spawn_chance: 0.9,
            group_size_range: (3, 6),
        }
    }

    /// Create a spider ambush predator archetype
    pub fn giant_spider() -> Self {
        EnemyArchetype {
            name: "Giant Spider".to_string(),
            entity_type: EntityType::Enemy,
            health: 50,
            behavior_pattern: BehaviorPatternConfig::ambush_predator(),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.8,
                courage: 0.5,
                intelligence: 0.4,
                curiosity: 0.2,
                loyalty: 0.1,
                alertness: 0.9,
            },
            spawn_weight: 0.3,
            min_level: 4,
            max_level: 8,
            group_spawn_chance: 0.1,
            group_size_range: (1, 2),
        }
    }

    /// Create a kobold coward archetype
    pub fn kobold_coward() -> Self {
        EnemyArchetype {
            name: "Kobold".to_string(),
            entity_type: EntityType::Enemy,
            health: 20,
            behavior_pattern: BehaviorPatternConfig::coward(),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.2,
                courage: 0.1,
                intelligence: 0.6,
                curiosity: 0.8,
                loyalty: 0.3,
                alertness: 0.9,
            },
            spawn_weight: 0.7,
            min_level: 1,
            max_level: 4,
            group_spawn_chance: 0.6,
            group_size_range: (3, 8),
        }
    }

    /// Create a troll elite archetype
    pub fn troll_elite() -> Self {
        EnemyArchetype {
            name: "Troll Chieftain".to_string(),
            entity_type: EntityType::Enemy,
            health: 120,
            behavior_pattern: BehaviorPatternConfig::elite(),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.8,
                courage: 0.9,
                intelligence: 0.7,
                curiosity: 0.4,
                loyalty: 0.6,
                alertness: 0.8,
            },
            spawn_weight: 0.1,
            min_level: 6,
            max_level: 10,
            group_spawn_chance: 0.7,
            group_size_range: (2, 4),
        }
    }

    /// Create a rat wanderer archetype
    pub fn giant_rat() -> Self {
        EnemyArchetype {
            name: "Giant Rat".to_string(),
            entity_type: EntityType::Enemy,
            health: 15,
            behavior_pattern: BehaviorPatternConfig::random_wander(5.0),
            ai_personality: crate::ai::AIPersonality {
                aggression: 0.3,
                courage: 0.2,
                intelligence: 0.2,
                curiosity: 0.9,
                loyalty: 0.1,
                alertness: 0.8,
            },
            spawn_weight: 1.2,
            min_level: 1,
            max_level: 3,
            group_spawn_chance: 0.4,
            group_size_range: (2, 5),
        }
    }
}

/// Enemy spawn configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemySpawnConfig {
    pub archetypes: Vec<EnemyArchetype>,
    pub spawn_density: f32,
    pub level_scaling: f32,
    pub group_spawn_modifier: f32,
    pub elite_spawn_chance: f32,
}

impl Default for EnemySpawnConfig {
    fn default() -> Self {
        EnemySpawnConfig {
            archetypes: vec![
                EnemyArchetype::goblin_scout(),
                EnemyArchetype::goblin_warrior(),
                EnemyArchetype::goblin_shaman(),
                EnemyArchetype::orc_berserker(),
                EnemyArchetype::skeleton_guard(),
                EnemyArchetype::wolf_pack_hunter(),
                EnemyArchetype::giant_spider(),
                EnemyArchetype::kobold_coward(),
                EnemyArchetype::troll_elite(),
                EnemyArchetype::giant_rat(),
            ],
            spawn_density: 0.1,
            level_scaling: 1.2,
            group_spawn_modifier: 1.5,
            elite_spawn_chance: 0.05,
        }
    }
}

/// Component for spawn points
#[derive(Component, Debug, Clone)]
pub struct EnemySpawnPoint {
    pub spawn_config: EnemySpawnConfig,
    pub area_level: u32,
    pub max_enemies: u32,
    pub current_enemies: u32,
    pub spawn_cooldown: f32,
    pub last_spawn_time: f32,
    pub spawn_radius: f32,
}

impl EnemySpawnPoint {
    pub fn new(area_level: u32, max_enemies: u32) -> Self {
        EnemySpawnPoint {
            spawn_config: EnemySpawnConfig::default(),
            area_level,
            max_enemies,
            current_enemies: 0,
            spawn_cooldown: 30.0, // 30 seconds between spawns
            last_spawn_time: 0.0,
            spawn_radius: 10.0,
        }
    }

    /// Check if spawning is allowed
    pub fn can_spawn(&self, current_time: f32) -> bool {
        self.current_enemies < self.max_enemies &&
        current_time - self.last_spawn_time >= self.spawn_cooldown
    }

    /// Select an appropriate archetype for the current level
    pub fn select_archetype(&self) -> Option<&EnemyArchetype> {
        let valid_archetypes: Vec<&EnemyArchetype> = self.spawn_config.archetypes
            .iter()
            .filter(|archetype| {
                archetype.min_level <= self.area_level && 
                archetype.max_level >= self.area_level
            })
            .collect();

        if valid_archetypes.is_empty() {
            return None;
        }

        // Weighted selection
        let total_weight: f32 = valid_archetypes.iter().map(|a| a.spawn_weight).sum();
        let mut random_value = (self.last_spawn_time.sin().abs() * total_weight) % total_weight;

        for archetype in valid_archetypes {
            random_value -= archetype.spawn_weight;
            if random_value <= 0.0 {
                return Some(archetype);
            }
        }

        valid_archetypes.first().copied()
    }
}

/// Resource for managing enemy spawning
#[derive(Resource, Default)]
pub struct EnemySpawnResource {
    pub global_spawn_config: EnemySpawnConfig,
    pub spawn_enabled: bool,
    pub total_enemies_spawned: u32,
    pub max_global_enemies: u32,
}

impl EnemySpawnResource {
    pub fn new(max_global_enemies: u32) -> Self {
        EnemySpawnResource {
            global_spawn_config: EnemySpawnConfig::default(),
            spawn_enabled: true,
            total_enemies_spawned: 0,
            max_global_enemies,
        }
    }
}

/// System for spawning enemies
pub fn enemy_spawn_system(
    time: Res<Time>,
    mut spawn_resource: ResMut<EnemySpawnResource>,
    mut spawn_points: Query<(Entity, &mut EnemySpawnPoint, &Position)>,
    mut commands: Commands,
) {
    if !spawn_resource.spawn_enabled {
        return;
    }

    let current_time = time.elapsed_seconds();

    for (spawn_entity, mut spawn_point, spawn_position) in spawn_points.iter_mut() {
        if !spawn_point.can_spawn(current_time) {
            continue;
        }

        if let Some(archetype) = spawn_point.select_archetype() {
            // Determine if this should be a group spawn
            let is_group_spawn = (current_time + spawn_position.0.x).sin().abs() < archetype.group_spawn_chance;
            
            let spawn_count = if is_group_spawn {
                let (min, max) = archetype.group_size_range;
                let range = max - min;
                let random_factor = (current_time + spawn_position.0.y).cos().abs();
                min + (random_factor * range as f32) as u32
            } else {
                1
            };

            let group_id = if is_group_spawn {
                Some(format!("group_{}_{}", spawn_entity.index(), current_time as u32))
            } else {
                None
            };

            // Spawn enemies
            for i in 0..spawn_count {
                if spawn_resource.total_enemies_spawned >= spawn_resource.max_global_enemies {
                    break;
                }

                let spawn_offset = if spawn_count > 1 {
                    let angle = (i as f32 / spawn_count as f32) * std::f32::consts::TAU;
                    Vec2::new(angle.cos(), angle.sin()) * 2.0
                } else {
                    Vec2::ZERO
                };

                let enemy_position = spawn_position.0 + spawn_offset;
                spawn_enemy(&mut commands, archetype, enemy_position, group_id.clone());
                
                spawn_point.current_enemies += 1;
                spawn_resource.total_enemies_spawned += 1;
            }

            spawn_point.last_spawn_time = current_time;
        }
    }
}

/// Spawn a single enemy
fn spawn_enemy(
    commands: &mut Commands,
    archetype: &EnemyArchetype,
    position: Vec2,
    group_id: Option<String>,
) {
    let mut behavior_config = archetype.behavior_pattern.clone();
    
    // Set group ID if this is a group spawn
    if let Some(group_id) = group_id {
        behavior_config.group_id = Some(group_id);
    }

    let mut ai_component = AIComponent::new(archetype.ai_personality.clone());
    ai_component.set_home_position(position);

    let entity = commands.spawn((
        Name::new(archetype.name.clone()),
        Position(position),
        Health::new(archetype.health),
        ai_component,
        BehaviorPatternComponent::new(behavior_config),
        // Add other components as needed
    )).id();

    info!("Spawned {} at {:?}", archetype.name, position);
}

/// System for cleaning up dead enemies
pub fn enemy_cleanup_system(
    mut spawn_points: Query<&mut EnemySpawnPoint>,
    mut spawn_resource: ResMut<EnemySpawnResource>,
    dead_enemies: Query<Entity, (With<AIComponent>, With<Health>, Changed<Health>)>,
    health_query: Query<&Health>,
) {
    let mut enemies_died = 0;

    for entity in dead_enemies.iter() {
        if let Ok(health) = health_query.get(entity) {
            if health.current <= 0 {
                enemies_died += 1;
            }
        }
    }

    if enemies_died > 0 {
        // Update spawn point counters
        for mut spawn_point in spawn_points.iter_mut() {
            spawn_point.current_enemies = spawn_point.current_enemies.saturating_sub(enemies_died);
        }

        // Update global counter
        spawn_resource.total_enemies_spawned = spawn_resource.total_enemies_spawned.saturating_sub(enemies_died);
    }
}

/// System for dynamic difficulty adjustment
pub fn dynamic_difficulty_system(
    time: Res<Time>,
    mut spawn_resource: ResMut<EnemySpawnResource>,
    player_query: Query<&Health, With<crate::components::Player>>,
    enemy_query: Query<&Health, (With<AIComponent>, Without<crate::components::Player>)>,
) {
    // Adjust spawn rates based on player performance
    if let Ok(player_health) = player_query.get_single() {
        let player_health_ratio = player_health.current as f32 / player_health.max as f32;
        let enemy_count = enemy_query.iter().count();

        // Increase difficulty if player is doing well
        if player_health_ratio > 0.8 && enemy_count < 5 {
            spawn_resource.global_spawn_config.spawn_density *= 1.01;
        }
        // Decrease difficulty if player is struggling
        else if player_health_ratio < 0.3 && enemy_count > 2 {
            spawn_resource.global_spawn_config.spawn_density *= 0.99;
        }

        // Clamp spawn density
        spawn_resource.global_spawn_config.spawn_density = spawn_resource.global_spawn_config.spawn_density.clamp(0.05, 0.3);
    }
}

/// Plugin for enemy spawning and behavior patterns
pub struct EnemySpawnerPlugin;

impl Plugin for EnemySpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemySpawnResource>()
            .add_systems(Update, (
                enemy_spawn_system,
                enemy_cleanup_system,
                dynamic_difficulty_system,
            ).chain());
    }
}