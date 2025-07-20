use rand::Rng;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GameError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Game error: {0}")]
    Game(String),
}

pub type Result<T> = std::result::Result<T, GameError>;

// Random number utilities
pub fn roll_dice(num: i32, sides: i32) -> i32 {
    let mut rng = rand::thread_rng();
    let mut total = 0;
    for _ in 0..num {
        total += rng.gen_range(1..=sides);
    }
    total
}

// Distance calculation
pub fn distance(x1: i32, y1: i32, x2: i32, y2: i32) -> f32 {
    let dx = (x2 - x1) as f32;
    let dy = (y2 - y1) as f32;
    (dx * dx + dy * dy).sqrt()
}

// Direction to delta conversion
pub fn direction_to_delta(dx: i32, dy: i32) -> (i32, i32) {
    let mut ndx = 0;
    let mut ndy = 0;
    
    if dx > 0 { ndx = 1; }
    else if dx < 0 { ndx = -1; }
    
    if dy > 0 { ndy = 1; }
    else if dy < 0 { ndy = -1; }
    
    (ndx, ndy)
}