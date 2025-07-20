#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::ai_component::*;

    #[test]
    fn test_ai_component_creation() {
        let ai = AIComponent::default();
        assert_eq!(ai.current_state, AIBehaviorState::Idle);
        assert!(ai.enabled);
        assert_eq!(ai.current_target, None);
    }

    #[test]
    fn test_ai_personality_presets() {
        let aggressive = AIComponent::aggressive();
        assert!(aggressive.personality.aggression > 0.7);
        assert!(aggressive.personality.courage > 0.6);

        let cowardly = AIComponent::cowardly();
        assert!(cowardly.personality.courage < 0.2);
        assert!(cowardly.personality.aggression < 0.3);

        let defensive = AIComponent::defensive();
        assert!(defensive.personality.loyalty > 0.7);
        assert!(defensive.personality.alertness > 0.7);
    }

    #[test]
    fn test_state_transitions() {
        let mut ai = AIComponent::default();
        
        // Test transition to hunt state
        ai.transition_to_state(AIBehaviorState::Hunt);
        assert_eq!(ai.current_state, AIBehaviorState::Hunt);
        assert_eq!(ai.previous_state, AIBehaviorState::Idle);
        assert_eq!(ai.state_timer, 0.0);
    }

    #[test]
    fn test_decision_factors_update() {
        let mut ai = AIComponent::default();
        
        let factors = AIDecisionFactors {
            health_percentage: 0.5,
            distance_to_target: 3.0,
            enemies_nearby: 2,
            ..Default::default()
        };
        
        ai.update_decision_factors(factors.clone());
        assert_eq!(ai.decision_factors.health_percentage, 0.5);
        assert_eq!(ai.decision_factors.distance_to_target, 3.0);
        assert_eq!(ai.decision_factors.enemies_nearby, 2);
    }

    #[test]
    fn test_patrol_system() {
        let mut ai = AIComponent::default();
        
        // Add patrol points
        ai.add_patrol_point(Vec2::new(0.0, 0.0));
        ai.add_patrol_point(Vec2::new(5.0, 0.0));
        ai.add_patrol_point(Vec2::new(5.0, 5.0));
        
        // Test patrol point cycling
        assert_eq!(ai.get_current_patrol_target(), Some(Vec2::new(0.0, 0.0)));
        
        ai.advance_patrol();
        assert_eq!(ai.get_current_patrol_target(), Some(Vec2::new(5.0, 0.0)));
        
        ai.advance_patrol();
        assert_eq!(ai.get_current_patrol_target(), Some(Vec2::new(5.0, 5.0)));
        
        ai.advance_patrol();
        assert_eq!(ai.get_current_patrol_target(), Some(Vec2::new(0.0, 0.0))); // Wraps around
    }

    #[test]
    fn test_transition_conditions() {
        let factors = AIDecisionFactors {
            health_percentage: 0.2,
            distance_to_target: 1.0,
            enemies_nearby: 3,
            ..Default::default()
        };

        // Test health condition
        let health_condition = AITransitionCondition::HealthBelow(0.3);
        assert!(health_condition.evaluate(&factors, 0.0));

        // Test distance condition
        let distance_condition = AITransitionCondition::TargetInRange(2.0);
        assert!(distance_condition.evaluate(&factors, 0.0));

        // Test enemy count condition
        let enemy_condition = AITransitionCondition::EnemiesNearby(2);
        assert!(enemy_condition.evaluate(&factors, 0.0));

        // Test compound condition
        let compound_condition = AITransitionCondition::And(
            Box::new(AITransitionCondition::HealthBelow(0.3)),
            Box::new(AITransitionCondition::EnemiesNearby(2))
        );
        assert!(compound_condition.evaluate(&factors, 0.0));
    }

    #[test]
    fn test_behavior_config_transitions() {
        let config = AIBehaviorConfig::default();
        
        let factors = AIDecisionFactors {
            health_percentage: 0.1, // Very low health
            distance_to_target: 5.0,
            ..Default::default()
        };

        // Should transition to flee when health is low
        let next_state = config.get_next_state(&AIBehaviorState::Hunt, &factors, 0.0);
        assert_eq!(next_state, Some(AIBehaviorState::Flee));
    }

    #[test]
    fn test_target_selector() {
        let selector = AITargetSelector::new(
            vec!["Player".to_string(), "Enemy".to_string()],
            10.0
        );

        // Test priority calculation
        let priority1 = selector.calculate_target_priority(2.0, 0.5, 0.8, 1.0);
        let priority2 = selector.calculate_target_priority(8.0, 0.5, 0.8, 1.0);
        
        // Closer target should have higher priority
        assert!(priority1 > priority2);
    }

    #[test]
    fn test_decision_tree() {
        let decision_tree = DecisionNode::Condition {
            test: DecisionTest::HealthBelow(0.3),
            true_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Flee),
            }),
            false_branch: Box::new(DecisionNode::Action {
                action: AIAction::SetState(AIBehaviorState::Hunt),
            }),
        };

        let mut ai = AIComponent::default();
        ai.decision_factors.health_percentage = 0.2;

        let action = decision_tree.evaluate(&ai, &std::collections::HashMap::new());
        assert_eq!(action, AIAction::SetState(AIBehaviorState::Flee));

        ai.decision_factors.health_percentage = 0.8;
        let action = decision_tree.evaluate(&ai, &std::collections::HashMap::new());
        assert_eq!(action, AIAction::SetState(AIBehaviorState::Hunt));
    }

    #[test]
    fn test_ai_personality_behavior() {
        let mut aggressive_ai = AIComponent::aggressive();
        aggressive_ai.decision_factors.health_percentage = 0.6;
        aggressive_ai.decision_factors.enemies_nearby = 1;
        assert!(aggressive_ai.should_be_aggressive());

        let mut cowardly_ai = AIComponent::cowardly();
        cowardly_ai.decision_factors.health_percentage = 0.3;
        cowardly_ai.decision_factors.enemies_nearby = 2;
        assert!(cowardly_ai.should_flee());
    }

    #[test]
    fn test_ai_memory() {
        let mut ai = AIComponent::default();
        let enemy_entity = bevy::prelude::Entity::from_raw(1);
        let position = Vec2::new(5.0, 5.0);

        // Test enemy memory
        ai.remember_enemy(enemy_entity, position, 1.0);
        assert!(ai.memory.known_enemies.contains_key(&enemy_entity));
        assert_eq!(ai.memory.known_enemies[&enemy_entity].0, position);

        // Test home position
        ai.set_home_position(Vec2::new(10.0, 10.0));
        assert_eq!(ai.memory.home_position, Vec2::new(10.0, 10.0));
    }
} 
   #[test]
    fn test_pathfinding_integration() {
        use crate::ai::pathfinding::*;
        
        let mut pathfinder = AStarPathfinder::new();
        
        // Test cache functionality
        let (total_entries, valid_entries) = pathfinder.get_cache_stats();
        assert_eq!(total_entries, 0);
        assert_eq!(valid_entries, 0);
        
        // Test cache clearing
        pathfinder.clear_cache();
        let (total_entries, _) = pathfinder.get_cache_stats();
        assert_eq!(total_entries, 0);
    }

    #[test]
    fn test_path_follower_integration() {
        use crate::ai::pathfinding::*;
        
        let mut follower = PathFollower::new(2.0);
        let waypoints = vec![
            IVec2::new(0, 0),
            IVec2::new(5, 0),
            IVec2::new(5, 5),
        ];
        let path = Path::new(waypoints, 30, 0.0);
        
        follower.set_path(path);
        assert!(follower.has_path());
        assert_eq!(follower.current_target(), Some(IVec2::new(0, 0)));
        
        // Simulate movement updates
        let target = follower.update(IVec2::new(0, 0), 0.1);
        assert!(target.is_some());
    }

    #[test]
    fn test_pathfinding_request() {
        use crate::ai::pathfinding::*;
        
        let request = PathfindingRequest::new(Vec2::new(10.0, 10.0));
        assert_eq!(request.target, Vec2::new(10.0, 10.0));
    }

    #[test]
    fn test_path_invalidation() {
        use crate::ai::pathfinding::*;
        
        let waypoints = vec![
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(2, 0),
        ];
        let mut path = Path::new(waypoints, 20, 0.0);
        
        assert!(path.is_valid);
        path.invalidate();
        assert!(!path.is_valid);
    }

    #[test]
    fn test_path_staleness() {
        use crate::ai::pathfinding::*;
        
        let waypoints = vec![IVec2::new(0, 0), IVec2::new(1, 0)];
        let path = Path::new(waypoints, 10, 0.0);
        
        assert!(!path.is_stale(5.0, 10.0)); // Not stale
        assert!(path.is_stale(15.0, 10.0));  // Stale
    } 
   #[test]
    fn test_behavior_patterns() {
        use crate::ai::behavior_patterns::*;
        
        // Test simple patrol pattern
        let patrol_config = BehaviorPatternConfig::simple_patrol(5.0);
        assert_eq!(patrol_config.pattern, BehaviorPattern::SimplePatrol);
        assert_eq!(patrol_config.get_parameter("patrol_radius", 0.0), 5.0);
        
        // Test aggressive hunter pattern
        let hunter_config = BehaviorPatternConfig::aggressive_hunter();
        assert_eq!(hunter_config.pattern, BehaviorPattern::AggressiveHunter);
        assert!(hunter_config.aggression_modifier > 1.0);
        
        // Test pack hunter pattern
        let pack_config = BehaviorPatternConfig::pack_hunter("test_pack".to_string());
        assert_eq!(pack_config.pattern, BehaviorPattern::PackHunter);
        assert_eq!(pack_config.group_id, Some("test_pack".to_string()));
    }

    #[test]
    fn test_behavior_pattern_component() {
        use crate::ai::behavior_patterns::*;
        
        let config = BehaviorPatternConfig::simple_patrol(3.0);
        let mut pattern = BehaviorPatternComponent::new(config);
        
        // Test state management
        pattern.set_state("test_value".to_string(), 42.0);
        assert_eq!(pattern.get_state("test_value", 0.0), 42.0);
        assert_eq!(pattern.get_state("nonexistent", 10.0), 10.0);
        
        // Test timer updates
        pattern.update_timer(1.0);
        assert_eq!(pattern.pattern_timer, 1.0);
    }

    #[test]
    fn test_group_coordination() {
        use crate::ai::behavior_patterns::*;
        
        let mut group = GroupCoordination::new("test_group".to_string());
        let entity1 = bevy::prelude::Entity::from_raw(1);
        let entity2 = bevy::prelude::Entity::from_raw(2);
        
        // Test member management
        group.add_member(entity1);
        group.add_member(entity2);
        assert_eq!(group.size(), 2);
        assert!(group.is_leader(entity1)); // First member becomes leader
        
        // Test member removal
        group.remove_member(entity1);
        assert_eq!(group.size(), 1);
        assert!(group.is_leader(entity2)); // Leadership transfers
    }

    #[test]
    fn test_enemy_archetypes() {
        use crate::ai::enemy_spawner::*;
        
        let goblin = EnemyArchetype::goblin_scout();
        assert_eq!(goblin.name, "Goblin Scout");
        assert!(goblin.health > 0);
        assert!(goblin.spawn_weight > 0.0);
        
        let troll = EnemyArchetype::troll_elite();
        assert_eq!(troll.name, "Troll Chieftain");
        assert!(troll.health > goblin.health); // Elite should have more health
        assert!(troll.min_level > goblin.min_level); // Elite should be higher level
    }

    #[test]
    fn test_enemy_spawn_point() {
        use crate::ai::enemy_spawner::*;
        
        let mut spawn_point = EnemySpawnPoint::new(3, 5);
        assert_eq!(spawn_point.area_level, 3);
        assert_eq!(spawn_point.max_enemies, 5);
        
        // Test spawn conditions
        assert!(spawn_point.can_spawn(100.0)); // Should be able to spawn initially
        
        spawn_point.current_enemies = spawn_point.max_enemies;
        assert!(!spawn_point.can_spawn(100.0)); // Should not spawn when at max
        
        // Test archetype selection
        let archetype = spawn_point.select_archetype();
        assert!(archetype.is_some());
        
        if let Some(arch) = archetype {
            assert!(arch.min_level <= spawn_point.area_level);
            assert!(arch.max_level >= spawn_point.area_level);
        }
    }

    #[test]
    fn test_spawn_config() {
        use crate::ai::enemy_spawner::*;
        
        let config = EnemySpawnConfig::default();
        assert!(!config.archetypes.is_empty());
        assert!(config.spawn_density > 0.0);
        assert!(config.level_scaling > 1.0);
    } 
   #[test]
    fn test_perception_system() {
        use crate::ai::perception_system::*;
        
        // Test Viewshed
        let mut viewshed = Viewshed::new(5);
        assert_eq!(viewshed.range, 5);
        assert!(viewshed.dirty);
        
        viewshed.mark_dirty();
        assert!(viewshed.dirty);
        
        // Test visibility
        viewshed.visible_tiles.insert(IVec2::new(1, 1));
        assert!(viewshed.can_see(IVec2::new(1, 1)));
        assert!(!viewshed.can_see(IVec2::new(5, 5)));
        
        assert_eq!(viewshed.visible_count(), 1);
    }

    #[test]
    fn test_perception_memory() {
        use crate::ai::perception_system::*;
        
        let entity = bevy::prelude::Entity::from_raw(42);
        let mut memory = PerceptionMemory::new(
            entity,
            IVec2::new(10, 10),
            0.0,
            "Enemy".to_string(),
            0.8,
            50,
            true,
        );
        
        assert_eq!(memory.entity, entity);
        assert_eq!(memory.last_known_position, IVec2::new(10, 10));
        assert_eq!(memory.threat_level, 0.8);
        assert!(memory.was_hostile);
        assert_eq!(memory.confidence, 1.0);
        
        // Test memory update
        memory.update(IVec2::new(12, 12), 5.0, 40, false);
        assert_eq!(memory.last_known_position, IVec2::new(12, 12));
        assert_eq!(memory.last_seen_time, 5.0);
        assert_eq!(memory.health_when_last_seen, 40);
        assert!(!memory.was_hostile);
        
        // Test confidence decay
        memory.decay_confidence(10.0, 0.1);
        assert!(memory.confidence < 1.0);
        
        // Test reliability
        assert!(memory.is_reliable(10.0, 30.0));
        assert!(!memory.is_reliable(50.0, 30.0));
    }

    #[test]
    fn test_perception_component() {
        use crate::ai::perception_system::*;
        
        let mut perception = PerceptionComponent::new(8.0, 12.0);
        assert_eq!(perception.detection_range, 8.0);
        assert_eq!(perception.hearing_range, 12.0);
        
        // Test entity perception
        let entity = bevy::prelude::Entity::from_raw(1);
        perception.perceive_entity(
            entity,
            IVec2::new(5, 5),
            0.0,
            "Player".to_string(),
            0.9,
            100,
            true,
        );
        
        assert!(perception.get_memory(entity).is_some());
        let memory = perception.get_memory(entity).unwrap();
        assert_eq!(memory.entity_type, "Player");
        assert_eq!(memory.threat_level, 0.9);
        
        // Test reliable memories
        let reliable = perception.get_reliable_memories(5.0);
        assert_eq!(reliable.len(), 1);
        
        // Test hostile memories
        let hostile = perception.get_hostile_memories(5.0);
        assert_eq!(hostile.len(), 1);
        
        // Test detection probability
        let prob = perception.calculate_detection_probability(3.0, 0.5, 0.8, 0.2);
        assert!(prob > 0.0 && prob <= 1.0);
        
        // Test noise hearing
        assert!(perception.can_hear_noise(10.0, 0.5));
        assert!(!perception.can_hear_noise(20.0, 0.1));
    }

    #[test]
    fn test_noise_emitter() {
        use crate::ai::perception_system::*;
        
        let mut noise = NoiseEmitter::new(0.2);
        assert_eq!(noise.base_noise_level, 0.2);
        assert_eq!(noise.current_noise_level, 0.2);
        
        // Test movement noise
        noise.add_movement_noise(3.0);
        assert!(noise.current_noise_level > 0.2);
        
        // Test action noise
        let before_action = noise.current_noise_level;
        noise.add_action_noise(2.0);
        assert!(noise.current_noise_level > before_action);
        
        // Test noise decay
        let before_decay = noise.current_noise_level;
        noise.decay_noise(1.0);
        assert!(noise.current_noise_level < before_decay);
        assert!(noise.current_noise_level >= noise.base_noise_level);
        
        // Test noise at distance
        let close_noise = noise.get_noise_at_distance(1.0);
        let far_noise = noise.get_noise_at_distance(10.0);
        assert!(close_noise > far_noise);
    }

    #[test]
    fn test_stealth_component() {
        use crate::ai::perception_system::*;
        
        let mut stealth = StealthComponent::new(0.6);
        assert_eq!(stealth.base_stealth, 0.6);
        assert_eq!(stealth.stealth_level, 0.6);
        
        // Test effective stealth with no penalties
        let effective = stealth.get_effective_stealth(false, false, 0.0);
        assert_eq!(effective, 0.6);
        
        // Test effective stealth with movement penalty
        let moving = stealth.get_effective_stealth(true, false, 0.0);
        assert!(moving < 0.6);
        
        // Test effective stealth with action penalty
        let acting = stealth.get_effective_stealth(false, true, 0.0);
        assert!(acting < 0.6);
        
        // Test effective stealth with light penalty
        let lit = stealth.get_effective_stealth(false, false, 1.0);
        assert!(lit < 0.6);
        
        // Test stealth update
        stealth.update_stealth(0.2);
        assert_eq!(stealth.stealth_level, 0.8);
        
        // Test clamping
        stealth.update_stealth(0.5);
        assert_eq!(stealth.stealth_level, 1.0);
    }

    #[test]
    fn test_field_of_view() {
        use crate::ai::perception_system::*;
        use crate::map::{DungeonMap, TileType};
        
        // Create a simple test map
        let mut map = DungeonMap::new(10, 10);
        
        // Fill with floors
        for x in 0..10 {
            for y in 0..10 {
                map.set_tile(x, y, TileType::Floor);
            }
        }
        
        // Add some walls
        map.set_tile(3, 3, TileType::Wall);
        map.set_tile(4, 3, TileType::Wall);
        
        // Calculate FOV from center
        let center = IVec2::new(5, 5);
        let visible = FieldOfView::calculate_fov(center, 3, &map);
        
        // Center should always be visible
        assert!(visible.contains(&center));
        
        // Should be able to see some adjacent tiles
        assert!(visible.contains(&IVec2::new(4, 5)));
        assert!(visible.contains(&IVec2::new(6, 5)));
        
        // Should not see beyond walls (depending on implementation)
        // This is a basic test - more complex scenarios would need detailed testing
        assert!(visible.len() > 1);
    } 
   #[test]
    fn test_special_enemies() {
        use crate::ai::special_enemies::*;
        
        // Test boss creation
        let boss = SpecialEnemyComponent::boss("Dragon Lord");
        assert_eq!(boss.enemy_type, SpecialEnemyType::Boss);
        assert!(boss.special_attacks.len() > 0);
        assert!(boss.environmental_interactions.len() > 0);
        assert_eq!(boss.enrage_threshold, 0.3);
        
        // Test teleporter creation
        let teleporter = SpecialEnemyComponent::teleporter();
        assert_eq!(teleporter.enemy_type, SpecialEnemyType::Teleporter);
        assert!(matches!(teleporter.movement_type, SpecialMovementType::Teleport { .. }));
        
        // Test summoner creation
        let summoner = SpecialEnemyComponent::summoner();
        assert_eq!(summoner.enemy_type, SpecialEnemyType::Summoner);
        assert!(summoner.special_attacks.iter().any(|attack| matches!(attack, SpecialAttackPattern::Summon { .. })));
        
        // Test berserker creation
        let berserker = SpecialEnemyComponent::berserker();
        assert_eq!(berserker.enemy_type, SpecialEnemyType::Berserker);
        assert_eq!(berserker.enrage_threshold, 0.5);
    }

    #[test]
    fn test_special_attack_patterns() {
        use crate::ai::special_enemies::*;
        
        // Test AoE attack
        let aoe = SpecialAttackPattern::AreaOfEffect {
            radius: 5.0,
            damage_falloff: true,
            warning_time: 2.0,
        };
        
        match aoe {
            SpecialAttackPattern::AreaOfEffect { radius, damage_falloff, warning_time } => {
                assert_eq!(radius, 5.0);
                assert!(damage_falloff);
                assert_eq!(warning_time, 2.0);
            }
            _ => panic!("Wrong attack pattern type"),
        }
        
        // Test charge attack
        let charge = SpecialAttackPattern::Charge {
            range: 6.0,
            speed_multiplier: 2.0,
            knockback: 3.0,
        };
        
        match charge {
            SpecialAttackPattern::Charge { range, speed_multiplier, knockback } => {
                assert_eq!(range, 6.0);
                assert_eq!(speed_multiplier, 2.0);
                assert_eq!(knockback, 3.0);
            }
            _ => panic!("Wrong attack pattern type"),
        }
    }

    #[test]
    fn test_special_movement_types() {
        use crate::ai::special_enemies::*;
        
        // Test teleport movement
        let teleport = SpecialMovementType::Teleport {
            teleport_range: 8.0,
            teleport_cooldown: 3.0,
        };
        
        match teleport {
            SpecialMovementType::Teleport { teleport_range, teleport_cooldown } => {
                assert_eq!(teleport_range, 8.0);
                assert_eq!(teleport_cooldown, 3.0);
            }
            _ => panic!("Wrong movement type"),
        }
        
        // Test phasing movement
        let phasing = SpecialMovementType::Phasing {
            phase_duration: 2.0,
            phase_cooldown: 10.0,
        };
        
        match phasing {
            SpecialMovementType::Phasing { phase_duration, phase_cooldown } => {
                assert_eq!(phase_duration, 2.0);
                assert_eq!(phase_cooldown, 10.0);
            }
            _ => panic!("Wrong movement type"),
        }
    }

    #[test]
    fn test_environmental_interactions() {
        use crate::ai::special_enemies::*;
        
        // Test destruction interaction
        let destruction = EnvironmentalInteraction::Destruction {
            destruction_range: 3.0,
            destruction_power: 1.0,
        };
        
        match destruction {
            EnvironmentalInteraction::Destruction { destruction_range, destruction_power } => {
                assert_eq!(destruction_range, 3.0);
                assert_eq!(destruction_power, 1.0);
            }
            _ => panic!("Wrong interaction type"),
        }
        
        // Test creation interaction
        let creation = EnvironmentalInteraction::Creation {
            creation_type: "wall".to_string(),
            creation_range: 4.0,
            creation_duration: 15.0,
        };
        
        match creation {
            EnvironmentalInteraction::Creation { creation_type, creation_range, creation_duration } => {
                assert_eq!(creation_type, "wall");
                assert_eq!(creation_range, 4.0);
                assert_eq!(creation_duration, 15.0);
            }
            _ => panic!("Wrong interaction type"),
        }
    }

    #[test]
    fn test_special_enemy_abilities() {
        use crate::ai::special_enemies::*;
        
        let mut special = SpecialEnemyComponent::default();
        
        // Test ability cooldown tracking
        special.special_abilities_cooldown.insert("test_ability".to_string(), 5.0);
        
        // Should be ready initially
        assert!(special.is_ability_ready("test_ability", 0.0));
        
        // Use the ability
        special.use_ability("test_ability".to_string(), 0.0);
        
        // Should not be ready immediately after use
        assert!(!special.is_ability_ready("test_ability", 2.0));
        
        // Should be ready after cooldown
        assert!(special.is_ability_ready("test_ability", 6.0));
    }

    #[test]
    fn test_phase_transitions() {
        use crate::ai::special_enemies::*;
        
        let mut special = SpecialEnemyComponent::default();
        special.phase_transitions.insert("enrage".to_string(), 0.3);
        special.phase_transitions.insert("desperate".to_string(), 0.1);
        
        // Should not transition at high health
        assert!(special.check_phase_transition(0.8).is_none());
        assert_eq!(special.current_phase, "normal");
        
        // Should transition to enrage at 30% health
        let old_phase = special.check_phase_transition(0.25);
        assert!(old_phase.is_some());
        assert_eq!(special.current_phase, "enrage");
        
        // Should transition to desperate at 10% health
        let old_phase = special.check_phase_transition(0.05);
        assert!(old_phase.is_some());
        assert_eq!(special.current_phase, "desperate");
    }

    #[test]
    fn test_enrage_system() {
        use crate::ai::special_enemies::*;
        
        let mut special = SpecialEnemyComponent::default();
        special.enrage_threshold = 0.25;
        
        // Should not enrage at high health
        assert!(!special.check_enrage(0.5));
        assert!(!special.is_enraged);
        
        // Should enrage at low health
        assert!(special.check_enrage(0.2));
        assert!(special.is_enraged);
        
        // Should not enrage again
        assert!(!special.check_enrage(0.1));
        assert!(special.is_enraged);
    }

    #[test]
    fn test_summoned_entity() {
        use crate::ai::special_enemies::*;
        
        let summoner = bevy::prelude::Entity::from_raw(1);
        let mut summoned = SummonedEntity::new(summoner, "skeleton".to_string(), 10.0);
        
        assert_eq!(summoned.summoner, summoner);
        assert_eq!(summoned.summon_type, "skeleton");
        assert_eq!(summoned.duration_remaining, 10.0);
        assert_eq!(summoned.max_duration, 10.0);
        
        // Test duration update
        assert!(summoned.update(5.0));
        assert_eq!(summoned.duration_remaining, 5.0);
        
        // Test expiration
        assert!(!summoned.update(10.0));
        assert!(summoned.duration_remaining <= 0.0);
    }

    #[test]
    fn test_attack_warning() {
        use crate::ai::special_enemies::*;
        
        let area = vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(0, 1)];
        let mut warning = AttackWarning::new("fireball".to_string(), 3.0, area.clone());
        
        assert_eq!(warning.attack_type, "fireball");
        assert_eq!(warning.warning_time_remaining, 3.0);
        assert_eq!(warning.affected_area, area);
        assert_eq!(warning.warning_intensity, 1.0);
        
        // Test warning update
        assert!(warning.update(1.0));
        assert_eq!(warning.warning_time_remaining, 2.0);
        assert!(warning.warning_intensity < 1.0);
        
        // Test warning expiration
        assert!(!warning.update(5.0));
        assert!(warning.warning_time_remaining <= 0.0);
    } 
   #[test]
    fn test_state_indicators() {
        use crate::ai::state_indicators::*;
        
        // Test default state indicator component
        let mut indicators = StateIndicatorComponent::default();
        assert!(indicators.indicators.len() > 0);
        assert_eq!(indicators.visibility_range, 15.0);
        assert!(!indicators.always_visible);
        
        // Test data management
        indicators.update_data("test_key".to_string(), 42.0);
        assert_eq!(indicators.get_data("test_key"), 42.0);
        assert_eq!(indicators.get_data("missing_key"), 0.0);
        
        // Test visibility calculations
        assert!(indicators.should_show_at_distance(10.0));
        assert!(!indicators.should_show_at_distance(20.0));
        
        // Test fade alpha calculation
        let close_alpha = indicators.calculate_fade_alpha(5.0);
        let far_alpha = indicators.calculate_fade_alpha(14.0);
        let too_far_alpha = indicators.calculate_fade_alpha(20.0);
        
        assert_eq!(close_alpha, 1.0);
        assert!(far_alpha < 1.0 && far_alpha > 0.0);
        assert_eq!(too_far_alpha, 0.0);
    }

    #[test]
    fn test_specialized_indicators() {
        use crate::ai::state_indicators::*;
        
        // Test boss indicators
        let boss_indicators = StateIndicatorComponent::boss_indicators();
        assert!(boss_indicators.always_visible);
        assert_eq!(boss_indicators.visibility_range, 25.0);
        assert!(boss_indicators.indicators.len() >= 4);
        
        // Test stealth indicators
        let stealth_indicators = StateIndicatorComponent::stealth_indicators();
        assert!(!stealth_indicators.always_visible);
        assert_eq!(stealth_indicators.visibility_range, 12.0);
        
        // Test spellcaster indicators
        let spell_indicators = StateIndicatorComponent::spellcaster_indicators();
        assert!(spell_indicators.indicators.iter().any(|i| matches!(i, StateIndicatorType::AbilityCharging { .. })));
    }

    #[test]
    fn test_indicator_types() {
        use crate::ai::state_indicators::*;
        
        // Test health bar indicator
        let health_bar = StateIndicatorType::HealthBar {
            show_percentage: true,
            color_coding: true,
            show_when_full: false,
        };
        
        match health_bar {
            StateIndicatorType::HealthBar { show_percentage, color_coding, show_when_full } => {
                assert!(show_percentage);
                assert!(color_coding);
                assert!(!show_when_full);
            }
            _ => panic!("Wrong indicator type"),
        }
        
        // Test behavior state indicator
        let behavior_state = StateIndicatorType::BehaviorState {
            show_icon: true,
            show_text: false,
            color_coded: true,
        };
        
        match behavior_state {
            StateIndicatorType::BehaviorState { show_icon, show_text, color_coded } => {
                assert!(show_icon);
                assert!(!show_text);
                assert!(color_coded);
            }
            _ => panic!("Wrong indicator type"),
        }
    }

    #[test]
    fn test_color_schemes() {
        use crate::ai::state_indicators::*;
        
        // Test default color scheme
        let default_scheme = IndicatorColorScheme::default();
        assert_eq!(default_scheme.primary, Color::WHITE);
        assert_eq!(default_scheme.danger, Color::RED);
        
        // Test health color scheme
        let health_scheme = IndicatorColorScheme::health_scheme();
        assert_eq!(health_scheme.primary, Color::GREEN);
        assert_eq!(health_scheme.danger, Color::RED);
        
        // Test aggression color scheme
        let aggression_scheme = IndicatorColorScheme::aggression_scheme();
        assert_eq!(aggression_scheme.primary, Color::BLUE);
        assert_eq!(aggression_scheme.danger, Color::RED);
        
        // Test alert color scheme
        let alert_scheme = IndicatorColorScheme::alert_scheme();
        assert_eq!(alert_scheme.primary, Color::GREEN);
        assert_eq!(alert_scheme.danger, Color::RED);
    }

    #[test]
    fn test_attack_warning_visual() {
        use crate::ai::state_indicators::*;
        
        let affected_tiles = vec![IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(0, 1)];
        let mut warning = AttackWarningVisual::new("fireball".to_string(), affected_tiles.clone());
        
        assert_eq!(warning.warning_type, "fireball");
        assert_eq!(warning.affected_tiles, affected_tiles);
        assert_eq!(warning.intensity, 1.0);
        assert!(warning.show_countdown);
        
        // Test different attack types get different colors
        let aoe_warning = AttackWarningVisual::new("aoe_attack".to_string(), vec![]);
        assert_eq!(aoe_warning.color, Color::RED);
        
        let charge_warning = AttackWarningVisual::new("charge_attack".to_string(), vec![]);
        assert_eq!(charge_warning.color, Color::ORANGE);
        
        let teleport_warning = AttackWarningVisual::new("teleport_attack".to_string(), vec![]);
        assert_eq!(teleport_warning.color, Color::PURPLE);
        
        // Test update
        warning.update(0.1, 2.5);
        assert!(!warning.countdown_text.is_empty());
        assert_eq!(warning.countdown_text, "2.5");
    }

    #[test]
    fn test_behavior_feedback_visual() {
        use crate::ai::state_indicators::*;
        
        let mut feedback = BehaviorFeedbackVisual::new(AIBehaviorState::Idle);
        assert_eq!(feedback.current_behavior, AIBehaviorState::Idle);
        assert_eq!(feedback.behavior_icon, "💤");
        assert_eq!(feedback.behavior_color, Color::GRAY);
        assert_eq!(feedback.behavior_description, "Idle");
        
        // Test behavior change
        feedback.update_behavior(AIBehaviorState::Attack);
        assert_eq!(feedback.current_behavior, AIBehaviorState::Attack);
        assert_eq!(feedback.behavior_icon, "⚔");
        assert_eq!(feedback.behavior_color, Color::RED);
        assert_eq!(feedback.behavior_description, "Attacking");
        assert!(feedback.transition_animation > 0.0);
        
        // Test no change when same behavior
        let old_animation = feedback.transition_animation;
        feedback.update_behavior(AIBehaviorState::Attack);
        assert_eq!(feedback.transition_animation, old_animation);
        
        // Test animation decay
        feedback.update(0.5);
        assert!(feedback.transition_animation < old_animation);
    }

    #[test]
    fn test_indicator_rendering_utilities() {
        use crate::ai::state_indicators::indicator_rendering::*;
        
        let scheme = IndicatorColorScheme::health_scheme();
        
        // Test health bar color calculation
        let full_health = get_health_bar_color(1.0, &scheme);
        assert_eq!(full_health, scheme.primary);
        
        let medium_health = get_health_bar_color(0.5, &scheme);
        assert_eq!(medium_health, scheme.secondary);
        
        let low_health = get_health_bar_color(0.1, &scheme);
        assert_eq!(low_health, scheme.danger);
        
        // Test aggression color calculation
        let high_aggression = get_aggression_color(0.9, &scheme);
        assert_eq!(high_aggression, scheme.danger);
        
        let medium_aggression = get_aggression_color(0.6, &scheme);
        assert_eq!(medium_aggression, scheme.warning);
        
        let low_aggression = get_aggression_color(0.1, &scheme);
        assert_eq!(low_aggression, scheme.neutral);
        
        // Test alert color calculation
        let high_alert = get_alert_color(0.9, &scheme);
        assert_eq!(high_alert, scheme.danger);
        
        let medium_alert = get_alert_color(0.5, &scheme);
        assert_eq!(medium_alert, scheme.secondary);
        
        let low_alert = get_alert_color(0.2, &scheme);
        assert_eq!(low_alert, scheme.primary);
        
        // Test behavior color mapping
        assert_eq!(get_behavior_color(&AIBehaviorState::Idle), Color::GRAY);
        assert_eq!(get_behavior_color(&AIBehaviorState::Attack), Color::RED);
        assert_eq!(get_behavior_color(&AIBehaviorState::Hunt), Color::ORANGE);
        assert_eq!(get_behavior_color(&AIBehaviorState::Flee), Color::YELLOW);
        
        // Test warning intensity calculation
        let early_intensity = calculate_warning_intensity(0.5, 2.0);
        assert_eq!(early_intensity, 0.5);
        
        let late_intensity = calculate_warning_intensity(1.5, 2.0);
        assert_eq!(late_intensity, 1.0);
        
        let full_intensity = calculate_warning_intensity(2.0, 2.0);
        assert_eq!(full_intensity, 1.0);
    }  
  #[test]
    fn test_reaction_system() {
        use crate::ai::reaction_system::*;
        
        // Test reaction trigger creation and properties
        let trigger = ReactionTrigger::PlayerAction {
            action_type: "spell_cast".to_string(),
            position: Vec2::new(10.0, 5.0),
            intensity: 0.9,
            timestamp: 15.0,
        };
        
        assert_eq!(trigger.get_position(), Vec2::new(10.0, 5.0));
        assert_eq!(trigger.get_timestamp(), 15.0);
        assert_eq!(trigger.get_intensity(), 0.9);
        
        // Test different trigger types
        let combat_trigger = ReactionTrigger::CombatEvent {
            event_type: "attack".to_string(),
            attacker: None,
            target: None,
            damage: 75.0,
            position: Vec2::new(0.0, 0.0),
            timestamp: 20.0,
        };
        
        assert_eq!(combat_trigger.get_intensity(), 0.75); // damage normalized to 0-1
        
        let sound_trigger = ReactionTrigger::SoundEvent {
            sound_type: "explosion".to_string(),
            position: Vec2::new(5.0, 5.0),
            volume: 1.0,
            timestamp: 25.0,
        };
        
        assert_eq!(sound_trigger.get_intensity(), 1.0);
    }

    #[test]
    fn test_reaction_conditions() {
        use crate::ai::reaction_system::*;
        
        let trigger = ReactionTrigger::CombatEvent {
            event_type: "damage_taken".to_string(),
            attacker: None,
            target: None,
            damage: 60.0,
            position: Vec2::new(3.0, 4.0),
            timestamp: 0.0,
        };
        
        let mut ai = AIComponent::default();
        ai.decision_factors.health_percentage = 0.3;
        ai.current_state = AIBehaviorState::Patrol;
        
        // Test trigger type condition
        let type_condition = ReactionCondition::TriggerType("damage_taken".to_string());
        assert!(type_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        let wrong_type_condition = ReactionCondition::TriggerType("healing".to_string());
        assert!(!wrong_type_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        // Test distance condition
        let close_distance_condition = ReactionCondition::DistanceWithin(10.0);
        assert!(close_distance_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        let far_distance_condition = ReactionCondition::DistanceWithin(2.0);
        assert!(!far_distance_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        // Test intensity condition
        let low_intensity_condition = ReactionCondition::IntensityAbove(0.5);
        assert!(low_intensity_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        let high_intensity_condition = ReactionCondition::IntensityAbove(0.8);
        assert!(!high_intensity_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        // Test state condition
        let state_condition = ReactionCondition::CurrentState(AIBehaviorState::Patrol);
        assert!(state_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        let wrong_state_condition = ReactionCondition::CurrentState(AIBehaviorState::Attack);
        assert!(!wrong_state_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        // Test health condition
        let health_condition = ReactionCondition::HealthBelow(0.5);
        assert!(health_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
        
        let high_health_condition = ReactionCondition::HealthBelow(0.2);
        assert!(!high_health_condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0));
    }

    #[test]
    fn test_reaction_responses() {
        use crate::ai::reaction_system::*;
        
        // Test behavior change response
        let behavior_response = ReactionResponse::BehaviorChange {
            new_state: AIBehaviorState::Hunt,
            duration: Some(10.0),
            priority: 5,
        };
        
        match behavior_response {
            ReactionResponse::BehaviorChange { new_state, duration, priority } => {
                assert_eq!(new_state, AIBehaviorState::Hunt);
                assert_eq!(duration, Some(10.0));
                assert_eq!(priority, 5);
            }
            _ => panic!("Wrong response type"),
        }
        
        // Test move to response
        let move_response = ReactionResponse::MoveTo {
            target_position: Vec2::new(15.0, 20.0),
            urgency: 0.8,
            investigate_time: 12.0,
        };
        
        match move_response {
            ReactionResponse::MoveTo { target_position, urgency, investigate_time } => {
                assert_eq!(target_position, Vec2::new(15.0, 20.0));
                assert_eq!(urgency, 0.8);
                assert_eq!(investigate_time, 12.0);
            }
            _ => panic!("Wrong response type"),
        }
        
        // Test flee response
        let flee_response = ReactionResponse::FleeArea {
            flee_distance: 25.0,
            flee_duration: 8.0,
            panic_level: 0.9,
        };
        
        match flee_response {
            ReactionResponse::FleeArea { flee_distance, flee_duration, panic_level } => {
                assert_eq!(flee_distance, 25.0);
                assert_eq!(flee_duration, 8.0);
                assert_eq!(panic_level, 0.9);
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_reaction_component() {
        use crate::ai::reaction_system::*;
        
        // Test default reaction component
        let mut reaction = ReactionComponent::default();
        assert!(reaction.reaction_rules.len() > 0);
        assert_eq!(reaction.reaction_sensitivity, 1.0);
        assert_eq!(reaction.memory_duration, 30.0);
        assert_eq!(reaction.max_recent_triggers, 10);
        
        // Test trigger memory management
        let trigger1 = ReactionTrigger::SoundEvent {
            sound_type: "footstep".to_string(),
            position: Vec2::new(1.0, 1.0),
            volume: 0.3,
            timestamp: 0.0,
        };
        
        let trigger2 = ReactionTrigger::PlayerAction {
            action_type: "movement".to_string(),
            position: Vec2::new(2.0, 2.0),
            intensity: 0.5,
            timestamp: 5.0,
        };
        
        reaction.add_trigger(trigger1);
        reaction.add_trigger(trigger2);
        assert_eq!(reaction.recent_triggers.len(), 2);
        
        // Test cooldown management
        let rule = &reaction.reaction_rules[0];
        assert!(reaction.can_trigger_rule(rule, 0.0));
        
        reaction.record_rule_trigger("test_rule".to_string(), 0.0);
        assert!(!reaction.can_trigger_rule(&ReactionRule {
            name: "test_rule".to_string(),
            trigger_conditions: vec![],
            response: ReactionResponse::BehaviorChange {
                new_state: AIBehaviorState::Idle,
                duration: None,
                priority: 1,
            },
            cooldown: 5.0,
            priority: 1,
            max_distance: 10.0,
        }, 2.0));
        
        // Test cleanup
        reaction.cleanup_old_triggers(100.0);
        assert_eq!(reaction.recent_triggers.len(), 0);
    }

    #[test]
    fn test_specialized_reaction_components() {
        use crate::ai::reaction_system::*;
        
        // Test guard reactions
        let guard_reaction = ReactionComponent::for_enemy_type("guard");
        assert!(guard_reaction.reaction_rules.len() > 2);
        assert!(guard_reaction.reaction_rules.iter().any(|rule| rule.name.contains("guard")));
        
        // Test coward reactions
        let coward_reaction = ReactionComponent::for_enemy_type("coward");
        assert!(coward_reaction.reaction_rules.iter().any(|rule| rule.name.contains("flee")));
        
        // Test berserker reactions
        let berserker_reaction = ReactionComponent::for_enemy_type("berserker");
        assert!(berserker_reaction.reaction_rules.iter().any(|rule| rule.name.contains("rage")));
        
        // Test unknown type (should get defaults)
        let unknown_reaction = ReactionComponent::for_enemy_type("unknown");
        assert_eq!(unknown_reaction.reaction_rules.len(), ReactionComponent::default().reaction_rules.len());
    }

    #[test]
    fn test_reaction_event_resource() {
        use crate::ai::reaction_system::*;
        
        let mut resource = ReactionEventResource::default();
        assert_eq!(resource.global_alert_level, 0.0);
        assert_eq!(resource.difficulty_modifier, 0.0);
        assert_eq!(resource.pending_triggers.len(), 0);
        
        // Test trigger addition
        let trigger = ReactionTrigger::EnvironmentalChange {
            change_type: "wall_destroyed".to_string(),
            affected_area: vec![IVec2::new(5, 5), IVec2::new(6, 5)],
            severity: 0.8,
            timestamp: 10.0,
        };
        
        resource.add_trigger(trigger);
        assert_eq!(resource.pending_triggers.len(), 1);
        
        // Test difficulty adjustment
        resource.adjust_difficulty(0.9); // High performance
        assert!(resource.difficulty_modifier > 0.0);
        
        resource.adjust_difficulty(0.2); // Low performance  
        assert!(resource.difficulty_modifier < 0.0);
        
        // Test clamping
        for _ in 0..200 {
            resource.adjust_difficulty(0.9);
        }
        assert!(resource.difficulty_modifier <= 2.0);
        
        for _ in 0..200 {
            resource.adjust_difficulty(0.1);
        }
        assert!(resource.difficulty_modifier >= 0.5);
    }

    #[test]
    fn test_reaction_rule_evaluation() {
        use crate::ai::reaction_system::*;
        
        let rule = ReactionRule {
            name: "test_rule".to_string(),
            trigger_conditions: vec![
                ReactionCondition::TriggerType("combat".to_string()),
                ReactionCondition::DistanceWithin(5.0),
                ReactionCondition::IntensityAbove(0.5),
            ],
            response: ReactionResponse::BehaviorChange {
                new_state: AIBehaviorState::Hunt,
                duration: Some(10.0),
                priority: 5,
            },
            cooldown: 3.0,
            priority: 5,
            max_distance: 5.0,
        };
        
        let trigger = ReactionTrigger::CombatEvent {
            event_type: "combat".to_string(),
            attacker: None,
            target: None,
            damage: 80.0, // 0.8 intensity
            position: Vec2::new(2.0, 0.0), // 2 units away
            timestamp: 0.0,
        };
        
        let ai = AIComponent::default();
        
        // All conditions should be met
        let all_met = rule.trigger_conditions.iter().all(|condition| {
            condition.evaluate(&trigger, &ai, Vec2::ZERO, 0.0)
        });
        assert!(all_met);
        
        // Test with trigger that doesn't meet distance condition
        let far_trigger = ReactionTrigger::CombatEvent {
            event_type: "combat".to_string(),
            attacker: None,
            target: None,
            damage: 80.0,
            position: Vec2::new(10.0, 0.0), // 10 units away
            timestamp: 0.0,
        };
        
        let distance_met = rule.trigger_conditions.iter().all(|condition| {
            condition.evaluate(&far_trigger, &ai, Vec2::ZERO, 0.0)
        });
        assert!(!distance_met);
    }