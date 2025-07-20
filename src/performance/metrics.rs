use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Performance metrics collector
pub struct MetricsCollector {
    counters: HashMap<String, u64>,
    gauges: HashMap<String, f64>,
    histograms: HashMap<String, Vec<f64>>,
    timers: HashMap<String, Vec<Duration>>,
    start_time: Instant,
    last_reset: Instant,
}

/// Metric type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
    Timer,
}

/// Metric value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
    Timer(Vec<f64>), // Duration in seconds
}

/// Metric entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub timestamp: u64,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
}

/// Metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub timestamp: u64,
    pub uptime_seconds: f64,
    pub metrics: Vec<Metric>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new() -> Self {
        let now = Instant::now();
        MetricsCollector {
            counters: HashMap::new(),
            gauges: HashMap::new(),
            histograms: HashMap::new(),
            timers: HashMap::new(),
            start_time: now,
            last_reset: now,
        }
    }
    
    /// Increment a counter
    pub fn increment_counter(&mut self, name: &str, value: u64) {
        *self.counters.entry(name.to_string()).or_insert(0) += value;
    }
    
    /// Set a gauge value
    pub fn set_gauge(&mut self, name: &str, value: f64) {
        self.gauges.insert(name.to_string(), value);
    }
    
    /// Add a value to a histogram
    pub fn record_histogram(&mut self, name: &str, value: f64) {
        self.histograms.entry(name.to_string()).or_insert_with(Vec::new).push(value);
    }
    
    /// Record a timer value
    pub fn record_timer(&mut self, name: &str, duration: Duration) {
        self.timers.entry(name.to_string()).or_insert_with(Vec::new).push(duration);
    }
    
    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.get(name).copied().unwrap_or(0)
    }
    
    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauges.get(name).copied()
    }
    
    /// Get histogram values
    pub fn get_histogram(&self, name: &str) -> Option<&Vec<f64>> {
        self.histograms.get(name)
    }
    
    /// Get timer values
    pub fn get_timer(&self, name: &str) -> Option<&Vec<Duration>> {
        self.timers.get(name)
    }
    
    /// Calculate histogram statistics
    pub fn get_histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        if let Some(values) = self.histograms.get(name) {
            if values.is_empty() {
                return None;
            }
            
            let mut sorted_values = values.clone();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let count = sorted_values.len();
            let sum: f64 = sorted_values.iter().sum();
            let mean = sum / count as f64;
            let min = sorted_values[0];
            let max = sorted_values[count - 1];
            
            // Calculate percentiles
            let p50 = percentile(&sorted_values, 0.5);
            let p90 = percentile(&sorted_values, 0.9);
            let p95 = percentile(&sorted_values, 0.95);
            let p99 = percentile(&sorted_values, 0.99);
            
            // Calculate standard deviation
            let variance: f64 = sorted_values.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / count as f64;
            let std_dev = variance.sqrt();
            
            Some(HistogramStats {
                count,
                sum,
                mean,
                min,
                max,
                std_dev,
                p50,
                p90,
                p95,
                p99,
            })
        } else {
            None
        }
    }
    
    /// Calculate timer statistics
    pub fn get_timer_stats(&self, name: &str) -> Option<TimerStats> {
        if let Some(durations) = self.timers.get(name) {
            if durations.is_empty() {
                return None;
            }
            
            let mut sorted_durations = durations.clone();
            sorted_durations.sort();
            
            let count = sorted_durations.len();
            let sum: Duration = sorted_durations.iter().sum();
            let mean = sum / count as u32;
            let min = sorted_durations[0];
            let max = sorted_durations[count - 1];
            
            // Calculate percentiles
            let p50 = duration_percentile(&sorted_durations, 0.5);
            let p90 = duration_percentile(&sorted_durations, 0.9);
            let p95 = duration_percentile(&sorted_durations, 0.95);
            let p99 = duration_percentile(&sorted_durations, 0.99);
            
            Some(TimerStats {
                count,
                sum,
                mean,
                min,
                max,
                p50,
                p90,
                p95,
                p99,
            })
        } else {
            None
        }
    }
    
    /// Get all metric names
    pub fn get_metric_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        names.extend(self.counters.keys().cloned());
        names.extend(self.gauges.keys().cloned());
        names.extend(self.histograms.keys().cloned());
        names.extend(self.timers.keys().cloned());
        names.sort();
        names.dedup();
        names
    }
    
    /// Create a metrics snapshot
    pub fn create_snapshot(&self) -> MetricsSnapshot {
        let mut metrics = Vec::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Add counters
        for (name, value) in &self.counters {
            metrics.push(Metric {
                name: name.clone(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(*value),
                timestamp,
                description: None,
                tags: HashMap::new(),
            });
        }
        
        // Add gauges
        for (name, value) in &self.gauges {
            metrics.push(Metric {
                name: name.clone(),
                metric_type: MetricType::Gauge,
                value: MetricValue::Gauge(*value),
                timestamp,
                description: None,
                tags: HashMap::new(),
            });
        }
        
        // Add histograms
        for (name, values) in &self.histograms {
            metrics.push(Metric {
                name: name.clone(),
                metric_type: MetricType::Histogram,
                value: MetricValue::Histogram(values.clone()),
                timestamp,
                description: None,
                tags: HashMap::new(),
            });
        }
        
        // Add timers
        for (name, durations) in &self.timers {
            let values: Vec<f64> = durations.iter().map(|d| d.as_secs_f64()).collect();
            metrics.push(Metric {
                name: name.clone(),
                metric_type: MetricType::Timer,
                value: MetricValue::Timer(values),
                timestamp,
                description: None,
                tags: HashMap::new(),
            });
        }
        
        MetricsSnapshot {
            timestamp,
            uptime_seconds: self.start_time.elapsed().as_secs_f64(),
            metrics,
        }
    }
    
    /// Reset all metrics
    pub fn reset(&mut self) {
        self.counters.clear();
        self.gauges.clear();
        self.histograms.clear();
        self.timers.clear();
        self.last_reset = Instant::now();
    }
    
    /// Get uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Get time since last reset
    pub fn get_time_since_reset(&self) -> Duration {
        self.last_reset.elapsed()
    }
    
    /// Trim old histogram values
    pub fn trim_histograms(&mut self, max_values: usize) {
        for values in self.histograms.values_mut() {
            if values.len() > max_values {
                values.drain(0..values.len() - max_values);
            }
        }
    }
    
    /// Trim old timer values
    pub fn trim_timers(&mut self, max_values: usize) {
        for durations in self.timers.values_mut() {
            if durations.len() > max_values {
                durations.drain(0..durations.len() - max_values);
            }
        }
    }
    
    /// Export metrics to JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        let snapshot = self.create_snapshot();
        serde_json::to_string_pretty(&snapshot)
    }
    
    /// Import metrics from JSON
    pub fn import_json(&mut self, json: &str) -> Result<(), Box<dyn std::error::Error>> {
        let snapshot: MetricsSnapshot = serde_json::from_str(json)?;
        
        // Clear existing metrics
        self.reset();
        
        // Import metrics
        for metric in snapshot.metrics {
            match metric.value {
                MetricValue::Counter(value) => {
                    self.counters.insert(metric.name, value);
                },
                MetricValue::Gauge(value) => {
                    self.gauges.insert(metric.name, value);
                },
                MetricValue::Histogram(values) => {
                    self.histograms.insert(metric.name, values);
                },
                MetricValue::Timer(values) => {
                    let durations: Vec<Duration> = values.iter()
                        .map(|&v| Duration::from_secs_f64(v))
                        .collect();
                    self.timers.insert(metric.name, durations);
                },
            }
        }
        
        Ok(())
    }
}

