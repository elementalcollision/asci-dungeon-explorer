use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::io::{self, Write};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use log::{info, warn, error};

use super::dialogue_ui::{DialogueUI, DialogueInput, DialogueUIEvent, CharacterPortrait, DialogueLayout, TypingConfig};
use super::dialogue_system_trait::{DialogueEntry, DialogueOptions};
use super::conversation_manager::ConversationManager;
use super::llama_dialogue_system::LlamaDialogueSystem;
use super::model_manager::ModelManager;

/// Dialogue UI manager that coordinates between the UI and dialogue systems
pub struct DialogueUIManager {
    dialogue_ui: DialogueUI,
    conversation_manager: Arc<Mutex<ConversationManager>>,
    current_character: Option<String>,
    current_location: String,
    pending_response: Option<String>,
    ui_enabled: bool,
}

impl DialogueUIManager {
    /// Create a new dialogue UI manager
    pub fn new(conversation_manager: Arc<Mutex<ConversationManager>>) -> Self {
        let mut ui_manager = DialogueUIManager {
            dialogue_ui: DialogueUI::new(),
            conversation_manager,
            current_character: None,
            current_location: "Unknown".to_string(),
            pending_response: None,
            ui_enabled: true,
        };
        
        // Set up default portraits
        ui_manager.setup_default_portraits();
        
        ui_manager
    }
    
    /// Set up default character portraits
    fn setup_default_portraits(&mut self) {
        use super::dialogue_ui::portrait_builder::{create_simple_portrait, PortraitStyle};
        
        // Add some default portraits
        self.dialogue_ui.add_portrait(create_simple_portrait("wizard", "Wizard", PortraitStyle::Wizard));
        self.dialogue_ui.add_portrait(create_simple_portrait("merchant", "Merchant", PortraitStyle::Merchant));
        self.dialogue_ui.add_portrait(create_simple_portrait("guard", "Guard", PortraitStyle::Guard));
        self.dialogue_ui.add_portrait(create_simple_portrait("warrior", "Warrior", PortraitStyle::Warrior));
    }
    
    /// Start a conversation with a character
    pub fn start_conversation(&mut self, character_id: &str, location: &str) -> Result<(), String> {
        if !self.ui_enabled {
            return Err("Dialogue UI is disabled".to_string());
        }
        
        // Start conversation with the conversation manager
        let context = match self.conversation_manager.lock() {
            Ok(mut manager) => {
                match manager.start_conversation(character_id, location) {
                    Ok(context) => context,
                    Err(e) => return Err(format!("Failed to start conversation: {}", e)),
                }
            },
            Err(_) => return Err("Failed to lock conversation manager".to_string()),
        };
        
        self.current_character = Some(character_id.to_string());
        self.current_location = location.to_string();
        
        // Show the dialogue UI
        self.dialogue_ui.show();
        
        // Generate initial greeting or response
        self.generate_initial_response(character_id)?;
        
        Ok(())
    }
    
    /// Generate initial response from character
    fn generate_initial_response(&mut self, character_id: &str) -> Result<(), String> {
        let greeting = "Hello there!"; // Default greeting
        
        match self.conversation_manager.lock() {
            Ok(mut manager) => {
                match manager.generate_response(character_id, greeting) {
                    Ok(response) => {
                        self.dialogue_ui.show_dialogue(&response);
                        Ok(())
                    },
                    Err(e) => {
                        warn!("Failed to generate initial response: {}", e);
                        
                        // Show fallback response
                        let fallback = DialogueEntry {
                            speaker: character_id.to_string(),
                            text: "Greetings, traveler.".to_string(),
                            emotion: Some("neutral".to_string()),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            metadata: HashMap::new(),
                        };
                        
                        self.dialogue_ui.show_dialogue(&fallback);
                        Ok(())
                    }
                }
            },
            Err(_) => Err("Failed to lock conversation manager".to_string()),
        }
    }
    
    /// End the current conversation
    pub fn end_conversation(&mut self) -> Result<(), String> {
        if let Some(character_id) = &self.current_character {
            match self.conversation_manager.lock() {
                Ok(mut manager) => {
                    if let Err(e) = manager.end_conversation(character_id) {
                        warn!("Failed to properly end conversation: {}", e);
                    }
                },
                Err(_) => return Err("Failed to lock conversation manager".to_string()),
            }
        }
        
        self.current_character = None;
        self.pending_response = None;
        self.dialogue_ui.hide();
        
        Ok(())
    }
    
