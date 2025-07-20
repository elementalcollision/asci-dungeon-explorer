use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use log::{info, warn, error};

/// Memory usage tracker
pub struct MemoryTracker {
    enabled: bool,
    allocations: HashMap<String, AllocationInfo>,
    total_allocated: u64,
    total_deallocated: u64,
    peak_usage: u64,
    current_usage: u64,
    allocation_count: u64,
    deallocation_count: u64,
    samples: Vec<MemorySample>,
    max_samples: usize,
    last_gc_time: Option<Instant>,
    gc_threshold: u64,
}

/// Allocation information
#[derive(Debug, Clone)]
pub struct AllocationInfo {
    pub size: u64,
    pub timestamp: Instant,
    pub category: String,
    pub description: String,
    pub stack_trace: Option<String>,
}

/// Memory sample
#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub total_usage: u64,
    pub heap_usage: u64,
    pub stack_usage: u64,
    pub system_usage: u64,
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: u64,
    pub peak_usage: u64,
    pub total_allocated: u64,
    pub total_deallocated: u64,
    pub allocation_count: u64,
    pub deallocation_count: u64,
    pub average_allocation_size: f64,
    pub fragmentation_ratio: f64,
    pub gc_count: u64,
    pub last_gc_time: Option<Instant>,
    pub categories: HashMap<String, CategoryStats>,
}

