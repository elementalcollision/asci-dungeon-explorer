mod components;
mod systems;
mod map;
mod game_state;
mod rendering;
mod input;
mod combat;
mod items;
mod ui;
mod ai;
mod guild;
mod language_model;
mod utils;
mod resources;
mod entity_factory;
mod character_creation;
mod inventory;

use crossterm::event::{Event, KeyCode};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use log::info;
use simplelog::{Config, LevelFilter, WriteLogger};
use std::fs::File;

use game_state::{GameState, StateType};
use rendering::terminal::with_terminal;

const FRAME_DURATION: Duration = Duration::from_millis(33); // ~30 FPS
const PERFORMANCE_SAMPLE_COUNT: usize = 100;

fn main() -> Result<(), Box<dyn Error>> {
    // Setup logging
    WriteLogger::init(
        LevelFilter::Info,
        Config::default(),
        File::create("ascii-dungeon-explorer.log")?,
    )?;
    
    info!("Starting ASCII Dungeon Explorer");
    
    // Setup terminal is handled by with_terminal
    
    // Create game state
    let mut game_state = GameState::new();
    
    // Performance monitoring
    let mut frame_times = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
    let mut update_times = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
    let mut render_times = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
    let mut input_times = Vec::with_capacity(PERFORMANCE_SAMPLE_COUNT);
    
    // Game loop
    let mut last_frame_time = Instant::now();
    let mut last_fps_update = Instant::now();
    let mut frames = 0;
    let mut current_fps = 0.0;
    
    'main_loop: loop {
        // Handle timing
        let frame_start = Instant::now();
        let elapsed = last_frame_time.elapsed();
        
        // Fixed time step
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
        
        last_frame_time = Instant::now();
        frames += 1;
        
        // Update FPS counter every second
        if last_fps_update.elapsed() >= Duration::from_secs(1) {
            current_fps = frames as f64 / last_fps_update.elapsed().as_secs_f64();
            frames = 0;
            last_fps_update = Instant::now();
            
            // Log performance metrics
            if !frame_times.is_empty() {
                let avg_frame_time = frame_times.iter().sum::<u128>() as f64 / frame_times.len() as f64;
                let avg_update_time = update_times.iter().sum::<u128>() as f64 / update_times.len() as f64;
                let avg_render_time = render_times.iter().sum::<u128>() as f64 / render_times.len() as f64;
                let avg_input_time = input_times.iter().sum::<u128>() as f64 / input_times.len() as f64;
                
                info!("FPS: {:.2}, Frame: {:.2}ms, Update: {:.2}ms, Render: {:.2}ms, Input: {:.2}ms", 
                    current_fps, 
                    avg_frame_time / 1_000.0, 
                    avg_update_time / 1_000.0, 
                    avg_render_time / 1_000.0,
                    avg_input_time / 1_000.0);
                
                frame_times.clear();
                update_times.clear();
                render_times.clear();
                input_times.clear();
            }
        }
        
        // Handle input
        let input_start = Instant::now();
        let key_event_opt = with_terminal(|terminal| {
            terminal.poll_key(0)
        }).unwrap_or(None);
        
        if let Some(key_event) = key_event_opt {
                    match key_event.code {
                        KeyCode::Char('q') => {
                            if game_state.state_stack.current() == StateType::MainMenu {
                                break 'main_loop;
                            } else {
                                game_state.state_stack.clear();
                            }
                        },
                        _ => game_state.handle_input(key_event),
                    }
                }
        let input_time = input_start.elapsed().as_nanos();
        input_times.push(input_time);
        
        // Update game state
        let update_start = Instant::now();
        game_state.update();
        let update_time = update_start.elapsed().as_nanos();
        update_times.push(update_time);
        
        // Render
        let render_start = Instant::now();
        game_state.render();
        let render_time = render_start.elapsed().as_nanos();
        render_times.push(render_time);
        
        // Check if game should exit
        if !game_state.running {
            break 'main_loop;
        }
        
        // Record frame time
        let frame_time = frame_start.elapsed().as_nanos();
        frame_times.push(frame_time);
        
        // Limit frame times array size
        if frame_times.len() > PERFORMANCE_SAMPLE_COUNT {
            frame_times.remove(0);
            update_times.remove(0);
            render_times.remove(0);
            input_times.remove(0);
        }
    }
    
    // Cleanup terminal is handled by with_terminal
    
    info!("Exiting ASCII Dungeon Explorer");
    
    Ok(())
}