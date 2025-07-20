use crossterm::{event::KeyCode, style::Color};
use specs::{World, Entity};
use std::path::PathBuf;
use crate::persistence::{SaveSystem, SaveSlot, SaveMetadata, SaveFile, SaveError};
use crate::ui::{
    ui_components::{UIComponent, UIRenderCommand, UIPanel, UIText, TextAlignment},
    menu_system::{MenuRenderer, MenuInput},
};

/// Save/Load UI state
#[derive(Debug, Clone, PartialEq)]
pub enum SaveLoadUIState {
    SaveMenu,
    LoadMenu,
    SlotDetails,
    ConfirmSave,
    ConfirmLoad,
    ConfirmDelete,
    ConfirmOverwrite,
    SaveInProgress,
    LoadInProgress,
    Error,
    Closed,
}

/// Save/Load operation type
#[derive(Debug, Clone, PartialEq)]
pub enum SaveLoadOperation {
    Save,
    Load,
    Delete,
    Overwrite,
}

/// Save/Load UI component
pub struct SaveLoadUI {
    pub state: SaveLoadUIState,
    pub operation: SaveLoadOperation,
    pub selected_slot: usize,
    pub scroll_offset: usize,
    pub save_slots: Vec<SaveSlot>,
    pub save_system: Option<SaveSystem>,
    pub current_save_name: String,
    pub error_message: String,
    pub confirmation_message: String,
    pub show_details: bool,
    pub slots_per_page: usize,
    pub last_operation_result: Option<Result<(), SaveError>>,
}

impl SaveLoadUI {
    pub fn new() -> Self {
        SaveLoadUI {
            state: SaveLoadUIState::Closed,
            operation: SaveLoadOperation::Save,
            selected_slot: 0,
            scroll_offset: 0,
            save_slots: Vec::new(),
            save_system: None,
            current_save_name: String::new(),
            error_message: String::new(),
            confirmation_message: String::new(),
            show_details: false,
            slots_per_page: 8,
            last_operation_result: None,
        }
    }

    pub fn open_save_menu(&mut self, save_system: SaveSystem, current_save_name: String) {
        self.save_system = Some(save_system);
        self.current_save_name = current_save_name;
        self.operation = SaveLoadOperation::Save;
        self.state = SaveLoadUIState::SaveMenu;
        self.refresh_save_slots();
        self.selected_slot = 0;
        self.scroll_offset = 0;
    }

    pub fn open_load_menu(&mut self, save_system: SaveSystem) {
        self.save_system = Some(save_system);
        self.operation = SaveLoadOperation::Load;
        self.state = SaveLoadUIState::LoadMenu;
        self.refresh_save_slots();
        self.selected_slot = 0;
        self.scroll_offset = 0;
    }

    pub fn close(&mut self) {
        self.state = SaveLoadUIState::Closed;
        self.error_message.clear();
        self.confirmation_message.clear();
        self.last_operation_result = None;
    }

    pub fn is_open(&self) -> bool {
        self.state != SaveLoadUIState::Closed
    }

