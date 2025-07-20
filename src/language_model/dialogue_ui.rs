use std::collections::HashMap;
use std::time::{Duration, Instant};
use crossterm::{
    cursor,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand, QueueableCommand,
};
use serde::{Serialize, Deserialize};

use super::dialogue_system_trait::{DialogueEntry, DialogueOptions, DialogueOption};

/// Character portrait represented as ASCII art
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterPortrait {
    pub character_id: String,
    pub name: String,
    pub ascii_art: Vec<String>,
    pub width: u16,
    pub height: u16,
    pub colors: HashMap<char, Color>,
}

/// Typing effect configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingConfig {
    pub chars_per_second: f32,
    pub pause_on_punctuation: Duration,
    pub skip_on_input: bool,
    pub sound_enabled: bool,
}

impl Default for TypingConfig {
    fn default() -> Self {
        TypingConfig {
            chars_per_second: 30.0,
            pause_on_punctuation: Duration::from_millis(200),
            skip_on_input: true,
            sound_enabled: false,
        }
    }
}

/// Dialogue UI state
#[derive(Debug, Clone)]
pub enum DialogueUIState {
    Hidden,
    ShowingText {
        current_text: String,
        target_text: String,
        char_index: usize,
        last_char_time: Instant,
    },
    WaitingForInput,
    ShowingOptions {
        options: DialogueOptions,
        selected_index: usize,
    },
    Transitioning,
}

/// Dialogue UI layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueLayout {
    pub dialogue_box_x: u16,
    pub dialogue_box_y: u16,
    pub dialogue_box_width: u16,
    pub dialogue_box_height: u16,
    pub portrait_x: u16,
    pub portrait_y: u16,
    pub text_x: u16,
    pub text_y: u16,
    pub text_width: u16,
    pub text_height: u16,
    pub options_x: u16,
    pub options_y: u16,
    pub options_width: u16,
    pub name_x: u16,
    pub name_y: u16,
}

impl Default for DialogueLayout {
    fn default() -> Self {
        DialogueLayout {
            dialogue_box_x: 2,
            dialogue_box_y: 20,
            dialogue_box_width: 76,
            dialogue_box_height: 8,
            portrait_x: 4,
            portrait_y: 21,
            text_x: 15,
            text_y: 21,
            text_width: 60,
            text_height: 6,
            options_x: 15,
            options_y: 22,
            options_width: 60,
            name_x: 15,
            name_y: 20,
        }
    }
}

/// Dialogue UI manager
pub struct DialogueUI {
    state: DialogueUIState,
    layout: DialogueLayout,
    typing_config: TypingConfig,
    portraits: HashMap<String, CharacterPortrait>,
    current_speaker: Option<String>,
    dialogue_history: Vec<DialogueEntry>,
    max_history: usize,
    visible: bool,
}

impl DialogueUI {
    /// Create a new dialogue UI
    pub fn new() -> Self {
        DialogueUI {
            state: DialogueUIState::Hidden,
            layout: DialogueLayout::default(),
            typing_config: TypingConfig::default(),
            portraits: HashMap::new(),
            current_speaker: None,
            dialogue_history: Vec::new(),
            max_history: 10,
            visible: false,
        }
    }
    
    /// Set the layout configuration
    pub fn set_layout(&mut self, layout: DialogueLayout) {
        self.layout = layout;
    }
    
    /// Set the typing configuration
    pub fn set_typing_config(&mut self, config: TypingConfig) {
        self.typing_config = config;
    }
    
    /// Add a character portrait
    pub fn add_portrait(&mut self, portrait: CharacterPortrait) {
        self.portraits.insert(portrait.character_id.clone(), portrait);
    }
    
    /// Show dialogue text with typing effect
    pub fn show_dialogue(&mut self, entry: &DialogueEntry) {
        self.current_speaker = Some(entry.speaker.clone());
        self.dialogue_history.push(entry.clone());
        
        // Trim history if it exceeds max
        while self.dialogue_history.len() > self.max_history {
            self.dialogue_history.remove(0);
        }
        
        // Start typing effect
        self.state = DialogueUIState::ShowingText {
            current_text: String::new(),
            target_text: entry.text.clone(),
            char_index: 0,
            last_char_time: Instant::now(),
        };
        
        self.visible = true;
    }
    
    /// Show dialogue options
    pub fn show_options(&mut self, options: DialogueOptions) {
        self.state = DialogueUIState::ShowingOptions {
            options,
            selected_index: 0,
        };
    }
    
