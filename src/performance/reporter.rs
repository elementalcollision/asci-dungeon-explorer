use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use log::{info, warn, error};

use super::profiler::{PerformanceProfiler, PerformanceStats, PerformanceGrade};
use super::metrics::{MetricsCollector, MetricsSnapshot};
use super::memory_tracker::{MemoryTracker, MemoryStats};

/// Performance report generator
pub struct PerformanceReporter {
    enabled: bool,
    report_interval: Duration,
    last_report: Instant,
    output_directory: String,
    auto_save: bool,
    report_history: Vec<PerformanceReport>,
    max_history: usize,
}

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: u64,
    pub duration_seconds: f64,
    pub performance_stats: SerializablePerformanceStats,
    pub memory_stats: SerializableMemoryStats,
    pub metrics_snapshot: MetricsSnapshot,
    pub system_info: SystemInfo,
    pub recommendations: Vec<PerformanceRecommendation>,
    pub grade: String,
}

/// Serializable performance stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializablePerformanceStats {
    pub average_fps: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub average_frame_time_ms: f64,
    pub min_frame_time_ms: f64,
    pub max_frame_time_ms: f64,
    pub frame_count: u64,
    pub total_time_seconds: f64,
    pub system_stats: HashMap<String, SerializableSystemStats>,
    pub warning_count: usize,
}

/// Serializable system stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableSystemStats {
    pub average_time_ms: f64,
    pub min_time_ms: f64,
    pub max_time_ms: f64,
    pub total_time_ms: f64,
    pub call_count: u64,
    pub percentage_of_frame: f64,
}

/// Serializable memory stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableMemoryStats {
    pub current_usage_mb: f64,
    pub peak_usage_mb: f64,
    pub total_allocated_mb: f64,
    pub total_deallocated_mb: f64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub average_allocation_size_kb: f64,
    pub fragmentation_ratio: f64,
    pub categories: HashMap<String, SerializableCategoryStats>,
}

/// Serializable category stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableCategoryStats {
    pub current_mb: f64,
    pub count: u64,
    pub average_size_kb: f64,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub architecture: String,
    pub cpu_count: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub game_version: String,
    pub build_type: String,
}

/// Performance recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceRecommendation {
    pub category: String,
    pub severity: RecommendationSeverity,
    pub title: String,
    pub description: String,
    pub suggested_action: String,
    pub impact: String,
}

