use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

// Game log resource
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct GameLog {
    pub entries: VecDeque<String>,
    pub max_entries: usize,
}

impl GameLog {
    pub fn new(max_entries: usize) -> Self {
        GameLog {
            entries: VecDeque::with_capacity(max_entries),
            max_entries,
        }
    }
    
    pub fn add_entry(&mut self, entry: String) {
        self.entries.push_back(entry);
        if self.entries.len() > self.max_entries {
            self.entries.pop_front();
        }
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

// Random number generator resource
#[derive(Serialize, Deserialize, Clone)]
pub struct RandomNumberGenerator {
    pub seed: u64,
}

impl RandomNumberGenerator {
    pub fn new(seed: u64) -> Self {
        RandomNumberGenerator { seed }
    }
    
    pub fn new_with_random_seed() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        RandomNumberGenerator { seed: rng.gen() }
    }
    
    pub fn roll_dice(&mut self, num: i32, sides: i32) -> i32 {
        use rand::Rng;
        use rand_chacha::ChaCha8Rng;
        use rand::SeedableRng;
        
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.seed = rng.gen();
        
        let mut total = 0;
        for _ in 0..num {
            total += rng.gen_range(1..=sides);
        }
        total
    }
    
    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        use rand::Rng;
        use rand_chacha::ChaCha8Rng;
        use rand::SeedableRng;
        
        let mut rng = ChaCha8Rng::seed_from_u64(self.seed);
        self.seed = rng.gen();
        
        rng.gen_range(min..=max)
    }
}

// Player resource
#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerResource {
    pub entity: usize, // Entity ID
    pub name: String,
}

// Map resource is already defined in the map module

// Game state resource
#[derive(Serialize, Deserialize, Clone)]
pub struct GameStateResource {
    pub turn_count: u32,
    pub depth: i32,
    pub game_over: bool,
}

impl Default for GameStateResource {
    fn default() -> Self {
        GameStateResource {
            turn_count: 0,
            depth: 1,
            game_over: false,
        }
    }
}