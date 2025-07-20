use std::io::{self, stdout, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use super::dialogue_ui::{DialogueUI, DialogueLayout, TypingConfig, CharacterPortrait};
use super::dialogue_ui::portrait_builder::{create_simple_portrait, PortraitStyle};
use super::dialogue_ui_manager::DialogueUIManager;
use super::conversation_manager::ConversationManager;
use super::model_manager::ModelManager;
use super::llama_dialogue_system::LlamaDialogueSystem;
use super::dialogue_system_trait::{DialogueSystem, DialogueConfig, CharacterPersona, DialogueEntry};

/// Example of how to use the dialogue UI components
pub fn dialogue_ui_example() -> crossterm::Result<()> {
    println!("Starting dialogue UI example...");
    println!("This will demonstrate the dialogue interface with typing effects and character portraits.");
    println!("Press Enter to continue, or Ctrl+C to exit.");
    
    // Wait for user input
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    
    // Initialize components
    let mut model_manager = ModelManager::new();
    model_manager.initialize().expect("Failed to initialize model manager");
    
    // Add a model path (this would normally point to a real model)
    let model_path = PathBuf::from("models/tinyllama-1.1b.gguf");
    model_manager.add_model_path("tiny", model_path);
    
    // Try to load the model (will use fallback if not available)
    if let Err(e) = model_manager.load_model("tiny", None) {
        println!("Model not available, using fallback responses: {}", e);
    }
    
    let model_manager = Arc::new(Mutex::new(model_manager));
    
    // Create dialogue system
    let mut dialogue_system = LlamaDialogueSystem::new(model_manager.clone());
    dialogue_system.initialize().expect("Failed to initialize dialogue system");
    
    let config = DialogueConfig {
        max_history_length: 10,
        system_prompt: "You are a wise wizard in a fantasy world. Respond in character with mystical and helpful dialogue.".to_string(),
        temperature: 0.7,
        max_tokens: 100,
        stop_sequences: vec!["\n".to_string()],
        model_name: Some("tiny".to_string()),
        timeout_seconds: 10,
    };
    dialogue_system.set_config(config);
    
    // Create conversation manager
    let mut conversation_manager = ConversationManager::new(model_manager.clone());
    conversation_manager.initialize().expect("Failed to initialize conversation manager");
    conversation_manager.register_dialogue_system("default", dialogue_system);
    
    let conversation_manager = Arc::new(Mutex::new(conversation_manager));
    
    // Create dialogue UI manager
    let mut ui_manager = DialogueUIManager::new(conversation_manager.clone());
    
    // Customize typing config for demonstration
    let typing_config = TypingConfig {
        chars_per_second: 20.0, // Slower for demonstration
        pause_on_punctuation: Duration::from_millis(300),
        skip_on_input: true,
        sound_enabled: false,
    };
    ui_manager.set_typing_config(typing_config);
    
    // Add custom portraits
    let wizard_portrait = create_simple_portrait("eldrin", "Eldrin the Wise", PortraitStyle::Wizard);
    ui_manager.add_portrait(wizard_portrait);
    
    // Adjust layout for current terminal size
    let (width, height) = crossterm::terminal::size()?;
    ui_manager.adjust_layout_for_terminal_size(width, height);
    
    // Create a wizard persona for the conversation manager
    {
        let mut manager = conversation_manager.lock().unwrap();
        let wizard_persona = CharacterPersona {
            id: "eldrin".to_string(),
            name: "Eldrin the Wise".to_string(),
            description: "An ancient wizard with flowing robes and a long white beard".to_string(),
            background: "Eldrin has studied magic for centuries and knows many secrets of the arcane arts.".to_string(),
            traits: vec!["wise".to_string(), "patient".to_string(), "mysterious".to_string()],
            speech_style: "Formal and mystical, often speaking in riddles".to_string(),
            knowledge_base: vec![
                "The ancient dragons have returned to the northern mountains.".to_string(),
                "A powerful artifact lies hidden in the Forgotten Depths.".to_string(),
                "The stars speak of great changes coming to the realm.".to_string(),
            ],
            faction: Some("Order of the Azure Flame".to_string()),
            default_emotion: "calm".to_string(),
            available_emotions: vec!["calm".to_string(), "curious".to_string(), "concerned".to_string()],
            dialogue_templates: std::collections::HashMap::new(),
            metadata: std::collections::HashMap::new(),
        };
        
        // This would normally be done through the persona system
        // For this example, we'll just start the conversation directly
    }
    
    // Start the dialogue demonstration
    run_dialogue_demo(&mut ui_manager, &mut stdout)?;
    
    // Clean up terminal
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    
    println!("Dialogue UI example completed.");
    Ok(())
}

/// Run the interactive dialogue demonstration
fn run_dialogue_demo<W: Write>(ui_manager: &mut DialogueUIManager, stdout: &mut W) -> crossterm::Result<()> {
    // Start conversation with the wizard
    if let Err(e) = ui_manager.start_conversation("eldrin", "Tower of Magic") {
        // If conversation manager fails, show a demo dialogue directly
        ui_manager.show_quick_dialogue(
            "Eldrin the Wise",
            "Greetings, young adventurer. I sense you seek knowledge of the arcane arts. How may I assist you on your mystical journey?"
        );
    }
    
    let mut running = true;
    let mut input_buffer = String::new();
    let mut input_mode = false;
    
    // Show initial instructions
    show_instructions(stdout)?;
    
    while running {
        // Update UI
        if let Some(event) = ui_manager.update() {
            match event {
                super::dialogue_ui::DialogueUIEvent::TypingComplete => {
                    // Typing animation finished
                },
                super::dialogue_ui::DialogueUIEvent::ContinueRequested => {
                    // Player wants to continue - show some options or prompt for input
                    show_input_prompt(stdout)?;
                    input_mode = true;
                },
                super::dialogue_ui::DialogueUIEvent::OptionSelected(index) => {
                    // Player selected a dialogue option
                    show_status(stdout, &format!("Selected option {}", index + 1))?;
                },
                super::dialogue_ui::DialogueUIEvent::DialogueCancelled => {
                    running = false;
                },
                _ => {}
            }
        }
        
        // Render UI
        ui_manager.render(stdout)?;
        stdout.flush()?;
        
        // Handle input
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) => {
                    match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            running = false;
                        },
                        KeyCode::Char('1') => {
                            // Demo: Show different character response
                            ui_manager.show_quick_dialogue(
                                "Eldrin the Wise",
                                "Ah, you wish to learn of the ancient magics. Very well, but know that such knowledge comes with great responsibility."
                            );
                        },
                        KeyCode::Char('2') => {
                            // Demo: Show dialogue options
                            let options = super::dialogue_system_trait::DialogueOptions {
                                options: vec![
                                    super::dialogue_system_trait::DialogueOption {
                                        text: "Tell me about the dragons.".to_string(),
                                        next_state: None,
                                        effects: Vec::new(),
                                        requirements: Vec::new(),
                                        metadata: std::collections::HashMap::new(),
                                    },
                                    super::dialogue_system_trait::DialogueOption {
                                        text: "What is this artifact you mentioned?".to_string(),
                                        next_state: None,
                                        effects: Vec::new(),
                                        requirements: Vec::new(),
                                        metadata: std::collections::HashMap::new(),
                                    },
                                    super::dialogue_system_trait::DialogueOption {
                                        text: "I must go now.".to_string(),
                                        next_state: Some("farewell".to_string()),
                                        effects: Vec::new(),
                                        requirements: Vec::new(),
                                        metadata: std::collections::HashMap::new(),
                                    },
                                ],
                                timeout_seconds: Some(30),
                                default_option: Some(0),
                            };
                            
                            // This is a direct call for demo purposes
                            // Normally this would be handled through the conversation manager
                        },
                        KeyCode::Char('3') => {
                            // Demo: Show system message
                            ui_manager.show_system_message("The wizard's eyes glow with ancient wisdom as he prepares to share his knowledge.");
                        },
                        KeyCode::Char('s') => {
                            // Skip typing animation
                            ui_manager.skip_typing();
                        },
                        KeyCode::Char('h') => {
                            // Show help
                            show_help(stdout)?;
                        },
                        KeyCode::Enter if input_mode => {
                            // Send input to character
                            if !input_buffer.is_empty() {
                                if let Err(e) = ui_manager.send_player_input(&input_buffer) {
                                    // If conversation manager fails, show a demo response
                                    let responses = vec![
                                        "Indeed, your words show wisdom beyond your years.",
                                        "Hmm, an interesting perspective. Let me ponder this...",
                                        "The ancient texts speak of such things, yes.",
                                        "You ask the right questions, young seeker.",
                                        "Perhaps... but the path is fraught with danger.",
                                    ];
                                    
                                    let response_text = responses[input_buffer.len() % responses.len()];
                                    ui_manager.show_quick_dialogue("Eldrin the Wise", response_text);
                                }
                                input_buffer.clear();
                                input_mode = false;
                                clear_input_line(stdout)?;
                            }
                        },
                        KeyCode::Char(c) if input_mode => {
                            input_buffer.push(c);
                            show_input_line(stdout, &input_buffer)?;
                        },
                        KeyCode::Backspace if input_mode => {
                            input_buffer.pop();
                            show_input_line(stdout, &input_buffer)?;
                        },
                        _ => {
                            // Pass other input to UI manager
                            ui_manager.handle_input(&Event::Key(key_event));
                        }
                    }
                },
                Event::Resize(width, height) => {
                    ui_manager.adjust_layout_for_terminal_size(width, height);
                },
                _ => {}
            }
        }
    }
    
    Ok(())
}

