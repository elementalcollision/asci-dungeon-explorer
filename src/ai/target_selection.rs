use specs::{System, ReadStorage, WriteStorage, Entities, Entity, Join};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::ai::ai_components::{AI, AITargetSelector, AITargetType, TargetSelectionStrategy, AIMemory};
use crate::components::{Position, Health, Player, Name, Faction};

/// Target information for selection algorithms
#[derive(Debug, Clone)]
pub struct TargetInfo {
    pub entity: Entity,
    pub position: Position,
    pub distance: f32,
    pub health_percentage: f32,
    pub threat_level: f32,
    pub priority: f32,
    pub last_seen: Option<Instant>,
    pub target_type: AITargetType,
}

/// Target selection system that manages AI target acquisition
pub struct TargetSelectionSystem {
    last_update: Instant,
    update_frequency: Duration,
    target_cache: HashMap<Entity, Vec<TargetInfo>>, // AI entity -> potential targets
    cache_duration: Duration,
}

impl TargetSelectionSystem {
    pub fn new() -> Self {
        TargetSelectionSystem {
            last_update: Instant::now(),
            update_frequency: Duration::from_millis(250), // Update 4 times per second
            target_cache: HashMap::new(),
            cache_duration: Duration::from_millis(500),
        }
    }

    pub fn with_update_frequency(mut self, frequency: Duration) -> Self {
        self.update_frequency = frequency;
        self
    }

    pub fn with_cache_duration(mut self, duration: Duration) -> Self {
        self.cache_duration = duration;
        self
    }
}

impl<'a> System<'a> for TargetSelectionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, AI>,
        WriteStorage<'a, AITargetSelector>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, Health>,
        ReadStorage<'a, Player>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Faction>,
    );

    fn run(&mut self, (entities, mut ais, mut target_selectors, positions, healths, players, names, factions): Self::SystemData) {
        let now = Instant::now();
        if now.duration_since(self.last_update) < self.update_frequency {
            return;
        }

        let delta_time = now.duration_since(self.last_update);
        self.last_update = now;

        // Clear old cache entries
        self.target_cache.retain(|_, targets| {
            targets.iter().any(|target| {
                target.last_seen.map_or(true, |time| now.duration_since(time) < self.cache_duration)
            })
        });

        // Process each AI entity with a target selector
        for (ai_entity, ai, target_selector, ai_position) in (&entities, &mut ais, &mut target_selectors, &positions).join() {
            if !ai.enabled || !target_selector.can_update_target() {
                continue;
            }

            // Get potential targets
            let potential_targets = self.find_potential_targets(
                ai_entity,
                ai_position,
                target_selector,
                &entities,
                &positions,
                &healths,
                &players,
                &names,
                &factions,
                &ai.memory,
            );

            // Cache the targets
            self.target_cache.insert(ai_entity, potential_targets.clone());

            // Select the best target
            let selected_target = self.select_best_target(
                &potential_targets,
                &target_selector.selection_strategy,
                &ai.personality,
                &ai.memory,
            );

            // Update AI target
            if let Some(target_info) = selected_target {
                ai.current_target = Some(target_info.entity);
                
                // Update memory with target information
                ai.memory.remember_entity(target_info.entity, target_info.position);
                
                // If it's a player, remember player position
                if target_info.target_type == AITargetType::Player {
                    ai.memory.remember_player_position(target_info.position);
                }
                
                // Add as threat if threatening
                if target_info.threat_level > 0.5 {
                    ai.memory.add_threat(target_info.entity, target_info.threat_level);
                }
            } else if ai.current_target.is_some() {
                // Check if current target is still valid
                let current_target_valid = potential_targets.iter()
                    .any(|target| Some(target.entity) == ai.current_target);
                
                if !current_target_valid {
                    ai.current_target = None;
                }
            }

            target_selector.mark_target_updated();
        }
    }
}

