use std::collections::VecDeque;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};

/// Memory usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub timestamp: Instant,
    pub heap_used: usize,
    pub heap_allocated: usize,
    pub stack_size: usize,
    pub virtual_memory: usize,
    pub resident_memory: usize,
}

/// Memory monitor for tracking memory usage over time
pub struct MemoryMonitor {
    history: VecDeque<MemoryStats>,
    max_history_size: usize,
    last_update: Instant,
    update_interval: Duration,
    
    // Statistics
    peak_heap_used: usize,
    peak_heap_allocated: usize,
    peak_virtual_memory: usize,
    peak_resident_memory: usize,
    
    // Memory pressure detection
    memory_pressure_threshold: f64, // Percentage of available memory
    is_memory_pressure: bool,
}

impl MemoryMonitor {
    /// Create a new memory monitor
    pub fn new() -> Self {
        MemoryMonitor {
            history: VecDeque::new(),
            max_history_size: 300, // Keep 5 minutes at 1 second intervals
            last_update: Instant::now(),
            update_interval: Duration::from_secs(1),
            peak_heap_used: 0,
            peak_heap_allocated: 0,
            peak_virtual_memory: 0,
            peak_resident_memory: 0,
            memory_pressure_threshold: 0.8, // 80% of available memory
            is_memory_pressure: false,
        }
    }
    
    /// Update memory statistics
    pub fn update(&mut self) -> Option<MemoryStats> {
        let now = Instant::now();
        
        // Only update if enough time has passed
        if now.duration_since(self.last_update) < self.update_interval {
            return None;
        }
        
        self.last_update = now;
        
        let stats = self.collect_memory_stats(now);
        
        // Update peaks
        self.peak_heap_used = self.peak_heap_used.max(stats.heap_used);
        self.peak_heap_allocated = self.peak_heap_allocated.max(stats.heap_allocated);
        self.peak_virtual_memory = self.peak_virtual_memory.max(stats.virtual_memory);
        self.peak_resident_memory = self.peak_resident_memory.max(stats.resident_memory);
        
        // Check for memory pressure
        self.check_memory_pressure(&stats);
        
        // Add to history
        self.history.push_back(stats.clone());
        
        // Trim history if needed
        while self.history.len() > self.max_history_size {
            self.history.pop_front();
        }
        
        Some(stats)
    }
    
    /// Collect current memory statistics
    fn collect_memory_stats(&self, timestamp: Instant) -> MemoryStats {
        // Note: This is a simplified implementation
        // In a real application, you would use platform-specific APIs
        // or crates like `sysinfo` to get actual memory usage
        
        #[cfg(target_os = "linux")]
        {
            self.collect_linux_memory_stats(timestamp)
        }
        
        #[cfg(target_os = "macos")]
        {
            self.collect_macos_memory_stats(timestamp)
        }
        
        #[cfg(target_os = "windows")]
        {
            self.collect_windows_memory_stats(timestamp)
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            self.collect_fallback_memory_stats(timestamp)
        }
    }
    
    #[cfg(target_os = "linux")]
    fn collect_linux_memory_stats(&self, timestamp: Instant) -> MemoryStats {
        use std::fs;
        
        // Read from /proc/self/status for process memory info
        let status = fs::read_to_string("/proc/self/status").unwrap_or_default();
        let mut vm_size = 0;
        let mut vm_rss = 0;
        
        for line in status.lines() {
            if line.starts_with("VmSize:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    vm_size = value.parse::<usize>().unwrap_or(0) * 1024; // Convert KB to bytes
                }
            } else if line.starts_with("VmRSS:") {
                if let Some(value) = line.split_whitespace().nth(1) {
                    vm_rss = value.parse::<usize>().unwrap_or(0) * 1024; // Convert KB to bytes
                }
            }
        }
        