/// Show initial instructions
fn show_instructions<W: Write>(stdout: &mut W) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Print};
    
    stdout.queue(cursor::MoveTo(2, 2))?;
    stdout.queue(Print("Dialogue UI Demo - Controls:"))?;
    stdout.queue(cursor::MoveTo(2, 3))?;
    stdout.queue(Print("1 - Show character response"))?;
    stdout.queue(cursor::MoveTo(2, 4))?;
    stdout.queue(Print("2 - Show dialogue options"))?;
    stdout.queue(cursor::MoveTo(2, 5))?;
    stdout.queue(Print("3 - Show system message"))?;
    stdout.queue(cursor::MoveTo(2, 6))?;
    stdout.queue(Print("s - Skip typing animation"))?;
    stdout.queue(cursor::MoveTo(2, 7))?;
    stdout.queue(Print("h - Show help"))?;
    stdout.queue(cursor::MoveTo(2, 8))?;
    stdout.queue(Print("q/Esc - Quit"))?;
    
    Ok(())
}

/// Show help information
fn show_help<W: Write>(stdout: &mut W) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Print, terminal::Clear, terminal::ClearType};
    
    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(cursor::MoveTo(2, 2))?;
    stdout.queue(Print("Dialogue UI Help:"))?;
    stdout.queue(cursor::MoveTo(2, 4))?;
    stdout.queue(Print("This demo shows the dialogue system with:"))?;
    stdout.queue(cursor::MoveTo(2, 5))?;
    stdout.queue(Print("- Character portraits (ASCII art)"))?;
    stdout.queue(cursor::MoveTo(2, 6))?;
    stdout.queue(Print("- Typing effects with configurable speed"))?;
    stdout.queue(cursor::MoveTo(2, 7))?;
    stdout.queue(Print("- Dialogue options with navigation"))?;
    stdout.queue(cursor::MoveTo(2, 8))?;
    stdout.queue(Print("- Word wrapping and text formatting"))?;
    stdout.queue(cursor::MoveTo(2, 9))?;
    stdout.queue(Print("- Integration with conversation management"))?;
    stdout.queue(cursor::MoveTo(2, 11))?;
    stdout.queue(Print("Press any key to return to demo..."))?;
    
    stdout.flush()?;
    
    // Wait for key press
    loop {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(_) = event::read()? {
                break;
            }
        }
    }
    
    // Clear and redraw instructions
    stdout.queue(Clear(ClearType::All))?;
    show_instructions(stdout)?;
    
    Ok(())
}

