use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use super::config_system::ConfigManager;
use super::config_ui::{ConfigUI, ConfigUIResult};

/// Example of how to use the configuration system
pub fn config_system_example() -> crossterm::Result<()> {
    println!("Language Model Configuration System Example");
    println!("This will demonstrate the configuration interface for managing language model settings.");
    println!("Press Enter to continue, or Ctrl+C to exit.");
    
    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    // Initialize configuration manager
    let config_path = PathBuf::from("config/language_model.json");
    let models_directory = PathBuf::from("models");
    let config_manager = ConfigManager::new(config_path, models_directory);
    
    // Create configuration UI
    let mut config_ui = match ConfigUI::new(config_manager) {
        Ok(ui) => ui,
        Err(e) => {
            execute!(stdout, LeaveAlternateScreen)?;
            disable_raw_mode()?;
            eprintln!("Failed to initialize configuration UI: {}", e);
            return Ok(());
        }
    };
    
    // Run the configuration interface
    run_config_interface(&mut config_ui, &mut stdout)?;
    
    // Clean up terminal
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    
    println!("Configuration system example completed.");
    Ok(())
}

/// Run the interactive configuration interface
fn run_config_interface<W: Write>(config_ui: &mut ConfigUI, stdout: &mut W) -> crossterm::Result<()> {
    let mut running = true;
    let mut message: Option<String> = None;
    let mut error: Option<String> = None;
    
    while running {
        // Render UI
        config_ui.render(stdout)?;
        
        // Show message or error if any
        if let Some(msg) = &message {
            show_message(stdout, msg, false)?;
            message = None;
        }
        
        if let Some(err) = &error {
            show_message(stdout, err, true)?;
            error = None;
        }
        
        stdout.flush()?;
        
        // Handle input
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            match config_ui.handle_input(c) {
                                ConfigUIResult::Continue => {},
                                ConfigUIResult::Exit => {
                                    if config_ui.has_pending_changes() {
                                        // Ask user if they want to save changes
                                        if confirm_exit_with_changes(stdout)? {
                                            running = false;
                                        }
                                    } else {
                                        running = false;
                                    }
                                },
                                ConfigUIResult::Message(msg) => {
                                    message = Some(msg);
                                },
                                ConfigUIResult::Error(err) => {
                                    error = Some(err);
                                },
                            }
                        },
                        KeyCode::Enter => {
                            match config_ui.handle_input('\n') {
                                ConfigUIResult::Continue => {},
                                ConfigUIResult::Exit => running = false,
                                ConfigUIResult::Message(msg) => message = Some(msg),
                                ConfigUIResult::Error(err) => error = Some(err),
                            }
                        },
                        KeyCode::Esc => {
                            match config_ui.handle_input('\x1b') {
                                ConfigUIResult::Continue => {},
                                ConfigUIResult::Exit => running = false,
                                ConfigUIResult::Message(msg) => message = Some(msg),
                                ConfigUIResult::Error(err) => error = Some(err),
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
    
    Ok(())
}

/// Show a message at the bottom of the screen
fn show_message<W: Write>(stdout: &mut W, message: &str, is_error: bool) -> crossterm::Result<()> {
    use crossterm::{cursor, style::{Color, Print, ResetColor, SetForegroundColor}};
    
    stdout.queue(cursor::MoveTo(2, 22))?;
    
    if is_error {
        stdout.queue(SetForegroundColor(Color::Red))?;
        stdout.queue(Print(format!("Error: {}", message)))?;
    } else {
        stdout.queue(SetForegroundColor(Color::Green))?;
        stdout.queue(Print(message))?;
    }
    
    stdout.queue(ResetColor)?;
    
    // Clear the message after a short delay
    std::thread::sleep(Duration::from_millis(2000));
    
    stdout.queue(cursor::MoveTo(2, 22))?;
    stdout.queue(Print(" ".repeat(60)))?; // Clear the line
    
    Ok(())
}

/// Confirm exit with unsaved changes
fn confirm_exit_with_changes<W: Write>(stdout: &mut W) -> crossterm::Result<bool> {
    use crossterm::{cursor, style::{Color, Print, ResetColor, SetForegroundColor}, terminal::{Clear, ClearType}};
    
    // Show confirmation dialog
    stdout.queue(cursor::MoveTo(10, 10))?;
    stdout.queue(SetForegroundColor(Color::Yellow))?;
    stdout.queue(Print("┌─────────────────────────────────────────┐"))?;
    stdout.queue(cursor::MoveTo(10, 11))?;
    stdout.queue(Print("│ You have unsaved changes!               │"))?;
    stdout.queue(cursor::MoveTo(10, 12))?;
    stdout.queue(Print("│ Are you sure you want to exit?          │"))?;
    stdout.queue(cursor::MoveTo(10, 13))?;
    stdout.queue(Print("│                                         │"))?;
    stdout.queue(cursor::MoveTo(10, 14))?;
    stdout.queue(Print("│ Press Y to exit, N to continue editing  │"))?;
    stdout.queue(cursor::MoveTo(10, 15))?;
    stdout.queue(Print("└─────────────────────────────────────────┘"))?;
    stdout.queue(ResetColor)?;
    
    stdout.flush()?;
    
    // Wait for user input
    loop {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            // Clear the dialog
                            for y in 10..16 {
                                stdout.queue(cursor::MoveTo(10, y))?;
                                stdout.queue(Print(" ".repeat(43)))?;
                            }
                            return Ok(true);
                        },
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            // Clear the dialog
                            for y in 10..16 {
                                stdout.queue(cursor::MoveTo(10, y))?;
                                stdout.queue(Print(" ".repeat(43)))?;
                            }
                            return Ok(false);
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }
    }
}

