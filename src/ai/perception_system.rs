use bevy::prelude::*;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use crate::components::{Position, Health};
use crate::map::{DungeonMap, TileType};
use crate::ai::AIComponent;

/// Field of view calculation using symmetric shadowcasting
pub struct FieldOfView;

impl FieldOfView {
    /// Calculate visible tiles from a position using shadowcasting algorithm
    pub fn calculate_fov(
        center: IVec2,
        range: i32,
        map: &DungeonMap,
    ) -> HashSet<IVec2> {
        let mut visible = HashSet::new();
        visible.insert(center); // Center is always visible

        // Cast shadows in 8 octants
        for octant in 0..8 {
            Self::cast_light(
                &mut visible,
                map,
                center.x,
                center.y,
                1,
                1.0,
                0.0,
                range,
                Self::get_octant_transform(octant),
            );
        }

        visible
    }

    /// Cast light in a specific octant using recursive shadowcasting
    fn cast_light(
        visible: &mut HashSet<IVec2>,
        map: &DungeonMap,
        cx: i32,
        cy: i32,
        row: i32,
        start_slope: f32,
        end_slope: f32,
        radius: i32,
        transform: fn(i32, i32) -> (i32, i32),
    ) {
        if start_slope < end_slope {
            return;
        }

        let mut next_start_slope = start_slope;

        for i in row..=radius {
            let mut blocked = false;
            let dy = -i;

            let mut dx = (dy as f32 * start_slope) as i32;
            let end_dx = (dy as f32 * end_slope) as i32;

            while dx >= end_dx {
                let (map_x, map_y) = transform(cx + dx, cy + dy);
                let current_pos = IVec2::new(map_x, map_y);

                let l_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
                let r_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);

                if start_slope < r_slope {
                    break;
                } else if end_slope > l_slope {
                    dx -= 1;
                    continue;
                }

                // Check if within radius
                let distance_sq = dx * dx + dy * dy;
                if distance_sq <= radius * radius {
                    visible.insert(current_pos);
                }

                if blocked {
                    if Self::is_wall(map, current_pos) {
                        next_start_slope = r_slope;
                        dx -= 1;
                        continue;
                    } else {
                        blocked = false;
                        start_slope = next_start_slope;
                    }
                } else if Self::is_wall(map, current_pos) && i < radius {
                    blocked = true;
                    Self::cast_light(
                        visible,
                        map,
                        cx,
                        cy,
                        i + 1,
                        start_slope,
                        l_slope,
                        radius,
                        transform,
                    );
                    next_start_slope = r_slope;
                }

                dx -= 1;
            }

            if blocked {
                break;
            }
        }
    }

    /// Get transformation function for each octant
    fn get_octant_transform(octant: usize) -> fn(i32, i32) -> (i32, i32) {
        match octant {
            0 => |x, y| (x, y),
            1 => |x, y| (y, x),
            2 => |x, y| (-y, x),
            3 => |x, y| (-x, y),
            4 => |x, y| (-x, -y),
            5 => |x, y| (-y, -x),
            6 => |x, y| (y, -x),
            7 => |x, y| (x, -y),
            _ => |x, y| (x, y),
        }
    }

    /// Check if a position blocks line of sight
    fn is_wall(map: &DungeonMap, pos: IVec2) -> bool {
        if let Some(tile) = map.get_tile(pos.x, pos.y) {
            matches!(tile, TileType::Wall | TileType::DeepWater)
        } else {
            true // Out of bounds blocks sight
        }
    }
}

/// Component for entities that can see
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Viewshed {
    pub visible_tiles: HashSet<IVec2>,
    pub range: i32,
    pub dirty: bool, // Needs recalculation
}

impl Viewshed {
    pub fn new(range: i32) -> Self {
        Viewshed {
            visible_tiles: HashSet::new(),
            range,
            dirty: true,
        }
    }

    /// Check if a position is visible
    pub fn can_see(&self, pos: IVec2) -> bool {
        self.visible_tiles.contains(&pos)
    }

    /// Mark viewshed as needing update
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Get all visible positions
    pub fn get_visible_positions(&self) -> Vec<IVec2> {
        self.visible_tiles.iter().copied().collect()
    }

