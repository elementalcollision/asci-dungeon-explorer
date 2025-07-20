# Performance Profiling System

The performance profiling system provides comprehensive monitoring and analysis of game performance, including frame timing, system execution times, memory usage, and automated reporting.

## Features

### 1. Performance Profiler
- **Frame Time Tracking**: Monitors frame duration and calculates FPS
- **System Profiling**: Tracks execution time of individual game systems
- **Performance Warnings**: Automatically detects performance issues
- **Performance Grading**: Provides overall performance assessment

### 2. Memory Tracker
- **Allocation Tracking**: Monitors memory allocations and deallocations
- **Category-based Tracking**: Organizes memory usage by categories
- **Leak Detection**: Identifies potential memory leaks
- **Memory Pool Support**: Efficient memory management utilities

### 3. Metrics Collector
- **Counters**: Track event counts (input events, render calls, etc.)
- **Gauges**: Monitor current values (entity count, memory usage)
- **Histograms**: Collect value distributions (damage dealt, path lengths)
- **Timers**: Measure operation durations

### 4. Performance Reporter
- **Automated Reports**: Generate periodic performance reports
- **Recommendations**: Provide actionable performance improvement suggestions
- **Export Capabilities**: Save reports in JSON and CSV formats
- **Historical Analysis**: Track performance trends over time

## Quick Start

### Basic Setup

```rust
use ascii_dungeon_explorer::performance::*;

// Initialize global profiler and memory tracker
init_profiler(60.0); // Target 60 FPS
init_memory_tracker();

// Create performance integration
let mut performance = PerformanceIntegration::new(60.0, "performance_reports");
```

### Frame Profiling

```rust
// In your main game loop
performance.start_frame();

// Your game logic here
update_game_systems();

performance.end_frame(&game_state);
```

### System Profiling

```rust
// Method 1: Using macros
let result = profile_system!("ai_system", {
    ai_system.run();
});

// Method 2: Using integration
performance.profile_system("render_system", || {
    render_system.run();
});

// Method 3: Using ProfiledSystem wrapper
let profiled_ai = ProfiledSystem::new(AISystem::new(), "ai");
```

### Memory Tracking

```rust
// Track allocations
track_allocation!("player_data", 1024, "entities", "Player entity data");

// Track deallocations
track_deallocation!("player_data");

// Manual tracking
performance.record_allocation("item_123", 256, "items", "Sword item");
performance.record_deallocation("item_123");
```

### Metrics Collection

```rust
// Record events
performance.record_input_event();
performance.record_render_call();
performance.record_ai_decision();
performance.record_combat_event(25.0); // 25 damage dealt
performance.record_pathfinding_request(15); // Path length of 15
```

## Configuration

### Profiler Settings

```rust
// Set target FPS for warnings
performance.set_target_fps(60.0);

// Enable/disable profiling
performance.set_enabled(true);

// Set maximum samples to keep
with_profiler(|p| p.set_max_samples(1000));
```

### Memory Tracker Settings

```rust
// Set GC threshold
performance.set_memory_gc_threshold(100 * 1024 * 1024); // 100MB

// Set maximum samples
with_memory_tracker(|t| t.set_max_samples(1000));
```

### Reporter Settings

```rust
// Set report interval
performance.set_report_interval(Duration::from_secs(60));

// Enable auto-save
performance.set_auto_save_reports(true);
```

## Performance Analysis

### Getting Current Stats

```rust
// Get performance summary
let summary = performance.get_performance_summary();
println!("{}", summary);

// Get performance grade
let grade = performance.get_performance_grade();
println!("Performance grade: {}", grade);

// Check if performance is acceptable
if !performance.is_performance_acceptable() {
    println!("Performance issues detected!");
}
```

### Accessing Detailed Stats

```rust
// Get profiler stats
if let Some(stats) = with_profiler(|p| p.get_stats()) {
    println!("Average FPS: {:.1}", stats.average_fps);
    println!("Frame count: {}", stats.frame_count);
    
    for (system_name, system_stats) in &stats.system_stats {
        println!("{}: {:.2}ms ({:.1}% of frame)",
            system_name,
            system_stats.average_time.as_secs_f64() * 1000.0,
            system_stats.percentage_of_frame);
    }
}

// Get memory stats
if let Some(mem_stats) = with_memory_tracker(|t| t.get_stats()) {
    println!("Current memory: {}", 
        MemoryTracker::format_size(mem_stats.current_usage));
    println!("Peak memory: {}", 
        MemoryTracker::format_size(mem_stats.peak_usage));
}
```

