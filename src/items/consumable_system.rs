use specs::{Component, VecStorage, System, WriteStorage, ReadStorage, Entities, Entity, Join, Write, ReadExpect};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::components::{CombatStats, Player, Name, Position};
use crate::items::{ItemProperties, ItemType, ConsumableType};
use crate::resources::{GameLog, RandomNumberGenerator};

/// Component for consumable items
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct Consumable {
    pub consumable_type: ConsumableType,
    pub effects: Vec<ConsumableEffect>,
    pub use_time: f32,           // Time to consume in seconds
    pub cooldown: f32,           // Cooldown after use in seconds
    pub charges: Option<i32>,    // Limited use items (None = unlimited)
    pub requirements: ConsumableRequirements,
    pub restrictions: Vec<ConsumableRestriction>,
}

impl Consumable {
    pub fn new(consumable_type: ConsumableType) -> Self {
        let (effects, use_time, cooldown) = match consumable_type {
            ConsumableType::Potion => (
                vec![ConsumableEffect::Healing { amount: 25, over_time: false }],
                1.0,
                0.0,
            ),
            ConsumableType::Food => (
                vec![
                    ConsumableEffect::Healing { amount: 10, over_time: true },
                    ConsumableEffect::StatusEffect {
                        effect_type: StatusEffectType::WellFed,
                        duration: 300.0, // 5 minutes
                        power: 1,
                    }
                ],
                3.0,
                0.0,
            ),
            ConsumableType::Scroll => (
                vec![ConsumableEffect::SpellCast { spell_id: "magic_missile".to_string() }],
                2.0,
                0.0,
            ),
            ConsumableType::Ammunition => (
                vec![], // Ammunition doesn't have direct effects
                0.1,
                0.0,
            ),
        };

        Consumable {
            consumable_type,
            effects,
            use_time,
            cooldown,
            charges: None,
            requirements: ConsumableRequirements::default(),
            restrictions: Vec::new(),
        }
    }

    pub fn with_effects(mut self, effects: Vec<ConsumableEffect>) -> Self {
        self.effects = effects;
        self
    }

    pub fn with_charges(mut self, charges: i32) -> Self {
        self.charges = Some(charges);
        self
    }

    pub fn with_cooldown(mut self, cooldown: f32) -> Self {
        self.cooldown = cooldown;
        self
    }

    pub fn with_use_time(mut self, use_time: f32) -> Self {
        self.use_time = use_time;
        self
    }

    pub fn add_restriction(mut self, restriction: ConsumableRestriction) -> Self {
        self.restrictions.push(restriction);
        self
    }

    pub fn can_use(&self, user_entity: Entity, world: &specs::World) -> Result<(), String> {
        // Check charges
        if let Some(charges) = self.charges {
            if charges <= 0 {
                return Err("Item has no charges remaining".to_string());
            }
        }

        // Check requirements
        if !self.requirements.check(user_entity, world) {
            return Err("Requirements not met".to_string());
        }

        // Check restrictions
        for restriction in &self.restrictions {
            if let Err(msg) = restriction.check(user_entity, world) {
                return Err(msg);
            }
        }

        Ok(())
    }

    pub fn use_charge(&mut self) -> bool {
        if let Some(charges) = &mut self.charges {
            if *charges > 0 {
                *charges -= 1;
                true
            } else {
                false
            }
        } else {
            true // Unlimited charges
        }
    }

    pub fn is_depleted(&self) -> bool {
        if let Some(charges) = self.charges {
            charges <= 0
        } else {
            false
        }
    }
}

