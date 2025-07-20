use bevy::prelude::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::ai::{AIComponent, AIBehaviorState, BehaviorPatternComponent, BehaviorPatternConfig};
use crate::components::{Position, Health};

/// Types of special enemies
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialEnemyType {
    Boss,
    Elite,
    Champion,
    Miniboss,
    Environmental,
    Summoner,
    Shapeshifter,
    Teleporter,
    Berserker,
    Necromancer,
}

/// Special attack patterns
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecialAttackPattern {
    /// Area of effect attacks
    AreaOfEffect {
        radius: f32,
        damage_falloff: bool,
        warning_time: f32,
    },
    /// Charge attacks that move the enemy
    Charge {
        range: f32,
        speed_multiplier: f32,
        knockback: f32,
    },
    /// Multi-hit combo attacks
    Combo {
        hit_count: u32,
        hit_interval: f32,
        damage_per_hit: f32,
    },
    /// Projectile attacks
    Projectile {
        projectile_count: u32,
        spread_angle: f32,
        projectile_speed: f32,
    },
    /// Summoning attacks
    Summon {
        summon_type: String,
        summon_count: u32,
        summon_duration: f32,
    },
    /// Environmental manipulation
    Environmental {
        effect_type: String,
        area_size: f32,
        duration: f32,
    },
    /// Teleport attacks
    Teleport {
        teleport_range: f32,
        attack_after_teleport: bool,
        teleport_count: u32,
    },
    /// Healing abilities
    Heal {
        heal_amount: f32,
        heal_range: f32,
        can_heal_others: bool,
    },
}

/// Unique movement patterns for special enemies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpecialMovementType {
    /// Standard ground movement
    Ground,
    /// Flying movement (ignores some obstacles)
    Flying,
    /// Teleportation movement
    Teleport {
        teleport_range: f32,
        teleport_cooldown: f32,
    },
    /// Phase through walls
    Phasing {
        phase_duration: f32,
        phase_cooldown: f32,
    },
    /// Burrowing underground
    Burrowing {
        burrow_speed: f32,
        surface_time: f32,
    },
    /// Web-based movement (spiders)
    Web {
        web_creation_range: f32,
        web_movement_speed: f32,
    },
    /// Climbing on walls and ceilings
    Climbing,
    /// Shapeshifting movement
    Shapeshifting {
        forms: Vec<String>,
        transform_time: f32,
    },
}

/// Environmental interaction types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnvironmentalInteraction {
    /// Destroy walls and obstacles
    Destruction {
        destruction_range: f32,
        destruction_power: f32,
    },
    /// Create obstacles or barriers
    Creation {
        creation_type: String,
        creation_range: f32,
        creation_duration: f32,
    },
    /// Manipulate lighting
    Lighting {
        light_radius: f32,
        light_intensity: f32,
        can_create_darkness: bool,
    },
    /// Trigger traps
    TrapTrigger {
        trigger_range: f32,
        trap_types: Vec<String>,
    },
    /// Manipulate terrain
    TerrainManipulation {
        manipulation_type: String,
        area_size: f32,
        duration: f32,
    },
    /// Summon environmental hazards
    HazardSummon {
        hazard_type: String,
        hazard_count: u32,
        hazard_duration: f32,
    },
}

/// Component for special enemy configuration
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct SpecialEnemyComponent {
    pub enemy_type: SpecialEnemyType,
    pub special_attacks: Vec<SpecialAttackPattern>,
    pub movement_type: SpecialMovementType,
    pub environmental_interactions: Vec<EnvironmentalInteraction>,
    pub phase_transitions: HashMap<String, f32>, // health_percentage -> phase_name
    pub current_phase: String,
    pub special_abilities_cooldown: HashMap<String, f32>,
    pub last_ability_use: HashMap<String, f32>,
    pub enrage_threshold: f32,
    pub is_enraged: bool,
    pub unique_mechanics: HashMap<String, f32>,
}

impl Default for SpecialEnemyComponent {
    fn default() -> Self {
        SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Elite,
            special_attacks: Vec::new(),
            movement_type: SpecialMovementType::Ground,
            environmental_interactions: Vec::new(),
            phase_transitions: HashMap::new(),
            current_phase: "normal".to_string(),
            special_abilities_cooldown: HashMap::new(),
            last_ability_use: HashMap::new(),
            enrage_threshold: 0.25,
            is_enraged: false,
            unique_mechanics: HashMap::new(),
        }
    }
}

