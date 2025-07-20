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