impl TargetSelectionSystem {
    /// Find all potential targets for an AI entity
    fn find_potential_targets(
        &self,
        ai_entity: Entity,
        ai_position: &Position,
        target_selector: &AITargetSelector,
        entities: &Entities,
        positions: &ReadStorage<Position>,
        healths: &ReadStorage<Health>,
        players: &ReadStorage<Player>,
        names: &ReadStorage<Name>,
        factions: &ReadStorage<Faction>,
        memory: &AIMemory,
    ) -> Vec<TargetInfo> {
        let mut targets = Vec::new();
        let ai_faction = factions.get(ai_entity);

        for (entity, position) in (entities, positions).join() {
            if entity == ai_entity {
                continue; // Don't target self
            }

            let distance = self.calculate_distance(ai_position, position);
            if distance > target_selector.max_target_distance {
                continue; // Too far away
            }

            // Determine target type and check if it's a valid target
            let target_type = self.determine_target_type(
                entity,
                players,
                factions,
                ai_faction,
            );

            if !target_selector.target_types.contains(&target_type) {
                continue; // Not a target type we're interested in
            }

            // Calculate target information
            let health_percentage = if let Some(health) = healths.get(entity) {
                health.current as f32 / health.max as f32
            } else {
                1.0
            };

            let threat_level = self.calculate_threat_level(
                entity,
                &target_type,
                health_percentage,
                distance,
                memory,
            );

            let priority = self.calculate_target_priority(
                entity,
                &target_type,
                distance,
                health_percentage,
                threat_level,
                memory,
            );

            let last_seen = memory.seen_entities.get(&entity).map(|(_, time)| *time);

            targets.push(TargetInfo {
                entity,
                position: *position,
                distance,
                health_percentage,
                threat_level,
                priority,
                last_seen,
                target_type,
            });
        }

        targets
    }

    /// Determine the target type of an entity
    fn determine_target_type(
        &self,
        entity: Entity,
        players: &ReadStorage<Player>,
        factions: &ReadStorage<Faction>,
        ai_faction: Option<&Faction>,
    ) -> AITargetType {
        // Check if it's a player
        if players.get(entity).is_some() {
            return AITargetType::Player;
        }

        // Check faction relationships
        if let (Some(ai_faction), Some(target_faction)) = (ai_faction, factions.get(entity)) {
            match ai_faction.relationship_with(target_faction) {
                crate::components::FactionRelationship::Hostile => AITargetType::Enemy,
                crate::components::FactionRelationship::Friendly => AITargetType::Ally,
                crate::components::FactionRelationship::Neutral => AITargetType::Neutral,
            }
        } else {
            // Default to neutral if no faction information
            AITargetType::Neutral
        }
    }

    /// Calculate threat level of a target
    fn calculate_threat_level(
        &self,
        entity: Entity,
        target_type: &AITargetType,
        health_percentage: f32,
        distance: f32,
        memory: &AIMemory,
    ) -> f32 {
        let mut threat = 0.0;

        // Base threat by type
        match target_type {
            AITargetType::Player => threat += 0.8,
            AITargetType::Enemy => threat += 0.6,
            AITargetType::Neutral => threat += 0.2,
            AITargetType::Ally => threat += 0.0,
            _ => threat += 0.1,
        }

        // Health factor (healthier targets are more threatening)
        threat += health_percentage * 0.3;

        // Distance factor (closer targets are more threatening)
        let distance_factor = (20.0 - distance.min(20.0)) / 20.0;
        threat += distance_factor * 0.4;

        // Memory factor (recently seen threats are more threatening)
        if let Some((_, threat_level, time)) = memory.threats.iter()
            .find(|(e, _, _)| *e == entity) {
            let time_factor = 1.0 - (time.elapsed().as_secs() as f32 / 30.0).min(1.0);
            threat += threat_level * time_factor * 0.3;
        }

        threat.min(1.0)
    }

    /// Calculate target priority for selection
    fn calculate_target_priority(
        &self,
        entity: Entity,
        target_type: &AITargetType,
        distance: f32,
        health_percentage: f32,
        threat_level: f32,
        memory: &AIMemory,
    ) -> f32 {
        let mut priority = 0.0;

        // Base priority by type
        match target_type {
            AITargetType::Player => priority += 1.0,
            AITargetType::Enemy => priority += 0.8,
            AITargetType::Neutral => priority += 0.3,
            AITargetType::Ally => priority += 0.1,
            _ => priority += 0.2,
        }

        // Threat level increases priority
        priority += threat_level * 0.5;

        // Distance factor (closer targets have higher priority)
        let distance_factor = (10.0 - distance.min(10.0)) / 10.0;
        priority += distance_factor * 0.3;

        // Health factor (weaker targets might have higher priority for some strategies)
        priority += (1.0 - health_percentage) * 0.2;

        // Memory factor (recently seen targets have higher priority)
        if memory.has_seen_recently(entity, 10) {
            priority += 0.3;
        }

        priority
    }

