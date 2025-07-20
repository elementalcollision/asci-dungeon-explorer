use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use log::{info, warn, error};

/// Performance profiler for tracking system performance
pub struct PerformanceProfiler {
    enabled: bool,
    frame_times: Vec<Duration>,
    system_times: HashMap<String, Vec<Duration>>,
    current_frame_start: Option<Instant>,
    current_system_start: Option<(String, Instant)>,
    max_samples: usize,
    frame_count: u64,
    total_time: Duration,
    min_frame_time: Duration,
    max_frame_time: Duration,
    target_fps: f64,
    performance_warnings: Vec<PerformanceWarning>,
}

/// Performance warning
#[derive(Debug, Clone)]
pub struct PerformanceWarning {
    pub timestamp: Instant,
    pub warning_type: WarningType,
    pub message: String,
    pub value: f64,
    pub threshold: f64,
}

/// Warning type
#[derive(Debug, Clone, PartialEq)]
pub enum WarningType {
    LowFrameRate,
    HighFrameTime,
    SlowSystem,
    MemoryUsage,
    SystemStall,
}

/// Performance statistics
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub average_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub average_frame_time: Duration,
    pub min_frame_time: Duration,
    pub max_frame_time: Duration,
    pub frame_count: u64,
    pub total_time: Duration,
    pub system_stats: HashMap<String, SystemStats>,
    pub warnings: Vec<PerformanceWarning>,
}