/// Effects that consumables can have
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConsumableEffect {
    /// Restore health
    Healing {
        amount: i32,
        over_time: bool,
    },
    /// Restore mana/magic points
    ManaRestore {
        amount: i32,
        over_time: bool,
    },
    /// Restore stamina
    StaminaRestore {
        amount: i32,
        over_time: bool,
    },
    /// Apply a status effect
    StatusEffect {
        effect_type: StatusEffectType,
        duration: f32,
        power: i32,
    },
    /// Temporarily boost attributes
    AttributeBoost {
        attribute: String,
        amount: i32,
        duration: f32,
    },
    /// Cure conditions
    CureCondition {
        condition: StatusEffectType,
    },
    /// Cast a spell
    SpellCast {
        spell_id: String,
    },
    /// Teleport effect
    Teleport {
        range: i32,
        random: bool,
    },
    /// Reveal map area
    RevealMap {
        radius: i32,
    },
    /// Identify items
    Identify {
        count: i32,
    },
    /// Custom effect
    Custom {
        effect_id: String,
        parameters: HashMap<String, String>,
    },
}

/// Status effects that can be applied
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum StatusEffectType {
    // Beneficial effects
    Regeneration,
    ManaRegeneration,
    Haste,
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    Protection,
    Invisibility,
    WellFed,
    Blessed,
    
    // Harmful effects
    Poison,
    Disease,
    Curse,
    Weakness,
    Slow,
    Confusion,
    Fear,
    Paralysis,
    Sleep,
    Blind,
    Deaf,
    Mute,
    
    // Neutral effects
    Detect,
    Levitation,
    WaterWalking,
}

/// Requirements to use a consumable
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConsumableRequirements {
    pub min_level: i32,
    pub required_attributes: HashMap<String, i32>,
    pub required_skills: HashMap<String, i32>,
    pub required_class: Option<String>,
    pub required_items: Vec<String>,
}

impl ConsumableRequirements {
    pub fn check(&self, entity: Entity, world: &specs::World) -> bool {
        // TODO: Implement actual requirement checking
        // This would need to check player level, attributes, skills, etc.
        true
    }
}

/// Restrictions on consumable usage
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConsumableRestriction {
    /// Cannot use in combat
    NoCombat,
    /// Cannot use while moving
    NoMovement,
    /// Cannot use while certain status effects are active
    NoStatusEffect(StatusEffectType),
    /// Cannot use in certain locations
    NoLocation(String),
    /// Cannot use more than X times per day
    DailyLimit(i32),
    /// Cannot use while health is above X%
    HealthThreshold(f32),
    /// Cannot use while mana is above X%
    ManaThreshold(f32),
    /// Custom restriction
    Custom(String),
}

impl ConsumableRestriction {
    pub fn check(&self, entity: Entity, world: &specs::World) -> Result<(), String> {
        match self {
            ConsumableRestriction::NoCombat => {
                // TODO: Check if entity is in combat
                Ok(())
            },
            ConsumableRestriction::NoMovement => {
                // TODO: Check if entity is moving
                Ok(())
            },
            ConsumableRestriction::HealthThreshold(threshold) => {
                let combat_stats = world.read_storage::<CombatStats>();
                if let Some(stats) = combat_stats.get(entity) {
                    let health_percentage = stats.hp as f32 / stats.max_hp as f32;
                    if health_percentage > *threshold {
                        return Err(format!("Cannot use while health is above {}%", threshold * 100.0));
                    }
                }
                Ok(())
            },
            _ => Ok(()), // Other restrictions not implemented yet
        }
    }
}

/// Component for tracking consumable usage cooldowns
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct ConsumableCooldowns {
    pub cooldowns: HashMap<String, f32>, // consumable_type -> remaining cooldown
    pub global_cooldown: f32,            // Global cooldown for all consumables
}

impl ConsumableCooldowns {
    pub fn new() -> Self {
        ConsumableCooldowns {
            cooldowns: HashMap::new(),
            global_cooldown: 0.0,
        }
    }

    pub fn is_on_cooldown(&self, consumable_type: &str) -> bool {
        self.global_cooldown > 0.0 || 
        self.cooldowns.get(consumable_type).map_or(false, |&cd| cd > 0.0)
    }