    pub fn refresh_save_slots(&mut self) {
        if let Some(ref save_system) = self.save_system {
            match save_system.get_save_slots() {
                Ok(slots) => {
                    self.save_slots = slots;
                }
                Err(e) => {
                    self.error_message = format!("Failed to load save slots: {}", e);
                    self.state = SaveLoadUIState::Error;
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyCode) -> Option<SaveLoadResult> {
        match self.state {
            SaveLoadUIState::SaveMenu | SaveLoadUIState::LoadMenu => {
                self.handle_slot_selection_key(key)
            }
            SaveLoadUIState::SlotDetails => {
                self.handle_details_key(key)
            }
            SaveLoadUIState::ConfirmSave | SaveLoadUIState::ConfirmLoad | 
            SaveLoadUIState::ConfirmDelete | SaveLoadUIState::ConfirmOverwrite => {
                self.handle_confirmation_key(key)
            }
            SaveLoadUIState::Error => {
                self.handle_error_key(key)
            }
            _ => None,
        }
    }

    fn handle_slot_selection_key(&mut self, key: KeyCode) -> Option<SaveLoadResult> {
        match key {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                if self.selected_slot > 0 {
                    self.selected_slot -= 1;
                    self.ensure_slot_visible();
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                if self.selected_slot < self.save_slots.len().saturating_sub(1) {
                    self.selected_slot += 1;
                    self.ensure_slot_visible();
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.activate_selected_slot()
            }
            KeyCode::Char('d') => {
                if self.get_selected_slot().map(|s| s.is_occupied).unwrap_or(false) {
                    self.operation = SaveLoadOperation::Delete;
                    self.confirmation_message = format!(
                        "Are you sure you want to delete save slot {}?",
                        self.selected_slot + 1
                    );
                    self.state = SaveLoadUIState::ConfirmDelete;
                }
                None
            }
            KeyCode::Char('i') => {
                if self.get_selected_slot().map(|s| s.is_occupied).unwrap_or(false) {
                    self.show_details = true;
                    self.state = SaveLoadUIState::SlotDetails;
                }
                None
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close();
                Some(SaveLoadResult::Cancelled)
            }
            _ => None,
        }
    }

    fn handle_details_key(&mut self, key: KeyCode) -> Option<SaveLoadResult> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.show_details = false;
                self.state = match self.operation {
                    SaveLoadOperation::Save => SaveLoadUIState::SaveMenu,
                    SaveLoadOperation::Load => SaveLoadUIState::LoadMenu,
                    _ => SaveLoadUIState::SaveMenu,
                };
                None
            }
            _ => None,
        }
    }

    fn handle_confirmation_key(&mut self, key: KeyCode) -> Option<SaveLoadResult> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                self.execute_operation()
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state = match self.operation {
                    SaveLoadOperation::Save => SaveLoadUIState::SaveMenu,
                    SaveLoadOperation::Load => SaveLoadUIState::LoadMenu,
                    _ => SaveLoadUIState::SaveMenu,
                };
                None
            }
            _ => None,
        }
    }

    fn handle_error_key(&mut self, key: KeyCode) -> Option<SaveLoadResult> {
        match key {
            KeyCode::Enter | KeyCode::Esc | KeyCode::Char(' ') => {
                self.error_message.clear();
                self.state = match self.operation {
                    SaveLoadOperation::Save => SaveLoadUIState::SaveMenu,
                    SaveLoadOperation::Load => SaveLoadUIState::LoadMenu,
                    _ => SaveLoadUIState::SaveMenu,
                };
                None
            }
            _ => None,
        }
    }

    fn activate_selected_slot(&mut self) -> Option<SaveLoadResult> {
        if let Some(slot) = self.get_selected_slot() {
            match self.operation {
                SaveLoadOperation::Save => {
                    if slot.is_occupied {
                        self.confirmation_message = format!(
                            "Overwrite save in slot {}?",
                            self.selected_slot + 1
                        );
                        self.state = SaveLoadUIState::ConfirmOverwrite;
                    } else {
                        self.confirmation_message = format!(
                            "Save game to slot {}?",
                            self.selected_slot + 1
                        );
                        self.state = SaveLoadUIState::ConfirmSave;
                    }
                }
                SaveLoadOperation::Load => {
                    if slot.is_occupied && !slot.is_corrupted {
                        self.confirmation_message = format!(
                            "Load save from slot {}?",
                            self.selected_slot + 1
                        );
                        self.state = SaveLoadUIState::ConfirmLoad;
                    } else if slot.is_corrupted {
                        self.error_message = "This save file is corrupted and cannot be loaded.".to_string();
                        self.state = SaveLoadUIState::Error;
                    } else {
                        self.error_message = "This slot is empty.".to_string();
                        self.state = SaveLoadUIState::Error;
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn execute_operation(&mut self) -> Option<SaveLoadResult> {
        if let Some(ref save_system) = self.save_system.clone() {
            match self.operation {
                SaveLoadOperation::Save | SaveLoadOperation::Overwrite => {
                    self.state = SaveLoadUIState::SaveInProgress;
                    // In a real implementation, this would be async
                    // For now, we'll simulate the save operation
                    Some(SaveLoadResult::SaveRequested(self.selected_slot as u32))
                }
                SaveLoadOperation::Load => {
                    self.state = SaveLoadUIState::LoadInProgress;
                    match save_system.load_from_slot(self.selected_slot as u32) {
                        Ok(save_file) => {
                            Some(SaveLoadResult::LoadCompleted(save_file))
                        }
                        Err(e) => {
                            self.error_message = format!("Failed to load save: {}", e);
                            self.state = SaveLoadUIState::Error;
                            None
                        }
                    }
                }
                SaveLoadOperation::Delete => {
                    match save_system.delete_slot(self.selected_slot as u32) {
                        Ok(()) => {
                            self.refresh_save_slots();
                            self.state = match self.operation {
                                SaveLoadOperation::Save => SaveLoadUIState::SaveMenu,
                                _ => SaveLoadUIState::LoadMenu,
                            };
                            Some(SaveLoadResult::SlotDeleted(self.selected_slot as u32))
                        }
                        Err(e) => {
                            self.error_message = format!("Failed to delete save: {}", e);
                            self.state = SaveLoadUIState::Error;
                            None
                        }
                    }
                }
            }
        } else {
            self.error_message = "Save system not available.".to_string();
            self.state = SaveLoadUIState::Error;
            None
        }
    }

    fn get_selected_slot(&self) -> Option<&SaveSlot> {
        self.save_slots.get(self.selected_slot)
    }

    fn ensure_slot_visible(&mut self) {
        if self.selected_slot < self.scroll_offset {
            self.scroll_offset = self.selected_slot;
        } else if self.selected_slot >= self.scroll_offset + self.slots_per_page {
            self.scroll_offset = self.selected_slot - self.slots_per_page + 1;
        }
    }
}    
pub fn render(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        match self.state {
            SaveLoadUIState::SaveMenu => self.render_save_menu(screen_width, screen_height),
            SaveLoadUIState::LoadMenu => self.render_load_menu(screen_width, screen_height),
            SaveLoadUIState::SlotDetails => self.render_slot_details(screen_width, screen_height),
            SaveLoadUIState::ConfirmSave | SaveLoadUIState::ConfirmLoad | 
            SaveLoadUIState::ConfirmDelete | SaveLoadUIState::ConfirmOverwrite => {
                self.render_confirmation_dialog(screen_width, screen_height)
            }
            SaveLoadUIState::SaveInProgress => self.render_progress_dialog("Saving...", screen_width, screen_height),
            SaveLoadUIState::LoadInProgress => self.render_progress_dialog("Loading...", screen_width, screen_height),
            SaveLoadUIState::Error => self.render_error_dialog(screen_width, screen_height),
            SaveLoadUIState::Closed => Vec::new(),
        }
    }

    fn render_save_menu(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Main panel
        let panel_width = screen_width - 4;
        let panel_height = screen_height - 4;
        let panel = UIPanel::new(
            "Save Game".to_string(),
            2,
            2,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        // Current save name
        if !self.current_save_name.is_empty() {
            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y: 4,
                text: format!("Current Save: {}", self.current_save_name),
                fg: Color::Cyan,
                bg: Color::Black,
            });
        }

        // Render save slots
        commands.extend(self.render_save_slots(6, screen_width, screen_height));

        // Controls
        commands.extend(self.render_controls(screen_height - 3, "ENTER:Save D:Delete I:Info ESC:Cancel"));

        commands
    }

    fn render_load_menu(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Main panel
        let panel_width = screen_width - 4;
        let panel_height = screen_height - 4;
        let panel = UIPanel::new(
            "Load Game".to_string(),
            2,
            2,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        // Render save slots
        commands.extend(self.render_save_slots(4, screen_width, screen_height));

        // Controls
        commands.extend(self.render_controls(screen_height - 3, "ENTER:Load D:Delete I:Info ESC:Cancel"));

        commands
    }

    fn render_save_slots(&self, start_y: i32, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let visible_slots = self.save_slots.iter()
            .skip(self.scroll_offset)
            .take(self.slots_per_page);

        for (i, slot) in visible_slots.enumerate() {
            let actual_index = i + self.scroll_offset;
            let y = start_y + (i as i32 * 3);
            let is_selected = actual_index == self.selected_slot;

            if y + 2 >= screen_height - 4 {
                break; // Don't render beyond screen
            }

            commands.extend(self.render_single_slot(slot, 4, y, screen_width - 8, is_selected));
        }

        // Scroll indicators
        if self.scroll_offset > 0 {
            commands.push(UIRenderCommand::DrawText {
                x: screen_width - 10,
                y: start_y,
                text: "↑ More".to_string(),
                fg: Color::DarkGrey,
                bg: Color::Black,
            });
        }

        if self.scroll_offset + self.slots_per_page < self.save_slots.len() {
            commands.push(UIRenderCommand::DrawText {
                x: screen_width - 10,
                y: start_y + (self.slots_per_page as i32 * 3) - 1,
                text: "↓ More".to_string(),
                fg: Color::DarkGrey,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_single_slot(&self, slot: &SaveSlot, x: i32, y: i32, width: i32, is_selected: bool) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let (bg_color, border_color) = if is_selected {
            (Color::DarkBlue, Color::White)
        } else {
            (Color::Black, Color::DarkGrey)
        };

        // Slot background
        let slot_panel = UIPanel::new(
            "".to_string(),
            x,
            y,
            width,
            3,
        ).with_colors(border_color, bg_color, Color::White);

        commands.extend(slot_panel.render());

        // Slot number
        commands.push(UIRenderCommand::DrawText {
            x: x + 1,
            y: y + 1,
            text: format!("Slot {}", slot.slot_id + 1),
            fg: Color::Yellow,
            bg: bg_color,
        });

        if slot.is_occupied {
            if slot.is_corrupted {
                // Corrupted save
                commands.push(UIRenderCommand::DrawText {
                    x: x + 10,
                    y: y + 1,
                    text: "[CORRUPTED]".to_string(),
                    fg: Color::Red,
                    bg: bg_color,
                });
            } else {
                // Valid save
                let save_info = format!("{} - Level {} - {}",
                    slot.metadata.player_name,
                    slot.metadata.character_level,
                    slot.metadata.formatted_playtime()
                );

                commands.push(UIRenderCommand::DrawText {
                    x: x + 10,
                    y: y + 1,
                    text: save_info,
                    fg: Color::White,
                    bg: bg_color,
                });

                // Last saved
                commands.push(UIRenderCommand::DrawText {
                    x: x + 10,
                    y: y + 2,
                    text: format!("Saved: {}", slot.metadata.formatted_last_saved()),
                    fg: Color::DarkGrey,
                    bg: bg_color,
                });
            }

            // Backup indicator
            if slot.backup_available {
                commands.push(UIRenderCommand::DrawText {
                    x: x + width - 5,
                    y: y + 1,
                    text: "[B]".to_string(),
                    fg: Color::Green,
                    bg: bg_color,
                });
            }
        } else {
            // Empty slot
            commands.push(UIRenderCommand::DrawText {
                x: x + 10,
                y: y + 1,
                text: "[Empty Slot]".to_string(),
                fg: Color::DarkGrey,
                bg: bg_color,
            });
        }

        commands
    }

    fn render_slot_details(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if let Some(slot) = self.get_selected_slot() {
            let panel_width = screen_width - 8;
            let panel_height = screen_height - 8;
            let panel = UIPanel::new(
                format!("Slot {} Details", slot.slot_id + 1),
                4,
                4,
                panel_width,
                panel_height,
            ).with_colors(Color::White, Color::Black, Color::Yellow);

            commands.extend(panel.render());

            let mut y = 6;

            if slot.is_occupied && !slot.is_corrupted {
                let metadata = &slot.metadata;

                // Save name
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Save Name: {}", metadata.save_name),
                    fg: Color::White,
                    bg: Color::Black,
                });
                y += 1;

                // Player name
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Player: {}", metadata.player_name),
                    fg: Color::White,
                    bg: Color::Black,
                });
                y += 1;

                // Character level
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Level: {}", metadata.character_level),
                    fg: Color::Cyan,
                    bg: Color::Black,
                });
                y += 1;

                // Current depth
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Depth: {}", metadata.current_depth),
                    fg: Color::Cyan,
                    bg: Color::Black,
                });
                y += 1;

                // Playtime
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Playtime: {}", metadata.formatted_playtime()),
                    fg: Color::Green,
                    bg: Color::Black,
                });
                y += 1;

                // Difficulty
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Difficulty: {}", metadata.difficulty),
                    fg: Color::Yellow,
                    bg: Color::Black,
                });
                y += 1;

                // Game version
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Game Version: {}", metadata.game_version),
                    fg: Color::DarkGrey,
                    bg: Color::Black,
                });
                y += 1;

                // Achievements
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: format!("Achievements: {}", metadata.achievements_count),
                    fg: Color::Magenta,
                    bg: Color::Black,
                });
                y += 2;

                // File info
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: "File Information:".to_string(),
                    fg: Color::Yellow,
                    bg: Color::Black,
                });
                y += 1;

                commands.push(UIRenderCommand::DrawText {
                    x: 8,
                    y,
                    text: format!("Created: {}", metadata.formatted_last_saved()),
                    fg: Color::DarkGrey,
                    bg: Color::Black,
                });
                y += 1;

                commands.push(UIRenderCommand::DrawText {
                    x: 8,
                    y,
                    text: format!("Last Saved: {}", metadata.formatted_last_saved()),
                    fg: Color::DarkGrey,
                    bg: Color::Black,
                });
                y += 1;

                if slot.backup_available {
                    commands.push(UIRenderCommand::DrawText {
                        x: 8,
                        y,
                        text: "Backup Available: Yes".to_string(),
                        fg: Color::Green,
                        bg: Color::Black,
                    });
                } else {
                    commands.push(UIRenderCommand::DrawText {
                        x: 8,
                        y,
                        text: "Backup Available: No".to_string(),
                        fg: Color::Red,
                        bg: Color::Black,
                    });
                }
            } else if slot.is_corrupted {
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: "This save file is corrupted and cannot be loaded.".to_string(),
                    fg: Color::Red,
                    bg: Color::Black,
                });
                y += 2;

                if slot.backup_available {
                    commands.push(UIRenderCommand::DrawText {
                        x: 6,
                        y,
                        text: "A backup is available and may be recoverable.".to_string(),
                        fg: Color::Yellow,
                        bg: Color::Black,
                    });
                }
            } else {
                commands.push(UIRenderCommand::DrawText {
                    x: 6,
                    y,
                    text: "This slot is empty.".to_string(),
                    fg: Color::DarkGrey,
                    bg: Color::Black,
                });
            }

            // Controls
            commands.extend(self.render_controls(screen_height - 5, "ESC:Back"));
        }

        commands
    }

    fn render_confirmation_dialog(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let panel_width = 50;
        let panel_height = 8;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        let panel = UIPanel::new(
            "Confirmation".to_string(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::Yellow, Color::DarkBlue, Color::White);

        commands.extend(panel.render());

        // Confirmation message
        let wrapped_lines = self.wrap_text(&self.confirmation_message, (panel_width - 4) as usize);
        let mut y = panel_y + 2;

        for line in wrapped_lines {
            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y,
                text: line,
                fg: Color::White,
                bg: Color::DarkBlue,
            });
            y += 1;
        }

        // Options
        y += 1;
        commands.push(UIRenderCommand::DrawText {
            x: panel_x + 2,
            y,
            text: "Y: Yes    N: No    ESC: Cancel".to_string(),
            fg: Color::Yellow,
            bg: Color::DarkBlue,
        });

        commands
    }

    fn render_progress_dialog(&self, message: &str, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let panel_width = 40;
        let panel_height = 6;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        let panel = UIPanel::new(
            "Please Wait".to_string(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::Cyan, Color::DarkBlue, Color::White);

        commands.extend(panel.render());

        // Progress message
        commands.push(UIRenderCommand::DrawText {
            x: panel_x + 2,
            y: panel_y + 2,
            text: message.to_string(),
            fg: Color::White,
            bg: Color::DarkBlue,
        });

        // Simple progress animation (dots)
        let dots = "...".repeat((std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() % 4) as usize);

        commands.push(UIRenderCommand::DrawText {
            x: panel_x + 2,
            y: panel_y + 3,
            text: dots,
            fg: Color::Cyan,
            bg: Color::DarkBlue,
        });

        commands
    }

    fn render_error_dialog(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let panel_width = 60;
        let panel_height = 10;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        let panel = UIPanel::new(
            "Error".to_string(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::Red, Color::Black, Color::White);

        commands.extend(panel.render());

        // Error message
        let wrapped_lines = self.wrap_text(&self.error_message, (panel_width - 4) as usize);
        let mut y = panel_y + 2;

        for line in wrapped_lines {
            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y,
                text: line,
                fg: Color::White,
                bg: Color::Black,
            });
            y += 1;
        }

        // Continue instruction
        y += 1;
        commands.push(UIRenderCommand::DrawText {
            x: panel_x + 2,
            y,
            text: "Press any key to continue...".to_string(),
            fg: Color::Yellow,
            bg: Color::Black,
        });

        commands
    }

    fn render_controls(&self, y: i32, controls_text: &str) -> Vec<UIRenderCommand> {
        vec![UIRenderCommand::DrawText {
            x: 4,
            y,
            text: controls_text.to_string(),
            fg: Color::DarkGrey,
            bg: Color::Black,
        }]
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![text.to_string()];
        }

        let mut lines = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_line = String::new();

        for word in words {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= width {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }

        if !current_line.is_empty() {
            lines.push(current_line);
        }

        if lines.is_empty() {
            lines.push(String::new());
        }

        lines
    }

    /// Complete a save operation
    pub fn complete_save_operation(&mut self, result: Result<(), SaveError>) {
        self.last_operation_result = Some(result.clone());
        
        match result {
            Ok(()) => {
                self.refresh_save_slots();
                self.state = SaveLoadUIState::SaveMenu;
            }
            Err(e) => {
                self.error_message = format!("Save failed: {}", e);
                self.state = SaveLoadUIState::Error;
            }
        }
    }

    /// Get the current save operation result
    pub fn get_last_result(&self) -> Option<&Result<(), SaveError>> {
        self.last_operation_result.as_ref()
    }
}

