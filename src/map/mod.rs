use serde::{Serialize, Deserialize};
use std::cmp::{max, min};

mod dungeon_generator;
mod cave_generator;
mod feature_generator;
mod entity_placement;

pub use dungeon_generator::{MapGenerator, RoomBasedDungeonGenerator};
pub use cave_generator::CellularAutomataCaveGenerator;
pub use feature_generator::{DungeonFeatureGenerator, SpecialRoomType, EnvironmentalHazard};
pub use entity_placement::{EntityPlacementSystem, EnemyType, ItemType};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    UpStairs,
    Door(bool), // bool indicates if door is open
    Water,
    Lava,
    Trap(bool), // bool indicates if trap is visible
    Bridge,
    Grass,
    Tree,
    Rock,
    Sand,
    Ice,
    Void,
}

impl TileType {
    /// Returns true if this tile blocks movement
    pub fn blocks_movement(&self) -> bool {
        matches!(self, TileType::Wall | TileType::Tree | TileType::Rock | TileType::Void | TileType::Door(false))
    }
    
    /// Returns true if this tile blocks line of sight
    pub fn blocks_sight(&self) -> bool {
        matches!(self, TileType::Wall | TileType::Tree | TileType::Rock)
    }
    
    /// Returns true if this tile is dangerous to walk on
    pub fn is_dangerous(&self) -> bool {
        matches!(self, TileType::Lava | TileType::Void)
    }
    
    /// Returns the movement cost for this tile (1.0 = normal, higher = slower)
    pub fn movement_cost(&self) -> f32 {
        match self {
            TileType::Floor | TileType::Grass | TileType::Sand => 1.0,
            TileType::Water => 2.0,
            TileType::Ice => 0.5,
            TileType::Lava => 3.0,
            TileType::DownStairs | TileType::UpStairs => 1.0,
            TileType::Door(true) => 1.5,  // Open door
            TileType::Door(false) => f32::INFINITY,  // Closed door blocks
            TileType::Trap(_) => 1.0,  // Traps don't slow movement
            TileType::Bridge => 1.0,
            _ => f32::INFINITY, // Impassable tiles
        }
    }
    
