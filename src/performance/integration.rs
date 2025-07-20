use std::time::{Duration, Instant};
use log::{info, warn, error};

use crate::game_state::GameState;
use crate::systems::*;
use super::profiler::{init_profiler, with_profiler};
use super::memory_tracker::{init_memory_tracker, with_memory_tracker};
use super::metrics::GameMetrics;
use super::reporter::PerformanceReporter;

/// Performance integration for the game
pub struct PerformanceIntegration {
    metrics: GameMetrics,
    reporter: PerformanceReporter,
    last_sample_time: Instant,
    sample_interval: Duration,
    enabled: bool,
}

impl PerformanceIntegration {
    /// Create a new performance integration
    pub fn new(target_fps: f64, output_dir: &str) -> Self {
        // Initialize global profiler and memory tracker
        init_profiler(target_fps);
        init_memory_tracker();
        
        PerformanceIntegration {
            metrics: GameMetrics::new(),
            reporter: PerformanceReporter::new(output_dir),
            last_sample_time: Instant::now(),
            sample_interval: Duration::from_millis(100), // Sample every 100ms
            enabled: true,
        }
    }
    
    /// Enable or disable performance monitoring
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        
        with_profiler(|p| p.set_enabled(enabled));
        with_memory_tracker(|t| t.set_enabled(enabled));
        self.reporter.set_enabled(enabled);
        