impl SpecialEnemyComponent {
    /// Create a boss enemy
    pub fn boss(name: &str) -> Self {
        let mut special = SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Boss,
            enrage_threshold: 0.3,
            ..Default::default()
        };

        // Add phase transitions
        special.phase_transitions.insert("enrage".to_string(), 0.3);
        special.phase_transitions.insert("desperate".to_string(), 0.1);

        // Add boss-specific abilities
        special.special_attacks.push(SpecialAttackPattern::AreaOfEffect {
            radius: 5.0,
            damage_falloff: true,
            warning_time: 2.0,
        });

        special.special_attacks.push(SpecialAttackPattern::Summon {
            summon_type: "minion".to_string(),
            summon_count: 3,
            summon_duration: 30.0,
        });

        special.environmental_interactions.push(EnvironmentalInteraction::Destruction {
            destruction_range: 3.0,
            destruction_power: 1.0,
        });

        // Set cooldowns
        special.special_abilities_cooldown.insert("aoe_attack".to_string(), 8.0);
        special.special_abilities_cooldown.insert("summon_minions".to_string(), 15.0);
        special.special_abilities_cooldown.insert("environmental_destruction".to_string(), 12.0);

        special
    }

    /// Create a teleporter enemy
    pub fn teleporter() -> Self {
        let mut special = SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Teleporter,
            movement_type: SpecialMovementType::Teleport {
                teleport_range: 8.0,
                teleport_cooldown: 3.0,
            },
            ..Default::default()
        };

        special.special_attacks.push(SpecialAttackPattern::Teleport {
            teleport_range: 8.0,
            attack_after_teleport: true,
            teleport_count: 3,
        });

        special.special_abilities_cooldown.insert("teleport_attack".to_string(), 5.0);

        special
    }

    /// Create a summoner enemy
    pub fn summoner() -> Self {
        let mut special = SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Summoner,
            ..Default::default()
        };

        special.special_attacks.push(SpecialAttackPattern::Summon {
            summon_type: "skeleton".to_string(),
            summon_count: 2,
            summon_duration: 20.0,
        });

        special.special_attacks.push(SpecialAttackPattern::Heal {
            heal_amount: 25.0,
            heal_range: 6.0,
            can_heal_others: true,
        });

        special.special_abilities_cooldown.insert("summon_skeletons".to_string(), 10.0);
        special.special_abilities_cooldown.insert("heal_allies".to_string(), 8.0);

        special
    }

    /// Create a berserker enemy
    pub fn berserker() -> Self {
        let mut special = SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Berserker,
            enrage_threshold: 0.5,
            ..Default::default()
        };

        special.special_attacks.push(SpecialAttackPattern::Charge {
            range: 6.0,
            speed_multiplier: 2.0,
            knockback: 3.0,
        });

        special.special_attacks.push(SpecialAttackPattern::Combo {
            hit_count: 4,
            hit_interval: 0.3,
            damage_per_hit: 15.0,
        });

        special.special_abilities_cooldown.insert("charge_attack".to_string(), 6.0);
        special.special_abilities_cooldown.insert("berserker_combo".to_string(), 8.0);

        special
    }

    /// Create an environmental manipulator
    pub fn environmental_manipulator() -> Self {
        let mut special = SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Environmental,
            ..Default::default()
        };

        special.environmental_interactions.push(EnvironmentalInteraction::Creation {
            creation_type: "wall".to_string(),
            creation_range: 4.0,
            creation_duration: 15.0,
        });

        special.environmental_interactions.push(EnvironmentalInteraction::HazardSummon {
            hazard_type: "spike_trap".to_string(),
            hazard_count: 3,
            hazard_duration: 20.0,
        });

        special.special_abilities_cooldown.insert("create_walls".to_string(), 12.0);
        special.special_abilities_cooldown.insert("summon_traps".to_string(), 10.0);

        special
    }

    /// Create a necromancer enemy
    pub fn necromancer() -> Self {
        let mut special = SpecialEnemyComponent {
            enemy_type: SpecialEnemyType::Necromancer,
            ..Default::default()
        };

        special.special_attacks.push(SpecialAttackPattern::Summon {
            summon_type: "undead".to_string(),
            summon_count: 4,
            summon_duration: 25.0,
        });

        special.environmental_interactions.push(EnvironmentalInteraction::Lighting {
            light_radius: 8.0,
            light_intensity: -0.5, // Creates darkness
            can_create_darkness: true,
        });

        special.special_abilities_cooldown.insert("raise_undead".to_string(), 15.0);
        special.special_abilities_cooldown.insert("darkness_aura".to_string(), 20.0);

        special
    }

    /// Check if an ability is off cooldown
    pub fn is_ability_ready(&self, ability_name: &str, current_time: f32) -> bool {
        if let (Some(&cooldown), Some(&last_use)) = (
            self.special_abilities_cooldown.get(ability_name),
            self.last_ability_use.get(ability_name),
        ) {
            current_time - last_use >= cooldown
        } else {
            true // Ability not tracked or never used
        }
    }

    /// Use an ability (sets cooldown)
    pub fn use_ability(&mut self, ability_name: String, current_time: f32) {
        self.last_ability_use.insert(ability_name, current_time);
    }

    /// Check if should transition to a new phase
    pub fn check_phase_transition(&mut self, health_percentage: f32) -> Option<String> {
        for (phase_name, threshold) in &self.phase_transitions {
            if health_percentage <= *threshold && self.current_phase != *phase_name {
                let old_phase = self.current_phase.clone();
                self.current_phase = phase_name.clone();
                return Some(old_phase);
            }
        }
        None
    }

    /// Check if should enrage
    pub fn check_enrage(&mut self, health_percentage: f32) -> bool {
        if !self.is_enraged && health_percentage <= self.enrage_threshold {
            self.is_enraged = true;
            true
        } else {
            false
        }
    }
}