### Performance Warnings

```rust
// Get recent warnings
let warnings = performance.get_recent_warnings(Duration::from_secs(10));
for warning in warnings {
    println!("Warning: {}", warning);
}
```

## Memory Categories

The system uses predefined memory categories for better organization:

```rust
use ascii_dungeon_explorer::performance::GameMemoryCategories;

// Available categories:
// - GameMemoryCategories::ENTITIES
// - GameMemoryCategories::COMPONENTS
// - GameMemoryCategories::RESOURCES
// - GameMemoryCategories::RENDERING
// - GameMemoryCategories::AUDIO
// - GameMemoryCategories::AI
// - GameMemoryCategories::PATHFINDING
// - GameMemoryCategories::DUNGEON
// - GameMemoryCategories::ITEMS
// - GameMemoryCategories::UI
// - GameMemoryCategories::SAVE_DATA
// - GameMemoryCategories::TEMPORARY

track_allocation!("enemy_ai", 512, GameMemoryCategories::AI, "Enemy AI state");
```

## Memory Pools

For efficient memory management:

```rust
// Create a memory pool
let mut entity_pool: MemoryPool<Entity> = MemoryPool::new(1000);

// Allocate from pool
if let Some((index, entity)) = entity_pool.allocate() {
    // Use entity
    entity.initialize();
    
    // Later, return to pool
    entity_pool.deallocate(index);
}

// Get pool statistics
let stats = entity_pool.get_stats();
println!("Pool utilization: {:.1}%", stats.utilization * 100.0);
```

## Exporting Data

### CSV Export

```rust
// Export performance data to CSV
performance.export_data("performance_data.csv")?;
```

### JSON Reports

```rust
// Reports are automatically saved as JSON if auto-save is enabled
// Manual save:
if let Some(report) = with_profiler(|profiler| {
    with_memory_tracker(|memory_tracker| {
        reporter.generate_report(profiler, memory_tracker, &metrics_collector)
    })
}) {
    reporter.save_report(&report)?;
}
```

## Best Practices

### 1. Selective Profiling
- Only enable profiling when needed for performance analysis
- Use different profiling levels for development vs. production

### 2. System Granularity
- Profile systems at appropriate granularity
- Don't over-profile small operations

### 3. Memory Tracking
- Use meaningful allocation IDs and descriptions
- Categorize allocations appropriately
- Clean up tracking when objects are destroyed

### 4. Performance Budgets
- Set realistic target FPS based on your game's requirements
- Monitor system percentages to identify bottlenecks

### 5. Regular Monitoring
- Set up automated reporting for continuous monitoring
- Review performance trends regularly
- Act on recommendations promptly

## Troubleshooting

### Common Issues

1. **High Memory Usage**: Check for memory leaks using the leak detection feature
2. **Low FPS**: Review system percentages to identify bottlenecks
3. **Frame Time Spikes**: Look for systems with high maximum execution times
4. **Memory Fragmentation**: Consider using memory pools for frequently allocated objects

### Performance Optimization Tips

1. **Batch Operations**: Group similar operations together
2. **Object Pooling**: Reuse objects instead of frequent allocation/deallocation
3. **System Scheduling**: Spread expensive operations across multiple frames
4. **Caching**: Cache expensive calculations when possible
5. **Profiling-Guided Optimization**: Use profiling data to guide optimization efforts

## Integration with Game Systems

### ECS Integration

```rust
// Wrap systems with profiling
let dispatcher = DispatcherBuilder::new()
    .with(ProfiledSystem::new(InputSystem, "input"), "input", &[])
    .with(ProfiledSystem::new(AISystem, "ai"), "ai", &[])
    .with(ProfiledSystem::new(RenderSystem, "render"), "render", &[])
    .build();
```

### Custom System Integration

```rust
impl<'a> System<'a> for MyCustomSystem {
    type SystemData = (/* your data */);
    
    fn run(&mut self, data: Self::SystemData) {
        profile_system!("my_custom_system", {
            // Your system logic here
        });
    }
}
```

This performance profiling system provides comprehensive monitoring capabilities to help you identify and resolve performance issues in your ASCII Dungeon Explorer game.