    /// Count visible tiles
    pub fn visible_count(&self) -> usize {
        self.visible_tiles.len()
    }
}

/// Memory of perceived entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionMemory {
    pub entity: Entity,
    pub last_known_position: IVec2,
    pub last_seen_time: f32,
    pub entity_type: String,
    pub threat_level: f32,
    pub health_when_last_seen: i32,
    pub was_hostile: bool,
    pub confidence: f32, // How sure we are about this information
}

impl PerceptionMemory {
    pub fn new(
        entity: Entity,
        position: IVec2,
        time: f32,
        entity_type: String,
        threat_level: f32,
        health: i32,
        hostile: bool,
    ) -> Self {
        PerceptionMemory {
            entity,
            last_known_position: position,
            last_seen_time: time,
            entity_type,
            threat_level,
            health_when_last_seen: health,
            was_hostile: hostile,
            confidence: 1.0,
        }
    }

    /// Update memory with new observation
    pub fn update(&mut self, position: IVec2, time: f32, health: i32, hostile: bool) {
        self.last_known_position = position;
        self.last_seen_time = time;
        self.health_when_last_seen = health;
        self.was_hostile = hostile;
        self.confidence = 1.0;
    }

    /// Decay confidence over time
    pub fn decay_confidence(&mut self, current_time: f32, decay_rate: f32) {
        let time_passed = current_time - self.last_seen_time;
        self.confidence = (self.confidence - time_passed * decay_rate).max(0.0);
    }

    /// Check if memory is still reliable
    pub fn is_reliable(&self, current_time: f32, max_age: f32) -> bool {
        current_time - self.last_seen_time < max_age && self.confidence > 0.3
    }
}

/// Component for entity perception and memory
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionComponent {
    pub perceived_entities: HashMap<Entity, PerceptionMemory>,
    pub detection_range: f32,
    pub hearing_range: f32,
    pub memory_duration: f32,
    pub confidence_decay_rate: f32,
    pub noise_sensitivity: f32,
    pub light_sensitivity: f32,
    pub last_update_time: f32,
}

impl Default for PerceptionComponent {
    fn default() -> Self {
        PerceptionComponent {
            perceived_entities: HashMap::new(),
            detection_range: 8.0,
            hearing_range: 12.0,
            memory_duration: 30.0,
            confidence_decay_rate: 0.1,
            noise_sensitivity: 1.0,
            light_sensitivity: 1.0,
            last_update_time: 0.0,
        }
    }
}

impl PerceptionComponent {
    pub fn new(detection_range: f32, hearing_range: f32) -> Self {
        PerceptionComponent {
            detection_range,
            hearing_range,
            ..Default::default()
        }
    }

    /// Add or update perception of an entity
    pub fn perceive_entity(
        &mut self,
        entity: Entity,
        position: IVec2,
        time: f32,
        entity_type: String,
        threat_level: f32,
        health: i32,
        hostile: bool,
    ) {
        if let Some(memory) = self.perceived_entities.get_mut(&entity) {
            memory.update(position, time, health, hostile);
        } else {
            let memory = PerceptionMemory::new(
                entity, position, time, entity_type, threat_level, health, hostile,
            );
            self.perceived_entities.insert(entity, memory);
        }
    }

    /// Get memory of a specific entity
    pub fn get_memory(&self, entity: Entity) -> Option<&PerceptionMemory> {
        self.perceived_entities.get(&entity)
    }

    /// Get all reliable memories
    pub fn get_reliable_memories(&self, current_time: f32) -> Vec<&PerceptionMemory> {
        self.perceived_entities
            .values()
            .filter(|memory| memory.is_reliable(current_time, self.memory_duration))
            .collect()
    }

    /// Get hostile entities in memory
    pub fn get_hostile_memories(&self, current_time: f32) -> Vec<&PerceptionMemory> {
        self.get_reliable_memories(current_time)
            .into_iter()
            .filter(|memory| memory.was_hostile)
            .collect()
    }

    /// Clean up old memories
    pub fn cleanup_old_memories(&mut self, current_time: f32) {
        self.perceived_entities.retain(|_, memory| {
            memory.is_reliable(current_time, self.memory_duration)
        });
    }