    /// Select the best target based on strategy
    fn select_best_target(
        &self,
        targets: &[TargetInfo],
        strategy: &TargetSelectionStrategy,
        personality: &crate::ai::ai_components::AIPersonality,
        memory: &AIMemory,
    ) -> Option<TargetInfo> {
        if targets.is_empty() {
            return None;
        }

        match strategy {
            TargetSelectionStrategy::Nearest => {
                targets.iter()
                    .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
                    .cloned()
            },
            TargetSelectionStrategy::Weakest => {
                targets.iter()
                    .min_by(|a, b| a.health_percentage.partial_cmp(&b.health_percentage).unwrap())
                    .cloned()
            },
            TargetSelectionStrategy::Strongest => {
                targets.iter()
                    .max_by(|a, b| a.health_percentage.partial_cmp(&b.health_percentage).unwrap())
                    .cloned()
            },
            TargetSelectionStrategy::MostThreatening => {
                targets.iter()
                    .max_by(|a, b| a.threat_level.partial_cmp(&b.threat_level).unwrap())
                    .cloned()
            },
            TargetSelectionStrategy::HighestPriority => {
                targets.iter()
                    .max_by(|a, b| a.priority.partial_cmp(&b.priority).unwrap())
                    .cloned()
            },
            TargetSelectionStrategy::LastSeen => {
                // Find the most recently seen target
                targets.iter()
                    .filter(|target| target.last_seen.is_some())
                    .max_by(|a, b| {
                        a.last_seen.unwrap().cmp(&b.last_seen.unwrap())
                    })
                    .or_else(|| targets.first())
                    .cloned()
            },
            TargetSelectionStrategy::Random => {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..targets.len());
                Some(targets[index].clone())
            },
        }
    }

    /// Calculate distance between two positions
    fn calculate_distance(&self, pos1: &Position, pos2: &Position) -> f32 {
        let dx = (pos1.x - pos2.x) as f32;
        let dy = (pos1.y - pos2.y) as f32;
        (dx * dx + dy * dy).sqrt()
    }

    /// Get cached targets for an AI entity
    pub fn get_cached_targets(&self, ai_entity: Entity) -> Option<&Vec<TargetInfo>> {
        self.target_cache.get(&ai_entity)
    }

    /// Clear target cache for an entity
    pub fn clear_target_cache(&mut self, ai_entity: Entity) {
        self.target_cache.remove(&ai_entity);
    }

    /// Get target selection statistics
    pub fn get_target_statistics(&self) -> TargetSelectionStatistics {
        let total_ais = self.target_cache.len();
        let total_targets: usize = self.target_cache.values().map(|targets| targets.len()).sum();
        let average_targets = if total_ais > 0 {
            total_targets as f32 / total_ais as f32
        } else {
            0.0
        };

        let mut target_type_counts = HashMap::new();
        for targets in self.target_cache.values() {
            for target in targets {
                *target_type_counts.entry(target.target_type.clone()).or_insert(0) += 1;
            }
        }

        TargetSelectionStatistics {
            total_ais_with_targets: total_ais,
            total_potential_targets: total_targets,
            average_targets_per_ai: average_targets,
            target_type_distribution: target_type_counts,
        }
    }
}

/// Target selection statistics
#[derive(Debug, Clone)]
pub struct TargetSelectionStatistics {
    pub total_ais_with_targets: usize,
    pub total_potential_targets: usize,
    pub average_targets_per_ai: f32,
    pub target_type_distribution: HashMap<AITargetType, usize>,
}

/// Advanced target selection strategies
pub struct AdvancedTargetSelector {
    pub strategies: Vec<WeightedStrategy>,
    pub fallback_strategy: TargetSelectionStrategy,
}