/// Component for tracking summoned entities
#[derive(Component, Debug, Clone)]
pub struct SummonedEntity {
    pub summoner: Entity,
    pub summon_type: String,
    pub duration_remaining: f32,
    pub max_duration: f32,
}

impl SummonedEntity {
    pub fn new(summoner: Entity, summon_type: String, duration: f32) -> Self {
        SummonedEntity {
            summoner,
            summon_type,
            duration_remaining: duration,
            max_duration: duration,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        self.duration_remaining -= delta_time;
        self.duration_remaining > 0.0
    }
}

/// Component for special attack warnings
#[derive(Component, Debug, Clone)]
pub struct AttackWarning {
    pub attack_type: String,
    pub warning_time_remaining: f32,
    pub affected_area: Vec<IVec2>,
    pub warning_intensity: f32,
}

impl AttackWarning {
    pub fn new(attack_type: String, warning_time: f32, area: Vec<IVec2>) -> Self {
        AttackWarning {
            attack_type,
            warning_time_remaining: warning_time,
            affected_area: area,
            warning_intensity: 1.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        self.warning_time_remaining -= delta_time;
        self.warning_intensity = (self.warning_time_remaining / 2.0).clamp(0.0, 1.0);
        self.warning_time_remaining > 0.0
    }
}

/// System for updating special enemy behaviors
pub fn special_enemy_system(
    time: Res<Time>,
    mut special_enemies: Query<(Entity, &mut SpecialEnemyComponent, &mut AIComponent, &Health, &Position)>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_seconds();
    let delta_time = time.delta_seconds();

    for (entity, mut special, mut ai, health, position) in special_enemies.iter_mut() {
        if !ai.enabled {
            continue;
        }

        let health_percentage = health.current as f32 / health.max as f32;

        // Check for phase transitions
        if let Some(old_phase) = special.check_phase_transition(health_percentage) {
            handle_phase_transition(&mut special, &mut ai, &old_phase, &special.current_phase.clone());
        }

        // Check for enrage
        if special.check_enrage(health_percentage) {
            handle_enrage(&mut special, &mut ai);
        }

        // Execute special behaviors based on enemy type and current state
        match special.enemy_type {
            SpecialEnemyType::Boss => {
                handle_boss_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
            SpecialEnemyType::Teleporter => {
                handle_teleporter_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
            SpecialEnemyType::Summoner => {
                handle_summoner_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
            SpecialEnemyType::Berserker => {
                handle_berserker_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
            SpecialEnemyType::Environmental => {
                handle_environmental_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
            SpecialEnemyType::Necromancer => {
                handle_necromancer_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
            _ => {
                // Handle other special enemy types
                handle_generic_special_behavior(entity, &mut special, &mut ai, position, current_time, &mut commands);
            }
        }
    }
}

/// Handle phase transitions
fn handle_phase_transition(
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    old_phase: &str,
    new_phase: &str,
) {
    info!("Special enemy transitioning from {} to {} phase", old_phase, new_phase);

    match new_phase {
        "enrage" => {
            ai.personality.aggression = (ai.personality.aggression * 1.5).min(1.0);
            ai.personality.courage = (ai.personality.courage * 1.3).min(1.0);
            
            // Reduce cooldowns in enrage phase
            for cooldown in special.special_abilities_cooldown.values_mut() {
                *cooldown *= 0.7;
            }
        }
        "desperate" => {
            ai.personality.aggression = 1.0;
            ai.personality.courage = 1.0;
            
            // Further reduce cooldowns in desperate phase
            for cooldown in special.special_abilities_cooldown.values_mut() {
                *cooldown *= 0.5;
            }
        }
        _ => {}
    }
}

/// Handle enrage state
fn handle_enrage(special: &mut SpecialEnemyComponent, ai: &mut AIComponent) {
    info!("Special enemy entering enrage state!");
    
    // Boost aggression and reduce fear
    ai.personality.aggression = (ai.personality.aggression * 1.8).min(1.0);
    ai.personality.courage = (ai.personality.courage * 1.5).min(1.0);
    
    // Reduce all cooldowns
    for cooldown in special.special_abilities_cooldown.values_mut() {
        *cooldown *= 0.6;
    }
}

/// Handle boss-specific behavior
fn handle_boss_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    match ai.current_state {
        AIBehaviorState::Attack => {
            // Use special attacks more frequently
            if special.is_ability_ready("aoe_attack", current_time) && ai.decision_factors.enemies_nearby >= 1 {
                execute_aoe_attack(entity, special, position, current_time, commands);
                special.use_ability("aoe_attack".to_string(), current_time);
            }
        }
        AIBehaviorState::Hunt => {
            // Summon minions when hunting
            if special.is_ability_ready("summon_minions", current_time) && ai.decision_factors.distance_to_target > 5.0 {
                execute_summon_attack(entity, special, position, current_time, commands);
                special.use_ability("summon_minions".to_string(), current_time);
            }
        }
        AIBehaviorState::Idle => {
            // Use environmental destruction when idle and enraged
            if special.is_enraged && special.is_ability_ready("environmental_destruction", current_time) {
                execute_environmental_destruction(entity, special, position, current_time, commands);
                special.use_ability("environmental_destruction".to_string(), current_time);
            }
        }
        _ => {}
    }
}

/// Handle teleporter-specific behavior
fn handle_teleporter_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    if ai.current_state == AIBehaviorState::Hunt || ai.current_state == AIBehaviorState::Attack {
        if special.is_ability_ready("teleport_attack", current_time) {
            let distance_to_target = ai.decision_factors.distance_to_target;
            
            // Teleport if target is too far or too close
            if distance_to_target > 6.0 || distance_to_target < 2.0 {
                execute_teleport_attack(entity, special, ai, position, current_time, commands);
                special.use_ability("teleport_attack".to_string(), current_time);
            }
        }
    }
}

/// Handle summoner-specific behavior
fn handle_summoner_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    match ai.current_state {
        AIBehaviorState::Hunt | AIBehaviorState::Attack => {
            // Summon allies when in combat
            if special.is_ability_ready("summon_skeletons", current_time) {
                execute_summon_attack(entity, special, position, current_time, commands);
                special.use_ability("summon_skeletons".to_string(), current_time);
            }
            
            // Heal allies if any are nearby and wounded
            if special.is_ability_ready("heal_allies", current_time) && ai.decision_factors.allies_nearby > 0 {
                execute_heal_ability(entity, special, position, current_time, commands);
                special.use_ability("heal_allies".to_string(), current_time);
            }
        }
        _ => {}
    }
}

/// Handle berserker-specific behavior
fn handle_berserker_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    if ai.current_state == AIBehaviorState::Hunt && ai.decision_factors.distance_to_target > 3.0 {
        // Use charge attack to close distance
        if special.is_ability_ready("charge_attack", current_time) {
            execute_charge_attack(entity, special, ai, position, current_time, commands);
            special.use_ability("charge_attack".to_string(), current_time);
        }
    } else if ai.current_state == AIBehaviorState::Attack {
        // Use combo attack in melee range
        if special.is_ability_ready("berserker_combo", current_time) {
            execute_combo_attack(entity, special, position, current_time, commands);
            special.use_ability("berserker_combo".to_string(), current_time);
        }
    }
}

/// Handle environmental manipulator behavior
fn handle_environmental_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    match ai.current_state {
        AIBehaviorState::Hunt => {
            // Create walls to block player escape routes
            if special.is_ability_ready("create_walls", current_time) && ai.decision_factors.distance_to_target < 8.0 {
                execute_wall_creation(entity, special, position, current_time, commands);
                special.use_ability("create_walls".to_string(), current_time);
            }
        }
        AIBehaviorState::Attack => {
            // Summon traps around the combat area
            if special.is_ability_ready("summon_traps", current_time) {
                execute_trap_summoning(entity, special, position, current_time, commands);
                special.use_ability("summon_traps".to_string(), current_time);
            }
        }
        _ => {}
    }
}

/// Handle necromancer-specific behavior
fn handle_necromancer_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    if ai.current_state == AIBehaviorState::Hunt || ai.current_state == AIBehaviorState::Attack {
        // Raise undead minions
        if special.is_ability_ready("raise_undead", current_time) {
            execute_summon_attack(entity, special, position, current_time, commands);
            special.use_ability("raise_undead".to_string(), current_time);
        }
        
        // Create darkness aura
        if special.is_ability_ready("darkness_aura", current_time) {
            execute_darkness_aura(entity, special, position, current_time, commands);
            special.use_ability("darkness_aura".to_string(), current_time);
        }
    }
}

/// Handle generic special enemy behavior
fn handle_generic_special_behavior(
    entity: Entity,
    special: &mut SpecialEnemyComponent,
    ai: &mut AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    // Generic special ability usage based on situation
    if ai.current_state == AIBehaviorState::Attack || ai.current_state == AIBehaviorState::Hunt {
        // Use first available special attack
        for attack in &special.special_attacks {
            match attack {
                SpecialAttackPattern::AreaOfEffect { .. } => {
                    if special.is_ability_ready("aoe", current_time) {
                        execute_aoe_attack(entity, special, position, current_time, commands);
                        special.use_ability("aoe".to_string(), current_time);
                        break;
                    }
                }
                SpecialAttackPattern::Charge { .. } => {
                    if special.is_ability_ready("charge", current_time) && ai.decision_factors.distance_to_target > 3.0 {
                        execute_charge_attack(entity, special, ai, position, current_time, commands);
                        special.use_ability("charge".to_string(), current_time);
                        break;
                    }
                }
                _ => {}
            }
        }
    }
}

// Special attack execution functions
fn execute_aoe_attack(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    if let Some(SpecialAttackPattern::AreaOfEffect { radius, warning_time, .. }) = special.special_attacks.first() {
        // Create warning area
        let mut affected_area = Vec::new();
        let center = position.0.as_ivec2();
        let radius_i = *radius as i32;
        
        for x in -radius_i..=radius_i {
            for y in -radius_i..=radius_i {
                let pos = center + IVec2::new(x, y);
                if (Vec2::new(x as f32, y as f32).length() <= *radius) {
                    affected_area.push(pos);
                }
            }
        }
        
        commands.spawn(AttackWarning::new(
            "aoe_attack".to_string(),
            *warning_time,
            affected_area,
        ));
        
        info!("Special enemy {} executing AoE attack", entity.index());
    }
}

fn execute_summon_attack(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    for attack in &special.special_attacks {
        if let SpecialAttackPattern::Summon { summon_type, summon_count, summon_duration } = attack {
            for i in 0..*summon_count {
                let angle = (i as f32 / *summon_count as f32) * std::f32::consts::TAU;
                let offset = Vec2::new(angle.cos(), angle.sin()) * 2.0;
                let summon_pos = position.0 + offset;
                
                // Create summoned entity (placeholder)
                let summoned = commands.spawn((
                    Position(summon_pos),
                    Health::new(30),
                    SummonedEntity::new(entity, summon_type.clone(), *summon_duration),
                )).id();
                
                info!("Summoned {} at {:?}", summon_type, summon_pos);
            }
            break;
        }
    }
}

fn execute_teleport_attack(
    entity: Entity,
    special: &SpecialEnemyComponent,
    ai: &AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    if let Some(target_pos) = ai.memory.last_known_target_position {
        // Teleport near the target
        let direction = (target_pos - position.0).normalize_or_zero();
        let teleport_pos = target_pos - direction * 2.0; // Teleport 2 units away from target
        
        commands.entity(entity).insert(Position(teleport_pos));
        info!("Special enemy {} teleported to {:?}", entity.index(), teleport_pos);
    }
}

fn execute_charge_attack(
    entity: Entity,
    special: &SpecialEnemyComponent,
    ai: &AIComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    if let Some(target_pos) = ai.memory.last_known_target_position {
        let direction = (target_pos - position.0).normalize_or_zero();
        let charge_distance = 4.0;
        let charge_target = position.0 + direction * charge_distance;
        
        // Create charge effect (placeholder)
        commands.spawn((
            Position(charge_target),
            AttackWarning::new("charge_attack".to_string(), 0.5, vec![charge_target.as_ivec2()]),
        ));
        
        info!("Special enemy {} executing charge attack to {:?}", entity.index(), charge_target);
    }
}

fn execute_combo_attack(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    // Create combo attack effect
    commands.spawn(AttackWarning::new(
        "combo_attack".to_string(),
        1.2, // Duration of combo
        vec![position.0.as_ivec2()],
    ));
    
    info!("Special enemy {} executing combo attack", entity.index());
}

fn execute_heal_ability(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    info!("Special enemy {} casting heal", entity.index());
    // Healing logic would be implemented here
}

fn execute_wall_creation(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    info!("Special enemy {} creating walls", entity.index());
    // Wall creation logic would be implemented here
}

fn execute_trap_summoning(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    info!("Special enemy {} summoning traps", entity.index());
    // Trap summoning logic would be implemented here
}

fn execute_darkness_aura(
    entity: Entity,
    special: &SpecialEnemyComponent,
    position: &Position,
    current_time: f32,
    commands: &mut Commands,
) {
    info!("Special enemy {} creating darkness aura", entity.index());
    // Darkness aura logic would be implemented here
}

/// System for managing summoned entities
pub fn summoned_entity_system(
    time: Res<Time>,
    mut summoned_query: Query<(Entity, &mut SummonedEntity)>,
    mut commands: Commands,
) {
    let delta_time = time.delta_seconds();

    for (entity, mut summoned) in summoned_query.iter_mut() {
        if !summoned.update(delta_time) {
            // Summon duration expired, despawn entity
            commands.entity(entity).despawn();
            info!("Summoned entity {} expired and was despawned", entity.index());
        }
    }
}

/// System for managing attack warnings
pub fn attack_warning_system(
    time: Res<Time>,
    mut warning_query: Query<(Entity, &mut AttackWarning)>,
    mut commands: Commands,
) {
    let delta_time = time.delta_seconds();

    for (entity, mut warning) in warning_query.iter_mut() {
        if !warning.update(delta_time) {
            // Warning expired, execute the attack
            execute_warned_attack(&warning);
            commands.entity(entity).despawn();
        }
    }
}

fn execute_warned_attack(warning: &AttackWarning) {
    info!("Executing {} attack on {} tiles", warning.attack_type, warning.affected_area.len());
    // Attack execution logic would be implemented here
}

/// Plugin for special enemies
pub struct SpecialEnemiesPlugin;

impl Plugin for SpecialEnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            special_enemy_system,
            summoned_entity_system,
            attack_warning_system,
        ).chain());
    }
}