    /// Update confidence decay for all memories
    pub fn update_memory_confidence(&mut self, current_time: f32) {
        for memory in self.perceived_entities.values_mut() {
            memory.decay_confidence(current_time, self.confidence_decay_rate);
        }
    }

    /// Check if entity can detect noise at distance
    pub fn can_hear_noise(&self, distance: f32, noise_level: f32) -> bool {
        let effective_range = self.hearing_range * self.noise_sensitivity;
        distance <= effective_range && noise_level > 0.1
    }

    /// Calculate detection probability based on various factors
    pub fn calculate_detection_probability(
        &self,
        distance: f32,
        noise_level: f32,
        light_level: f32,
        target_stealth: f32,
    ) -> f32 {
        let distance_factor = (1.0 - (distance / self.detection_range).min(1.0)).max(0.0);
        let noise_factor = (noise_level * self.noise_sensitivity).min(1.0);
        let light_factor = light_level * self.light_sensitivity;
        let stealth_factor = 1.0 - target_stealth.min(1.0);

        let base_probability = distance_factor * 0.6 + noise_factor * 0.2 + light_factor * 0.2;
        (base_probability * stealth_factor).clamp(0.0, 1.0)
    }
}

/// Component for entities that make noise
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct NoiseEmitter {
    pub base_noise_level: f32,
    pub current_noise_level: f32,
    pub noise_decay_rate: f32,
    pub movement_noise_multiplier: f32,
    pub action_noise_multiplier: f32,
}

impl Default for NoiseEmitter {
    fn default() -> Self {
        NoiseEmitter {
            base_noise_level: 0.1,
            current_noise_level: 0.1,
            noise_decay_rate: 2.0,
            movement_noise_multiplier: 1.0,
            action_noise_multiplier: 2.0,
        }
    }
}

impl NoiseEmitter {
    pub fn new(base_noise: f32) -> Self {
        NoiseEmitter {
            base_noise_level: base_noise,
            current_noise_level: base_noise,
            ..Default::default()
        }
    }

    /// Add noise from movement
    pub fn add_movement_noise(&mut self, movement_speed: f32) {
        let movement_noise = movement_speed * self.movement_noise_multiplier * 0.1;
        self.current_noise_level = (self.current_noise_level + movement_noise).min(1.0);
    }

    /// Add noise from actions (combat, abilities, etc.)
    pub fn add_action_noise(&mut self, action_intensity: f32) {
        let action_noise = action_intensity * self.action_noise_multiplier * 0.2;
        self.current_noise_level = (self.current_noise_level + action_noise).min(1.0);
    }

    /// Decay noise over time
    pub fn decay_noise(&mut self, delta_time: f32) {
        self.current_noise_level = (self.current_noise_level - self.noise_decay_rate * delta_time)
            .max(self.base_noise_level);
    }

    /// Get effective noise level at a distance
    pub fn get_noise_at_distance(&self, distance: f32) -> f32 {
        if distance <= 0.0 {
            return self.current_noise_level;
        }
        
        // Noise falls off with distance
        let falloff = 1.0 / (1.0 + distance * 0.2);
        self.current_noise_level * falloff
    }
}

/// Component for stealth/concealment
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct StealthComponent {
    pub stealth_level: f32,
    pub movement_stealth_penalty: f32,
    pub action_stealth_penalty: f32,
    pub light_stealth_penalty: f32,
    pub base_stealth: f32,
}

impl Default for StealthComponent {
    fn default() -> Self {
        StealthComponent {
            stealth_level: 0.0,
            movement_stealth_penalty: 0.3,
            action_stealth_penalty: 0.5,
            light_stealth_penalty: 0.4,
            base_stealth: 0.0,
        }
    }
}

impl StealthComponent {
    pub fn new(base_stealth: f32) -> Self {
        StealthComponent {
            stealth_level: base_stealth,
            base_stealth,
            ..Default::default()
        }
    }

    /// Calculate effective stealth considering penalties
    pub fn get_effective_stealth(&self, is_moving: bool, is_acting: bool, light_level: f32) -> f32 {
        let mut effective_stealth = self.stealth_level;

        if is_moving {
            effective_stealth -= self.movement_stealth_penalty;
        }

        if is_acting {
            effective_stealth -= self.action_stealth_penalty;
        }

        effective_stealth -= light_level * self.light_stealth_penalty;

        effective_stealth.max(0.0)
    }