#[derive(Debug, Clone)]
pub struct WeightedStrategy {
    pub strategy: TargetSelectionStrategy,
    pub weight: f32,
    pub condition: TargetSelectionCondition,
}

#[derive(Debug, Clone)]
pub enum TargetSelectionCondition {
    Always,
    HealthBelow(f32),
    HealthAbove(f32),
    EnemiesNearby(u32),
    PersonalityTrait(String, f32), // trait name, minimum value
    TimeOfDay(u32, u32), // start hour, end hour
    Custom(String), // custom condition identifier
}

impl AdvancedTargetSelector {
    pub fn new() -> Self {
        AdvancedTargetSelector {
            strategies: Vec::new(),
            fallback_strategy: TargetSelectionStrategy::Nearest,
        }
    }

    pub fn add_strategy(
        &mut self,
        strategy: TargetSelectionStrategy,
        weight: f32,
        condition: TargetSelectionCondition,
    ) {
        self.strategies.push(WeightedStrategy {
            strategy,
            weight,
            condition,
        });
    }

    pub fn select_strategy(
        &self,
        ai: &crate::ai::ai_components::AI,
        targets: &[TargetInfo],
    ) -> TargetSelectionStrategy {
        let mut applicable_strategies = Vec::new();

        for weighted_strategy in &self.strategies {
            if self.evaluate_condition(&weighted_strategy.condition, ai, targets) {
                applicable_strategies.push((weighted_strategy.strategy.clone(), weighted_strategy.weight));
            }
        }

        if applicable_strategies.is_empty() {
            return self.fallback_strategy.clone();
        }

        // Select strategy based on weights
        let total_weight: f32 = applicable_strategies.iter().map(|(_, weight)| weight).sum();
        if total_weight <= 0.0 {
            return self.fallback_strategy.clone();
        }

        use rand::Rng;
        let mut rng = rand::thread_rng();
        let mut random_value = rng.gen::<f32>() * total_weight;

        for (strategy, weight) in applicable_strategies {
            random_value -= weight;
            if random_value <= 0.0 {
                return strategy;
            }
        }

        self.fallback_strategy.clone()
    }