/// Result of save/load operations
#[derive(Debug, Clone)]
pub enum SaveLoadResult {
    SaveRequested(u32),      // Slot ID
    LoadCompleted(SaveFile), // Loaded save file
    SlotDeleted(u32),        // Deleted slot ID
    Cancelled,
}

impl UIComponent for SaveLoadUI {
    fn render(&self, _x: i32, _y: i32, width: i32, height: i32) -> Vec<UIRenderCommand> {
        self.render(width, height)
    }

    fn handle_input(&mut self, input: char) -> bool {
        let key = match input {
            '\n' => KeyCode::Enter,
            '\x1b' => KeyCode::Esc,
            '\t' => KeyCode::Tab,
            'k' | 'w' => KeyCode::Up,
            'j' | 's' => KeyCode::Down,
            'h' | 'a' => KeyCode::Left,
            'l' | 'd' => KeyCode::Right,
            c => KeyCode::Char(c),
        };

        self.handle_key(key).is_some()
    }

    fn is_focused(&self) -> bool {
        self.is_open()
    }

    fn set_focus(&mut self, focused: bool) {
        if !focused {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_save_system() -> (SaveSystem, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let save_system = SaveSystem::new(temp_dir.path()).unwrap();
        (save_system, temp_dir)
    }

    #[test]
    fn test_save_load_ui_creation() {
        let ui = SaveLoadUI::new();
        
        assert_eq!(ui.state, SaveLoadUIState::Closed);
        assert!(!ui.is_open());
        assert_eq!(ui.selected_slot, 0);
        assert!(ui.save_slots.is_empty());
    }

    #[test]
    fn test_open_save_menu() {
        let mut ui = SaveLoadUI::new();
        let (save_system, _temp_dir) = create_test_save_system();
        
        ui.open_save_menu(save_system, "Test Save".to_string());
        
        assert!(ui.is_open());
        assert_eq!(ui.state, SaveLoadUIState::SaveMenu);
        assert_eq!(ui.operation, SaveLoadOperation::Save);
        assert_eq!(ui.current_save_name, "Test Save");
        assert!(!ui.save_slots.is_empty()); // Should have loaded slots
    }

    #[test]
    fn test_open_load_menu() {
        let mut ui = SaveLoadUI::new();
        let (save_system, _temp_dir) = create_test_save_system();
        
        ui.open_load_menu(save_system);
        
        assert!(ui.is_open());
        assert_eq!(ui.state, SaveLoadUIState::LoadMenu);
        assert_eq!(ui.operation, SaveLoadOperation::Load);
        assert!(!ui.save_slots.is_empty()); // Should have loaded slots
    }

    #[test]
    fn test_close() {
        let mut ui = SaveLoadUI::new();
        let (save_system, _temp_dir) = create_test_save_system();
        
        ui.open_save_menu(save_system, "Test".to_string());
        assert!(ui.is_open());
        
        ui.close();
        assert!(!ui.is_open());
        assert_eq!(ui.state, SaveLoadUIState::Closed);
        assert!(ui.error_message.is_empty());
        assert!(ui.confirmation_message.is_empty());
    }

    #[test]
    fn test_slot_navigation() {
        let mut ui = SaveLoadUI::new();
        let (save_system, _temp_dir) = create_test_save_system();
        
        ui.open_save_menu(save_system, "Test".to_string());
        
        assert_eq!(ui.selected_slot, 0);
        
        // Navigate down
        ui.handle_key(KeyCode::Down);
        assert_eq!(ui.selected_slot, 1);
        
        // Navigate up
        ui.handle_key(KeyCode::Up);
        assert_eq!(ui.selected_slot, 0);
        
        // Can't go below 0
        ui.handle_key(KeyCode::Up);
        assert_eq!(ui.selected_slot, 0);
    }

    #[test]
    fn test_text_wrapping() {
        let ui = SaveLoadUI::new();
        
        let text = "This is a long line that should be wrapped";
        let wrapped = ui.wrap_text(text, 20);
        
        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 20);
        }
    }

