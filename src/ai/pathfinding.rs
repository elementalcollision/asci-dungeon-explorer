use bevy::prelude::*;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use crate::map::{DungeonMap, TileType};

/// A node in the pathfinding graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PathNode {
    pub position: IVec2,
    pub g_cost: i32,      // Cost from start
    pub h_cost: i32,      // Heuristic cost to goal
    pub parent: Option<IVec2>,
}

impl PathNode {
    pub fn new(position: IVec2, g_cost: i32, h_cost: i32, parent: Option<IVec2>) -> Self {
        PathNode {
            position,
            g_cost,
            h_cost,
            parent,
        }
    }

    pub fn f_cost(&self) -> i32 {
        self.g_cost + self.h_cost
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.f_cost().cmp(&self.f_cost())
            .then_with(|| other.h_cost.cmp(&self.h_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A complete path from start to goal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Path {
    pub waypoints: Vec<IVec2>,
    pub current_waypoint: usize,
    pub total_cost: i32,
    pub created_time: f32,
    pub is_valid: bool,
}

impl Path {
    pub fn new(waypoints: Vec<IVec2>, total_cost: i32, created_time: f32) -> Self {
        Path {
            waypoints,
            current_waypoint: 0,
            total_cost,
            created_time,
            is_valid: true,
        }
    }

    /// Get the current target waypoint
    pub fn current_target(&self) -> Option<IVec2> {
        if self.current_waypoint < self.waypoints.len() {
            Some(self.waypoints[self.current_waypoint])
        } else {
            None
        }
    }

    /// Advance to the next waypoint
    pub fn advance_waypoint(&mut self) -> bool {
        if self.current_waypoint < self.waypoints.len() - 1 {
            self.current_waypoint += 1;
            true
        } else {
            false
        }
    }

    /// Check if the path is complete
    pub fn is_complete(&self) -> bool {
        self.current_waypoint >= self.waypoints.len()
    }

    /// Get the remaining waypoints
    pub fn remaining_waypoints(&self) -> &[IVec2] {
        if self.current_waypoint < self.waypoints.len() {
            &self.waypoints[self.current_waypoint..]
        } else {
            &[]
        }
    }

    /// Get the final destination
    pub fn destination(&self) -> Option<IVec2> {
        self.waypoints.last().copied()
    }

    /// Invalidate the path
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }

    /// Check if path is stale (older than given time)
    pub fn is_stale(&self, current_time: f32, max_age: f32) -> bool {
        current_time - self.created_time > max_age
    }
}

/// Pathfinding cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    path: Path,
    access_count: u32,
    last_accessed: f32,
}

/// A* pathfinding implementation
pub struct AStarPathfinder {
    cache: HashMap<(IVec2, IVec2), CacheEntry>,
    cache_max_size: usize,
    cache_max_age: f32,
    diagonal_movement: bool,
    movement_cost_base: i32,
    movement_cost_diagonal: i32,
}

impl Default for AStarPathfinder {
    fn default() -> Self {
        AStarPathfinder {
            cache: HashMap::new(),
            cache_max_size: 1000,
            cache_max_age: 30.0, // 30 seconds
            diagonal_movement: true,
            movement_cost_base: 10,
            movement_cost_diagonal: 14,
        }
    }
}

impl AStarPathfinder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Configure pathfinding parameters
    pub fn with_config(
        cache_max_size: usize,
        cache_max_age: f32,
        diagonal_movement: bool,
    ) -> Self {
        AStarPathfinder {
            cache: HashMap::new(),
            cache_max_size,
            cache_max_age,
            diagonal_movement,
            movement_cost_base: 10,
            movement_cost_diagonal: 14,
        }
    }

    /// Find a path from start to goal using A* algorithm
    pub fn find_path(
        &mut self,
        start: IVec2,
        goal: IVec2,
        map: &DungeonMap,
        current_time: f32,
    ) -> Option<Path> {
        // Check cache first
        if let Some(cached_path) = self.get_cached_path(start, goal, current_time) {
            return Some(cached_path);
        }

        // Perform A* search
        let path = self.a_star_search(start, goal, map, current_time)?;

        // Cache the result
        self.cache_path(start, goal, path.clone(), current_time);

        Some(path)
    }

    /// Get a cached path if available and valid
    fn get_cached_path(&mut self, start: IVec2, goal: IVec2, current_time: f32) -> Option<Path> {
        let key = (start, goal);
        
        if let Some(entry) = self.cache.get_mut(&key) {
            // Check if cache entry is still valid
            if !entry.path.is_stale(current_time, self.cache_max_age) && entry.path.is_valid {
                entry.access_count += 1;
                entry.last_accessed = current_time;
                return Some(entry.path.clone());
            }
        }

        None
    }

    /// Cache a path
    fn cache_path(&mut self, start: IVec2, goal: IVec2, path: Path, current_time: f32) {
        // Clean cache if it's getting too large
        if self.cache.len() >= self.cache_max_size {
            self.clean_cache(current_time);
        }

        let key = (start, goal);
        let entry = CacheEntry {
            path,
            access_count: 1,
            last_accessed: current_time,
        };

        self.cache.insert(key, entry);
    }

    /// Clean old entries from cache
    fn clean_cache(&mut self, current_time: f32) {
        // Remove stale entries
        self.cache.retain(|_, entry| {
            !entry.path.is_stale(current_time, self.cache_max_age) && entry.path.is_valid
        });

        // If still too large, remove least recently used entries
        if self.cache.len() >= self.cache_max_size {
            let mut entries: Vec<_> = self.cache.iter().collect();
            entries.sort_by(|a, b| a.1.last_accessed.partial_cmp(&b.1.last_accessed).unwrap());
            
            let to_remove = entries.len() - (self.cache_max_size / 2);
            for (key, _) in entries.iter().take(to_remove) {
                self.cache.remove(key);
            }
        }
    }

    /// Perform A* pathfinding search
    fn a_star_search(
        &self,
        start: IVec2,
        goal: IVec2,
        map: &DungeonMap,
        current_time: f32,
    ) -> Option<Path> {
        if start == goal {
            return Some(Path::new(vec![start], 0, current_time));
        }

        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();

        // Initialize start node
        let start_node = PathNode::new(
            start,
            0,
            self.heuristic_cost(start, goal),
            None,
        );

        open_set.push(start_node);
        g_score.insert(start, 0);

        while let Some(current) = open_set.pop() {
            if current.position == goal {
                // Reconstruct path
                let path_waypoints = self.reconstruct_path(&came_from, current.position);
                return Some(Path::new(path_waypoints, current.g_cost, current_time));
            }

            closed_set.insert(current.position);

            // Check all neighbors
            for neighbor_pos in self.get_neighbors(current.position, map) {
                if closed_set.contains(&neighbor_pos) {
                    continue;
                }

                let movement_cost = self.get_movement_cost(current.position, neighbor_pos);
                let tentative_g_score = current.g_cost + movement_cost;

                let neighbor_g_score = g_score.get(&neighbor_pos).copied().unwrap_or(i32::MAX);

                if tentative_g_score < neighbor_g_score {
                    came_from.insert(neighbor_pos, current.position);
                    g_score.insert(neighbor_pos, tentative_g_score);

                    let neighbor_node = PathNode::new(
                        neighbor_pos,
                        tentative_g_score,
                        self.heuristic_cost(neighbor_pos, goal),
                        Some(current.position),
                    );

                    open_set.push(neighbor_node);
                }
            }
        }

        None // No path found
    }

    /// Reconstruct the path from the came_from map
    fn reconstruct_path(&self, came_from: &HashMap<IVec2, IVec2>, mut current: IVec2) -> Vec<IVec2> {
        let mut path = vec![current];

        while let Some(&parent) = came_from.get(&current) {
            current = parent;
            path.push(current);
        }

        path.reverse();
        path
    }

    /// Get valid neighbors for a position
    fn get_neighbors(&self, pos: IVec2, map: &DungeonMap) -> Vec<IVec2> {
        let mut neighbors = Vec::new();

        // Cardinal directions
        let directions = if self.diagonal_movement {
            vec![
                IVec2::new(-1, -1), IVec2::new(0, -1), IVec2::new(1, -1),
                IVec2::new(-1,  0),                     IVec2::new(1,  0),
                IVec2::new(-1,  1), IVec2::new(0,  1), IVec2::new(1,  1),
            ]
        } else {
            vec![
                IVec2::new(0, -1),
                IVec2::new(-1, 0), IVec2::new(1, 0),
                IVec2::new(0, 1),
            ]
        };

        for direction in directions {
            let neighbor_pos = pos + direction;

            if self.is_walkable(neighbor_pos, map) {
                // For diagonal movement, check that we're not cutting corners
                if self.diagonal_movement && direction.x != 0 && direction.y != 0 {
                    let horizontal = pos + IVec2::new(direction.x, 0);
                    let vertical = pos + IVec2::new(0, direction.y);
                    
                    if self.is_walkable(horizontal, map) || self.is_walkable(vertical, map) {
                        neighbors.push(neighbor_pos);
                    }
                } else {
                    neighbors.push(neighbor_pos);
                }
            }
        }

        neighbors
    }

    /// Check if a position is walkable
    fn is_walkable(&self, pos: IVec2, map: &DungeonMap) -> bool {
        if let Some(tile) = map.get_tile(pos.x, pos.y) {
            match tile {
                TileType::Floor | TileType::Door | TileType::DownStairs | TileType::UpStairs => true,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Calculate movement cost between two adjacent positions
    fn get_movement_cost(&self, from: IVec2, to: IVec2) -> i32 {
        let diff = to - from;
        if diff.x.abs() == 1 && diff.y.abs() == 1 {
            self.movement_cost_diagonal
        } else {
            self.movement_cost_base
        }
    }

    /// Calculate heuristic cost (Manhattan distance for grid-based movement)
    fn heuristic_cost(&self, from: IVec2, to: IVec2) -> i32 {
        let diff = to - from;
        if self.diagonal_movement {
            // Diagonal distance heuristic
            let dx = diff.x.abs();
            let dy = diff.y.abs();
            self.movement_cost_base * (dx + dy) + (self.movement_cost_diagonal - 2 * self.movement_cost_base) * dx.min(dy)
        } else {
            // Manhattan distance
            (diff.x.abs() + diff.y.abs()) * self.movement_cost_base
        }
    }

    /// Invalidate cached paths that pass through a position
    pub fn invalidate_paths_through(&mut self, position: IVec2) {
        for entry in self.cache.values_mut() {
            if entry.path.waypoints.contains(&position) {
                entry.path.invalidate();
            }
        }
    }

    /// Clear all cached paths
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        let valid_entries = self.cache.values().filter(|entry| entry.path.is_valid).count();
        (self.cache.len(), valid_entries)
    }
}

/// Component for entities that can follow paths
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct PathFollower {
    pub current_path: Option<Path>,
    pub movement_speed: f32,
    pub path_recalculation_distance: f32, // Recalculate path if target moves this far
    pub stuck_threshold: f32,              // Time before considering entity stuck
    pub stuck_timer: f32,
    pub last_position: Option<IVec2>,
    pub arrival_threshold: f32,            // Distance to waypoint to consider it reached
}

impl Default for PathFollower {
    fn default() -> Self {
        PathFollower {
            current_path: None,
            movement_speed: 2.0,
            path_recalculation_distance: 3.0,
            stuck_threshold: 2.0,
            stuck_timer: 0.0,
            last_position: None,
            arrival_threshold: 0.5,
        }
    }
}

impl PathFollower {
    pub fn new(movement_speed: f32) -> Self {
        PathFollower {
            movement_speed,
            ..Default::default()
        }
    }

    /// Set a new path to follow
    pub fn set_path(&mut self, path: Path) {
        self.current_path = Some(path);
        self.stuck_timer = 0.0;
    }

    /// Clear the current path
    pub fn clear_path(&mut self) {
        self.current_path = None;
        self.stuck_timer = 0.0;
    }

    /// Check if currently following a path
    pub fn has_path(&self) -> bool {
        self.current_path.as_ref().map_or(false, |path| path.is_valid && !path.is_complete())
    }

    /// Get the current target position
    pub fn current_target(&self) -> Option<IVec2> {
        self.current_path.as_ref().and_then(|path| path.current_target())
    }

    /// Update path following logic
    pub fn update(&mut self, current_position: IVec2, delta_time: f32) -> Option<Vec2> {
        if let Some(path) = &mut self.current_path {
            if !path.is_valid {
                self.clear_path();
                return None;
            }

            if let Some(target) = path.current_target() {
                let current_pos_f32 = Vec2::new(current_position.x as f32, current_position.y as f32);
                let target_pos_f32 = Vec2::new(target.x as f32, target.y as f32);
                let distance = current_pos_f32.distance(target_pos_f32);

                // Check if we've reached the current waypoint
                if distance <= self.arrival_threshold {
                    if !path.advance_waypoint() {
                        // Path complete
                        self.clear_path();
                        return None;
                    }
                    
                    // Get next target
                    if let Some(next_target) = path.current_target() {
                        let next_target_f32 = Vec2::new(next_target.x as f32, next_target.y as f32);
                        return Some(next_target_f32);
                    }
                } else {
                    // Move towards current target
                    return Some(target_pos_f32);
                }
            }

            // Check if stuck
            if let Some(last_pos) = self.last_position {
                if last_pos == current_position {
                    self.stuck_timer += delta_time;
                    if self.stuck_timer >= self.stuck_threshold {
                        // Entity is stuck, invalidate path
                        path.invalidate();
                        self.clear_path();
                    }
                } else {
                    self.stuck_timer = 0.0;
                }
            }

            self.last_position = Some(current_position);
        }

        None
    }

    /// Check if the entity appears to be stuck
    pub fn is_stuck(&self) -> bool {
        self.stuck_timer >= self.stuck_threshold
    }
}

/// Resource for global pathfinding
#[derive(Resource)]
pub struct PathfindingResource {
    pub pathfinder: AStarPathfinder,
}

impl Default for PathfindingResource {
    fn default() -> Self {
        PathfindingResource {
            pathfinder: AStarPathfinder::new(),
        }
    }
}

/// System for updating path followers
pub fn path_following_system(
    time: Res<Time>,
    mut path_followers: Query<(&mut PathFollower, &mut Transform, &Position)>,
) {
    let delta_time = time.delta_seconds();

    for (mut follower, mut transform, position) in path_followers.iter_mut() {
        let current_pos = position.0.as_ivec2();
        
        if let Some(target_pos) = follower.update(current_pos, delta_time) {
            // Calculate movement direction
            let current_world_pos = Vec2::new(transform.translation.x, transform.translation.y);
            let direction = (target_pos - current_world_pos).normalize_or_zero();
            
            // Move towards target
            let movement = direction * follower.movement_speed * delta_time;
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;
        }
    }
}

/// System for pathfinding requests
pub fn pathfinding_request_system(
    mut pathfinding_res: ResMut<PathfindingResource>,
    map_res: Res<DungeonMap>,
    time: Res<Time>,
    mut path_requests: Query<(&PathfindingRequest, &mut PathFollower, &Position)>,
    mut commands: Commands,
) {
    let current_time = time.elapsed_seconds();

    for (request, mut follower, position) in path_requests.iter_mut() {
        let start = position.0.as_ivec2();
        let goal = request.target.as_ivec2();

        if let Some(path) = pathfinding_res.pathfinder.find_path(start, goal, &map_res, current_time) {
            follower.set_path(path);
        }
    }

    // Remove processed requests
    for entity in path_requests.iter().map(|(_, _, _)| entity) {
        commands.entity(entity).remove::<PathfindingRequest>();
    }
}

/// Component for requesting pathfinding
#[derive(Component)]
pub struct PathfindingRequest {
    pub target: Vec2,
}

impl PathfindingRequest {
    pub fn new(target: Vec2) -> Self {
        PathfindingRequest { target }
    }
}

/// Plugin for pathfinding systems
pub struct PathfindingPlugin;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PathfindingResource>()
            .add_systems(Update, (
                pathfinding_request_system,
                path_following_system,
            ).chain());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let waypoints = vec![
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(2, 0),
        ];
        let path = Path::new(waypoints.clone(), 20, 0.0);
        
        assert_eq!(path.waypoints, waypoints);
        assert_eq!(path.current_waypoint, 0);
        assert_eq!(path.total_cost, 20);
        assert!(path.is_valid);
        assert!(!path.is_complete());
    }

    #[test]
    fn test_path_following() {
        let waypoints = vec![
            IVec2::new(0, 0),
            IVec2::new(1, 0),
            IVec2::new(2, 0),
        ];
        let mut path = Path::new(waypoints, 20, 0.0);
        
        assert_eq!(path.current_target(), Some(IVec2::new(0, 0)));
        
        assert!(path.advance_waypoint());
        assert_eq!(path.current_target(), Some(IVec2::new(1, 0)));
        
        assert!(path.advance_waypoint());
        assert_eq!(path.current_target(), Some(IVec2::new(2, 0)));
        
        assert!(!path.advance_waypoint());
        assert!(path.is_complete());
    }

    #[test]
    fn test_pathfinder_heuristic() {
        let pathfinder = AStarPathfinder::new();
        
        let cost = pathfinder.heuristic_cost(IVec2::new(0, 0), IVec2::new(3, 4));
        assert!(cost > 0);
        
        // Distance to self should be 0
        let self_cost = pathfinder.heuristic_cost(IVec2::new(5, 5), IVec2::new(5, 5));
        assert_eq!(self_cost, 0);
    }

    #[test]
    fn test_path_follower() {
        let mut follower = PathFollower::new(2.0);
        assert!(!follower.has_path());
        
        let path = Path::new(vec![IVec2::new(0, 0), IVec2::new(1, 0)], 10, 0.0);
        follower.set_path(path);
        assert!(follower.has_path());
        
        follower.clear_path();
        assert!(!follower.has_path());
    }

    #[test]
    fn test_movement_cost() {
        let pathfinder = AStarPathfinder::new();
        
        // Cardinal movement
        let cardinal_cost = pathfinder.get_movement_cost(IVec2::new(0, 0), IVec2::new(1, 0));
        assert_eq!(cardinal_cost, 10);
        
        // Diagonal movement
        let diagonal_cost = pathfinder.get_movement_cost(IVec2::new(0, 0), IVec2::new(1, 1));
        assert_eq!(diagonal_cost, 14);
    }
}