        if enabled {
            info!("Performance monitoring enabled");
        } else {
            info!("Performance monitoring disabled");
        }
    }
    
    /// Start frame profiling
    pub fn start_frame(&mut self) {
        if !self.enabled {
            return;
        }
        
        with_profiler(|p| p.start_frame());
    }
    
    /// End frame profiling
    pub fn end_frame(&mut self, game_state: &GameState) {
        if !self.enabled {
            return;
        }
        
        with_profiler(|p| p.end_frame());
        
        // Take periodic samples
        if self.last_sample_time.elapsed() >= self.sample_interval {
            self.take_sample(game_state);
            self.last_sample_time = Instant::now();
        }
        
        // Generate reports if needed
        self.check_and_generate_report();
    }
    
    /// Profile a system execution
    pub fn profile_system<F, R>(&mut self, system_name: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if !self.enabled {
            return f();
        }
        
        let start = Instant::now();
        with_profiler(|p| p.start_system(system_name));
        
        let result = f();
        
        let duration = start.elapsed();
        with_profiler(|p| p.end_system());
        self.metrics.record_system_time(system_name, duration);
        
        result
    }
    
    /// Record frame time
    pub fn record_frame_time(&mut self, duration: Duration) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_frame_time(duration);
    }
    
    /// Record entity count
    pub fn record_entity_count(&mut self, count: usize) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_entity_count(count);
    }
    
    /// Record memory allocation
    pub fn record_allocation(&mut self, id: &str, size: u64, category: &str, description: &str) {
        if !self.enabled {
            return;
        }
        
        with_memory_tracker(|t| t.record_allocation(id, size, category, description));
        self.metrics.record_memory_usage(size);
    }
    
    /// Record memory deallocation
    pub fn record_deallocation(&mut self, id: &str) {
        if !self.enabled {
            return;
        }
        
        with_memory_tracker(|t| t.record_deallocation(id));
    }
    
    /// Record input event
    pub fn record_input_event(&mut self) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_input_event();
    }
    
    /// Record render call
    pub fn record_render_call(&mut self) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_render_call();
    }
    
    /// Record AI decision
    pub fn record_ai_decision(&mut self) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_ai_decision();
    }
    
    /// Record pathfinding request
    pub fn record_pathfinding_request(&mut self, path_length: usize) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_pathfinding_request(path_length);
    }
    
    /// Record combat event
    pub fn record_combat_event(&mut self, damage: f32) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_combat_event(damage);
    }
    
    /// Record dungeon generation
    pub fn record_dungeon_generation(&mut self, duration: Duration, room_count: usize) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_dungeon_generation(duration, room_count);
    }
    
    /// Record save operation
    pub fn record_save_operation(&mut self, duration: Duration, size_bytes: u64) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_save_operation(duration, size_bytes);
    }
    
    /// Record load operation
    pub fn record_load_operation(&mut self, duration: Duration) {
        if !self.enabled {
            return;
        }
        
        self.metrics.record_load_operation(duration);
    }
    
    /// Take a performance sample
    fn take_sample(&mut self, game_state: &GameState) {
        // Record current entity count
        let entity_count = game_state.world.entities().join().count();
        self.record_entity_count(entity_count);
        
        // Take memory sample
        with_memory_tracker(|t| t.take_sample());
        
        // Record current memory usage
        with_memory_tracker(|t| {
            let stats = t.get_stats();
            self.metrics.record_memory_usage(stats.current_usage);
        });
    }
    
    /// Check if a report should be generated and generate it
    fn check_and_generate_report(&mut self) {
        if !self.reporter.should_generate_report() {
            return;
        }
        
        // Generate report
        let report_result = with_profiler(|profiler| {
            with_memory_tracker(|memory_tracker| {
                self.reporter.generate_report(profiler, memory_tracker, &self.metrics.collector)
            })
        });
        
        if let Some(Some(report)) = report_result {
            info!("Generated performance report with grade: {}", report.grade);
            
            // Log any critical recommendations
            for rec in &report.recommendations {
                if rec.severity == super::reporter::RecommendationSeverity::Critical {
                    error!("Critical performance issue: {}", rec.title);
                } else if rec.severity == super::reporter::RecommendationSeverity::High {
                    warn!("High priority performance issue: {}", rec.title);
                }
            }
        }
    }
    
    /// Get current performance summary
    pub fn get_performance_summary(&self) -> String {
        let mut summary = String::new();
        
        // Get profiler stats
        if let Some(stats) = with_profiler(|p| p.get_stats()) {
            summary.push_str(&format!("FPS: {:.1} (min: {:.1}, max: {:.1})\n", 
                stats.average_fps, stats.min_fps, stats.max_fps));
            summary.push_str(&format!("Frame Time: {:.2}ms (min: {:.2}ms, max: {:.2}ms)\n",
                stats.average_frame_time.as_secs_f64() * 1000.0,
                stats.min_frame_time.as_secs_f64() * 1000.0,
                stats.max_frame_time.as_secs_f64() * 1000.0));
            summary.push_str(&format!("Frames: {}\n", stats.frame_count));
            summary.push_str(&format!("Warnings: {}\n", stats.warnings.len()));
        }
        
        // Get memory stats
        if let Some(mem_stats) = with_memory_tracker(|t| t.get_stats()) {
            summary.push_str(&format!("Memory: {} (peak: {})\n",
                super::memory_tracker::MemoryTracker::format_size(mem_stats.current_usage),
                super::memory_tracker::MemoryTracker::format_size(mem_stats.peak_usage)));
            summary.push_str(&format!("Allocations: {} (deallocations: {})\n",
                mem_stats.allocation_count, mem_stats.deallocation_count));
        }
        
        // Get metrics summary
        summary.push_str(&self.metrics.get_performance_summary());
        
        summary
    }
    
    /// Get performance grade
    pub fn get_performance_grade(&self) -> String {
        with_profiler(|p| p.get_performance_grade().as_str().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
    
    /// Check if performance is acceptable
    pub fn is_performance_acceptable(&self) -> bool {
        with_profiler(|p| p.is_performance_acceptable())
            .unwrap_or(true)
    }
    
    /// Reset all performance data
    pub fn reset(&mut self) {
        with_profiler(|p| p.reset());
        with_memory_tracker(|t| t.reset());
        self.metrics.collector.reset();
        self.reporter.clear_history();
        info!("Performance monitoring reset");
    }
    
    /// Export performance data
    pub fn export_data(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.reporter.export_csv(filepath)
    }
    
    /// Get recent performance warnings
    pub fn get_recent_warnings(&self, duration: Duration) -> Vec<String> {
        with_profiler(|p| {
            p.get_recent_warnings(duration)
                .into_iter()
                .map(|w| w.message.clone())
                .collect()
        }).unwrap_or_else(Vec::new)
    }
    
    /// Set profiler target FPS
    pub fn set_target_fps(&mut self, target_fps: f64) {
        with_profiler(|p| p.set_target_fps(target_fps));
    }
    
    /// Set memory GC threshold
    pub fn set_memory_gc_threshold(&mut self, threshold: u64) {
        with_memory_tracker(|t| t.set_gc_threshold(threshold));
    }
    
    /// Set report interval
    pub fn set_report_interval(&mut self, interval: Duration) {
        self.reporter.set_report_interval(interval);
    }
    
    /// Enable or disable auto-save reports
    pub fn set_auto_save_reports(&mut self, auto_save: bool) {
        self.reporter.set_auto_save(auto_save);
    }
}

/// Macro for easy system profiling with integration
#[macro_export]
macro_rules! profile_system_integrated {
    ($integration:expr, $system_name:expr, $code:block) => {
        $integration.profile_system($system_name, || $code)
    };
}

/// Performance monitoring wrapper for systems
pub struct ProfiledSystem<T> {
    system: T,
    name: String,
}

impl<T> ProfiledSystem<T> {
    pub fn new(system: T, name: &str) -> Self {
        ProfiledSystem {
            system,
            name: name.to_string(),
        }
    }
}

impl<'a, T> specs::System<'a> for ProfiledSystem<T>
where
    T: specs::System<'a>,
{
    type SystemData = T::SystemData;
    
    fn run(&mut self, data: Self::SystemData) {
        profile_system!(&self.name, {
            self.system.run(data);
        });
    }
    
    fn setup(&mut self, world: &mut specs::World) {
        self.system.setup(world);
    }
    
    fn dispose(self, world: &mut specs::World) {
        self.system.dispose(world);
    }
}