    /// Update the dialogue UI (call this every frame)
    pub fn update(&mut self) -> Option<DialogueUIEvent> {
        match &mut self.state {
            DialogueUIState::ShowingText {
                current_text,
                target_text,
                char_index,
                last_char_time,
            } => {
                let now = Instant::now();
                let char_interval = Duration::from_secs_f32(1.0 / self.typing_config.chars_per_second);
                
                if now.duration_since(*last_char_time) >= char_interval && *char_index < target_text.len() {
                    // Add next character
                    if let Some(next_char) = target_text.chars().nth(*char_index) {
                        current_text.push(next_char);
                        *char_index += 1;
                        *last_char_time = now;
                        
                        // Pause on punctuation
                        if matches!(next_char, '.' | '!' | '?' | ',' | ';' | ':') {
                            *last_char_time = now + self.typing_config.pause_on_punctuation;
                        }
                    }
                }
                
                // Check if typing is complete
                if *char_index >= target_text.len() {
                    self.state = DialogueUIState::WaitingForInput;
                    return Some(DialogueUIEvent::TypingComplete);
                }
            },
            _ => {}
        }
        
        None
    }
    
    /// Handle input events
    pub fn handle_input(&mut self, input: DialogueInput) -> Option<DialogueUIEvent> {
        match &mut self.state {
            DialogueUIState::ShowingText { target_text, .. } => {
                match input {
                    DialogueInput::Skip | DialogueInput::Confirm => {
                        if self.typing_config.skip_on_input {
                            // Skip to end of text
                            self.state = DialogueUIState::WaitingForInput;
                            return Some(DialogueUIEvent::TypingSkipped);
                        }
                    },
                    _ => {}
                }
            },
            DialogueUIState::WaitingForInput => {
                match input {
                    DialogueInput::Confirm => {
                        return Some(DialogueUIEvent::ContinueRequested);
                    },
                    DialogueInput::Cancel => {
                        return Some(DialogueUIEvent::DialogueCancelled);
                    },
                    _ => {}
                }
            },
            DialogueUIState::ShowingOptions { options, selected_index } => {
                match input {
                    DialogueInput::Up => {
                        if *selected_index > 0 {
                            *selected_index -= 1;
                            return Some(DialogueUIEvent::OptionChanged(*selected_index));
                        }
                    },
                    DialogueInput::Down => {
                        if *selected_index < options.options.len() - 1 {
                            *selected_index += 1;
                            return Some(DialogueUIEvent::OptionChanged(*selected_index));
                        }
                    },
                    DialogueInput::Confirm => {
                        return Some(DialogueUIEvent::OptionSelected(*selected_index));
                    },
                    DialogueInput::Cancel => {
                        return Some(DialogueUIEvent::DialogueCancelled);
                    },
                    _ => {}
                }
            },
            _ => {}
        }
        
        None
    }
    
    /// Render the dialogue UI
    pub fn render<W: std::io::Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        if !self.visible {
            return Ok(());
        }
        
        // Clear dialogue area
        self.clear_dialogue_area(writer)?;
        
        // Draw dialogue box border
        self.draw_dialogue_box(writer)?;
        
        // Draw character portrait
        if let Some(speaker) = &self.current_speaker {
            self.draw_portrait(writer, speaker)?;
            self.draw_speaker_name(writer, speaker)?;
        }
        
        // Draw dialogue content based on state
        match &self.state {
            DialogueUIState::ShowingText { current_text, .. } => {
                self.draw_text(writer, current_text)?;
            },
            DialogueUIState::WaitingForInput => {
                if let Some(last_entry) = self.dialogue_history.last() {
                    self.draw_text(writer, &last_entry.text)?;
                    self.draw_continue_prompt(writer)?;
                }
            },
            DialogueUIState::ShowingOptions { options, selected_index } => {
                if let Some(last_entry) = self.dialogue_history.last() {
                    self.draw_text(writer, &last_entry.text)?;
                }
                self.draw_options(writer, options, *selected_index)?;
            },
            _ => {}
        }
        