/// Show input prompt
fn show_input_prompt<W: Write>(stdout: &mut W) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Print};
    
    stdout.queue(cursor::MoveTo(2, 15))?;
    stdout.queue(Print("Enter your response: "))?;
    stdout.flush()?;
    
    Ok(())
}

/// Show input line
fn show_input_line<W: Write>(stdout: &mut W, input: &str) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Print};
    
    stdout.queue(cursor::MoveTo(22, 15))?;
    stdout.queue(Print(format!("{:<50}", input)))?; // Pad to clear previous text
    stdout.queue(cursor::MoveTo(22 + input.len() as u16, 15))?;
    stdout.flush()?;
    
    Ok(())
}

/// Clear input line
fn clear_input_line<W: Write>(stdout: &mut W) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Print};
    
    stdout.queue(cursor::MoveTo(2, 15))?;
    stdout.queue(Print(" ".repeat(70)))?; // Clear the line
    stdout.flush()?;
    
    Ok(())
}

/// Show status message
fn show_status<W: Write>(stdout: &mut W, message: &str) -> crossterm::Result<()> {
    use crossterm::{cursor, style::Print};
    
    stdout.queue(cursor::MoveTo(2, 17))?;
    stdout.queue(Print(format!("Status: {:<50}", message)))?;
    stdout.flush()?;
    
    Ok(())
}