    #[test]
    fn test_save_operation_completion() {
        let mut ui = SaveLoadUI::new();
        
        // Test successful save
        ui.complete_save_operation(Ok(()));
        assert_eq!(ui.state, SaveLoadUIState::SaveMenu);
        assert!(ui.get_last_result().unwrap().is_ok());
        
        // Test failed save
        ui.complete_save_operation(Err(SaveError::IoError("Test error".to_string())));
        assert_eq!(ui.state, SaveLoadUIState::Error);
        assert!(!ui.error_message.is_empty());
        assert!(ui.get_last_result().unwrap().is_err());
    }

    #[test]
    fn test_slot_selection() {
        let mut ui = SaveLoadUI::new();
        let (save_system, _temp_dir) = create_test_save_system();
        
        ui.open_save_menu(save_system, "Test".to_string());
        
        // Select empty slot for saving
        let result = ui.handle_key(KeyCode::Enter);
        assert!(result.is_none()); // Should show confirmation, not return result yet
        assert_eq!(ui.state, SaveLoadUIState::ConfirmSave);
        
        // Confirm save
        let result = ui.handle_key(KeyCode::Char('y'));
        assert!(result.is_some());
        
        if let Some(SaveLoadResult::SaveRequested(slot_id)) = result {
            assert_eq!(slot_id, 0);
        } else {
            panic!("Expected SaveRequested result");
        }
    }
} 
           export_path: PathBuf::from("./saves/export.sav"),
            import_path: PathBuf::from("./saves/"),
            scroll_offset: 0,
            max_visible_slots: 10,
        }
    }

    /// Open save menu
    pub fn open_save_menu(&mut self, save_slots: Vec<SaveSlot>) {
        self.state = SaveLoadUIState::SaveMenu;
        self.save_slots = save_slots;
        self.selected_slot = 0;
        self.scroll_offset = 0;
        self.clear_messages();
    }

    /// Open load menu
    pub fn open_load_menu(&mut self, save_slots: Vec<SaveSlot>) {
        self.state = SaveLoadUIState::LoadMenu;
        self.save_slots = save_slots;
        self.selected_slot = 0;
        self.scroll_offset = 0;
        self.clear_messages();
    }

    /// Close UI
    pub fn close(&mut self) {
        self.state = SaveLoadUIState::Closed;
        self.clear_messages();
    }

    /// Clear messages
    fn clear_messages(&mut self) {
        self.error_message.clear();
        self.success_message.clear();
    }

    /// Handle input
    pub fn handle_input(&mut self, key: KeyCode) -> Option<SaveLoadAction> {
        match self.state {
            SaveLoadUIState::SaveMenu | SaveLoadUIState::LoadMenu => {
                self.handle_slot_selection_input(key)
            },
            SaveLoadUIState::DeleteConfirm => {
                self.handle_delete_confirm_input(key)
            },
            SaveLoadUIState::ExportMenu => {
                self.handle_export_input(key)
            },
            SaveLoadUIState::ImportMenu => {
                self.handle_import_input(key)
            },
            SaveLoadUIState::ErrorDialog | SaveLoadUIState::SaveComplete | SaveLoadUIState::LoadComplete => {
                match key {
                    KeyCode::Enter | KeyCode::Esc => {
                        self.close();
                        None
                    },
                    _ => None,
                }
            },
            SaveLoadUIState::Closed => None,
        }
    }

    /// Handle slot selection input
    fn handle_slot_selection_input(&mut self, key: KeyCode) -> Option<SaveLoadAction> {
        match key {
            KeyCode::Up => {
                if self.selected_slot > 0 {
                    self.selected_slot -= 1;
                    if self.selected_slot < self.scroll_offset {
                        self.scroll_offset = self.selected_slot;
                    }
                }
                None
            },
            KeyCode::Down => {
                if self.selected_slot < self.save_slots.len().saturating_sub(1) {
                    self.selected_slot += 1;
                    if self.selected_slot >= self.scroll_offset + self.max_visible_slots {
                        self.scroll_offset = self.selected_slot - self.max_visible_slots + 1;
                    }
                }
                None
            },
            KeyCode::Enter => {
                if self.selected_slot < self.save_slots.len() {
                    let slot_id = self.save_slots[self.selected_slot].slot_id;
                    match self.state {
                        SaveLoadUIState::SaveMenu => Some(SaveLoadAction::Save(slot_id)),
                        SaveLoadUIState::LoadMenu => {
                            if self.save_slots[self.selected_slot].is_occupied {
                                Some(SaveLoadAction::Load(slot_id))
                            } else {
                                self.show_error("Cannot load from empty slot");
                                None
                            }
                        },
                        _ => None,
                    }
                } else {
                    None
                }
            },
            KeyCode::Delete => {
                if self.selected_slot < self.save_slots.len() && 
                   self.save_slots[self.selected_slot].is_occupied {
                    let slot_id = self.save_slots[self.selected_slot].slot_id;
                    self.confirm_action = Some(SaveLoadAction::Delete(slot_id));
                    self.state = SaveLoadUIState::DeleteConfirm;
                }
                None
            },
            KeyCode::Char('e') | KeyCode::Char('E') => {
                self.state = SaveLoadUIState::ExportMenu;
                None
            },
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.state = SaveLoadUIState::ImportMenu;
                None
            },
            KeyCode::Esc => Some(SaveLoadAction::Cancel),
            _ => None,
        }
    }

    /// Handle delete confirmation input
    fn handle_delete_confirm_input(&mut self, key: KeyCode) -> Option<SaveLoadAction> {
        match key {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                if let Some(action) = self.confirm_action.take() {
                    Some(action)
                } else {
                    None
                }
            },
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.confirm_action = None;
                self.state = SaveLoadUIState::SaveMenu; // Return to previous state
                None
            },
            _ => None,
        }
    }

    /// Handle export input
    fn handle_export_input(&mut self, key: KeyCode) -> Option<SaveLoadAction> {
        match key {
            KeyCode::Enter => {
                Some(SaveLoadAction::Export(self.export_path.clone()))
            },
            KeyCode::Esc => {
                self.state = SaveLoadUIState::SaveMenu;
                None
            },
            _ => None, // In a real implementation, handle text input for path
        }
    }

    /// Handle import input
    fn handle_import_input(&mut self, key: KeyCode) -> Option<SaveLoadAction> {
        match key {
            KeyCode::Enter => {
                Some(SaveLoadAction::Import(self.import_path.clone()))
            },
            KeyCode::Esc => {
                self.state = SaveLoadUIState::LoadMenu;
                None
            },
            _ => None, // In a real implementation, handle text input for path
        }
    }

    /// Show error message
    pub fn show_error(&mut self, message: &str) {
        self.error_message = message.to_string();
        self.state = SaveLoadUIState::ErrorDialog;
    }

    /// Show success message
    pub fn show_success(&mut self, message: &str) {
        self.success_message = message.to_string();
        match self.state {
            SaveLoadUIState::SaveMenu => self.state = SaveLoadUIState::SaveComplete,
            SaveLoadUIState::LoadMenu => self.state = SaveLoadUIState::LoadComplete,
            _ => {},
        }
    }

    /// Update save slots
    pub fn update_save_slots(&mut self, save_slots: Vec<SaveSlot>) {
        self.save_slots = save_slots;
        // Adjust selected slot if it's out of bounds
        if self.selected_slot >= self.save_slots.len() && !self.save_slots.is_empty() {
            self.selected_slot = self.save_slots.len() - 1;
        }
    }

    /// Get visible save slots
    fn get_visible_slots(&self) -> &[SaveSlot] {
        let start = self.scroll_offset;
        let end = (start + self.max_visible_slots).min(self.save_slots.len());
        &self.save_slots[start..end]
    }

    /// Format slot display text
    fn format_slot_text(&self, slot: &SaveSlot, index: usize) -> String {
        if slot.is_occupied {
            if let Some(ref metadata) = slot.metadata {
                format!("Slot {}: {} - Level {} - Depth {} ({})", 
                    slot.slot_id,
                    metadata.player_name,
                    metadata.character_level,
                    metadata.current_depth,
                    metadata.created_at.format("%Y-%m-%d %H:%M")
                )
            } else {
                format!("Slot {}: Occupied", slot.slot_id)
            }
        } else {
            format!("Slot {}: Empty", slot.slot_id)
        }
    }

    /// Check if UI is open
    pub fn is_open(&self) -> bool {
        self.state != SaveLoadUIState::Closed
    }

    /// Get current state
    pub fn get_state(&self) -> &SaveLoadUIState {
        &self.state
    }
}