        writer.flush()?;
        Ok(())
    }
    
    /// Clear the dialogue area
    fn clear_dialogue_area<W: std::io::Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        for y in self.layout.dialogue_box_y..self.layout.dialogue_box_y + self.layout.dialogue_box_height {
            writer.queue(cursor::MoveTo(self.layout.dialogue_box_x, y))?;
            writer.queue(Print(" ".repeat(self.layout.dialogue_box_width as usize)))?;
        }
        Ok(())
    }
    
    /// Draw the dialogue box border
    fn draw_dialogue_box<W: std::io::Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        let x = self.layout.dialogue_box_x;
        let y = self.layout.dialogue_box_y;
        let width = self.layout.dialogue_box_width;
        let height = self.layout.dialogue_box_height;
        
        // Top border
        writer.queue(cursor::MoveTo(x, y))?;
        writer.queue(Print("┌"))?;
        writer.queue(Print("─".repeat((width - 2) as usize)))?;
        writer.queue(Print("┐"))?;
        
        // Side borders
        for row in 1..height - 1 {
            writer.queue(cursor::MoveTo(x, y + row))?;
            writer.queue(Print("│"))?;
            writer.queue(cursor::MoveTo(x + width - 1, y + row))?;
            writer.queue(Print("│"))?;
        }
        
        // Bottom border
        writer.queue(cursor::MoveTo(x, y + height - 1))?;
        writer.queue(Print("└"))?;
        writer.queue(Print("─".repeat((width - 2) as usize)))?;
        writer.queue(Print("┘"))?;
        
        Ok(())
    }
    
    /// Draw character portrait
    fn draw_portrait<W: std::io::Write>(&self, writer: &mut W, speaker: &str) -> crossterm::Result<()> {
        if let Some(portrait) = self.portraits.get(speaker) {
            for (i, line) in portrait.ascii_art.iter().enumerate() {
                writer.queue(cursor::MoveTo(self.layout.portrait_x, self.layout.portrait_y + i as u16))?;
                
                // Apply colors if specified
                for ch in line.chars() {
                    if let Some(color) = portrait.colors.get(&ch) {
                        writer.queue(SetForegroundColor(*color))?;
                    }
                    writer.queue(Print(ch))?;
                    writer.queue(ResetColor)?;
                }
            }
        } else {
            // Default portrait placeholder
            let placeholder = vec![
                "┌─────┐",
                "│ ◉ ◉ │",
                "│  ─  │",
                "│ \\_/ │",
                "└─────┘",
            ];
            
            for (i, line) in placeholder.iter().enumerate() {
                writer.queue(cursor::MoveTo(self.layout.portrait_x, self.layout.portrait_y + i as u16))?;
                writer.queue(Print(line))?;
            }
        }
        
        Ok(())
    }
    
    /// Draw speaker name
    fn draw_speaker_name<W: std::io::Write>(&self, writer: &mut W, speaker: &str) -> crossterm::Result<()> {
        writer.queue(cursor::MoveTo(self.layout.name_x, self.layout.name_y))?;
        writer.queue(SetForegroundColor(Color::Yellow))?;
        writer.queue(Print(format!("{}: ", speaker)))?;
        writer.queue(ResetColor)?;
        Ok(())
    }
    
    /// Draw dialogue text with word wrapping
    fn draw_text<W: std::io::Write>(&self, writer: &mut W, text: &str) -> crossterm::Result<()> {
        let wrapped_lines = self.wrap_text(text, self.layout.text_width as usize);
        
        for (i, line) in wrapped_lines.iter().enumerate() {
            if i >= self.layout.text_height as usize {
                break; // Don't exceed text area height
            }
            
            writer.queue(cursor::MoveTo(self.layout.text_x, self.layout.text_y + i as u16))?;
            writer.queue(Print(line))?;
        }
        
        Ok(())
    }
    
    /// Draw continue prompt
    fn draw_continue_prompt<W: std::io::Write>(&self, writer: &mut W) -> crossterm::Result<()> {
        let prompt_y = self.layout.text_y + self.layout.text_height - 1;
        let prompt_x = self.layout.text_x + self.layout.text_width - 20;
        
        writer.queue(cursor::MoveTo(prompt_x, prompt_y))?;
        writer.queue(SetForegroundColor(Color::DarkGrey))?;
        writer.queue(Print("[Press Enter to continue]"))?;
        writer.queue(ResetColor)?;
        
        Ok(())
    }
    
    /// Draw dialogue options
    fn draw_options<W: std::io::Write>(&self, writer: &mut W, options: &DialogueOptions, selected_index: usize) -> crossterm::Result<()> {
        let start_y = self.layout.options_y;
        
        for (i, option) in options.options.iter().enumerate() {
            let y = start_y + i as u16;
            writer.queue(cursor::MoveTo(self.layout.options_x, y))?;
            
            if i == selected_index {
                writer.queue(SetForegroundColor(Color::Yellow))?;
                writer.queue(Print(format!("> {}", option.text)))?;
            } else {
                writer.queue(SetForegroundColor(Color::White))?;
                writer.queue(Print(format!("  {}", option.text)))?;
            }
            
            writer.queue(ResetColor)?;
        }
        
        Ok(())
    }
    
    /// Wrap text to fit within specified width
    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.len() + word.len() + 1 > width {
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                }
            }
            
            if !current_line.is_empty() {
                current_line.push(' ');
            }
            current_line.push_str(word);
        }
        
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
    }
    
    /// Hide the dialogue UI
    pub fn hide(&mut self) {
        self.visible = false;
        self.state = DialogueUIState::Hidden;
    }
    
    /// Show the dialogue UI
    pub fn show(&mut self) {
        self.visible = true;
    }
    
    /// Check if the dialogue UI is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Get the current state
    pub fn get_state(&self) -> &DialogueUIState {
        &self.state
    }
    
    /// Skip current typing animation
    pub fn skip_typing(&mut self) {
        if let DialogueUIState::ShowingText { target_text, .. } = &self.state {
            let target = target_text.clone();
            self.state = DialogueUIState::WaitingForInput;
        }
    }
    
    /// Clear dialogue history
    pub fn clear_history(&mut self) {
        self.dialogue_history.clear();
    }
    
    /// Get dialogue history
    pub fn get_history(&self) -> &[DialogueEntry] {
        &self.dialogue_history
    }
    
    /// Set maximum history length
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        
        // Trim current history if needed
        while self.dialogue_history.len() > self.max_history {
            self.dialogue_history.remove(0);
        }
    }
}

