use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Frame timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameStats {
    pub frame_time: Duration,
    pub fps: f64,
    pub delta_time: Duration,
    pub frame_number: u64,
    pub timestamp: Instant,
}

/// Frame timer for tracking game loop performance
pub struct FrameTimer {
    last_frame_time: Instant,
    frame_history: VecDeque<FrameStats>,
    max_history_size: usize,
    frame_count: u64,
    total_time: Duration,
    target_fps: f64,
    target_frame_time: Duration,
    
    // Statistics
    min_frame_time: Duration,
    max_frame_time: Duration,
    avg_frame_time: Duration,
    current_fps: f64,
    
    // Frame time buckets for histogram
    frame_time_buckets: [u32; 10],
    bucket_ranges: [Duration; 10],
}

impl FrameTimer {
    /// Create a new frame timer
    pub fn new(target_fps: f64) -> Self {
        let target_frame_time = Duration::from_secs_f64(1.0 / target_fps);
        
        // Create buckets for frame time histogram (in milliseconds)
        let bucket_ranges = [
            Duration::from_millis(1),   // 0-1ms
            Duration::from_millis(5),   // 1-5ms
            Duration::from_millis(10),  // 5-10ms
            Duration::from_millis(16),  // 10-16ms (60 FPS)
            Duration::from_millis(20),  // 16-20ms
            Duration::from_millis(33),  // 20-33ms (30 FPS)
            Duration::from_millis(50),  // 33-50ms (20 FPS)
            Duration::from_millis(100), // 50-100ms (10 FPS)
            Duration::from_millis(200), // 100-200ms (5 FPS)
            Duration::MAX,              // 200ms+ (very slow)
        ];
        
        FrameTimer {
            last_frame_time: Instant::now(),
            frame_history: VecDeque::new(),
            max_history_size: 300, // Keep 5 seconds at 60 FPS
            frame_count: 0,
            total_time: Duration::ZERO,
            target_fps,
            target_frame_time,
            min_frame_time: Duration::MAX,
            max_frame_time: Duration::ZERO,
            avg_frame_time: Duration::ZERO,
            current_fps: 0.0,
            frame_time_buckets: [0; 10],
            bucket_ranges,
        }
    }
    
    /// Mark the start of a new frame and return frame statistics
    pub fn tick(&mut self) -> FrameStats {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        let delta_time = frame_time;
        
        self.last_frame_time = now;
        self.frame_count += 1;
        self.total_time += frame_time;
        
        // Update statistics
        self.min_frame_time = self.min_frame_time.min(frame_time);
        self.max_frame_time = self.max_frame_time.max(frame_time);
        
        // Calculate current FPS
        self.current_fps = if frame_time.as_secs_f64() > 0.0 {
            1.0 / frame_time.as_secs_f64()
        } else {
            0.0
        };
        
        // Update frame time histogram
        self.update_histogram(frame_time);
        
        // Create frame stats
        let stats = FrameStats {
            frame_time,
            fps: self.current_fps,
            delta_time,
            frame_number: self.frame_count,
            timestamp: now,
        };
        
        // Add to history
        self.frame_history.push_back(stats.clone());
        
        // Trim history if needed
        while self.frame_history.len() > self.max_history_size {
            self.frame_history.pop_front();
        }
        
        // Update average frame time from recent history
        self.update_average_frame_time();
        
        stats
    }
    
    /// Update the frame time histogram
    fn update_histogram(&mut self, frame_time: Duration) {
        for (i, &bucket_max) in self.bucket_ranges.iter().enumerate() {
            if frame_time <= bucket_max {
                self.frame_time_buckets[i] += 1;
                break;
            }
        }
    }
    
    /// Update average frame time from recent history
    fn update_average_frame_time(&mut self) {
        if self.frame_history.is_empty() {
            return;
        }
        
        let sum: Duration = self.frame_history.iter()
            .map(|stats| stats.frame_time)
            .sum();
        
        self.avg_frame_time = sum / self.frame_history.len() as u32;
    }
    
    /// Get current FPS
    pub fn get_fps(&self) -> f64 {
        self.current_fps
    }
    
    /// Get average FPS over recent history
    pub fn get_average_fps(&self) -> f64 {
        if self.avg_frame_time.as_secs_f64() > 0.0 {
            1.0 / self.avg_frame_time.as_secs_f64()
        } else {
            0.0
        }
    }
    
