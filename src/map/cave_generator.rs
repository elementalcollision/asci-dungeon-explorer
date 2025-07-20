use rand::Rng;
use crate::map::{Map, TileType, MapTheme};
use crate::resources::RandomNumberGenerator;
use super::dungeon_generator::MapGenerator;

pub struct CellularAutomataCaveGenerator {
    pub rng: RandomNumberGenerator,
    pub noise_density: f32,
    pub iterations: i32,
    pub birth_limit: i32,
    pub death_limit: i32,
}

impl CellularAutomataCaveGenerator {
    pub fn new(rng: RandomNumberGenerator) -> Self {
        CellularAutomataCaveGenerator {
            rng,
            noise_density: 0.45,
            iterations: 5,
            birth_limit: 4,
            death_limit: 3,
        }
    }
    
    fn count_neighbors(&self, map: &Map, x: i32, y: i32) -> i32 {
        let mut count = 0;
        
        for nx in -1..=1 {
            for ny in -1..=1 {
                // Don't count the cell itself
                if nx == 0 && ny == 0 { continue; }
                
                let tx = x + nx;
                let ty = y + ny;
                
                // Count walls outside the map as neighbors
                if !map.in_bounds(tx, ty) {
                    count += 1;
                } else {
                    let idx = map.xy_idx(tx, ty);
                    if map.tiles[idx] == TileType::Wall {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
    
    fn apply_cellular_automaton(&mut self, map: &mut Map) {
        let mut new_tiles = map.tiles.clone();
        
        for y in 1..map.height-1 {
            for x in 1..map.width-1 {
                let idx = map.xy_idx(x, y);
                let neighbors = self.count_neighbors(map, x, y);
                
                // Apply cellular automaton rules
                if map.tiles[idx] == TileType::Wall {
                    // Wall cell
                    if neighbors < self.death_limit {
                        // Change to floor if too few neighbors
                        new_tiles[idx] = TileType::Floor;
                    }
                } else {
                    // Floor cell
                    if neighbors > self.birth_limit {
                        // Change to wall if too many neighbors
                        new_tiles[idx] = TileType::Wall;
                    }
                }
            }
        }
        
        map.tiles = new_tiles;
    }
    
    fn connect_regions(&mut self, map: &mut Map) {
        // Find all separate regions
        let regions = self.find_regions(map);
        
        if regions.len() <= 1 {
            return; // Already connected
        }
        
        // Sort regions by size (largest first)
        let mut sorted_regions = regions.clone();
        sorted_regions.sort_by(|a, b| b.len().cmp(&a.len()));
        
        // The largest region is our main cave
        let main_region = &sorted_regions[0];
        
        // Connect each other region to the main region
        for region in sorted_regions.iter().skip(1) {
            // Find closest points between regions
            let (p1, p2) = self.find_closest_points(map, main_region, region);
            
            // Create a tunnel between these points
            self.create_tunnel(map, p1, p2);
        }
    }
    
    fn find_regions(&self, map: &Map) -> Vec<Vec<(i32, i32)>> {
        let mut regions = Vec::new();
        let mut visited = vec![false; (map.width * map.height) as usize];
        
        for y in 1..map.height-1 {
            for x in 1..map.width-1 {
                let idx = map.xy_idx(x, y);
                
                if !visited[idx] && map.tiles[idx] == TileType::Floor {
                    // Found an unvisited floor tile, start a new region
                    let mut region = Vec::new();
                    self.flood_fill(map, x, y, &mut visited, &mut region);
                    regions.push(region);
                }
            }
        }
        
        regions
    }
    
    fn flood_fill(&self, map: &Map, x: i32, y: i32, visited: &mut Vec<bool>, region: &mut Vec<(i32, i32)>) {
        let mut stack = vec![(x, y)];
        
        while let Some((cx, cy)) = stack.pop() {
            let idx = map.xy_idx(cx, cy);
            
            if visited[idx] || map.tiles[idx] != TileType::Floor {
                continue;
            }
            
            // Mark as visited and add to region
            visited[idx] = true;
            region.push((cx, cy));
            
            // Add neighbors to stack
            for &(dx, dy) in &[(0, -1), (1, 0), (0, 1), (-1, 0)] {
                let nx = cx + dx;
                let ny = cy + dy;
                
                if map.in_bounds(nx, ny) {
                    stack.push((nx, ny));
                }
            }
        }
    }
    
    fn find_closest_points(&self, map: &Map, region1: &[(i32, i32)], region2: &[(i32, i32)]) -> ((i32, i32), (i32, i32)) {
        let mut min_distance = f32::MAX;
        let mut closest_pair = ((0, 0), (0, 0));
        
        for &(x1, y1) in region1 {
            for &(x2, y2) in region2 {
                let dx = (x2 - x1) as f32;
                let dy = (y2 - y1) as f32;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance < min_distance {
                    min_distance = distance;
                    closest_pair = ((x1, y1), (x2, y2));
                }
            }
        }
        
        closest_pair
    }
    
    fn create_tunnel(&mut self, map: &mut Map, p1: (i32, i32), p2: (i32, i32)) {
        let (x1, y1) = p1;
        let (x2, y2) = p2;
        
        // Use a simple L-shaped tunnel
        let mut current_x = x1;
        let mut current_y = y1;
        
        // Go horizontally first
        while current_x != x2 {
            let idx = map.xy_idx(current_x, current_y);
            map.tiles[idx] = TileType::Floor;
            map.blocked[idx] = false;
            
            current_x += if x2 > x1 { 1 } else { -1 };
        }
        
        // Then go vertically
        while current_y != y2 {
            let idx = map.xy_idx(current_x, current_y);
            map.tiles[idx] = TileType::Floor;
            map.blocked[idx] = false;
            
            current_y += if y2 > y1 { 1 } else { -1 };
        }
    }
    
    fn place_stairs(&mut self, map: &mut Map) {
        // Find the most distant floor tiles to place stairs
        let mut floor_tiles = Vec::new();
        
        for y in 1..map.height-1 {
            for x in 1..map.width-1 {
                let idx = map.xy_idx(x, y);
                if map.tiles[idx] == TileType::Floor {
                    floor_tiles.push((x, y));
                }
            }
        }
        
        if floor_tiles.is_empty() {
            return;
        }
        
        // Place stairs up at a random floor tile
        let up_idx = self.rng.range(0, floor_tiles.len() as i32) as usize;
        let (up_x, up_y) = floor_tiles[up_idx];
        let up_tile_idx = map.xy_idx(up_x, up_y);
        map.tiles[up_tile_idx] = TileType::UpStairs;
        map.entrance = (up_x, up_y);
        
        // Find the floor tile farthest from the stairs up for stairs down
        let mut max_distance = 0.0;
        let mut down_pos = (up_x, up_y);
        
        for &(x, y) in &floor_tiles {
            let dx = (x - up_x) as f32;
            let dy = (y - up_y) as f32;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance > max_distance {
                max_distance = distance;
                down_pos = (x, y);
            }
        }
        
        let down_tile_idx = map.xy_idx(down_pos.0, down_pos.1);
        map.tiles[down_tile_idx] = TileType::DownStairs;
        map.exit = down_pos;
    }
}

impl MapGenerator for CellularAutomataCaveGenerator {
    fn generate_map(&mut self, width: i32, height: i32, depth: i32) -> Map {
        let mut map = Map::new(width, height, depth);
        map.theme = MapTheme::Cave;
        
        // Initialize with random noise
        for y in 1..height-1 {
            for x in 1..width-1 {
                let idx = map.xy_idx(x, y);
                if (self.rng.range(0, 1000) as f32 / 1000.0) < self.noise_density {
                    map.tiles[idx] = TileType::Wall;
                    map.blocked[idx] = true;
                } else {
                    map.tiles[idx] = TileType::Floor;
                    map.blocked[idx] = false;
                }
            }
        }
        
        // Apply cellular automaton iterations
        for _ in 0..self.iterations {
            self.apply_cellular_automaton(&mut map);
        }
        
        // Connect separate regions
        self.connect_regions(&mut map);
        
        // Place stairs
        self.place_stairs(&mut map);
        
        // Update the blocked array
        map.populate_blocked();
        
        map
    }
}