/// Histogram statistics
#[derive(Debug, Clone)]
pub struct HistogramStats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub min: f64,
    pub max: f64,
    pub std_dev: f64,
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
}

/// Timer statistics
#[derive(Debug, Clone)]
pub struct TimerStats {
    pub count: usize,
    pub sum: Duration,
    pub mean: Duration,
    pub min: Duration,
    pub max: Duration,
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

/// Calculate percentile for f64 values
fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    
    let index = (p * (sorted_values.len() - 1) as f64).round() as usize;
    sorted_values[index.min(sorted_values.len() - 1)]
}

/// Calculate percentile for Duration values
fn duration_percentile(sorted_durations: &[Duration], p: f64) -> Duration {
    if sorted_durations.is_empty() {
        return Duration::ZERO;
    }
    
    let index = (p * (sorted_durations.len() - 1) as f64).round() as usize;
    sorted_durations[index.min(sorted_durations.len() - 1)]
}

/// Game-specific metrics
pub struct GameMetrics {
    pub collector: MetricsCollector,
}

impl GameMetrics {
    /// Create new game metrics
    pub fn new() -> Self {
        GameMetrics {
            collector: MetricsCollector::new(),
        }
    }
    
    /// Record frame time
    pub fn record_frame_time(&mut self, duration: Duration) {
        self.collector.record_timer("frame_time", duration);
        self.collector.set_gauge("current_fps", 1.0 / duration.as_secs_f64());
    }
    
