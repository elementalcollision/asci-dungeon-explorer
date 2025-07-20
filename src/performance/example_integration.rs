use std::time::{Duration, Instant};
use specs::prelude::*;
use log::info;

use crate::game_state::GameState;
use crate::systems::*;
use super::integration::PerformanceIntegration;

/// Example of how to integrate performance monitoring into the main game loop
pub struct GameWithPerformanceMonitoring {
    pub game_state: GameState,
    pub performance: PerformanceIntegration,
    pub dispatcher: Dispatcher<'static, 'static>,
}

impl GameWithPerformanceMonitoring {
    /// Create a new game with performance monitoring
    pub fn new() -> Self {
        let mut game_state = GameState::new();
        let performance = PerformanceIntegration::new(60.0, "performance_reports");
        
        // Create dispatcher with profiled systems
        let dispatcher = DispatcherBuilder::new()
            .with(ProfiledSystem::new(InputSystem, "input"), "input", &[])
            .with(ProfiledSystem::new(PlayerSystem, "player"), "player", &["input"])
            .with(ProfiledSystem::new(AISystem, "ai"), "ai", &["input"])
            .with(ProfiledSystem::new(MovementSystem, "movement"), "movement", &["player", "ai"])
            .with(ProfiledSystem::new(CombatSystem, "combat"), "combat", &["movement"])
            .with(ProfiledSystem::new(RenderSystem, "render"), "render", &["combat"])
            .build();
        
        GameWithPerformanceMonitoring {
            game_state,
            performance,
            dispatcher,
        }
    }
    
    /// Run the main game loop with performance monitoring
    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let target_frame_time = Duration::from_millis(16); // ~60 FPS
        let mut last_frame = Instant::now();
        
        info!("Starting game with performance monitoring");
        
        loop {
            let frame_start = Instant::now();
            
            // Start frame profiling
            self.performance.start_frame();
            
            // Handle input
            self.performance.profile_system("input_handling", || {
                // Input handling code would go here
                self.handle_input();
            });
            
            // Update game systems
            self.performance.profile_system("system_update", || {
                self.dispatcher.dispatch(&self.game_state.world);
                self.game_state.world.maintain();
            });
            
            // Record frame time
            let frame_time = frame_start.elapsed();
            self.performance.record_frame_time(frame_time);
            
            // End frame profiling
            self.performance.end_frame(&self.game_state);
            
            // Check performance and log warnings
            if !self.performance.is_performance_acceptable() {
                let warnings = self.performance.get_recent_warnings(Duration::from_secs(1));
                for warning in warnings {
                    log::warn!("Performance warning: {}", warning);
                }
            }
            
            // Sleep to maintain target frame rate
            let elapsed = frame_start.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
            
            last_frame = frame_start;
            
            // Example: Break after some condition (in real game, this would be user input)
            if self.should_exit() {
                break;
            }
        }
        
        // Print final performance summary
        info!("Final performance summary:\n{}", self.performance.get_performance_summary());
        
        // Export performance data
        if let Err(e) = self.performance.export_data("final_performance_report.csv") {
            log::error!("Failed to export performance data: {}", e);
        }
        
        Ok(())
    }
    
    /// Handle input (placeholder)
    fn handle_input(&mut self) {
        // Record input event
        self.performance.record_input_event();
        
        // Input handling logic would go here
        // For now, just simulate some work
        std::thread::sleep(Duration::from_micros(100));
    }
    
    /// Check if game should exit (placeholder)
    fn should_exit(&self) -> bool {
        // In a real game, this would check for quit conditions
        // For this example, we'll run for a limited time
        false // This would be replaced with actual exit logic
    }
    
    /// Get current performance grade
    pub fn get_performance_grade(&self) -> String {
        self.performance.get_performance_grade()
    }
    
    /// Reset performance monitoring
    pub fn reset_performance(&mut self) {
        self.performance.reset();
    }
    
    /// Enable/disable performance monitoring
    pub fn set_performance_enabled(&mut self, enabled: bool) {
        self.performance.set_enabled(enabled);
    }
    
    /// Set target FPS
    pub fn set_target_fps(&mut self, fps: f64) {
        self.performance.set_target_fps(fps);
    }
}

/// Example system implementations for demonstration
use super::integration::ProfiledSystem;

pub struct InputSystem;
impl<'a> System<'a> for InputSystem {
    type SystemData = ();
    
    fn run(&mut self, _data: Self::SystemData) {
        // Simulate input processing
        std::thread::sleep(Duration::from_micros(50));
    }
}

pub struct PlayerSystem;
impl<'a> System<'a> for PlayerSystem {
    type SystemData = ();
    
    fn run(&mut self, _data: Self::SystemData) {
        // Simulate player system processing
        std::thread::sleep(Duration::from_micros(100));
    }
}

pub struct AISystem;
impl<'a> System<'a> for AISystem {
    type SystemData = ();
    
    fn run(&mut self, _data: Self::SystemData) {
        // Simulate AI processing
        std::thread::sleep(Duration::from_micros(200));
    }
}

pub struct MovementSystem;
impl<'a> System<'a> for MovementSystem {
    type SystemData = ();
    
    fn run(&mut self, _data: Self::SystemData) {
        // Simulate movement processing
        std::thread::sleep(Duration::from_micros(75));
    }
}

pub struct CombatSystem;
impl<'a> System<'a> for CombatSystem {
    type SystemData = ();
    
    fn run(&mut self, _data: Self::SystemData) {
        // Simulate combat processing
        std::thread::sleep(Duration::from_micros(150));
    }
}

pub struct RenderSystem;
impl<'a> System<'a> for RenderSystem {
    type SystemData = ();
    
    fn run(&mut self, _data: Self::SystemData) {
        // Simulate rendering
        std::thread::sleep(Duration::from_micros(300));
    }
}

/// Example usage function
pub fn example_usage() {
    // Create game with performance monitoring
    let mut game = GameWithPerformanceMonitoring::new();
    
    // Configure performance monitoring
    game.set_target_fps(60.0);
    game.performance.set_report_interval(Duration::from_secs(30));
    game.performance.set_auto_save_reports(true);
    
    // Run the game
    if let Err(e) = game.run() {
        log::error!("Game error: {}", e);
    }
    
    // Print final grade
    info!("Final performance grade: {}", game.get_performance_grade());
}