/// Category statistics
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub allocated: u64,
    pub deallocated: u64,
    pub current: u64,
    pub count: u64,
    pub average_size: f64,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        MemoryTracker {
            enabled: true,
            allocations: HashMap::new(),
            total_allocated: 0,
            total_deallocated: 0,
            peak_usage: 0,
            current_usage: 0,
            allocation_count: 0,
            deallocation_count: 0,
            samples: Vec::new(),
            max_samples: 1000,
            last_gc_time: None,
            gc_threshold: 100 * 1024 * 1024, // 100MB
        }
    }
    
    /// Enable or disable tracking
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled {
            info!("Memory tracking enabled");
        } else {
            info!("Memory tracking disabled");
        }
    }
    
    /// Record an allocation
    pub fn record_allocation(&mut self, id: &str, size: u64, category: &str, description: &str) {
        if !self.enabled {
            return;
        }
        
        let info = AllocationInfo {
            size,
            timestamp: Instant::now(),
            category: category.to_string(),
            description: description.to_string(),
            stack_trace: None, // Could be implemented with backtrace crate
        };
        
        self.allocations.insert(id.to_string(), info);
        self.total_allocated += size;
        self.current_usage += size;
        self.allocation_count += 1;
        
        // Update peak usage
        if self.current_usage > self.peak_usage {
            self.peak_usage = self.current_usage;
        }
        
        // Check if GC is needed
        if self.current_usage > self.gc_threshold {
            self.suggest_gc();
        }
    }
    
    /// Record a deallocation
    pub fn record_deallocation(&mut self, id: &str) {
        if !self.enabled {
            return;
        }
        
        if let Some(info) = self.allocations.remove(id) {
            self.total_deallocated += info.size;
            self.current_usage = self.current_usage.saturating_sub(info.size);
            self.deallocation_count += 1;
        }
    }
    
    /// Take a memory sample
    pub fn take_sample(&mut self) {
        if !self.enabled {
            return;
        }
        
        let sample = MemorySample {
            timestamp: Instant::now(),
            total_usage: self.current_usage,
            heap_usage: self.get_heap_usage(),
            stack_usage: self.get_stack_usage(),
            system_usage: self.get_system_usage(),
        };
        
        self.samples.push(sample);
        
        // Trim samples if needed
        if self.samples.len() > self.max_samples {
            self.samples.remove(0);
        }
    }
    
    /// Get current memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let mut categories = HashMap::new();
        
        // Calculate category statistics
        for info in self.allocations.values() {
            let entry = categories.entry(info.category.clone()).or_insert(CategoryStats {
                allocated: 0,
                deallocated: 0,
                current: 0,
                count: 0,
                average_size: 0.0,
            });
            
            entry.current += info.size;
            entry.count += 1;
        }
        
        // Calculate average sizes
        for stats in categories.values_mut() {
            if stats.count > 0 {
                stats.average_size = stats.current as f64 / stats.count as f64;
            }
        }
        
        let average_allocation_size = if self.allocation_count > 0 {
            self.total_allocated as f64 / self.allocation_count as f64
        } else {
            0.0
        };
        
        let fragmentation_ratio = if self.total_allocated > 0 {
            (self.total_allocated - self.current_usage) as f64 / self.total_allocated as f64
        } else {
            0.0
        };
        
        MemoryStats {
            current_usage: self.current_usage,
            peak_usage: self.peak_usage,
            total_allocated: self.total_allocated,
            total_deallocated: self.total_deallocated,
            allocation_count: self.allocation_count,
            deallocation_count: self.deallocation_count,
            average_allocation_size,
            fragmentation_ratio,
            gc_count: 0, // Would need to track GC events
            last_gc_time: self.last_gc_time,
            categories,
        }
    }
    
    /// Get memory usage over time
    pub fn get_usage_history(&self, duration: Duration) -> Vec<&MemorySample> {
        let cutoff = Instant::now() - duration;
        self.samples.iter()
            .filter(|s| s.timestamp > cutoff)
            .collect()
    }
    
    /// Get allocations by category
    pub fn get_allocations_by_category(&self, category: &str) -> Vec<(&String, &AllocationInfo)> {
        self.allocations.iter()
            .filter(|(_, info)| info.category == category)
            .collect()
    }
    
    /// Get largest allocations
    pub fn get_largest_allocations(&self, count: usize) -> Vec<(&String, &AllocationInfo)> {
        let mut allocations: Vec<_> = self.allocations.iter().collect();
        allocations.sort_by(|a, b| b.1.size.cmp(&a.1.size));
        allocations.into_iter().take(count).collect()
    }
    
    /// Get oldest allocations
    pub fn get_oldest_allocations(&self, count: usize) -> Vec<(&String, &AllocationInfo)> {
        let mut allocations: Vec<_> = self.allocations.iter().collect();
        allocations.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
        allocations.into_iter().take(count).collect()
    }
    
    /// Check for memory leaks
    pub fn check_for_leaks(&self, age_threshold: Duration) -> Vec<(&String, &AllocationInfo)> {
        let cutoff = Instant::now() - age_threshold;
        self.allocations.iter()
            .filter(|(_, info)| info.timestamp < cutoff)
            .collect()
    }
    
    /// Reset statistics
    pub fn reset(&mut self) {
        self.allocations.clear();
        self.total_allocated = 0;
        self.total_deallocated = 0;
        self.peak_usage = 0;
        self.current_usage = 0;
        self.allocation_count = 0;
        self.deallocation_count = 0;
        self.samples.clear();
        self.last_gc_time = None;
        info!("Memory tracker reset");
    }
    
    /// Set GC threshold
    pub fn set_gc_threshold(&mut self, threshold: u64) {
        self.gc_threshold = threshold;
    }
    
    /// Set maximum samples
    pub fn set_max_samples(&mut self, max_samples: usize) {
        self.max_samples = max_samples;
        
        // Trim existing samples if needed
        if self.samples.len() > max_samples {
            self.samples.drain(0..self.samples.len() - max_samples);
        }
    }
    
    /// Suggest garbage collection
    fn suggest_gc(&mut self) {
        warn!("Memory usage ({} bytes) exceeded GC threshold ({} bytes)", 
              self.current_usage, self.gc_threshold);
        
        // In a real implementation, this might trigger actual GC
        // For now, just record the suggestion
        self.last_gc_time = Some(Instant::now());
    }
    
    /// Get heap usage (simplified - would need platform-specific implementation)
    fn get_heap_usage(&self) -> u64 {
        // This is a placeholder - real implementation would use platform-specific APIs
        self.current_usage
    }
    
    /// Get stack usage (simplified)
    fn get_stack_usage(&self) -> u64 {
        // This is a placeholder - real implementation would use platform-specific APIs
        0
    }
    
    /// Get system memory usage (simplified)
    fn get_system_usage(&self) -> u64 {
        // This is a placeholder - real implementation would use platform-specific APIs
        self.current_usage
    }
    
    /// Format memory size for display
    pub fn format_size(bytes: u64) -> String {
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
    
    /// Generate memory report
    pub fn generate_report(&self) -> String {
        let stats = self.get_stats();
        let mut report = String::new();
        
        report.push_str("=== Memory Usage Report ===\n");
        report.push_str(&format!("Current Usage: {}\n", Self::format_size(stats.current_usage)));
        report.push_str(&format!("Peak Usage: {}\n", Self::format_size(stats.peak_usage)));
        report.push_str(&format!("Total Allocated: {}\n", Self::format_size(stats.total_allocated)));
        report.push_str(&format!("Total Deallocated: {}\n", Self::format_size(stats.total_deallocated)));
        report.push_str(&format!("Allocations: {}\n", stats.allocation_count));
        report.push_str(&format!("Deallocations: {}\n", stats.deallocation_count));
        report.push_str(&format!("Average Allocation Size: {}\n", Self::format_size(stats.average_allocation_size as u64)));
        report.push_str(&format!("Fragmentation Ratio: {:.2}%\n", stats.fragmentation_ratio * 100.0));
        
        report.push_str("\n=== Categories ===\n");
        for (category, cat_stats) in &stats.categories {
            report.push_str(&format!("{}: {} ({} allocations, avg {})\n",
                category,
                Self::format_size(cat_stats.current),
                cat_stats.count,
                Self::format_size(cat_stats.average_size as u64)));
        }
        
        // Show largest allocations
        report.push_str("\n=== Largest Allocations ===\n");
        for (id, info) in self.get_largest_allocations(10) {
            report.push_str(&format!("{}: {} ({})\n",
                id,
                Self::format_size(info.size),
                info.description));
        }
        
        // Check for potential leaks
        let potential_leaks = self.check_for_leaks(Duration::from_secs(300)); // 5 minutes
        if !potential_leaks.is_empty() {
            report.push_str("\n=== Potential Memory Leaks ===\n");
            for (id, info) in potential_leaks.iter().take(10) {
                let age = info.timestamp.elapsed();
                report.push_str(&format!("{}: {} (age: {:.1}s, {})\n",
                    id,
                    Self::format_size(info.size),
                    age.as_secs_f64(),
                    info.description));
            }
        }
        
        report
    }
}