/// Recommendation severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl PerformanceReporter {
    /// Create a new performance reporter
    pub fn new(output_directory: &str) -> Self {
        PerformanceReporter {
            enabled: true,
            report_interval: Duration::from_secs(60), // 1 minute
            last_report: Instant::now(),
            output_directory: output_directory.to_string(),
            auto_save: true,
            report_history: Vec::new(),
            max_history: 100,
        }
    }
    
    /// Enable or disable reporting
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("Performance reporting enabled");
        } else {
            info!("Performance reporting disabled");
        }
    }
    
    /// Set report interval
    pub fn set_report_interval(&mut self, interval: Duration) {
        self.report_interval = interval;
    }
    
    /// Set auto-save
    pub fn set_auto_save(&mut self, auto_save: bool) {
        self.auto_save = auto_save;
    }
    
    /// Check if it's time to generate a report
    pub fn should_generate_report(&self) -> bool {
        self.enabled && self.last_report.elapsed() >= self.report_interval
    }
    
    /// Generate a performance report
    pub fn generate_report(
        &mut self,
        profiler: &PerformanceProfiler,
        memory_tracker: &MemoryTracker,
        metrics_collector: &MetricsCollector,
    ) -> PerformanceReport {
        let now = Instant::now();
        let duration = now.duration_since(self.last_report);
        self.last_report = now;
        
        // Get performance stats
        let perf_stats = profiler.get_stats();
        let serializable_perf_stats = self.convert_performance_stats(&perf_stats);
        
        // Get memory stats
        let mem_stats = memory_tracker.get_stats();
        let serializable_mem_stats = self.convert_memory_stats(&mem_stats);
        
        // Get metrics snapshot
        let metrics_snapshot = metrics_collector.create_snapshot();
        
        // Get system info
        let system_info = self.get_system_info();
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(&perf_stats, &mem_stats, &metrics_snapshot);
        
        // Get performance grade
        let grade = profiler.get_performance_grade();
        
        let report = PerformanceReport {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            duration_seconds: duration.as_secs_f64(),
            performance_stats: serializable_perf_stats,
            memory_stats: serializable_mem_stats,
            metrics_snapshot,
            system_info,
            recommendations,
            grade: grade.as_str().to_string(),
        };
        
        // Add to history
        self.report_history.push(report.clone());
        
        // Trim history if needed
        if self.report_history.len() > self.max_history {
            self.report_history.remove(0);
        }
        
        // Auto-save if enabled
        if self.auto_save {
            if let Err(e) = self.save_report(&report) {
                error!("Failed to auto-save performance report: {}", e);
            }
        }
        
        info!("Generated performance report (grade: {})", grade.as_str());
        report
    }
    
    /// Convert performance stats to serializable format
    fn convert_performance_stats(&self, stats: &PerformanceStats) -> SerializablePerformanceStats {
        let mut system_stats = HashMap::new();
        
        for (name, sys_stats) in &stats.system_stats {
            system_stats.insert(name.clone(), SerializableSystemStats {
                average_time_ms: sys_stats.average_time.as_secs_f64() * 1000.0,
                min_time_ms: sys_stats.min_time.as_secs_f64() * 1000.0,
                max_time_ms: sys_stats.max_time.as_secs_f64() * 1000.0,
                total_time_ms: sys_stats.total_time.as_secs_f64() * 1000.0,
                call_count: sys_stats.call_count,
                percentage_of_frame: sys_stats.percentage_of_frame,
            });
        }
        
        SerializablePerformanceStats {
            average_fps: stats.average_fps,
            min_fps: stats.min_fps,
            max_fps: stats.max_fps,
            average_frame_time_ms: stats.average_frame_time.as_secs_f64() * 1000.0,
            min_frame_time_ms: stats.min_frame_time.as_secs_f64() * 1000.0,
            max_frame_time_ms: stats.max_frame_time.as_secs_f64() * 1000.0,
            frame_count: stats.frame_count,
            total_time_seconds: stats.total_time.as_secs_f64(),
            system_stats,
            warning_count: stats.warnings.len(),
        }
    }
    
    /// Convert memory stats to serializable format
    fn convert_memory_stats(&self, stats: &MemoryStats) -> SerializableMemoryStats {
        let mut categories = HashMap::new();
        
        for (name, cat_stats) in &stats.categories {
            categories.insert(name.clone(), SerializableCategoryStats {
                current_mb: cat_stats.current as f64 / 1024.0 / 1024.0,
                count: cat_stats.count,
                average_size_kb: cat_stats.average_size / 1024.0,
            });
        }
        
        SerializableMemoryStats {
            current_usage_mb: stats.current_usage as f64 / 1024.0 / 1024.0,
            peak_usage_mb: stats.peak_usage as f64 / 1024.0 / 1024.0,
            total_allocated_mb: stats.total_allocated as f64 / 1024.0 / 1024.0,
            total_deallocated_mb: stats.total_deallocated as f64 / 1024.0 / 1024.0,
            allocation_count: stats.allocation_count,
            deallocation_count: stats.deallocation_count,
            average_allocation_size_kb: stats.average_allocation_size / 1024.0,
            fragmentation_ratio: stats.fragmentation_ratio,
            categories,
        }
    }
    
    /// Get system information
    fn get_system_info(&self) -> SystemInfo {
        SystemInfo {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cpu_count: num_cpus::get(),
            total_memory_mb: self.get_total_memory_mb(),
            available_memory_mb: self.get_available_memory_mb(),
            game_version: env!("CARGO_PKG_VERSION").to_string(),
            build_type: if cfg!(debug_assertions) { "debug".to_string() } else { "release".to_string() },
        }
    }
    
    /// Get total system memory (simplified)
    fn get_total_memory_mb(&self) -> u64 {
        // This is a placeholder - real implementation would use platform-specific APIs
        8192 // 8GB default
    }
    
    /// Get available system memory (simplified)
    fn get_available_memory_mb(&self) -> u64 {
        // This is a placeholder - real implementation would use platform-specific APIs
        4096 // 4GB default
    }
    
    /// Generate performance recommendations
    fn generate_recommendations(
        &self,
        perf_stats: &PerformanceStats,
        mem_stats: &MemoryStats,
        _metrics: &MetricsSnapshot,
    ) -> Vec<PerformanceRecommendation> {
        let mut recommendations = Vec::new();
        
        // Check frame rate
        if perf_stats.average_fps < 30.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Performance".to_string(),
                severity: RecommendationSeverity::High,
                title: "Low Frame Rate".to_string(),
                description: format!("Average FPS is {:.1}, which is below acceptable levels", perf_stats.average_fps),
                suggested_action: "Consider reducing visual effects, optimizing rendering, or lowering game settings".to_string(),
                impact: "Improved gameplay smoothness".to_string(),
            });
        } else if perf_stats.average_fps < 45.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Performance".to_string(),
                severity: RecommendationSeverity::Medium,
                title: "Suboptimal Frame Rate".to_string(),
                description: format!("Average FPS is {:.1}, which could be improved", perf_stats.average_fps),
                suggested_action: "Minor optimizations to rendering or AI systems may help".to_string(),
                impact: "Smoother gameplay experience".to_string(),
            });
        }
        
        // Check memory usage
        let memory_mb = mem_stats.current_usage as f64 / 1024.0 / 1024.0;
        if memory_mb > 1000.0 {
            recommendations.push(PerformanceRecommendation {
                category: "Memory".to_string(),
                severity: RecommendationSeverity::High,
                title: "High Memory Usage".to_string(),
                description: format!("Current memory usage is {:.1} MB", memory_mb),
                suggested_action: "Review memory allocations and consider implementing object pooling".to_string(),
                impact: "Reduced memory pressure and potential performance improvement".to_string(),
            });
        }
        
        // Check fragmentation
        if mem_stats.fragmentation_ratio > 0.3 {
            recommendations.push(PerformanceRecommendation {
                category: "Memory".to_string(),
                severity: RecommendationSeverity::Medium,
                title: "Memory Fragmentation".to_string(),
                description: format!("Memory fragmentation ratio is {:.1}%", mem_stats.fragmentation_ratio * 100.0),
                suggested_action: "Consider implementing memory pools or compaction strategies".to_string(),
                impact: "More efficient memory usage".to_string(),
            });
        }
        
        // Check slow systems
        for (system_name, sys_stats) in &perf_stats.system_stats {
            if sys_stats.percentage_of_frame > 25.0 {
                recommendations.push(PerformanceRecommendation {
                    category: "Systems".to_string(),
                    severity: RecommendationSeverity::Medium,
                    title: format!("Slow System: {}", system_name),
                    description: format!("System '{}' is using {:.1}% of frame time", system_name, sys_stats.percentage_of_frame),
                    suggested_action: format!("Optimize the {} system or consider spreading work across frames", system_name),
                    impact: "Improved overall frame rate".to_string(),
                });
            }
        }
        
        // Check warnings
        if perf_stats.warning_count > 10 {
            recommendations.push(PerformanceRecommendation {
                category: "Stability".to_string(),
                severity: RecommendationSeverity::Medium,
                title: "Frequent Performance Warnings".to_string(),
                description: format!("{} performance warnings detected", perf_stats.warning_count),
                suggested_action: "Review performance warnings and address underlying issues".to_string(),
                impact: "More stable performance".to_string(),
            });
        }
        
        recommendations
    }
    
    /// Save report to file
    pub fn save_report(&self, report: &PerformanceReport) -> Result<(), Box<dyn std::error::Error>> {
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&self.output_directory)?;
        
        // Generate filename with timestamp
        let filename = format!("performance_report_{}.json", report.timestamp);
        let filepath = Path::new(&self.output_directory).join(filename);
        
        // Serialize and write report
        let json = serde_json::to_string_pretty(report)?;
        let mut file = File::create(filepath)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
    
    /// Load report from file
    pub fn load_report(&self, filepath: &str) -> Result<PerformanceReport, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(filepath)?;
        let report: PerformanceReport = serde_json::from_str(&content)?;
        Ok(report)
    }
    
    /// Generate summary report
    pub fn generate_summary_report(&self) -> String {
        if self.report_history.is_empty() {
            return "No performance reports available".to_string();
        }
        
        let mut summary = String::new();
        summary.push_str("=== Performance Summary ===\n");
        
        // Calculate averages
        let total_reports = self.report_history.len();
        let avg_fps: f64 = self.report_history.iter()
            .map(|r| r.performance_stats.average_fps)
            .sum::<f64>() / total_reports as f64;
        
        let avg_memory: f64 = self.report_history.iter()
            .map(|r| r.memory_stats.current_usage_mb)
            .sum::<f64>() / total_reports as f64;
        
        summary.push_str(&format!("Reports: {}\n", total_reports));
        summary.push_str(&format!("Average FPS: {:.1}\n", avg_fps));
        summary.push_str(&format!("Average Memory: {:.1} MB\n", avg_memory));
        
        // Grade distribution
        let mut grade_counts = HashMap::new();
        for report in &self.report_history {
            *grade_counts.entry(report.grade.clone()).or_insert(0) += 1;
        }
        
        summary.push_str("\n=== Grade Distribution ===\n");
        for (grade, count) in grade_counts {
            let percentage = (count as f64 / total_reports as f64) * 100.0;
            summary.push_str(&format!("{}: {} ({:.1}%)\n", grade, count, percentage));
        }
        
        // Recent trends
        if self.report_history.len() >= 2 {
            let recent = &self.report_history[self.report_history.len() - 1];
            let previous = &self.report_history[self.report_history.len() - 2];
            
            let fps_change = recent.performance_stats.average_fps - previous.performance_stats.average_fps;
            let memory_change = recent.memory_stats.current_usage_mb - previous.memory_stats.current_usage_mb;
            
            summary.push_str("\n=== Recent Trends ===\n");
            summary.push_str(&format!("FPS Change: {:+.1}\n", fps_change));
            summary.push_str(&format!("Memory Change: {:+.1} MB\n", memory_change));
        }
        
        // Top recommendations
        let mut all_recommendations = Vec::new();
        for report in &self.report_history {
            all_recommendations.extend(report.recommendations.iter());
        }
        
        // Count recommendation categories
        let mut category_counts = HashMap::new();
        for rec in &all_recommendations {
            *category_counts.entry(rec.category.clone()).or_insert(0) += 1;
        }
        
        if !category_counts.is_empty() {
            summary.push_str("\n=== Top Recommendation Categories ===\n");
            let mut sorted_categories: Vec<_> = category_counts.into_iter().collect();
            sorted_categories.sort_by(|a, b| b.1.cmp(&a.1));
            
            for (category, count) in sorted_categories.into_iter().take(5) {
                summary.push_str(&format!("{}: {} occurrences\n", category, count));
            }
        }
        
        summary
    }
    
    /// Get report history
    pub fn get_report_history(&self) -> &Vec<PerformanceReport> {
        &self.report_history
    }
    
    /// Clear report history
    pub fn clear_history(&mut self) {
        self.report_history.clear();
        info!("Performance report history cleared");
    }
    
    /// Export all reports to CSV
    pub fn export_csv(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = File::create(filepath)?;
        
        // Write header
        writeln!(file, "timestamp,duration_seconds,average_fps,min_fps,max_fps,frame_time_ms,memory_mb,grade,warning_count")?;
        
        // Write data
        for report in &self.report_history {
            writeln!(file, "{},{},{:.2},{:.2},{:.2},{:.2},{:.2},{},{}",
                report.timestamp,
                report.duration_seconds,
                report.performance_stats.average_fps,
                report.performance_stats.min_fps,
                report.performance_stats.max_fps,
                report.performance_stats.average_frame_time_ms,
                report.memory_stats.current_usage_mb,
                report.grade,
                report.performance_stats.warning_count)?;
        }
        
        Ok(())
    }
    
    /// Set maximum history size
    pub fn set_max_history(&mut self, max_history: usize) {
        self.max_history = max_history;
        
        // Trim existing history if needed
        if self.report_history.len() > max_history {
            self.report_history.drain(0..self.report_history.len() - max_history);
        }
    }
}