    /// Update stealth level
    pub fn update_stealth(&mut self, modifier: f32) {
        self.stealth_level = (self.base_stealth + modifier).clamp(0.0, 1.0);
    }
}

/// System for updating viewsheds
pub fn viewshed_system(
    mut viewshed_query: Query<(&mut Viewshed, &Position), Changed<Position>>,
    map: Res<DungeonMap>,
) {
    for (mut viewshed, position) in viewshed_query.iter_mut() {
        if viewshed.dirty {
            let center = position.0.as_ivec2();
            viewshed.visible_tiles = FieldOfView::calculate_fov(center, viewshed.range, &map);
            viewshed.dirty = false;
        }
    }
}

/// System for entity perception
pub fn perception_system(
    time: Res<Time>,
    mut perceivers: Query<(Entity, &mut PerceptionComponent, &Position, &Viewshed, &AIComponent)>,
    targets: Query<(Entity, &Position, &Health, Option<&NoiseEmitter>, Option<&StealthComponent>), Without<AIComponent>>,
    mut ai_query: Query<&mut AIComponent>,
) {
    let current_time = time.elapsed_seconds();
    let delta_time = time.delta_seconds();

    for (perceiver_entity, mut perception, perceiver_pos, viewshed, ai) in perceivers.iter_mut() {
        if !ai.enabled {
            continue;
        }

        // Update memory confidence and clean up old memories
        perception.update_memory_confidence(current_time);
        perception.cleanup_old_memories(current_time);

        // Perceive entities
        for (target_entity, target_pos, target_health, noise_emitter, stealth) in targets.iter() {
            let distance = perceiver_pos.0.distance(target_pos.0);
            let target_ivec = target_pos.0.as_ivec2();

            // Check if target is within detection range
            if distance > perception.detection_range {
                continue;
            }

            // Calculate detection factors
            let noise_level = noise_emitter
                .map(|ne| ne.get_noise_at_distance(distance))
                .unwrap_or(0.1);

            let light_level = 1.0; // TODO: Implement proper lighting system

            let target_stealth = stealth
                .map(|s| s.get_effective_stealth(false, false, light_level))
                .unwrap_or(0.0);

            // Check line of sight for visual detection
            let can_see = viewshed.can_see(target_ivec);

            // Check noise-based detection
            let can_hear = perception.can_hear_noise(distance, noise_level);

            // Calculate detection probability
            let detection_prob = perception.calculate_detection_probability(
                distance,
                noise_level,
                light_level,
                target_stealth,
            );

            // Determine if entity is detected
            let is_detected = can_see || (can_hear && detection_prob > 0.5);

            if is_detected {
                // Determine threat level and hostility
                let threat_level = calculate_threat_level(target_health, distance);
                let is_hostile = true; // TODO: Implement faction system

                perception.perceive_entity(
                    target_entity,
                    target_ivec,
                    current_time,
                    "Unknown".to_string(), // TODO: Get actual entity type
                    threat_level,
                    target_health.current,
                    is_hostile,
                );

                // Update AI with perceived target
                if let Ok(mut target_ai) = ai_query.get_mut(perceiver_entity) {
                    if is_hostile && target_ai.current_target.is_none() {
                        target_ai.set_target(Some(target_entity));
                        target_ai.memory.last_known_target_position = Some(target_pos.0);
                    }
                }
            }
        }

        perception.last_update_time = current_time;
    }
}

/// System for updating noise levels
pub fn noise_system(
    time: Res<Time>,
    mut noise_emitters: Query<&mut NoiseEmitter>,
) {
    let delta_time = time.delta_seconds();

    for mut noise_emitter in noise_emitters.iter_mut() {
        noise_emitter.decay_noise(delta_time);
    }
}

/// System for marking viewsheds as dirty when entities move
pub fn viewshed_dirty_system(
    mut moved_entities: Query<&mut Viewshed, (Changed<Position>, With<Viewshed>)>,
) {
    for mut viewshed in moved_entities.iter_mut() {
        viewshed.mark_dirty();
    }
}