    /// Returns the ASCII character representation of this tile
    pub fn glyph(&self) -> char {
        match self {
            TileType::Wall => '#',
            TileType::Floor => '.',
            TileType::DownStairs => '>',
            TileType::UpStairs => '<',
            TileType::Door(true) => '/',   // Open door
            TileType::Door(false) => '+',  // Closed door
            TileType::Water => '~',
            TileType::Lava => '≈',
            TileType::Trap(true) => '^',   // Visible trap
            TileType::Trap(false) => '.',  // Hidden trap (looks like floor)
            TileType::Bridge => '=',
            TileType::Grass => '"',
            TileType::Tree => '♠',
            TileType::Rock => '○',
            TileType::Sand => '·',
            TileType::Ice => '*',
            TileType::Void => ' ',
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub opaque: Vec<bool>, // Blocks line of sight
    pub depth: i32,
    pub rooms: Vec<Rect>,
    pub corridors: Vec<Vec<(i32, i32)>>,
    pub entrance: (i32, i32),
    pub exit: (i32, i32),
    pub theme: MapTheme,
    pub generation_seed: u64,
    pub tile_content: Vec<Vec<u32>>, // Entity IDs at each tile
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MapTheme {
    Dungeon,
    Cave,
    Forest,
    Desert,
    Ice,
    Volcanic,
    Underwater,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Corridor {
    pub start: (i32, i32),
    pub end: (i32, i32),
    pub points: Vec<(i32, i32)>,
}

impl Map {
    pub fn new(width: i32, height: i32, depth: i32) -> Self {
        Self::new_with_theme(width, height, depth, MapTheme::Dungeon, 0)
    }
    
    pub fn new_with_theme(width: i32, height: i32, depth: i32, theme: MapTheme, seed: u64) -> Self {
        let size = (width * height) as usize;
        Map {
            tiles: vec![TileType::Wall; size],
            width,
            height,
            revealed_tiles: vec![false; size],
            visible_tiles: vec![false; size],
            blocked: vec![true; size],
            opaque: vec![true; size],
            depth,
            rooms: Vec::new(),
            corridors: Vec::new(),
            entrance: (0, 0),
            exit: (0, 0),
            theme,
            generation_seed: seed,
            tile_content: vec![Vec::new(); size],
        }
    }
    
    /// Convert x, y coordinates to array index
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }
    
    /// Convert array index to x, y coordinates
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        let x = (idx % self.width as usize) as i32;
        let y = (idx / self.width as usize) as i32;
        (x, y)
    }
    
    /// Check if coordinates are within map bounds
    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && x < self.width && y >= 0 && y < self.height
    }
    
    /// Check if a tile blocks movement
    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return true;
        }
        let idx = self.xy_idx(x, y);
        self.blocked[idx]
    }
    
    /// Check if a tile blocks line of sight
    pub fn is_opaque(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return true;
        }
        let idx = self.xy_idx(x, y);
        self.opaque[idx]
    }
    
    /// Set a tile at the given coordinates
    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            self.tiles[idx] = tile;
            self.blocked[idx] = tile.blocks_movement();
            self.opaque[idx] = tile.blocks_sight();
        }
    }
    
    /// Get the tile type at the given coordinates
    pub fn get_tile(&self, x: i32, y: i32) -> Option<TileType> {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            Some(self.tiles[idx])
        } else {
            None
        }
    }
    
    /// Fill a rectangular area with a specific tile type
    pub fn fill_rect(&mut self, rect: &Rect, tile: TileType) {
        for y in rect.y1..rect.y2 {
            for x in rect.x1..rect.x2 {
                self.set_tile(x, y, tile);
            }
        }
    }
    
    /// Create a horizontal corridor between two points
    pub fn create_h_corridor(&mut self, x1: i32, x2: i32, y: i32) -> Vec<(i32, i32)> {
        let mut points = Vec::new();
        let start_x = x1.min(x2);
        let end_x = x1.max(x2);
        
        for x in start_x..=end_x {
            self.set_tile(x, y, TileType::Floor);
            points.push((x, y));
        }
        
        points
    }
    
    /// Create a vertical corridor between two points
    pub fn create_v_corridor(&mut self, y1: i32, y2: i32, x: i32) -> Vec<(i32, i32)> {
        let mut points = Vec::new();
        let start_y = y1.min(y2);
        let end_y = y1.max(y2);
        
        for y in start_y..=end_y {
            self.set_tile(x, y, TileType::Floor);
            points.push((x, y));
        }
        
        points
    }
    
    /// Create an L-shaped corridor between two points
    pub fn create_l_corridor(&mut self, start: (i32, i32), end: (i32, i32)) -> Corridor {
        let mut points = Vec::new();
        
        // First, go horizontally
        let h_points = self.create_h_corridor(start.0, end.0, start.1);
        points.extend(h_points);
        
        // Then, go vertically
        let v_points = self.create_v_corridor(start.1, end.1, end.0);
        points.extend(v_points);
        
        Corridor {
            start,
            end,
            points,
        }
    }
    
    /// Get all neighbors of a given position
    pub fn get_neighbors(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        let mut neighbors = Vec::new();
        
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                
                let nx = x + dx;
                let ny = y + dy;
                
                if self.in_bounds(nx, ny) {
                    neighbors.push((nx, ny));
                }
            }
        }
        
        neighbors
    }
    
    /// Get orthogonal neighbors (no diagonals)
    pub fn get_orthogonal_neighbors(&self, x: i32, y: i32) -> Vec<(i32, i32)> {
        let mut neighbors = Vec::new();
        let directions = [(0, -1), (1, 0), (0, 1), (-1, 0)];
        
        for (dx, dy) in directions.iter() {
            let nx = x + dx;
            let ny = y + dy;
            
            if self.in_bounds(nx, ny) {
                neighbors.push((nx, ny));
            }
        }
        
        neighbors
    }
    
    /// Count the number of wall neighbors around a position
    pub fn count_wall_neighbors(&self, x: i32, y: i32) -> usize {
        self.get_neighbors(x, y)
            .iter()
            .filter(|(nx, ny)| {
                let idx = self.xy_idx(*nx, *ny);
                self.tiles[idx] == TileType::Wall
            })
            .count()
    }
    
    /// Find a random floor tile
    pub fn find_random_floor_tile(&self, rng: &mut dyn rand::RngCore) -> Option<(i32, i32)> {
        use rand::Rng;
        
        let floor_tiles: Vec<(i32, i32)> = (0..self.width)
            .flat_map(|x| (0..self.height).map(move |y| (x, y)))
            .filter(|(x, y)| {
                let idx = self.xy_idx(*x, *y);
                self.tiles[idx] == TileType::Floor
            })
            .collect();
        
        if floor_tiles.is_empty() {
            None
        } else {
            let index = rng.gen_range(0..floor_tiles.len());
            Some(floor_tiles[index])
        }
    }
    
    /// Clear visibility for all tiles
    pub fn clear_visibility(&mut self) {
        for visible in self.visible_tiles.iter_mut() {
            *visible = false;
        }
    }
    
    /// Reveal a tile (mark it as seen)
    pub fn reveal_tile(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            self.revealed_tiles[idx] = true;
        }
    }
    
    /// Set a tile as visible
    pub fn set_visible(&mut self, x: i32, y: i32, visible: bool) {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            self.visible_tiles[idx] = visible;
            if visible {
                self.revealed_tiles[idx] = true;
            }
        }
    }
    
    /// Check if a tile is revealed
    pub fn is_revealed(&self, x: i32, y: i32) -> bool {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            self.revealed_tiles[idx]
        } else {
            false
        }
    }
    
    /// Check if a tile is visible
    pub fn is_visible(&self, x: i32, y: i32) -> bool {
        if self.in_bounds(x, y) {
            let idx = self.xy_idx(x, y);
            self.visible_tiles[idx]
        } else {
            false
        }
    }
    
    /// Get the movement cost for a tile
    pub fn get_movement_cost(&self, x: i32, y: i32) -> f32 {
        if let Some(tile) = self.get_tile(x, y) {
            tile.movement_cost()
        } else {
            f32::INFINITY
        }
    }
    
    /// Check if a tile is dangerous
    pub fn is_dangerous(&self, x: i32, y: i32) -> bool {
        if let Some(tile) = self.get_tile(x, y) {
            tile.is_dangerous()
        } else {
            true
        }
    }
    
    /// Populate the blocked array based on current tiles
    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter().enumerate() {
            self.blocked[i] = tile.blocks_movement();
            self.opaque[i] = tile.blocks_sight();
        }
    }
    
    /// Get tile glyph by index
    pub fn get_tile_glyph(&self, idx: usize) -> char {
        self.tiles[idx].glyph()
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Rect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        }
    }
    
    /// Create a rectangle from two corners
    pub fn from_corners(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        Rect {
            x1: x1.min(x2),
            y1: y1.min(y2),
            x2: x1.max(x2),
            y2: y1.max(y2),
        }
    }
    
    /// Check if this rectangle intersects with another
    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }
    
    /// Get the center point of the rectangle
    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
    
    /// Get the width of the rectangle
    pub fn width(&self) -> i32 {
        self.x2 - self.x1
    }
    
    /// Get the height of the rectangle
    pub fn height(&self) -> i32 {
        self.y2 - self.y1
    }
    
    /// Get the area of the rectangle
    pub fn area(&self) -> i32 {
        self.width() * self.height()
    }
    
    /// Check if a point is inside the rectangle
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }
    
    /// Get a random point inside the rectangle
    pub fn random_point(&self, rng: &mut dyn rand::RngCore) -> (i32, i32) {
        use rand::Rng;
        let x = rng.gen_range(self.x1..self.x2);
        let y = rng.gen_range(self.y1..self.y2);
        (x, y)
    }
    
    /// Expand the rectangle by a given amount
    pub fn expand(&self, amount: i32) -> Rect {
        Rect {
            x1: self.x1 - amount,
            y1: self.y1 - amount,
            x2: self.x2 + amount,
            y2: self.y2 + amount,
        }
    }
    
    /// Shrink the rectangle by a given amount
    pub fn shrink(&self, amount: i32) -> Rect {
        Rect {
            x1: self.x1 + amount,
            y1: self.y1 + amount,
            x2: self.x2 - amount,
            y2: self.y2 - amount,
        }
    }
    
    /// Get all points along the perimeter of the rectangle
    pub fn perimeter_points(&self) -> Vec<(i32, i32)> {
        let mut points = Vec::new();
        
        // Top and bottom edges
        for x in self.x1..self.x2 {
            points.push((x, self.y1));
            points.push((x, self.y2 - 1));
        }
        
        // Left and right edges (excluding corners already added)
        for y in (self.y1 + 1)..(self.y2 - 1) {
            points.push((self.x1, y));
            points.push((self.x2 - 1, y));
        }
        
        points
    }
    
    /// Get all points inside the rectangle
    pub fn interior_points(&self) -> Vec<(i32, i32)> {
        let mut points = Vec::new();
        
        for y in self.y1..self.y2 {
            for x in self.x1..self.x2 {
                points.push((x, y));
            }
        }
        
        points
    }
}

// Direction enum for map generation and movement
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Direction {
    pub fn delta(&self) -> (i32, i32) {
        match *self {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::NorthEast => (1, -1),
            Direction::NorthWest => (-1, -1),
            Direction::SouthEast => (1, 1),
            Direction::SouthWest => (-1, 1),
        }
    }
    
    pub fn opposite(&self) -> Direction {
        match *self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
            Direction::NorthEast => Direction::SouthWest,
            Direction::NorthWest => Direction::SouthEast,
            Direction::SouthEast => Direction::NorthWest,
            Direction::SouthWest => Direction::NorthEast,
        }
    }
    
    pub fn cardinals() -> Vec<Direction> {
        vec![Direction::North, Direction::South, Direction::East, Direction::West]
    }
    
    pub fn all_directions() -> Vec<Direction> {
        vec![
            Direction::North, Direction::South, Direction::East, Direction::West,
            Direction::NorthEast, Direction::NorthWest, Direction::SouthEast, Direction::SouthWest
        ]
    }
}