/// System performance statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub total_time: Duration,
    pub call_count: u64,
    pub percentage_of_frame: f64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new(target_fps: f64) -> Self {
        PerformanceProfiler {
            enabled: true,
            frame_times: Vec::new(),
            system_times: HashMap::new(),
            current_frame_start: None,
            current_system_start: None,
            max_samples: 1000,
            frame_count: 0,
            total_time: Duration::ZERO,
            min_frame_time: Duration::from_secs(1),
            max_frame_time: Duration::ZERO,
            target_fps,
            performance_warnings: Vec::new(),
        }
    }
    
    /// Enable or disable profiling
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("Performance profiling enabled");
        } else {
            info!("Performance profiling disabled");
        }
    }
    
    /// Check if profiling is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
    
    /// Start frame timing
    pub fn start_frame(&mut self) {
        if !self.enabled {
            return;
        }
        
        self.current_frame_start = Some(Instant::now());
    }
    
    /// End frame timing
    pub fn end_frame(&mut self) {
        if !self.enabled {
            return;
        }
        
        if let Some(start) = self.current_frame_start.take() {
            let frame_time = start.elapsed();
            
            // Update frame statistics
            self.frame_times.push(frame_time);
            self.frame_count += 1;
            self.total_time += frame_time;
            
            // Update min/max frame times
            if frame_time < self.min_frame_time {
                self.min_frame_time = frame_time;
            }
            if frame_time > self.max_frame_time {
                self.max_frame_time = frame_time;
            }
            
            // Trim samples if needed
            if self.frame_times.len() > self.max_samples {
                self.frame_times.remove(0);
            }
            
            // Check for performance warnings
            self.check_frame_warnings(frame_time);
        }
    }
    
    /// Start system timing
    pub fn start_system(&mut self, system_name: &str) {
        if !self.enabled {
            return;
        }
        
        self.current_system_start = Some((system_name.to_string(), Instant::now()));
    }
    
    /// End system timing
    pub fn end_system(&mut self) {
        if !self.enabled {
            return;
        }
        
        if let Some((system_name, start)) = self.current_system_start.take() {
            let system_time = start.elapsed();
            
            // Add to system times
            let times = self.system_times.entry(system_name.clone()).or_insert_with(Vec::new);
            times.push(system_time);
            
            // Trim samples if needed
            if times.len() > self.max_samples {
                times.remove(0);
            }
            
            // Check for system warnings
            self.check_system_warnings(&system_name, system_time);
        }
    }
    
    /// Get current performance statistics
    pub fn get_stats(&self) -> PerformanceStats {
        let mut system_stats = HashMap::new();
        
        // Calculate system statistics
        for (system_name, times) in &self.system_times {
            if !times.is_empty() {
                let total_time: Duration = times.iter().sum();
                let average_time = total_time / times.len() as u32;
                let min_time = *times.iter().min().unwrap_or(&Duration::ZERO);
                let max_time = *times.iter().max().unwrap_or(&Duration::ZERO);
                
                // Calculate percentage of frame time
                let percentage_of_frame = if !self.frame_times.is_empty() {
                    let avg_frame_time: Duration = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
                    if avg_frame_time.as_nanos() > 0 {
                        (average_time.as_nanos() as f64 / avg_frame_time.as_nanos() as f64) * 100.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                
                system_stats.insert(system_name.clone(), SystemStats {
                    average_time,
                    min_time,
                    max_time,
                    total_time,
                    call_count: times.len() as u64,
                    percentage_of_frame,
                });
            }
        }
        
        // Calculate frame statistics
        let (average_fps, min_fps, max_fps, average_frame_time) = if !self.frame_times.is_empty() {
            let total_frame_time: Duration = self.frame_times.iter().sum();
            let avg_frame_time = total_frame_time / self.frame_times.len() as u32;
            
            let avg_fps = if avg_frame_time.as_secs_f64() > 0.0 {
                1.0 / avg_frame_time.as_secs_f64()
            } else {
                0.0
            };
            
            let min_fps = if self.max_frame_time.as_secs_f64() > 0.0 {
                1.0 / self.max_frame_time.as_secs_f64()
            } else {
                0.0
            };
            
            let max_fps = if self.min_frame_time.as_secs_f64() > 0.0 {
                1.0 / self.min_frame_time.as_secs_f64()
            } else {
                0.0
            };
            
            (avg_fps, min_fps, max_fps, avg_frame_time)
        } else {
            (0.0, 0.0, 0.0, Duration::ZERO)
        };
        
        PerformanceStats {
            average_fps,
            min_fps,
            max_fps,
            average_frame_time,
            min_frame_time: self.min_frame_time,
            max_frame_time: self.max_frame_time,
            frame_count: self.frame_count,
            total_time: self.total_time,
            system_stats,
            warnings: self.performance_warnings.clone(),
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.frame_times.clear();
        self.system_times.clear();
        self.frame_count = 0;
        self.total_time = Duration::ZERO;
        self.min_frame_time = Duration::from_secs(1);
        self.max_frame_time = Duration::ZERO;
        self.performance_warnings.clear();
        info!("Performance profiler reset");
    }
    
    /// Set maximum number of samples to keep
    pub fn set_max_samples(&mut self, max_samples: usize) {
        self.max_samples = max_samples;
        
        // Trim existing samples if needed
        if self.frame_times.len() > max_samples {
            self.frame_times.drain(0..self.frame_times.len() - max_samples);
        }
        
        for times in self.system_times.values_mut() {
            if times.len() > max_samples {
                times.drain(0..times.len() - max_samples);
            }
        }
    }
    
    /// Set target FPS for warnings
    pub fn set_target_fps(&mut self, target_fps: f64) {
        self.target_fps = target_fps;
    }
    
    /// Check for frame-related performance warnings
    fn check_frame_warnings(&mut self, frame_time: Duration) {
        let fps = if frame_time.as_secs_f64() > 0.0 {
            1.0 / frame_time.as_secs_f64()
        } else {
            0.0
        };
        
        // Check for low frame rate
        if fps < self.target_fps * 0.8 {
            self.add_warning(WarningType::LowFrameRate, 
                format!("Frame rate dropped to {:.1} FPS (target: {:.1})", fps, self.target_fps),
                fps, self.target_fps * 0.8);
        }
        
        // Check for high frame time
        let target_frame_time = 1.0 / self.target_fps;
        if frame_time.as_secs_f64() > target_frame_time * 1.5 {
            self.add_warning(WarningType::HighFrameTime,
                format!("Frame time exceeded {:.2}ms (target: {:.2}ms)", 
                    frame_time.as_secs_f64() * 1000.0, target_frame_time * 1000.0),
                frame_time.as_secs_f64() * 1000.0, target_frame_time * 1000.0);
        }
    }
    
    /// Check for system-related performance warnings
    fn check_system_warnings(&mut self, system_name: &str, system_time: Duration) {
        let target_frame_time = Duration::from_secs_f64(1.0 / self.target_fps);
        
        // Check if system is taking too much time
        if system_time > target_frame_time / 4 {
            self.add_warning(WarningType::SlowSystem,
                format!("System '{}' took {:.2}ms (>{:.1}% of frame budget)", 
                    system_name, 
                    system_time.as_secs_f64() * 1000.0,
                    (system_time.as_secs_f64() / target_frame_time.as_secs_f64()) * 100.0),
                system_time.as_secs_f64() * 1000.0,
                (target_frame_time.as_secs_f64() / 4.0) * 1000.0);
        }
        
        // Check for system stalls (very long execution)
        if system_time > target_frame_time {
            self.add_warning(WarningType::SystemStall,
                format!("System '{}' stalled for {:.2}ms (>100% of frame budget)", 
                    system_name, system_time.as_secs_f64() * 1000.0),
                system_time.as_secs_f64() * 1000.0,
                target_frame_time.as_secs_f64() * 1000.0);
        }
    }
    
    /// Add a performance warning
    fn add_warning(&mut self, warning_type: WarningType, message: String, value: f64, threshold: f64) {
        let warning = PerformanceWarning {
            timestamp: Instant::now(),
            warning_type,
            message: message.clone(),
            value,
            threshold,
        };
        
        self.performance_warnings.push(warning);
        
        // Trim warnings if too many
        if self.performance_warnings.len() > 100 {
            self.performance_warnings.remove(0);
        }
        
        // Log warning
        warn!("Performance warning: {}", message);
    }
    
    /// Get recent warnings
    pub fn get_recent_warnings(&self, duration: Duration) -> Vec<&PerformanceWarning> {
        let cutoff = Instant::now() - duration;
        self.performance_warnings.iter()
            .filter(|w| w.timestamp > cutoff)
            .collect()
    }
    
    /// Clear warnings
    pub fn clear_warnings(&mut self) {
        self.performance_warnings.clear();
    }
    
    /// Get average FPS over recent frames
    pub fn get_recent_fps(&self, sample_count: usize) -> f64 {
        if self.frame_times.is_empty() {
            return 0.0;
        }
        
        let samples = std::cmp::min(sample_count, self.frame_times.len());
        let recent_times = &self.frame_times[self.frame_times.len() - samples..];
        
        let total_time: Duration = recent_times.iter().sum();
        let avg_time = total_time / samples as u32;
        
        if avg_time.as_secs_f64() > 0.0 {
            1.0 / avg_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get system performance percentage
    pub fn get_system_percentage(&self, system_name: &str) -> f64 {
        if let Some(times) = self.system_times.get(system_name) {
            if !times.is_empty() && !self.frame_times.is_empty() {
                let avg_system_time: Duration = times.iter().sum::<Duration>() / times.len() as u32;
                let avg_frame_time: Duration = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
                
                if avg_frame_time.as_nanos() > 0 {
                    (avg_system_time.as_nanos() as f64 / avg_frame_time.as_nanos() as f64) * 100.0
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    /// Check if performance is acceptable
    pub fn is_performance_acceptable(&self) -> bool {
        let recent_fps = self.get_recent_fps(60); // Check last 60 frames
        recent_fps >= self.target_fps * 0.9 // Allow 10% tolerance
    }
    
    /// Get performance grade
    pub fn get_performance_grade(&self) -> PerformanceGrade {
        let recent_fps = self.get_recent_fps(60);
        let target = self.target_fps;
        
        if recent_fps >= target * 0.95 {
            PerformanceGrade::Excellent
        } else if recent_fps >= target * 0.85 {
            PerformanceGrade::Good
        } else if recent_fps >= target * 0.70 {
            PerformanceGrade::Fair
        } else if recent_fps >= target * 0.50 {
            PerformanceGrade::Poor
        } else {
            PerformanceGrade::Unacceptable
        }
    }
}

/// Performance grade
#[derive(Debug, Clone, PartialEq)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    Unacceptable,
}

impl PerformanceGrade {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "Excellent",
            PerformanceGrade::Good => "Good",
            PerformanceGrade::Fair => "Fair",
            PerformanceGrade::Poor => "Poor",
            PerformanceGrade::Unacceptable => "Unacceptable",
        }
    }
    
    pub fn color(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "green",
            PerformanceGrade::Good => "cyan",
            PerformanceGrade::Fair => "yellow",
            PerformanceGrade::Poor => "red",
            PerformanceGrade::Unacceptable => "magenta",
        }
    }
}

/// Global profiler instance
static GLOBAL_PROFILER: Mutex<Option<PerformanceProfiler>> = Mutex::new(None);

/// Initialize global profiler
pub fn init_profiler(target_fps: f64) {
    let mut profiler = GLOBAL_PROFILER.lock().unwrap();
    *profiler = Some(PerformanceProfiler::new(target_fps));
}

/// Get global profiler
pub fn with_profiler<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut PerformanceProfiler) -> R,
{
    if let Ok(mut profiler) = GLOBAL_PROFILER.lock() {
        if let Some(ref mut p) = *profiler {
            Some(f(p))
        } else {
            None
        }
    } else {
        None
    }
}

/// Macro for easy system profiling
#[macro_export]
macro_rules! profile_system {
    ($system_name:expr, $code:block) => {
        $crate::performance::with_profiler(|p| p.start_system($system_name));
        let result = $code;
        $crate::performance::with_profiler(|p| p.end_system());
        result
    };
}

/// Macro for easy frame profiling
#[macro_export]
macro_rules! profile_frame {
    ($code:block) => {
        $crate::performance::with_profiler(|p| p.start_frame());
        let result = $code;
        $crate::performance::with_profiler(|p| p.end_frame());
        result
    };
}