    pub fn get_cooldown(&self, consumable_type: &str) -> f32 {
        self.global_cooldown.max(
            *self.cooldowns.get(consumable_type).unwrap_or(&0.0)
        )
    }

    pub fn set_cooldown(&mut self, consumable_type: String, cooldown: f32) {
        self.cooldowns.insert(consumable_type, cooldown);
    }

    pub fn set_global_cooldown(&mut self, cooldown: f32) {
        self.global_cooldown = cooldown;
    }

    pub fn update(&mut self, delta_time: f32) {
        // Update global cooldown
        if self.global_cooldown > 0.0 {
            self.global_cooldown = (self.global_cooldown - delta_time).max(0.0);
        }

        // Update individual cooldowns
        for cooldown in self.cooldowns.values_mut() {
            if *cooldown > 0.0 {
                *cooldown = (*cooldown - delta_time).max(0.0);
            }
        }

        // Remove expired cooldowns
        self.cooldowns.retain(|_, &mut cooldown| cooldown > 0.0);
    }
}

/// Component for active status effects
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
#[storage(VecStorage)]
pub struct StatusEffects {
    pub effects: HashMap<StatusEffectType, StatusEffect>,
}

impl StatusEffects {
    pub fn new() -> Self {
        StatusEffects {
            effects: HashMap::new(),
        }
    }

    pub fn add_effect(&mut self, effect_type: StatusEffectType, effect: StatusEffect) {
        // If effect already exists, either stack or replace based on type
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

    pub fn remove_effect(&mut self, effect_type: &StatusEffectType) {
        self.effects.remove(effect_type);
    }

    pub fn has_effect(&self, effect_type: &StatusEffectType) -> bool {
        self.effects.contains_key(effect_type)
    }

    pub fn get_effect(&self, effect_type: &StatusEffectType) -> Option<&StatusEffect> {
        self.effects.get(effect_type)
    }

    pub fn update(&mut self, delta_time: f32) -> Vec<StatusEffectType> {
        let mut expired = Vec::new();

        for (effect_type, effect) in &mut self.effects {
            effect.duration -= delta_time;
            if effect.duration <= 0.0 {
                expired.push(effect_type.clone());
            }
        }

        // Remove expired effects
        for effect_type in &expired {
            self.effects.remove(effect_type);
        }

        expired
    }
}

/// Individual status effect
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusEffect {
    pub power: i32,
    pub duration: f32,
    pub tick_interval: f32,
    pub last_tick: f32,
    pub source: Option<Entity>,
}

impl StatusEffect {
    pub fn new(power: i32, duration: f32) -> Self {
        StatusEffect {
            power,
            duration,
            tick_interval: 1.0, // Default to 1 second ticks
            last_tick: 0.0,
            source: None,
        }
    }

    pub fn with_tick_interval(mut self, interval: f32) -> Self {
        self.tick_interval = interval;
        self
    }

    pub fn with_source(mut self, source: Entity) -> Self {
        self.source = Some(source);
        self
    }