/// Global memory tracker instance
static GLOBAL_MEMORY_TRACKER: Mutex<Option<MemoryTracker>> = Mutex::new(None);

/// Initialize global memory tracker
pub fn init_memory_tracker() {
    let mut tracker = GLOBAL_MEMORY_TRACKER.lock().unwrap();
    *tracker = Some(MemoryTracker::new());
}

/// Get global memory tracker
pub fn with_memory_tracker<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut MemoryTracker) -> R,
{
    if let Ok(mut tracker) = GLOBAL_MEMORY_TRACKER.lock() {
        if let Some(ref mut t) = *tracker {
            Some(f(t))
        } else {
            None
        }
    } else {
        None
    }
}

/// Macro for tracking allocations
#[macro_export]
macro_rules! track_allocation {
    ($id:expr, $size:expr, $category:expr, $description:expr) => {
        $crate::performance::with_memory_tracker(|t| {
            t.record_allocation($id, $size, $category, $description);
        });
    };
}

/// Macro for tracking deallocations
#[macro_export]
macro_rules! track_deallocation {
    ($id:expr) => {
        $crate::performance::with_memory_tracker(|t| {
            t.record_deallocation($id);
        });
    };
}

/// Game-specific memory categories
pub struct GameMemoryCategories;

impl GameMemoryCategories {
    pub const ENTITIES: &'static str = "entities";
    pub const COMPONENTS: &'static str = "components";
    pub const RESOURCES: &'static str = "resources";
    pub const RENDERING: &'static str = "rendering";
    pub const AUDIO: &'static str = "audio";
    pub const AI: &'static str = "ai";
    pub const PATHFINDING: &'static str = "pathfinding";
    pub const DUNGEON: &'static str = "dungeon";
    pub const ITEMS: &'static str = "items";
    pub const UI: &'static str = "ui";
    pub const SAVE_DATA: &'static str = "save_data";
    pub const TEMPORARY: &'static str = "temporary";
}

/// Memory pool for efficient allocation
pub struct MemoryPool<T> {
    pool: Vec<T>,
    available: Vec<usize>,
    capacity: usize,
    allocated_count: usize,
}

impl<T: Default + Clone> MemoryPool<T> {
    /// Create a new memory pool
    pub fn new(capacity: usize) -> Self {
        let mut pool = Vec::with_capacity(capacity);
        let mut available = Vec::with_capacity(capacity);
        
        for i in 0..capacity {
            pool.push(T::default());
            available.push(i);
        }
        
        MemoryPool {
            pool,
            available,
            capacity,
            allocated_count: 0,
        }
    }
    
    /// Allocate an item from the pool
    pub fn allocate(&mut self) -> Option<(usize, &mut T)> {
        if let Some(index) = self.available.pop() {
            self.allocated_count += 1;
            Some((index, &mut self.pool[index]))
        } else {
            None
        }
    }
    
    /// Deallocate an item back to the pool
    pub fn deallocate(&mut self, index: usize) {
        if index < self.capacity && !self.available.contains(&index) {
            self.pool[index] = T::default(); // Reset to default state
            self.available.push(index);
            self.allocated_count = self.allocated_count.saturating_sub(1);
        }
    }
    
    /// Get item by index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.pool.get(index)
    }
    
    /// Get mutable item by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.pool.get_mut(index)
    }
    
    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            capacity: self.capacity,
            allocated: self.allocated_count,
            available: self.available.len(),
            utilization: self.allocated_count as f64 / self.capacity as f64,
        }
    }
    
    /// Reset the pool
    pub fn reset(&mut self) {
        self.available.clear();
        for i in 0..self.capacity {
            self.pool[i] = T::default();
            self.available.push(i);
        }
        self.allocated_count = 0;
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub capacity: usize,
    pub allocated: usize,
    pub available: usize,
    pub utilization: f64,
}