    fn evaluate_condition(
        &self,
        condition: &TargetSelectionCondition,
        ai: &crate::ai::ai_components::AI,
        _targets: &[TargetInfo],
    ) -> bool {
        match condition {
            TargetSelectionCondition::Always => true,
            TargetSelectionCondition::HealthBelow(threshold) => {
                ai.decision_factors.health_percentage < *threshold
            },
            TargetSelectionCondition::HealthAbove(threshold) => {
                ai.decision_factors.health_percentage > *threshold
            },
            TargetSelectionCondition::EnemiesNearby(count) => {
                ai.decision_factors.number_of_enemies >= *count
            },
            TargetSelectionCondition::PersonalityTrait(trait_name, min_value) => {
                match trait_name.as_str() {
                    "aggression" => ai.personality.aggression >= *min_value,
                    "courage" => ai.personality.courage >= *min_value,
                    "intelligence" => ai.personality.intelligence >= *min_value,
                    "alertness" => ai.personality.alertness >= *min_value,
                    "loyalty" => ai.personality.loyalty >= *min_value,
                    "curiosity" => ai.personality.curiosity >= *min_value,
                    _ => false,
                }
            },
            TargetSelectionCondition::TimeOfDay(_start, _end) => {
                // Would need game time system to implement
                true
            },
            TargetSelectionCondition::Custom(_) => {
                // Custom conditions would be implemented based on game needs
                false
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{World, WorldExt, Builder};
    use crate::ai::ai_components::{AI, AIPersonality};

    #[test]
    fn test_target_selection_system() {
        let system = TargetSelectionSystem::new();
        assert_eq!(system.update_frequency, Duration::from_millis(250));
        assert_eq!(system.cache_duration, Duration::from_millis(500));
    }

    #[test]
    fn test_target_info_creation() {
        let target_info = TargetInfo {
            entity: Entity::from_raw_index(0),
            position: Position { x: 5, y: 5, z: 0 },
            distance: 7.07,
            health_percentage: 0.8,
            threat_level: 0.6,
            priority: 0.9,
            last_seen: None,
            target_type: AITargetType::Player,
        };

        assert_eq!(target_info.target_type, AITargetType::Player);
        assert_eq!(target_info.health_percentage, 0.8);
    }

    #[test]
    fn test_distance_calculation() {
        let system = TargetSelectionSystem::new();
        let pos1 = Position { x: 0, y: 0, z: 0 };
        let pos2 = Position { x: 3, y: 4, z: 0 };
        
        let distance = system.calculate_distance(&pos1, &pos2);
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_threat_level_calculation() {
        let system = TargetSelectionSystem::new();
        let memory = crate::ai::ai_components::AIMemory::default();
        
        let threat = system.calculate_threat_level(
            Entity::from_raw_index(0),
            &AITargetType::Player,
            1.0,
            5.0,
            &memory,
        );
        
        assert!(threat > 0.0);
        assert!(threat <= 1.0);
    }

    #[test]
    fn test_target_priority_calculation() {
        let system = TargetSelectionSystem::new();
        let memory = crate::ai::ai_components::AIMemory::default();
        
        let priority = system.calculate_target_priority(
            Entity::from_raw_index(0),
            &AITargetType::Player,
            5.0,
            0.8,
            0.6,
            &memory,
        );
        
        assert!(priority > 0.0);
    }

    #[test]
    fn test_target_selection_strategies() {
        let targets = vec![
            TargetInfo {
                entity: Entity::from_raw_index(0),
                position: Position { x: 5, y: 5, z: 0 },
                distance: 7.07,
                health_percentage: 0.8,
                threat_level: 0.6,
                priority: 0.9,
                last_seen: None,
                target_type: AITargetType::Player,
            },
            TargetInfo {
                entity: Entity::from_raw_index(1),
                position: Position { x: 2, y: 2, z: 0 },
                distance: 2.83,
                health_percentage: 0.3,
                threat_level: 0.4,
                priority: 0.7,
                last_seen: None,
                target_type: AITargetType::Enemy,
            },
        ];

        let system = TargetSelectionSystem::new();
        let personality = AIPersonality::default();
        let memory = crate::ai::ai_components::AIMemory::default();

        // Test nearest strategy
        let nearest = system.select_best_target(&targets, &TargetSelectionStrategy::Nearest, &personality, &memory);
        assert_eq!(nearest.unwrap().entity, Entity::from_raw_index(1));

        // Test weakest strategy
        let weakest = system.select_best_target(&targets, &TargetSelectionStrategy::Weakest, &personality, &memory);
        assert_eq!(weakest.unwrap().entity, Entity::from_raw_index(1));

        // Test strongest strategy
        let strongest = system.select_best_target(&targets, &TargetSelectionStrategy::Strongest, &personality, &memory);
        assert_eq!(strongest.unwrap().entity, Entity::from_raw_index(0));
    }

    #[test]
    fn test_advanced_target_selector() {
        let mut selector = AdvancedTargetSelector::new();
        
        selector.add_strategy(
            TargetSelectionStrategy::Weakest,
            1.0,
            TargetSelectionCondition::HealthBelow(0.5),
        );
        
        selector.add_strategy(
            TargetSelectionStrategy::MostThreatening,
            0.8,
            TargetSelectionCondition::HealthAbove(0.5),
        );

        let ai = AI::default();
        let targets = vec![];
        
        let strategy = selector.select_strategy(&ai, &targets);
        assert_eq!(strategy, TargetSelectionStrategy::MostThreatening);
    }

    #[test]
    fn test_target_statistics() {
        let mut system = TargetSelectionSystem::new();
        
        // Add some mock data to cache
        system.target_cache.insert(Entity::from_raw_index(0), vec![
            TargetInfo {
                entity: Entity::from_raw_index(1),
                position: Position { x: 0, y: 0, z: 0 },
                distance: 5.0,
                health_percentage: 1.0,
                threat_level: 0.5,
                priority: 0.8,
                last_seen: None,
                target_type: AITargetType::Player,
            }
        ]);
        
        let stats = system.get_target_statistics();
        assert_eq!(stats.total_ais_with_targets, 1);
        assert_eq!(stats.total_potential_targets, 1);
        assert_eq!(stats.average_targets_per_ai, 1.0);
    }
}