impl UIComponent for SaveLoadUI {
    fn render(&self) -> Vec<UIRenderCommand> {
        match self.state {
            SaveLoadUIState::SaveMenu => self.render_save_menu(),
            SaveLoadUIState::LoadMenu => self.render_load_menu(),
            SaveLoadUIState::DeleteConfirm => self.render_delete_confirm(),
            SaveLoadUIState::ExportMenu => self.render_export_menu(),
            SaveLoadUIState::ImportMenu => self.render_import_menu(),
            SaveLoadUIState::SaveComplete => self.render_success_dialog("Game Saved Successfully!"),
            SaveLoadUIState::LoadComplete => self.render_success_dialog("Game Loaded Successfully!"),
            SaveLoadUIState::ErrorDialog => self.render_error_dialog(),
            SaveLoadUIState::Closed => Vec::new(),
        }
    }
}

impl SaveLoadUI {
    /// Render save menu
    fn render_save_menu(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        // Main panel
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 10,
            y: 5,
            width: 60,
            height: 20,
            title: Some("Save Game".to_string()),
            border_color: Color::White,
            background_color: Color::Black,
        }));

        // Instructions
        commands.push(UIRenderCommand::Text(UIText {
            x: 12,
            y: 7,
            text: "Select a save slot:".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 12,
            y: 8,
            text: "↑/↓: Navigate, Enter: Save, Del: Delete, E: Export, Esc: Cancel".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        // Render save slots
        self.render_slot_list(&mut commands, 10);

        commands
    }

    /// Render load menu
    fn render_load_menu(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        // Main panel
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 10,
            y: 5,
            width: 60,
            height: 20,
            title: Some("Load Game".to_string()),
            border_color: Color::White,
            background_color: Color::Black,
        }));

        // Instructions
        commands.push(UIRenderCommand::Text(UIText {
            x: 12,
            y: 7,
            text: "Select a save slot to load:".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 12,
            y: 8,
            text: "↑/↓: Navigate, Enter: Load, Del: Delete, I: Import, Esc: Cancel".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        // Render save slots
        self.render_slot_list(&mut commands, 10);

        commands
    }

    /// Render slot list
    fn render_slot_list(&self, commands: &mut Vec<UIRenderCommand>, start_y: i32) {
        let visible_slots = self.get_visible_slots();
        
        for (i, slot) in visible_slots.iter().enumerate() {
            let y = start_y + i as i32;
            let global_index = self.scroll_offset + i;
            let is_selected = global_index == self.selected_slot;
            
            let color = if is_selected {
                Color::Yellow
            } else if slot.is_occupied {
                Color::White
            } else {
                Color::DarkGrey
            };

            let prefix = if is_selected { "> " } else { "  " };
            let text = format!("{}{}", prefix, self.format_slot_text(slot, global_index));

            commands.push(UIRenderCommand::Text(UIText {
                x: 12,
                y,
                text,
                color,
                alignment: TextAlignment::Left,
            }));
        }

        // Show scroll indicators
        if self.scroll_offset > 0 {
            commands.push(UIRenderCommand::Text(UIText {
                x: 65,
                y: start_y,
                text: "↑".to_string(),
                color: Color::Yellow,
                alignment: TextAlignment::Left,
            }));
        }

        if self.scroll_offset + self.max_visible_slots < self.save_slots.len() {
            commands.push(UIRenderCommand::Text(UIText {
                x: 65,
                y: start_y + self.max_visible_slots as i32 - 1,
                text: "↓".to_string(),
                color: Color::Yellow,
                alignment: TextAlignment::Left,
            }));
        }
    }

    /// Render delete confirmation
    fn render_delete_confirm(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 25,
            y: 10,
            width: 30,
            height: 8,
            title: Some("Confirm Delete".to_string()),
            border_color: Color::Red,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 27,
            y: 12,
            text: "Delete this save?".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        if let Some(SaveLoadAction::Delete(slot_id)) = &self.confirm_action {
            commands.push(UIRenderCommand::Text(UIText {
                x: 27,
                y: 13,
                text: format!("Slot {}", slot_id),
                color: Color::Yellow,
                alignment: TextAlignment::Left,
            }));
        }

        commands.push(UIRenderCommand::Text(UIText {
            x: 27,
            y: 15,
            text: "Y: Yes, N: No".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render export menu
    fn render_export_menu(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 20,
            y: 8,
            width: 40,
            height: 10,
            title: Some("Export Save".to_string()),
            border_color: Color::Green,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 22,
            y: 10,
            text: "Export current game to:".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 22,
            y: 12,
            text: self.export_path.display().to_string(),
            color: Color::Yellow,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 22,
            y: 15,
            text: "Enter: Export, Esc: Cancel".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render import menu
    fn render_import_menu(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 20,
            y: 8,
            width: 40,
            height: 10,
            title: Some("Import Save".to_string()),
            border_color: Color::Blue,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 22,
            y: 10,
            text: "Import save from:".to_string(),
            color: Color::White,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 22,
            y: 12,
            text: self.import_path.display().to_string(),
            color: Color::Yellow,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 22,
            y: 15,
            text: "Enter: Import, Esc: Cancel".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render success dialog
    fn render_success_dialog(&self, title: &str) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 25,
            y: 10,
            width: 30,
            height: 8,
            title: Some(title.to_string()),
            border_color: Color::Green,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 27,
            y: 13,
            text: self.success_message.clone(),
            color: Color::Green,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 27,
            y: 15,
            text: "Press Enter to continue".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        commands
    }

    /// Render error dialog
    fn render_error_dialog(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        
        commands.push(UIRenderCommand::Panel(UIPanel {
            x: 25,
            y: 10,
            width: 30,
            height: 8,
            title: Some("Error".to_string()),
            border_color: Color::Red,
            background_color: Color::Black,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 27,
            y: 13,
            text: self.error_message.clone(),
            color: Color::Red,
            alignment: TextAlignment::Left,
        }));

        commands.push(UIRenderCommand::Text(UIText {
            x: 27,
            y: 15,
            text: "Press Enter to continue".to_string(),
            color: Color::DarkGrey,
            alignment: TextAlignment::Left,
        }));

        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::save_system::SaveMetadata;
    use chrono::Utc;

    fn create_test_save_slots() -> Vec<SaveSlot> {
        vec![
            SaveSlot {
                slot_id: 0,
                is_occupied: true,
                metadata: Some(SaveMetadata {
                    save_name: "Test Save".to_string(),
                    player_name: "TestPlayer".to_string(),
                    character_level: 5,
                    current_depth: 3,
                    playtime_seconds: 1800,
                    created_at: Utc::now(),
                    last_modified: Utc::now(),
                }),
            },
            SaveSlot {
                slot_id: 1,
                is_occupied: false,
                metadata: None,
            },
            SaveSlot {
                slot_id: 2,
                is_occupied: true,
                metadata: Some(SaveMetadata {
                    save_name: "Another Save".to_string(),
                    player_name: "Player2".to_string(),
                    character_level: 10,
                    current_depth: 5,
                    playtime_seconds: 3600,
                    created_at: Utc::now(),
                    last_modified: Utc::now(),
                }),
            },
        ]
    }

    #[test]
    fn test_save_load_ui_creation() {
        let ui = SaveLoadUI::new();
        
        assert_eq!(ui.state, SaveLoadUIState::Closed);
        assert_eq!(ui.selected_slot, 0);
        assert!(ui.save_slots.is_empty());
        assert!(!ui.is_open());
    }

    #[test]
    fn test_open_save_menu() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        
        ui.open_save_menu(slots.clone());
        
        assert_eq!(ui.state, SaveLoadUIState::SaveMenu);
        assert_eq!(ui.save_slots.len(), 3);
        assert!(ui.is_open());
    }

    #[test]
    fn test_open_load_menu() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        
        ui.open_load_menu(slots.clone());
        
        assert_eq!(ui.state, SaveLoadUIState::LoadMenu);
        assert_eq!(ui.save_slots.len(), 3);
        assert!(ui.is_open());
    }

    #[test]
    fn test_navigation_input() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_save_menu(slots);
        
        // Test down navigation
        assert_eq!(ui.selected_slot, 0);
        ui.handle_input(KeyCode::Down);
        assert_eq!(ui.selected_slot, 1);
        
        // Test up navigation
        ui.handle_input(KeyCode::Up);
        assert_eq!(ui.selected_slot, 0);
        
        // Test boundary conditions
        ui.handle_input(KeyCode::Up);
        assert_eq!(ui.selected_slot, 0); // Should not go below 0
    }

    #[test]
    fn test_save_action() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_save_menu(slots);
        
        ui.selected_slot = 1; // Select empty slot
        let action = ui.handle_input(KeyCode::Enter);
        
        assert_eq!(action, Some(SaveLoadAction::Save(1)));
    }

    #[test]
    fn test_load_action() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_load_menu(slots);
        
        ui.selected_slot = 0; // Select occupied slot
        let action = ui.handle_input(KeyCode::Enter);
        
        assert_eq!(action, Some(SaveLoadAction::Load(0)));
    }

    #[test]
    fn test_load_empty_slot() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_load_menu(slots);
        
        ui.selected_slot = 1; // Select empty slot
        let action = ui.handle_input(KeyCode::Enter);
        
        assert_eq!(action, None); // Should not allow loading empty slot
        assert_eq!(ui.state, SaveLoadUIState::ErrorDialog);
    }

    #[test]
    fn test_delete_confirmation() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_save_menu(slots);
        
        ui.selected_slot = 0; // Select occupied slot
        ui.handle_input(KeyCode::Delete);
        
        assert_eq!(ui.state, SaveLoadUIState::DeleteConfirm);
        assert_eq!(ui.confirm_action, Some(SaveLoadAction::Delete(0)));
        
        // Confirm deletion
        let action = ui.handle_input(KeyCode::Char('y'));
        assert_eq!(action, Some(SaveLoadAction::Delete(0)));
    }

    #[test]
    fn test_cancel_delete() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_save_menu(slots);
        
        ui.selected_slot = 0;
        ui.handle_input(KeyCode::Delete);
        
        // Cancel deletion
        let action = ui.handle_input(KeyCode::Char('n'));
        assert_eq!(action, None);
        assert_eq!(ui.confirm_action, None);
        assert_eq!(ui.state, SaveLoadUIState::SaveMenu);
    }

    #[test]
    fn test_export_menu() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_save_menu(slots);
        
        ui.handle_input(KeyCode::Char('e'));
        assert_eq!(ui.state, SaveLoadUIState::ExportMenu);
        
        let action = ui.handle_input(KeyCode::Enter);
        assert!(matches!(action, Some(SaveLoadAction::Export(_))));
    }

    #[test]
    fn test_import_menu() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_load_menu(slots);
        
        ui.handle_input(KeyCode::Char('i'));
        assert_eq!(ui.state, SaveLoadUIState::ImportMenu);
        
        let action = ui.handle_input(KeyCode::Enter);
        assert!(matches!(action, Some(SaveLoadAction::Import(_))));
    }

    #[test]
    fn test_error_handling() {
        let mut ui = SaveLoadUI::new();
        
        ui.show_error("Test error message");
        
        assert_eq!(ui.state, SaveLoadUIState::ErrorDialog);
        assert_eq!(ui.error_message, "Test error message");
        
        // Clear error
        ui.handle_input(KeyCode::Enter);
        assert_eq!(ui.state, SaveLoadUIState::Closed);
    }

    #[test]
    fn test_success_handling() {
        let mut ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        ui.open_save_menu(slots);
        
        ui.show_success("Save successful");
        
        assert_eq!(ui.state, SaveLoadUIState::SaveComplete);
        assert_eq!(ui.success_message, "Save successful");
    }

    #[test]
    fn test_slot_formatting() {
        let ui = SaveLoadUI::new();
        let slots = create_test_save_slots();
        
        let occupied_text = ui.format_slot_text(&slots[0], 0);
        assert!(occupied_text.contains("TestPlayer"));
        assert!(occupied_text.contains("Level 5"));
        
        let empty_text = ui.format_slot_text(&slots[1], 1);
        assert!(empty_text.contains("Empty"));
    }

    #[test]
    fn test_scrolling() {
        let mut ui = SaveLoadUI::new();
        ui.max_visible_slots = 2;
        
        // Create more slots than visible
        let mut slots = Vec::new();
        for i in 0..5 {
            slots.push(SaveSlot {
                slot_id: i,
                is_occupied: false,
                metadata: None,
            });
        }
        
        ui.open_save_menu(slots);
        
        // Navigate to bottom
        for _ in 0..4 {
            ui.handle_input(KeyCode::Down);
        }
        
        assert_eq!(ui.selected_slot, 4);
        assert_eq!(ui.scroll_offset, 3); // Should scroll to show selected item
    }
}