    /// Get minimum frame time
    pub fn get_min_frame_time(&self) -> Duration {
        self.min_frame_time
    }
    
    /// Get maximum frame time
    pub fn get_max_frame_time(&self) -> Duration {
        self.max_frame_time
    }
    
    /// Get average frame time
    pub fn get_average_frame_time(&self) -> Duration {
        self.avg_frame_time
    }
    
    /// Get target FPS
    pub fn get_target_fps(&self) -> f64 {
        self.target_fps
    }
    
    /// Get target frame time
    pub fn get_target_frame_time(&self) -> Duration {
        self.target_frame_time
    }
    
    /// Get total frame count
    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }
    
    /// Get total elapsed time
    pub fn get_total_time(&self) -> Duration {
        self.total_time
    }
    
    /// Get frame history
    pub fn get_frame_history(&self) -> &VecDeque<FrameStats> {
        &self.frame_history
    }
    
    /// Get frame time histogram
    pub fn get_frame_time_histogram(&self) -> &[u32; 10] {
        &self.frame_time_buckets
    }
    
    /// Get histogram bucket ranges
    pub fn get_histogram_ranges(&self) -> &[Duration; 10] {
        &self.bucket_ranges
    }
    
    /// Check if the current frame rate is below target
    pub fn is_below_target_fps(&self) -> bool {
        self.current_fps < self.target_fps * 0.9 // 10% tolerance
    }
    
    /// Check if frame time is consistently high
    pub fn is_frame_time_unstable(&self) -> bool {
        if self.frame_history.len() < 10 {
            return false;
        }
        
        // Check if recent frame times vary significantly
        let recent_frames: Vec<_> = self.frame_history.iter()
            .rev()
            .take(10)
            .collect();
        
        let min_recent = recent_frames.iter()
            .map(|stats| stats.frame_time)
            .min()
            .unwrap_or(Duration::ZERO);
        
        let max_recent = recent_frames.iter()
            .map(|stats| stats.frame_time)
            .max()
            .unwrap_or(Duration::ZERO);
        
        // Consider unstable if max is more than 2x min
        max_recent > min_recent * 2
    }
    
    /// Get frame time percentile from history
    pub fn get_frame_time_percentile(&self, percentile: f64) -> Duration {
        if self.frame_history.is_empty() {
            return Duration::ZERO;
        }
        
        let mut frame_times: Vec<Duration> = self.frame_history.iter()
            .map(|stats| stats.frame_time)
            .collect();
        
        frame_times.sort();
        
        let index = ((frame_times.len() as f64 - 1.0) * percentile / 100.0) as usize;
        frame_times[index.min(frame_times.len() - 1)]
    }
    
    /// Get performance grade based on frame rate consistency
    pub fn get_performance_grade(&self) -> PerformanceGrade {
        let avg_fps = self.get_average_fps();
        let target_fps = self.target_fps;
        let fps_ratio = avg_fps / target_fps;
        
        let p99_frame_time = self.get_frame_time_percentile(99.0);
        let target_frame_time = self.target_frame_time;
        let frame_time_ratio = p99_frame_time.as_secs_f64() / target_frame_time.as_secs_f64();
        
        if fps_ratio >= 0.95 && frame_time_ratio <= 1.1 {
            PerformanceGrade::Excellent
        } else if fps_ratio >= 0.85 && frame_time_ratio <= 1.3 {
            PerformanceGrade::Good
        } else if fps_ratio >= 0.70 && frame_time_ratio <= 1.5 {
            PerformanceGrade::Fair
        } else if fps_ratio >= 0.50 && frame_time_ratio <= 2.0 {
            PerformanceGrade::Poor
        } else {
            PerformanceGrade::Terrible
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.frame_history.clear();
        self.frame_count = 0;
        self.total_time = Duration::ZERO;
        self.min_frame_time = Duration::MAX;
        self.max_frame_time = Duration::ZERO;
        self.avg_frame_time = Duration::ZERO;
        self.current_fps = 0.0;
        self.frame_time_buckets = [0; 10];
        self.last_frame_time = Instant::now();
    }
    
    /// Set target FPS
    pub fn set_target_fps(&mut self, target_fps: f64) {
        self.target_fps = target_fps;
        self.target_frame_time = Duration::from_secs_f64(1.0 / target_fps);
    }
    
    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // Trim current history if needed
        while self.frame_history.len() > self.max_history_size {
            self.frame_history.pop_front();
        }
    }
    
    /// Get detailed frame timing report
    pub fn get_timing_report(&self) -> FrameTimingReport {
        FrameTimingReport {
            current_fps: self.current_fps,
            average_fps: self.get_average_fps(),
            target_fps: self.target_fps,
            min_frame_time: self.min_frame_time,
            max_frame_time: self.max_frame_time,
            avg_frame_time: self.avg_frame_time,
            target_frame_time: self.target_frame_time,
            frame_count: self.frame_count,
            total_time: self.total_time,
            p50_frame_time: self.get_frame_time_percentile(50.0),
            p95_frame_time: self.get_frame_time_percentile(95.0),
            p99_frame_time: self.get_frame_time_percentile(99.0),
            performance_grade: self.get_performance_grade(),
            is_below_target: self.is_below_target_fps(),
            is_unstable: self.is_frame_time_unstable(),
            histogram: self.frame_time_buckets.clone(),
        }
    }
}