    pub fn should_tick(&mut self, delta_time: f32) -> bool {
        self.last_tick += delta_time;
        if self.last_tick >= self.tick_interval {
            self.last_tick = 0.0;
            true
        } else {
            false
        }
    }
}

/// Intent component for using consumables
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct WantsToUseConsumable {
    pub item: Entity,
    pub target: Option<Entity>,
}

/// System for handling consumable usage
pub struct ConsumableUsageSystem;

impl<'a> System<'a> for ConsumableUsageSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToUseConsumable>,
        WriteStorage<'a, Consumable>,
        WriteStorage<'a, ConsumableCooldowns>,
        WriteStorage<'a, StatusEffects>,
        WriteStorage<'a, CombatStats>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Player>,
        Write<'a, GameLog>,
        Write<'a, RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_to_use,
            mut consumables,
            mut cooldowns,
            mut status_effects,
            mut combat_stats,
            names,
            players,
            mut gamelog,
            mut rng,
        ) = data;

        let mut to_remove = Vec::new();

        for (entity, use_intent) in (&entities, &wants_to_use).join() {
            let item_entity = use_intent.item;
            let target_entity = use_intent.target.unwrap_or(entity);

            // Get consumable component
            if let Some(consumable) = consumables.get_mut(item_entity) {
                // Check if can use
                match consumable.can_use(entity, &entities.system_data()) {
                    Ok(()) => {
                        // Check cooldowns
                        let consumable_type = format!("{:?}", consumable.consumable_type);
                        let mut can_use = true;

                        if let Some(cd) = cooldowns.get(entity) {
                            if cd.is_on_cooldown(&consumable_type) {
                                let remaining = cd.get_cooldown(&consumable_type);
                                gamelog.entries.push(format!("Must wait {:.1} seconds before using another consumable", remaining));
                                can_use = false;
                            }
                        }

                        if can_use {
                            // Use the consumable
                            self.apply_consumable_effects(
                                &consumable.effects.clone(),
                                target_entity,
                                item_entity,
                                &mut status_effects,
                                &mut combat_stats,
                                &mut gamelog,
                                &mut rng,
                            );

                            // Set cooldown
                            if consumable.cooldown > 0.0 {
                                cooldowns.entry(entity)
                                    .or_insert_with(ConsumableCooldowns::new)
                                    .set_cooldown(consumable_type, consumable.cooldown);
                            }

                            // Use charge
                            if !consumable.use_charge() {
                                gamelog.entries.push("Item has no charges remaining".to_string());
                            }

                            // Log usage
                            let item_name = names.get(item_entity)
                                .map(|n| n.name.clone())
                                .unwrap_or("Unknown Item".to_string());

                            if players.get(entity).is_some() {
                                gamelog.entries.push(format!("You use the {}", item_name));
                            } else {
                                let user_name = names.get(entity)
                                    .map(|n| n.name.clone())
                                    .unwrap_or("Someone".to_string());
                                gamelog.entries.push(format!("{} uses {}", user_name, item_name));
                            }

                            // Remove item if depleted
                            if consumable.is_depleted() {
                                entities.delete(item_entity).expect("Failed to delete depleted consumable");
                            }
                        }
                    },
                    Err(msg) => {
                        gamelog.entries.push(msg);
                    }
                }
            }

            to_remove.push(entity);
        }

        // Clean up usage intents
        for entity in to_remove {
            wants_to_use.remove(entity);
        }
    }
}