/// Calculate threat level based on health and distance
fn calculate_threat_level(health: &Health, distance: f32) -> f32 {
    let health_factor = health.current as f32 / health.max as f32;
    let distance_factor = (10.0 - distance.min(10.0)) / 10.0;
    
    (health_factor * 0.7 + distance_factor * 0.3).clamp(0.0, 1.0)
}

/// System for sharing perception information between group members
pub fn group_perception_sharing_system(
    mut perceivers: Query<(&mut PerceptionComponent, &AIComponent)>,
    group_resource: Res<crate::ai::GroupCoordinationResource>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_seconds();

    // Share information within groups
    for group in group_resource.groups.values() {
        let mut shared_memories: Vec<PerceptionMemory> = Vec::new();

        // Collect all memories from group members
        for &member in &group.members {
            if let Ok((perception, ai)) = perceivers.get(member) {
                if ai.enabled {
                    for memory in perception.get_reliable_memories(current_time) {
                        shared_memories.push(memory.clone());
                    }
                }
            }
        }

        // Share memories with all group members
        for &member in &group.members {
            if let Ok((mut perception, ai)) = perceivers.get_mut(member) {
                if ai.enabled {
                    for shared_memory in &shared_memories {
                        // Add shared memory with reduced confidence
                        if !perception.perceived_entities.contains_key(&shared_memory.entity) {
                            let mut memory = shared_memory.clone();
                            memory.confidence *= 0.7; // Shared information is less reliable
                            perception.perceived_entities.insert(shared_memory.entity, memory);
                        }
                    }
                }
            }
        }
    }
}

/// Plugin for perception systems
pub struct PerceptionPlugin;

impl Plugin for PerceptionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            viewshed_dirty_system,
            viewshed_system,
            perception_system,
            noise_system,
            group_perception_sharing_system,
        ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewshed_creation() {
        let viewshed = Viewshed::new(8);
        assert_eq!(viewshed.range, 8);
        assert!(viewshed.dirty);
        assert!(viewshed.visible_tiles.is_empty());
    }

    #[test]
    fn test_perception_memory() {
        let entity = Entity::from_raw(1);
        let mut memory = PerceptionMemory::new(
            entity,
            IVec2::new(5, 5),
            0.0,
            "Enemy".to_string(),
            0.8,
            100,
            true,
        );

        assert_eq!(memory.entity, entity);
        assert_eq!(memory.last_known_position, IVec2::new(5, 5));
        assert_eq!(memory.confidence, 1.0);

        // Test confidence decay
        memory.decay_confidence(10.0, 0.1);
        assert!(memory.confidence < 1.0);

        // Test reliability
        assert!(memory.is_reliable(5.0, 30.0));
        assert!(!memory.is_reliable(50.0, 30.0));
    }

    #[test]
    fn test_noise_emitter() {
        let mut noise = NoiseEmitter::new(0.1);
        assert_eq!(noise.current_noise_level, 0.1);

        // Test adding movement noise
        noise.add_movement_noise(2.0);
        assert!(noise.current_noise_level > 0.1);

        // Test noise decay
        let initial_noise = noise.current_noise_level;
        noise.decay_noise(1.0);
        assert!(noise.current_noise_level < initial_noise);

        // Test noise at distance
        let distant_noise = noise.get_noise_at_distance(10.0);
        assert!(distant_noise < noise.current_noise_level);
    }

    #[test]
    fn test_stealth_component() {
        let mut stealth = StealthComponent::new(0.5);
        assert_eq!(stealth.stealth_level, 0.5);

        // Test effective stealth with penalties
        let effective = stealth.get_effective_stealth(true, true, 0.8);
        assert!(effective < stealth.stealth_level);

        // Test stealth update
        stealth.update_stealth(0.2);
        assert_eq!(stealth.stealth_level, 0.7);
    }

    #[test]
    fn test_perception_component() {
        let mut perception = PerceptionComponent::new(8.0, 12.0);
        assert_eq!(perception.detection_range, 8.0);
        assert_eq!(perception.hearing_range, 12.0);

        // Test detection probability calculation
        let prob = perception.calculate_detection_probability(5.0, 0.5, 0.8, 0.2);
        assert!(prob > 0.0 && prob <= 1.0);

        // Test noise hearing
        assert!(perception.can_hear_noise(10.0, 0.5));
        assert!(!perception.can_hear_noise(20.0, 0.1));
    }
}