/// Performance grade based on frame rate consistency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerformanceGrade {
    Excellent,
    Good,
    Fair,
    Poor,
    Terrible,
}

impl PerformanceGrade {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "Excellent",
            PerformanceGrade::Good => "Good",
            PerformanceGrade::Fair => "Fair",
            PerformanceGrade::Poor => "Poor",
            PerformanceGrade::Terrible => "Terrible",
        }
    }
    
    pub fn get_color(&self) -> &'static str {
        match self {
            PerformanceGrade::Excellent => "green",
            PerformanceGrade::Good => "cyan",
            PerformanceGrade::Fair => "yellow",
            PerformanceGrade::Poor => "red",
            PerformanceGrade::Terrible => "magenta",
        }
    }
}

/// Detailed frame timing report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameTimingReport {
    pub current_fps: f64,
    pub average_fps: f64,
    pub target_fps: f64,
    pub min_frame_time: Duration,
    pub max_frame_time: Duration,
    pub avg_frame_time: Duration,
    pub target_frame_time: Duration,
    pub frame_count: u64,
    pub total_time: Duration,
    pub p50_frame_time: Duration,
    pub p95_frame_time: Duration,
    pub p99_frame_time: Duration,
    pub performance_grade: PerformanceGrade,
    pub is_below_target: bool,
    pub is_unstable: bool,
    pub histogram: [u32; 10],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_frame_timer_basic() {
        let mut timer = FrameTimer::new(60.0);
        
        // Simulate a few frames
        for _ in 0..5 {
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
            let stats = timer.tick();
            assert!(stats.frame_time >= Duration::from_millis(16));
        }
        
        assert_eq!(timer.get_frame_count(), 5);
        assert!(timer.get_average_fps() > 0.0);
    }
    
    #[test]
    fn test_performance_grade() {
        let mut timer = FrameTimer::new(60.0);
        
        // Simulate consistent 60 FPS
        for _ in 0..10 {
            timer.tick();
            thread::sleep(Duration::from_millis(16));
        }
        
        let grade = timer.get_performance_grade();
        // Grade should be good or excellent for consistent timing
        assert!(matches!(grade, PerformanceGrade::Good | PerformanceGrade::Excellent));
    }
    
    #[test]
    fn test_histogram() {
        let mut timer = FrameTimer::new(60.0);
        
        // Add some frame times
        timer.tick();
        thread::sleep(Duration::from_millis(10));
        timer.tick();
        thread::sleep(Duration::from_millis(20));
        timer.tick();
        
        let histogram = timer.get_frame_time_histogram();
        
        // Should have entries in the histogram
        let total_frames: u32 = histogram.iter().sum();
        assert!(total_frames > 0);
    }
    
    #[test]
    fn test_percentiles() {
        let mut timer = FrameTimer::new(60.0);
        
        // Add frames with known timings
        for i in 1..=10 {
            timer.tick();
            thread::sleep(Duration::from_millis(i * 2));
        }
        
        let p50 = timer.get_frame_time_percentile(50.0);
        let p95 = timer.get_frame_time_percentile(95.0);
        
        // P95 should be higher than P50
        assert!(p95 >= p50);
    }
}