/// Input events for dialogue UI
#[derive(Debug, Clone, Copy)]
pub enum DialogueInput {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
    Skip,
}

/// Events emitted by dialogue UI
#[derive(Debug, Clone)]
pub enum DialogueUIEvent {
    TypingComplete,
    TypingSkipped,
    ContinueRequested,
    OptionSelected(usize),
    OptionChanged(usize),
    DialogueCancelled,
}

/// Helper functions for creating character portraits
pub mod portrait_builder {
    use super::*;
    
    /// Create a simple character portrait
    pub fn create_simple_portrait(character_id: &str, name: &str, style: PortraitStyle) -> CharacterPortrait {
        let (ascii_art, colors) = match style {
            PortraitStyle::Wizard => {
                let art = vec![
                    "  /\\   ".to_string(),
                    " /  \\  ".to_string(),
                    "| ◉◉ | ".to_string(),
                    "|  ─  |".to_string(),
                    "| \\_/ |".to_string(),
                    " \\___/ ".to_string(),
                    "  |||  ".to_string(),
                ];
                
                let mut colors = HashMap::new();
                colors.insert('/', Color::DarkBlue);
                colors.insert('\\', Color::DarkBlue);
                colors.insert('|', Color::DarkGrey);
                colors.insert('◉', Color::Blue);
                
                (art, colors)
            },
            PortraitStyle::Warrior => {
                let art = vec![
                    " [===] ".to_string(),
                    "│ ◉◉ │".to_string(),
                    "│  ─  │".to_string(),
                    "│ \\_/ │".to_string(),
                    " \\___/ ".to_string(),
                    "  |||  ".to_string(),
                ];
                
                let mut colors = HashMap::new();
                colors.insert('[', Color::DarkRed);
                colors.insert(']', Color::DarkRed);
                colors.insert('=', Color::DarkRed);
                colors.insert('│', Color::DarkGrey);
                colors.insert('◉', Color::Red);
                
                (art, colors)
            },
            PortraitStyle::Merchant => {
                let art = vec![
                    " $$$$$ ".to_string(),
                    "│ ◉◉ │".to_string(),
                    "│  ─  │".to_string(),
                    "│ \\_/ │".to_string(),
                    " \\___/ ".to_string(),
                    "  $$$  ".to_string(),
                ];
                
                let mut colors = HashMap::new();
                colors.insert('$', Color::Yellow);
                colors.insert('│', Color::DarkGrey);
                colors.insert('◉', Color::Green);
                
                (art, colors)
            },
            PortraitStyle::Guard => {
                let art = vec![
                    " ##### ".to_string(),
                    "│ ◉◉ │".to_string(),
                    "│  ─  │".to_string(),
                    "│ \\_/ │".to_string(),
                    " \\___/ ".to_string(),
                    "  |||  ".to_string(),
                ];
                
                let mut colors = HashMap::new();
                colors.insert('#', Color::DarkCyan);
                colors.insert('│', Color::DarkGrey);
                colors.insert('◉', Color::Cyan);
                
                (art, colors)
            },
        };
        
        CharacterPortrait {
            character_id: character_id.to_string(),
            name: name.to_string(),
            width: 7,
            height: ascii_art.len() as u16,
            ascii_art,
            colors,
        }
    }
    
    /// Portrait styles
    pub enum PortraitStyle {
        Wizard,
        Warrior,
        Merchant,
        Guard,
    }
}