/// Demonstrate configuration system programmatically
pub fn demonstrate_config_system() {
    println!("Demonstrating configuration system programmatically...");
    
    // Create configuration manager
    let config_path = PathBuf::from("config/demo_language_model.json");
    let models_directory = PathBuf::from("models");
    let mut config_manager = ConfigManager::new(config_path, models_directory);
    
    // Initialize
    if let Err(e) = config_manager.initialize() {
        println!("Failed to initialize config manager: {}", e);
        return;
    }
    
    // Show current configuration
    println!("\nCurrent Configuration:");
    let summary = config_manager.get_config_summary();
    for (key, value) in summary {
        println!("  {}: {}", key, value);
    }
    
    // Show available models
    println!("\nAvailable Models:");
    for model in config_manager.get_available_models() {
        let status = if model.available { "Available" } else { "Not Available" };
        println!("  {} - {} ({})", model.name, model.display_name, status);
        println!("    Path: {}", model.path.display());
        println!("    Description: {}", model.description);
        println!("    Requirements: {} MB RAM, {} CPU threads", 
            model.requirements.min_memory_mb, model.requirements.min_cpu_threads);
        println!();
    }
    
    // Demonstrate model switching
    println!("Demonstrating model switching...");
    let available_models: Vec<_> = config_manager.get_available_models().iter()
        .filter(|m| m.available)
        .collect();
    
    if available_models.len() > 1 {
        let first_model = &available_models[0];
        let second_model = &available_models[1];
        
        println!("Switching from {} to {}", first_model.name, second_model.name);
        
        match config_manager.switch_model(&second_model.name) {
            Ok(_) => println!("Successfully switched to {}", second_model.name),
            Err(e) => println!("Failed to switch model: {}", e),
        }
    } else {
        println!("Not enough available models to demonstrate switching");
    }
    
    // Demonstrate performance settings
    println!("\nDemonstrating performance settings...");
    let mut perf_settings = config_manager.get_config().language_model.performance_settings.clone();
    
    println!("Current cache enabled: {}", perf_settings.cache_enabled);
    perf_settings.cache_enabled = !perf_settings.cache_enabled;
    println!("Toggling cache to: {}", perf_settings.cache_enabled);
    
    config_manager.update_performance_settings(perf_settings);
    
    // Demonstrate UI settings
    println!("\nDemonstrating UI settings...");
    let mut ui_settings = config_manager.get_config().ui_settings.clone();
    
    println!("Current typing speed: {} chars/sec", ui_settings.typing_config.chars_per_second);
    ui_settings.typing_config.chars_per_second = 50.0;
    println!("Changed typing speed to: {} chars/sec", ui_settings.typing_config.chars_per_second);
    
    config_manager.update_ui_settings(ui_settings);
    
    // Save configuration
    println!("\nSaving configuration...");
    if let Err(e) = config_manager.save_config() {
        println!("Failed to save configuration: {}", e);
    } else {
        println!("Configuration saved successfully");
    }
    
    // Show final configuration
    println!("\nFinal Configuration Summary:");
    let final_summary = config_manager.get_config_summary();
    for (key, value) in final_summary {
        println!("  {}: {}", key, value);
    }
    
    println!("\nConfiguration system demonstration completed.");
}