        MemoryStats {
            timestamp,
            heap_used: vm_rss / 2, // Rough estimate
            heap_allocated: vm_rss,
            stack_size: 8 * 1024 * 1024, // Typical stack size
            virtual_memory: vm_size,
            resident_memory: vm_rss,
        }
    }
    
    #[cfg(target_os = "macos")]
    fn collect_macos_memory_stats(&self, timestamp: Instant) -> MemoryStats {
        // On macOS, we would use mach APIs or sysctl
        // This is a simplified fallback implementation
        self.collect_fallback_memory_stats(timestamp)
    }
    
    #[cfg(target_os = "windows")]
    fn collect_windows_memory_stats(&self, timestamp: Instant) -> MemoryStats {
        // On Windows, we would use Windows APIs like GetProcessMemoryInfo
        // This is a simplified fallback implementation
        self.collect_fallback_memory_stats(timestamp)
    }
    
    fn collect_fallback_memory_stats(&self, timestamp: Instant) -> MemoryStats {
        // Fallback implementation using rough estimates
        // In a real implementation, you would use a crate like `sysinfo`
        
        let estimated_heap = 10 * 1024 * 1024; // 10 MB estimate
        let estimated_virtual = 50 * 1024 * 1024; // 50 MB estimate
        
        MemoryStats {
            timestamp,
            heap_used: estimated_heap / 2,
            heap_allocated: estimated_heap,
            stack_size: 8 * 1024 * 1024, // 8 MB typical stack
            virtual_memory: estimated_virtual,
            resident_memory: estimated_heap,
        }
    }
    
    /// Check for memory pressure conditions
    fn check_memory_pressure(&mut self, stats: &MemoryStats) {
        // Simple heuristic: if resident memory is growing rapidly
        if self.history.len() >= 10 {
            let recent_stats: Vec<_> = self.history.iter().rev().take(10).collect();
            let oldest_memory = recent_stats.last().unwrap().resident_memory;
            let current_memory = stats.resident_memory;
            
            let growth_rate = if oldest_memory > 0 {
                (current_memory as f64 - oldest_memory as f64) / oldest_memory as f64
            } else {
                0.0
            };
            
            // Consider memory pressure if growth rate > 50% in recent history
            self.is_memory_pressure = growth_rate > 0.5;
        }
    }
    
    /// Get current memory statistics
    pub fn get_current_stats(&self) -> Option<&MemoryStats> {
        self.history.back()
    }
    
    /// Get memory history
    pub fn get_history(&self) -> &VecDeque<MemoryStats> {
        &self.history
    }
    
    /// Get peak memory usage
    pub fn get_peak_heap_used(&self) -> usize {
        self.peak_heap_used
    }
    
    /// Get peak heap allocated
    pub fn get_peak_heap_allocated(&self) -> usize {
        self.peak_heap_allocated
    }
    
    /// Get peak virtual memory
    pub fn get_peak_virtual_memory(&self) -> usize {
        self.peak_virtual_memory
    }
    
    /// Get peak resident memory
    pub fn get_peak_resident_memory(&self) -> usize {
        self.peak_resident_memory
    }
    
    /// Check if under memory pressure
    pub fn is_memory_pressure(&self) -> bool {
        self.is_memory_pressure
    }
    
    /// Get memory usage trend over recent history
    pub fn get_memory_trend(&self, samples: usize) -> MemoryTrend {
        if self.history.len() < 2 {
            return MemoryTrend::Stable;
        }
        
        let sample_count = samples.min(self.history.len());
        let recent_stats: Vec<_> = self.history.iter().rev().take(sample_count).collect();
        
        if recent_stats.len() < 2 {
            return MemoryTrend::Stable;
        }
        
        let oldest = recent_stats.last().unwrap();
        let newest = recent_stats.first().unwrap();
        
        let change_rate = if oldest.resident_memory > 0 {
            (newest.resident_memory as f64 - oldest.resident_memory as f64) / oldest.resident_memory as f64
        } else {
            0.0
        };
        
        if change_rate > 0.1 {
            MemoryTrend::Increasing
        } else if change_rate < -0.1 {
            MemoryTrend::Decreasing
        } else {
            MemoryTrend::Stable
        }
    }
    
    /// Get average memory usage over recent history
    pub fn get_average_memory_usage(&self, samples: usize) -> MemoryAverages {
        if self.history.is_empty() {
            return MemoryAverages::default();
        }
        
        let sample_count = samples.min(self.history.len());
        let recent_stats: Vec<_> = self.history.iter().rev().take(sample_count).collect();
        
        let heap_used_sum: usize = recent_stats.iter().map(|s| s.heap_used).sum();
        let heap_allocated_sum: usize = recent_stats.iter().map(|s| s.heap_allocated).sum();
        let virtual_memory_sum: usize = recent_stats.iter().map(|s| s.virtual_memory).sum();
        let resident_memory_sum: usize = recent_stats.iter().map(|s| s.resident_memory).sum();
        
        let count = recent_stats.len();
        
        MemoryAverages {
            avg_heap_used: heap_used_sum / count,
            avg_heap_allocated: heap_allocated_sum / count,
            avg_virtual_memory: virtual_memory_sum / count,
            avg_resident_memory: resident_memory_sum / count,
        }
    }
    
    /// Set update interval
    pub fn set_update_interval(&mut self, interval: Duration) {
        self.update_interval = interval;
    }
    
    /// Set maximum history size
    pub fn set_max_history_size(&mut self, size: usize) {
        self.max_history_size = size;
        
        // Trim current history if needed
        while self.history.len() > self.max_history_size {
            self.history.pop_front();
        }
    }
    
    /// Reset all statistics
    pub fn reset(&mut self) {
        self.history.clear();
        self.peak_heap_used = 0;
        self.peak_heap_allocated = 0;
        self.peak_virtual_memory = 0;
        self.peak_resident_memory = 0;
        self.is_memory_pressure = false;
        self.last_update = Instant::now();
    }
    
    /// Get detailed memory report
    pub fn get_memory_report(&self) -> MemoryReport {
        let current = self.get_current_stats().cloned();
        let averages = self.get_average_memory_usage(60); // Last minute
        let trend = self.get_memory_trend(10);
        
        MemoryReport {
            current_stats: current,
            peak_heap_used: self.peak_heap_used,
            peak_heap_allocated: self.peak_heap_allocated,
            peak_virtual_memory: self.peak_virtual_memory,
            peak_resident_memory: self.peak_resident_memory,
            averages,
            trend,
            is_memory_pressure: self.is_memory_pressure,
            history_size: self.history.len(),
        }
    }
    
    /// Format memory size for display
    pub fn format_memory_size(bytes: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size = bytes as f64;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", bytes, UNITS[unit_index])
        } else {
            format!("{:.2} {}", size, UNITS[unit_index])
        }
    }
}