    /// Record system execution time
    pub fn record_system_time(&mut self, system_name: &str, duration: Duration) {
        self.collector.record_timer(&format!("system_{}", system_name), duration);
    }
    
    /// Record entity count
    pub fn record_entity_count(&mut self, count: usize) {
        self.collector.set_gauge("entity_count", count as f64);
    }
    
    /// Record memory usage
    pub fn record_memory_usage(&mut self, bytes: u64) {
        self.collector.set_gauge("memory_usage_bytes", bytes as f64);
        self.collector.set_gauge("memory_usage_mb", bytes as f64 / 1024.0 / 1024.0);
    }
    
    /// Record input events
    pub fn record_input_event(&mut self) {
        self.collector.increment_counter("input_events", 1);
    }
    
    /// Record render calls
    pub fn record_render_call(&mut self) {
        self.collector.increment_counter("render_calls", 1);
    }
    
    /// Record AI decisions
    pub fn record_ai_decision(&mut self) {
        self.collector.increment_counter("ai_decisions", 1);
    }
    
    /// Record pathfinding requests
    pub fn record_pathfinding_request(&mut self, path_length: usize) {
        self.collector.increment_counter("pathfinding_requests", 1);
        self.collector.record_histogram("path_length", path_length as f64);
    }
    
    /// Record combat events
    pub fn record_combat_event(&mut self, damage: f32) {
        self.collector.increment_counter("combat_events", 1);
        self.collector.record_histogram("damage_dealt", damage as f64);
    }
    
    /// Record dungeon generation time
    pub fn record_dungeon_generation(&mut self, duration: Duration, room_count: usize) {
        self.collector.record_timer("dungeon_generation", duration);
        self.collector.record_histogram("room_count", room_count as f64);
    }
    
    /// Record save/load operations
    pub fn record_save_operation(&mut self, duration: Duration, size_bytes: u64) {
        self.collector.record_timer("save_operation", duration);
        self.collector.record_histogram("save_size_bytes", size_bytes as f64);
    }
    
    /// Record load operation
    pub fn record_load_operation(&mut self, duration: Duration) {
        self.collector.record_timer("load_operation", duration);
    }
    
    /// Get performance summary
    pub fn get_performance_summary(&self) -> String {
        let mut summary = String::new();
        
        // Frame time stats
        if let Some(stats) = self.collector.get_timer_stats("frame_time") {
            summary.push_str(&format!("Frame Time: avg={:.2}ms, p95={:.2}ms, p99={:.2}ms\n",
                stats.mean.as_secs_f64() * 1000.0,
                stats.p95.as_secs_f64() * 1000.0,
                stats.p99.as_secs_f64() * 1000.0));
        }
        
        // FPS
        if let Some(fps) = self.collector.get_gauge("current_fps") {
            summary.push_str(&format!("Current FPS: {:.1}\n", fps));
        }
        
        // Entity count
        if let Some(count) = self.collector.get_gauge("entity_count") {
            summary.push_str(&format!("Entity Count: {}\n", count as u64));
        }
        
        // Memory usage
        if let Some(memory_mb) = self.collector.get_gauge("memory_usage_mb") {
            summary.push_str(&format!("Memory Usage: {:.1} MB\n", memory_mb));
        }
        
        // Event counts
        let input_events = self.collector.get_counter("input_events");
        let render_calls = self.collector.get_counter("render_calls");
        let ai_decisions = self.collector.get_counter("ai_decisions");
        
        summary.push_str(&format!("Events: Input={}, Render={}, AI={}\n", 
            input_events, render_calls, ai_decisions));
        
        summary
    }
}