impl ConsumableUsageSystem {
    fn apply_consumable_effects(
        &self,
        effects: &[ConsumableEffect],
        target: Entity,
        source: Entity,
        status_effects: &mut WriteStorage<StatusEffects>,
        combat_stats: &mut WriteStorage<CombatStats>,
        gamelog: &mut GameLog,
        rng: &mut RandomNumberGenerator,
    ) {
        for effect in effects {
            match effect {
                ConsumableEffect::Healing { amount, over_time } => {
                    if *over_time {
                        // Apply regeneration effect
                        let regen_effect = StatusEffect::new(*amount / 10, 10.0) // Heal over 10 seconds
                            .with_tick_interval(1.0)
                            .with_source(source);
                        
                        status_effects.entry(target)
                            .or_insert_with(StatusEffects::new)
                            .add_effect(StatusEffectType::Regeneration, regen_effect);
                        
                        gamelog.entries.push(format!("Regeneration effect applied"));
                    } else {
                        // Instant healing
                        if let Some(stats) = combat_stats.get_mut(target) {
                            let old_hp = stats.hp;
                            stats.hp = (stats.hp + amount).min(stats.max_hp);
                            let healed = stats.hp - old_hp;
                            
                            if healed > 0 {
                                gamelog.entries.push(format!("Restored {} health", healed));
                            } else {
                                gamelog.entries.push("Already at full health".to_string());
                            }
                        }
                    }
                },
                ConsumableEffect::StatusEffect { effect_type, duration, power } => {
                    let effect = StatusEffect::new(*power, *duration).with_source(source);
                    status_effects.entry(target)
                        .or_insert_with(StatusEffects::new)
                        .add_effect(effect_type.clone(), effect);
                    
                    gamelog.entries.push(format!("{:?} effect applied", effect_type));
                },
                ConsumableEffect::AttributeBoost { attribute, amount, duration } => {
                    // Convert attribute boost to status effect
                    let effect_type = match attribute.as_str() {
                        "Strength" => StatusEffectType::Strength,
                        "Dexterity" => StatusEffectType::Dexterity,
                        "Constitution" => StatusEffectType::Constitution,
                        "Intelligence" => StatusEffectType::Intelligence,
                        "Wisdom" => StatusEffectType::Wisdom,
                        "Charisma" => StatusEffectType::Charisma,
                        _ => StatusEffectType::Blessed, // Generic boost
                    };
                    
                    let effect = StatusEffect::new(*amount, *duration).with_source(source);
                    status_effects.entry(target)
                        .or_insert_with(StatusEffects::new)
                        .add_effect(effect_type, effect);
                    
                    gamelog.entries.push(format!("{} increased by {}", attribute, amount));
                },
                ConsumableEffect::CureCondition { condition } => {
                    if let Some(effects) = status_effects.get_mut(target) {
                        if effects.has_effect(condition) {
                            effects.remove_effect(condition);
                            gamelog.entries.push(format!("{:?} cured", condition));
                        } else {
                            gamelog.entries.push("No condition to cure".to_string());
                        }
                    }
                },
                _ => {
                    // TODO: Implement other effect types
                    gamelog.entries.push("Effect not yet implemented".to_string());
                }
            }
        }
    }
}

/// System for updating cooldowns and status effects
pub struct ConsumableUpdateSystem;

impl<'a> System<'a> for ConsumableUpdateSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, ConsumableCooldowns>,
        WriteStorage<'a, StatusEffects>,
        WriteStorage<'a, CombatStats>,
        Write<'a, GameLog>,
        ReadExpect<'a, f32>, // Delta time
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut cooldowns, mut status_effects, mut combat_stats, mut gamelog, delta_time) = data;

        // Update cooldowns
        for (_, cooldown) in (&entities, &mut cooldowns).join() {
            cooldown.update(*delta_time);
        }

        // Update status effects
        for (entity, effects) in (&entities, &mut status_effects).join() {
            let expired = effects.update(*delta_time);
            
            // Apply status effect ticks
            for (effect_type, effect) in &mut effects.effects {
                if effect.should_tick(*delta_time) {
                    self.apply_status_effect_tick(entity, effect_type, effect, &mut combat_stats, &mut gamelog);
                }
            }

            // Log expired effects
            for effect_type in expired {
                gamelog.entries.push(format!("{:?} effect has worn off", effect_type));
            }
        }
    }
}

