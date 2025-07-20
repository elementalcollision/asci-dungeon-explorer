#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_profiler_basic_functionality() {
        let mut profiler = PerformanceProfiler::new(60.0);
        
        // Test frame timing
        profiler.start_frame();
        thread::sleep(Duration::from_millis(10));
        profiler.end_frame();
        
        let stats = profiler.get_stats();
        assert!(stats.frame_count == 1);
        assert!(stats.average_frame_time >= Duration::from_millis(10));
        assert!(stats.average_fps > 0.0);
    }
    
    #[test]
    fn test_profiler_system_timing() {
        let mut profiler = PerformanceProfiler::new(60.0);
        
        // Test system timing
        profiler.start_system("test_system");
        thread::sleep(Duration::from_millis(5));
        profiler.end_system();
        
        let stats = profiler.get_stats();
        assert!(stats.system_stats.contains_key("test_system"));
        
        let system_stats = &stats.system_stats["test_system"];
        assert!(system_stats.call_count == 1);
        assert!(system_stats.average_time >= Duration::from_millis(5));
    }
    
    #[test]
    fn test_metrics_collector() {
        let mut collector = MetricsCollector::new();
        
        // Test counter
        collector.increment_counter("test_counter", 5);
        assert_eq!(collector.get_counter("test_counter"), 5);
        
        collector.increment_counter("test_counter", 3);
        assert_eq!(collector.get_counter("test_counter"), 8);
        
        // Test gauge
        collector.set_gauge("test_gauge", 42.5);
        assert_eq!(collector.get_gauge("test_gauge"), Some(42.5));
        
        // Test histogram
        collector.record_histogram("test_histogram", 10.0);
        collector.record_histogram("test_histogram", 20.0);
        collector.record_histogram("test_histogram", 30.0);
        
        let stats = collector.get_histogram_stats("test_histogram").unwrap();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 20.0);
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 30.0);
    }
    
    #[test]
    fn test_memory_tracker() {
        let mut tracker = MemoryTracker::new();
        
        // Test allocation tracking
        tracker.record_allocation("test_alloc", 1024, "test", "Test allocation");
        let stats = tracker.get_stats();
        assert_eq!(stats.current_usage, 1024);
        assert_eq!(stats.allocation_count, 1);
        
        // Test deallocation
        tracker.record_deallocation("test_alloc");
        let stats = tracker.get_stats();
        assert_eq!(stats.current_usage, 0);
        assert_eq!(stats.deallocation_count, 1);
    }
    
    #[test]
    fn test_performance_grade() {
        let mut profiler = PerformanceProfiler::new(60.0);
        
        // Simulate good performance
        for _ in 0..10 {
            profiler.start_frame();
            thread::sleep(Duration::from_millis(16)); // ~60 FPS
            profiler.end_frame();
        }
        
        let grade = profiler.get_performance_grade();
        assert!(matches!(grade, PerformanceGrade::Excellent | PerformanceGrade::Good));
    }
    
    #[test]
    fn test_performance_warnings() {
        let mut profiler = PerformanceProfiler::new(60.0);
        
        // Simulate poor performance to trigger warnings
        profiler.start_frame();
        thread::sleep(Duration::from_millis(100)); // Very slow frame
        profiler.end_frame();
        
        let stats = profiler.get_stats();
        assert!(!stats.warnings.is_empty());
    }
    
    #[test]
    fn test_memory_pool() {
        let mut pool: MemoryPool<i32> = MemoryPool::new(10);
        
        // Test allocation
        let (index1, item1) = pool.allocate().unwrap();
        *item1 = 42;
        
        let (index2, item2) = pool.allocate().unwrap();
        *item2 = 84;
        
        // Test retrieval
        assert_eq!(pool.get(index1), Some(&42));
        assert_eq!(pool.get(index2), Some(&84));
        
        // Test deallocation
        pool.deallocate(index1);
        assert_eq!(pool.get(index1), Some(&0)); // Should be reset to default
        
        // Test stats
        let stats = pool.get_stats();
        assert_eq!(stats.allocated, 1); // Only index2 is still allocated
        assert_eq!(stats.available, 9); // 8 never used + 1 deallocated
    }
    
    #[test]
    fn test_reporter_recommendations() {
        let mut profiler = PerformanceProfiler::new(60.0);
        let memory_tracker = MemoryTracker::new();
        let metrics_collector = MetricsCollector::new();
        let mut reporter = PerformanceReporter::new("test_reports");
        
        // Simulate poor performance
        profiler.start_frame();
        thread::sleep(Duration::from_millis(50)); // Very slow frame (20 FPS)
        profiler.end_frame();
        
        let report = reporter.generate_report(&profiler, &memory_tracker, &metrics_collector);
        
        // Should have recommendations for low frame rate
        assert!(!report.recommendations.is_empty());
        assert!(report.recommendations.iter().any(|r| r.category == "Performance"));
    }
    
    #[test]
    fn test_format_memory_size() {
        assert_eq!(MemoryTracker::format_size(512), "512 B");
        assert_eq!(MemoryTracker::format_size(1024), "1.00 KB");
        assert_eq!(MemoryTracker::format_size(1024 * 1024), "1.00 MB");
        assert_eq!(MemoryTracker::format_size(1024 * 1024 * 1024), "1.00 GB");
    }
    
    #[test]
    fn test_profiler_macros() {
        init_profiler(60.0);
        
        // Test system profiling macro
        let result = profile_system!("test_macro_system", {
            thread::sleep(Duration::from_millis(1));
            42
        });
        
        assert_eq!(result, 42);
        
        // Verify the system was recorded
        let stats = with_profiler(|p| p.get_stats()).unwrap();
        assert!(stats.system_stats.contains_key("test_macro_system"));
    }
    
    #[test]
    fn test_memory_tracker_macros() {
        init_memory_tracker();
        
        // Test allocation tracking macro
        track_allocation!("test_macro_alloc", 2048, "test", "Test macro allocation");
        
        let stats = with_memory_tracker(|t| t.get_stats()).unwrap();
        assert_eq!(stats.current_usage, 2048);
        
        // Test deallocation macro
        track_deallocation!("test_macro_alloc");
        
        let stats = with_memory_tracker(|t| t.get_stats()).unwrap();
        assert_eq!(stats.current_usage, 0);
    }
}