/// Memory usage trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryTrend {
    Increasing,
    Decreasing,
    Stable,
}

impl MemoryTrend {
    pub fn as_str(&self) -> &'static str {
        match self {
            MemoryTrend::Increasing => "Increasing",
            MemoryTrend::Decreasing => "Decreasing",
            MemoryTrend::Stable => "Stable",
        }
    }
}

/// Average memory usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryAverages {
    pub avg_heap_used: usize,
    pub avg_heap_allocated: usize,
    pub avg_virtual_memory: usize,
    pub avg_resident_memory: usize,
}

/// Comprehensive memory report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryReport {
    pub current_stats: Option<MemoryStats>,
    pub peak_heap_used: usize,
    pub peak_heap_allocated: usize,
    pub peak_virtual_memory: usize,
    pub peak_resident_memory: usize,
    pub averages: MemoryAverages,
    pub trend: MemoryTrend,
    pub is_memory_pressure: bool,
    pub history_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_memory_monitor_basic() {
        let mut monitor = MemoryMonitor::new();
        
        // Force an update
        let stats = monitor.update();
        assert!(stats.is_some());
        
        let stats = stats.unwrap();
        assert!(stats.heap_used > 0);
        assert!(stats.resident_memory > 0);
    }
    
    #[test]
    fn test_memory_history() {
        let mut monitor = MemoryMonitor::new();
        monitor.set_update_interval(Duration::from_millis(1)); // Very short interval for testing
        
        // Add several updates
        for _ in 0..5 {
            monitor.update();
            thread::sleep(Duration::from_millis(2));
        }
        
        let history = monitor.get_history();
        assert!(history.len() > 0);
    }
    
    #[test]
    fn test_memory_peaks() {
        let mut monitor = MemoryMonitor::new();
        monitor.set_update_interval(Duration::from_millis(1));
        
        // Update a few times
        for _ in 0..3 {
            monitor.update();
            thread::sleep(Duration::from_millis(2));
        }
        
        // Peaks should be set
        assert!(monitor.get_peak_heap_used() > 0);
        assert!(monitor.get_peak_resident_memory() > 0);
    }
    
    #[test]
    fn test_memory_trend() {
        let mut monitor = MemoryMonitor::new();
        monitor.set_update_interval(Duration::from_millis(1));
        
        // Add some history
        for _ in 0..10 {
            monitor.update();
            thread::sleep(Duration::from_millis(2));
        }
        
        let trend = monitor.get_memory_trend(5);
        // Should be one of the valid trends
        assert!(matches!(trend, MemoryTrend::Increasing | MemoryTrend::Decreasing | MemoryTrend::Stable));
    }
    
    #[test]
    fn test_memory_formatting() {
        assert_eq!(MemoryMonitor::format_memory_size(1024), "1.00 KB");
        assert_eq!(MemoryMonitor::format_memory_size(1024 * 1024), "1.00 MB");
        assert_eq!(MemoryMonitor::format_memory_size(1536), "1.50 KB");
        assert_eq!(MemoryMonitor::format_memory_size(512), "512 B");
    }
}