impl ConsumableUpdateSystem {
    fn apply_status_effect_tick(
        &self,
        entity: Entity,
        effect_type: &StatusEffectType,
        effect: &StatusEffect,
        combat_stats: &mut WriteStorage<CombatStats>,
        gamelog: &mut GameLog,
    ) {
        match effect_type {
            StatusEffectType::Regeneration => {
                if let Some(stats) = combat_stats.get_mut(entity) {
                    let old_hp = stats.hp;
                    stats.hp = (stats.hp + effect.power).min(stats.max_hp);
                    let healed = stats.hp - old_hp;
                    
                    if healed > 0 {
                        gamelog.entries.push(format!("Regenerated {} health", healed));
                    }
                }
            },
            StatusEffectType::Poison => {
                if let Some(stats) = combat_stats.get_mut(entity) {
                    stats.hp = (stats.hp - effect.power).max(0);
                    gamelog.entries.push(format!("Poison deals {} damage", effect.power));
                }
            },
            _ => {
                // Other status effects don't tick
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};

    fn setup_world() -> World {
        let mut world = World::new();
        world.register::<Consumable>();
        world.register::<ConsumableCooldowns>();
        world.register::<StatusEffects>();
        world.register::<CombatStats>();
        world.register::<Name>();
        world.register::<Player>();
        world.register::<WantsToUseConsumable>();
        world
    }

    #[test]
    fn test_consumable_creation() {
        let potion = Consumable::new(ConsumableType::Potion);
        assert_eq!(potion.consumable_type, ConsumableType::Potion);
        assert!(!potion.effects.is_empty());
        assert_eq!(potion.use_time, 1.0);
    }

    #[test]
    fn test_consumable_charges() {
        let mut consumable = Consumable::new(ConsumableType::Scroll).with_charges(3);
        
        assert_eq!(consumable.charges, Some(3));
        assert!(!consumable.is_depleted());
        
        assert!(consumable.use_charge());
        assert_eq!(consumable.charges, Some(2));
        
        consumable.use_charge();
        consumable.use_charge();
        assert_eq!(consumable.charges, Some(0));
        assert!(consumable.is_depleted());
        
        assert!(!consumable.use_charge());
    }

    #[test]
    fn test_cooldown_system() {
        let mut cooldowns = ConsumableCooldowns::new();
        
        cooldowns.set_cooldown("potion".to_string(), 5.0);
        assert!(cooldowns.is_on_cooldown("potion"));
        assert_eq!(cooldowns.get_cooldown("potion"), 5.0);
        
        cooldowns.update(2.0);
        assert_eq!(cooldowns.get_cooldown("potion"), 3.0);
        
        cooldowns.update(4.0);
        assert!(!cooldowns.is_on_cooldown("potion"));
    }

    #[test]
    fn test_status_effects() {
        let mut effects = StatusEffects::new();
        let regen_effect = StatusEffect::new(5, 10.0);
        
        effects.add_effect(StatusEffectType::Regeneration, regen_effect);
        assert!(effects.has_effect(&StatusEffectType::Regeneration));
        
        let expired = effects.update(15.0); // More than duration
        assert!(expired.contains(&StatusEffectType::Regeneration));
        assert!(!effects.has_effect(&StatusEffectType::Regeneration));
    }

    #[test]
    fn test_consumable_restrictions() {
        let restriction = ConsumableRestriction::HealthThreshold(0.5);
        
        let mut world = setup_world();
        let entity = world.create_entity()
            .with(CombatStats { max_hp: 100, hp: 75, defense: 0, power: 0 })
            .build();
        
        // Should fail because health is above 50%
        assert!(restriction.check(entity, &world).is_err());
        
        // Update health to below threshold
        {
            let mut stats = world.write_storage::<CombatStats>();
            if let Some(stat) = stats.get_mut(entity) {
                stat.hp = 25;
            }
        }
        
        // Should pass now
        assert!(restriction.check(entity, &world).is_ok());
    }

    #[test]
    fn test_consumable_effects() {
        let healing_effect = ConsumableEffect::Healing { amount: 25, over_time: false };
        let status_effect = ConsumableEffect::StatusEffect {
            effect_type: StatusEffectType::Strength,
            duration: 60.0,
            power: 3,
        };
        
        // Test effect creation
        match healing_effect {
            ConsumableEffect::Healing { amount, over_time } => {
                assert_eq!(amount, 25);
                assert!(!over_time);
            },
            _ => panic!("Wrong effect type"),
        }
        
        match status_effect {
            ConsumableEffect::StatusEffect { effect_type, duration, power } => {
                assert_eq!(effect_type, StatusEffectType::Strength);
                assert_eq!(duration, 60.0);
                assert_eq!(power, 3);
            },
            _ => panic!("Wrong effect type"),
        }
    }
}