    /// Send player input to the current character
    pub fn send_player_input(&mut self, input: &str) -> Result<(), String> {
        let character_id = match &self.current_character {
            Some(id) => id.clone(),
            None => return Err("No active conversation".to_string()),
        };
        
        // Generate response from character
        match self.conversation_manager.lock() {
            Ok(mut manager) => {
                match manager.generate_response(&character_id, input) {
                    Ok(response) => {
                        self.dialogue_ui.show_dialogue(&response);
                        Ok(())
                    },
                    Err(e) => {
                        error!("Failed to generate response: {}", e);
                        
                        // Show fallback response
                        let fallback = DialogueEntry {
                            speaker: character_id,
                            text: "I'm sorry, I didn't understand that.".to_string(),
                            emotion: Some("confused".to_string()),
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            metadata: HashMap::new(),
                        };
                        
                        self.dialogue_ui.show_dialogue(&fallback);
                        Ok(())
                    }
                }
            },
            Err(_) => Err("Failed to lock conversation manager".to_string()),
        }
    }
    
    /// Generate dialogue options for the player
    pub fn generate_dialogue_options(&mut self, topic: &str) -> Result<(), String> {
        let character_id = match &self.current_character {
            Some(id) => id.clone(),
            None => return Err("No active conversation".to_string()),
        };
        
        match self.conversation_manager.lock() {
            Ok(manager) => {
                match manager.generate_options(&character_id, topic) {
                    Ok(options) => {
                        self.dialogue_ui.show_options(options);
                        Ok(())
                    },
                    Err(e) => {
                        warn!("Failed to generate dialogue options: {}", e);
                        
                        // Show default options
                        let default_options = DialogueOptions {
                            options: vec![
                                super::dialogue_system_trait::DialogueOption {
                                    text: "Tell me more.".to_string(),
                                    next_state: None,
                                    effects: Vec::new(),
                                    requirements: Vec::new(),
                                    metadata: HashMap::new(),
                                },
                                super::dialogue_system_trait::DialogueOption {
                                    text: "I have to go.".to_string(),
                                    next_state: Some("farewell".to_string()),
                                    effects: Vec::new(),
                                    requirements: Vec::new(),
                                    metadata: HashMap::new(),
                                },
                            ],
                            timeout_seconds: Some(30),
                            default_option: Some(0),
                        };
                        
                        self.dialogue_ui.show_options(default_options);
                        Ok(())
                    }
                }
            },
            Err(_) => Err("Failed to lock conversation manager".to_string()),
        }
    }
    
    /// Handle input events
    pub fn handle_input(&mut self, event: &Event) -> Option<DialogueUIEvent> {
        if !self.ui_enabled || !self.dialogue_ui.is_visible() {
            return None;
        }
        
        let dialogue_input = match event {
            Event::Key(KeyEvent { code, modifiers, .. }) => {
                match code {
                    KeyCode::Up => Some(DialogueInput::Up),
                    KeyCode::Down => Some(DialogueInput::Down),
                    KeyCode::Left => Some(DialogueInput::Left),
                    KeyCode::Right => Some(DialogueInput::Right),
                    KeyCode::Enter => Some(DialogueInput::Confirm),
                    KeyCode::Esc => Some(DialogueInput::Cancel),
                    KeyCode::Char(' ') => Some(DialogueInput::Skip),
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        Some(DialogueInput::Cancel)
                    },
                    _ => None,
                }
            },
            _ => None,
        };
        
        if let Some(input) = dialogue_input {
            if let Some(ui_event) = self.dialogue_ui.handle_input(input) {
                self.handle_ui_event(&ui_event);
                return Some(ui_event);
            }
        }
        
        None
    }
    
    /// Handle UI events
    fn handle_ui_event(&mut self, event: &DialogueUIEvent) {
        match event {
            DialogueUIEvent::OptionSelected(index) => {
                if let Some(character_id) = &self.current_character {
                    match self.conversation_manager.lock() {
                        Ok(mut manager) => {
                            match manager.select_option(character_id, *index) {
                                Ok(response) => {
                                    self.dialogue_ui.show_dialogue(&response);
                                },
                                Err(e) => {
                                    error!("Failed to select dialogue option: {}", e);
                                }
                            }
                        },
                        Err(_) => {
                            error!("Failed to lock conversation manager");
                        }
                    }
                }
            },
            DialogueUIEvent::DialogueCancelled => {
                if let Err(e) = self.end_conversation() {
                    error!("Failed to end conversation: {}", e);
                }
            },
            DialogueUIEvent::ContinueRequested => {
                // Generate new dialogue options or continue conversation
                if let Err(e) = self.generate_dialogue_options("general") {
                    warn!("Failed to generate dialogue options: {}", e);
                }
            },
            _ => {}
        }
    }
    
    /// Update the dialogue UI (call this every frame)
    pub fn update(&mut self) -> Option<DialogueUIEvent> {
        if !self.ui_enabled {
            return None;
        }
        
        self.dialogue_ui.update()
    }
    
    /// Render the dialogue UI
    pub fn render<W: Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        if !self.ui_enabled {
            return Ok(());
        }
        
        self.dialogue_ui.render(writer)
    }
    
    /// Set the dialogue UI layout
    pub fn set_layout(&mut self, layout: DialogueLayout) {
        self.dialogue_ui.set_layout(layout);
    }
    
    /// Set the typing configuration
    pub fn set_typing_config(&mut self, config: TypingConfig) {
        self.dialogue_ui.set_typing_config(config);
    }
    
    /// Add a character portrait
    pub fn add_portrait(&mut self, portrait: CharacterPortrait) {
        self.dialogue_ui.add_portrait(portrait);
    }
    
    /// Enable or disable the dialogue UI
    pub fn set_enabled(&mut self, enabled: bool) {
        self.ui_enabled = enabled;
        
        if !enabled {
            self.dialogue_ui.hide();
        }
    }
    
    /// Check if the dialogue UI is enabled
    pub fn is_enabled(&self) -> bool {
        self.ui_enabled
    }
    
    /// Check if there's an active conversation
    pub fn has_active_conversation(&self) -> bool {
        self.current_character.is_some() && self.dialogue_ui.is_visible()
    }
    
    /// Get the current character ID
    pub fn get_current_character(&self) -> Option<&str> {
        self.current_character.as_deref()
    }
    
    /// Get the current location
    pub fn get_current_location(&self) -> &str {
        &self.current_location
    }
    
    /// Skip current typing animation
    pub fn skip_typing(&mut self) {
        self.dialogue_ui.skip_typing();
    }
    
    /// Clear dialogue history
    pub fn clear_history(&mut self) {
        self.dialogue_ui.clear_history();
    }
    
    /// Get dialogue history
    pub fn get_history(&self) -> &[DialogueEntry] {
        self.dialogue_ui.get_history()
    }
    
    /// Create a quick dialogue for testing
    pub fn show_quick_dialogue(&mut self, speaker: &str, text: &str) {
        let entry = DialogueEntry {
            speaker: speaker.to_string(),
            text: text.to_string(),
            emotion: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        };
        
        self.dialogue_ui.show();
        self.dialogue_ui.show_dialogue(&entry);
    }
    
    /// Show a system message
    pub fn show_system_message(&mut self, message: &str) {
        let entry = DialogueEntry {
            speaker: "System".to_string(),
            text: message.to_string(),
            emotion: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        };
        
        self.dialogue_ui.show();
        self.dialogue_ui.show_dialogue(&entry);
    }
    
    /// Adjust layout for different terminal sizes
    pub fn adjust_layout_for_terminal_size(&mut self, width: u16, height: u16) {
        let mut layout = DialogueLayout::default();
        
        // Adjust layout based on terminal size
        if height < 30 {
            // Small terminal - compact layout
            layout.dialogue_box_y = height - 10;
            layout.dialogue_box_height = 8;
        } else {
            // Normal terminal
            layout.dialogue_box_y = height - 12;
            layout.dialogue_box_height = 10;
        }
        
        if width < 80 {
            // Narrow terminal
            layout.dialogue_box_width = width - 4;
            layout.text_width = width - 20;
        } else {
            // Normal width
            layout.dialogue_box_width = 76;
            layout.text_width = 60;
        }
        
        // Update text and options positions
        layout.text_y = layout.dialogue_box_y + 1;
        layout.options_y = layout.text_y + 1;
        
        self.dialogue_ui.set_layout(layout);
    }
}