use crossterm::{
    event::{KeyCode, KeyEvent},
    style::{Color, SetForegroundColor, SetBackgroundColor, ResetColor},
};
use specs::{World, Entity, Join, WorldExt};
use crate::components::{Name, Player};
use crate::items::{ItemProperties, get_item_display_name, get_item_current_value};
use crate::items::inventory_system::{AdvancedInventory, InventorySortMode};
use std::io::{Write, stdout};

pub struct InventoryUI {
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub items_per_page: usize,
    pub show_details: bool,
    pub filter_mode: InventoryFilter,
    pub sort_mode: InventorySortMode,
}

impl InventoryUI {
    pub fn new() -> Self {
        InventoryUI {
            selected_index: 0,
            scroll_offset: 0,
            items_per_page: 20,
            show_details: false,
            filter_mode: InventoryFilter::All,
            sort_mode: InventorySortMode::None,
        }
    }

    pub fn render(&self, world: &World, player_entity: Entity, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        let inventories = world.read_storage::<AdvancedInventory>();
        
        if let Some(inventory) = inventories.get(player_entity) {
            self.render_inventory_screen(world, inventory, width, height)?;
        }
        
        Ok(())
    }

    fn render_inventory_screen(
        &self,
        world: &World,
        inventory: &AdvancedInventory,
        width: u16,
        height: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        
        // Clear screen
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        // Title
        crossterm::execute!(stdout, SetForegroundColor(Color::Yellow))?;
        println!("=== INVENTORY ===");
        crossterm::execute!(stdout, ResetColor)?;
        
        // Inventory stats
        println!("Capacity: {}/{} | Weight: {:.1}/{:.1} lbs | Gold: {}",
            inventory.items.len(),
            inventory.capacity,
            inventory.current_weight,
            inventory.weight_limit,
            inventory.gold
        );
        
        if inventory.is_overweight() {
            crossterm::execute!(stdout, SetForegroundColor(Color::Red))?;
            println!("OVERWEIGHT! Movement may be impaired.");
            crossterm::execute!(stdout, ResetColor)?;
        }
        
        println!(); // Empty line

        // Filter and sort info
        println!("Filter: {:?} | Sort: {:?}", self.filter_mode, self.sort_mode);
        println!(); // Empty line

        // Item list
        let filtered_items = self.get_filtered_items(world, inventory);
        let visible_items = self.get_visible_items(&filtered_items);
        
        for (display_index, (slot_index, slot)) in visible_items.iter().enumerate() {
            let is_selected = display_index + self.scroll_offset == self.selected_index;
            
            if is_selected {
                crossterm::execute!(stdout, SetBackgroundColor(Color::DarkGrey))?;
            }
            
            self.render_item_line(world, slot.entity, slot.quantity, *slot_index)?;
            
            if is_selected {
                crossterm::execute!(stdout, ResetColor)?;
            }
        }

        // Show item details if enabled
        if self.show_details && !filtered_items.is_empty() && self.selected_index < filtered_items.len() {
            let selected_slot = &filtered_items[self.selected_index].1;
            self.render_item_details(world, selected_slot.entity, height)?;
        }

        // Controls help
        self.render_controls_help(height)?;
        
        stdout.flush()?;
        Ok(())
    }

    fn render_item_line(
        &self,
        world: &World,
        entity: Entity,
        quantity: i32,
        slot_index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = get_item_display_name(world, entity).unwrap_or("Unknown".to_string());
        let value = get_item_current_value(world, entity);
        
        let properties = world.read_storage::<ItemProperties>();
        let weight = if let Some(props) = properties.get(entity) {
            props.weight * quantity as f32
        } else {
            0.0
        };

        // Color code by rarity
        if let Some(props) = properties.get(entity) {
            let (r, g, b) = props.rarity.color();
            crossterm::execute!(stdout(), SetForegroundColor(Color::Rgb { r, g, b }))?;
        }

        if quantity > 1 {
            println!("{:2}. {} x{} ({:.1} lbs, {} gold each)",
                slot_index + 1, name, quantity, weight, value);
        } else {
            println!("{:2}. {} ({:.1} lbs, {} gold)",
                slot_index + 1, name, weight, value);
        }
        
        crossterm::execute!(stdout(), ResetColor)?;
        Ok(())
    }

    fn render_item_details(&self, world: &World, entity: Entity, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        let detail_start_y = height - 10;
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(0, detail_start_y))?;
        
        // Draw separator
        crossterm::execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
        println!("{}", "─".repeat(80));
        crossterm::execute!(stdout(), ResetColor)?;
        
        // Item details
        let info = crate::items::get_item_info_string(world, entity);
        let lines: Vec<&str> = info.lines().take(8).collect(); // Limit to 8 lines
        
        for line in lines {
            println!("{}", line);
        }
        
        Ok(())
    }

    fn render_controls_help(&self, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(0, height - 2))?;
        crossterm::execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
        println!("Controls: ↑↓ Navigate | Enter Use/Equip | D Drop | I Details | S Sort | F Filter | ESC Exit");
        crossterm::execute!(stdout(), ResetColor)?;
        Ok(())
    }

    fn get_filtered_items(&self, world: &World, inventory: &AdvancedInventory) -> Vec<(usize, &crate::items::inventory_system::InventorySlot)> {
        let properties = world.read_storage::<ItemProperties>();
        
        inventory.items.iter().enumerate()
            .filter(|(_, slot)| {
                if let Some(props) = properties.get(slot.entity) {
                    match self.filter_mode {
                        InventoryFilter::All => true,
                        InventoryFilter::Weapons => matches!(props.item_type, crate::items::ItemType::Weapon(_)),
                        InventoryFilter::Armor => matches!(props.item_type, crate::items::ItemType::Armor(_)),
                        InventoryFilter::Consumables => matches!(props.item_type, crate::items::ItemType::Consumable(_)),
                        InventoryFilter::Tools => matches!(props.item_type, crate::items::ItemType::Tool(_)),
                        InventoryFilter::Materials => matches!(props.item_type, crate::items::ItemType::Material(_)),
                        InventoryFilter::Valuable => props.rarity >= crate::items::ItemRarity::Rare,
                    }
                } else {
                    false
                }
            })
            .collect()
    }

    fn get_visible_items<'a>(&self, filtered_items: &'a [(usize, &crate::items::inventory_system::InventorySlot)]) -> &'a [(usize, &crate::items::inventory_system::InventorySlot)] {
        let start = self.scroll_offset;
        let end = (start + self.items_per_page).min(filtered_items.len());
        &filtered_items[start..end]
    }

    pub fn handle_input(&mut self, key: KeyEvent, world: &mut World, player_entity: Entity) -> InventoryAction {
        match key.code {
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                    if self.selected_index < self.scroll_offset {
                        self.scroll_offset = self.selected_index;
                    }
                }
                InventoryAction::None
            },
            KeyCode::Down => {
                let inventories = world.read_storage::<AdvancedInventory>();
                if let Some(inventory) = inventories.get(player_entity) {
                    let filtered_items = self.get_filtered_items(world, inventory);
                    if self.selected_index < filtered_items.len().saturating_sub(1) {
                        self.selected_index += 1;
                        if self.selected_index >= self.scroll_offset + self.items_per_page {
                            self.scroll_offset = self.selected_index - self.items_per_page + 1;
                        }
                    }
                }
                InventoryAction::None
            },
            KeyCode::Enter => {
                // Use or equip selected item
                let inventories = world.read_storage::<AdvancedInventory>();
                if let Some(inventory) = inventories.get(player_entity) {
                    let filtered_items = self.get_filtered_items(world, inventory);
                    if self.selected_index < filtered_items.len() {
                        let (slot_index, slot) = filtered_items[self.selected_index];
                        return InventoryAction::UseItem(slot.entity);
                    }
                }
                InventoryAction::None
            },
            KeyCode::Char('d') | KeyCode::Char('D') => {
                // Drop selected item
                let inventories = world.read_storage::<AdvancedInventory>();
                if let Some(inventory) = inventories.get(player_entity) {
                    let filtered_items = self.get_filtered_items(world, inventory);
                    if self.selected_index < filtered_items.len() {
                        let (slot_index, slot) = filtered_items[self.selected_index];
                        return InventoryAction::DropItem(slot.entity);
                    }
                }
                InventoryAction::None
            },
            KeyCode::Char('i') | KeyCode::Char('I') => {
                self.show_details = !self.show_details;
                InventoryAction::None
            },
            KeyCode::Char('s') | KeyCode::Char('S') => {
                self.cycle_sort_mode();
                InventoryAction::SortInventory(self.sort_mode.clone())
            },
            KeyCode::Char('f') | KeyCode::Char('F') => {
                self.cycle_filter_mode();
                self.selected_index = 0;
                self.scroll_offset = 0;
                InventoryAction::None
            },
            KeyCode::Char('a') | KeyCode::Char('A') => {
                // Toggle auto-pickup
                InventoryAction::ToggleAutoPickup
            },
            KeyCode::Esc => {
                InventoryAction::Close
            },
            _ => InventoryAction::None,
        }
    }

    fn cycle_sort_mode(&mut self) {
        self.sort_mode = match self.sort_mode {
            InventorySortMode::None => InventorySortMode::Name,
            InventorySortMode::Name => InventorySortMode::Type,
            InventorySortMode::Type => InventorySortMode::Value,
            InventorySortMode::Value => InventorySortMode::Weight,
            InventorySortMode::Weight => InventorySortMode::None,
        };
    }

    fn cycle_filter_mode(&mut self) {
        self.filter_mode = match self.filter_mode {
            InventoryFilter::All => InventoryFilter::Weapons,
            InventoryFilter::Weapons => InventoryFilter::Armor,
            InventoryFilter::Armor => InventoryFilter::Consumables,
            InventoryFilter::Consumables => InventoryFilter::Tools,
            InventoryFilter::Tools => InventoryFilter::Materials,
            InventoryFilter::Materials => InventoryFilter::Valuable,
            InventoryFilter::Valuable => InventoryFilter::All,
        };
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InventoryFilter {
    All,
    Weapons,
    Armor,
    Consumables,
    Tools,
    Materials,
    Valuable,
}

#[derive(Debug, Clone)]
pub enum InventoryAction {
    None,
    UseItem(Entity),
    DropItem(Entity),
    SortInventory(InventorySortMode),
    ToggleAutoPickup,
    Close,
}

// Container UI for interacting with chests, corpses, etc.
pub struct ContainerUI {
    pub selected_container_index: usize,
    pub selected_inventory_index: usize,
    pub active_panel: ContainerPanel,
    pub transfer_mode: bool,
}

impl ContainerUI {
    pub fn new() -> Self {
        ContainerUI {
            selected_container_index: 0,
            selected_inventory_index: 0,
            active_panel: ContainerPanel::Container,
            transfer_mode: false,
        }
    }

    pub fn render(&self, world: &World, player_entity: Entity, container_entity: Entity, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        let mut stdout = stdout();
        
        // Clear screen
        crossterm::execute!(stdout, crossterm::terminal::Clear(crossterm::terminal::ClearType::All))?;
        crossterm::execute!(stdout, crossterm::cursor::MoveTo(0, 0))?;

        // Title
        crossterm::execute!(stdout, SetForegroundColor(Color::Yellow))?;
        let container_name = world.read_storage::<Name>()
            .get(container_entity)
            .map(|n| n.name.clone())
            .unwrap_or("Container".to_string());
        println!("=== {} ===", container_name);
        crossterm::execute!(stdout, ResetColor)?;
        
        // Split screen layout
        let panel_width = width / 2;
        
        // Render container panel
        self.render_container_panel(world, container_entity, 0, 3, panel_width, height - 6)?;
        
        // Render inventory panel
        self.render_inventory_panel(world, player_entity, panel_width, 3, panel_width, height - 6)?;
        
        // Controls
        self.render_container_controls(height)?;
        
        stdout.flush()?;
        Ok(())
    }

    fn render_container_panel(&self, world: &World, container_entity: Entity, x: u16, y: u16, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(x, y))?;
        
        if self.active_panel == ContainerPanel::Container {
            crossterm::execute!(stdout(), SetBackgroundColor(Color::DarkBlue))?;
        }
        
        println!("CONTAINER");
        crossterm::execute!(stdout(), ResetColor)?;
        
        // Render container contents
        let containers = world.read_storage::<crate::items::inventory_system::Container>();
        if let Some(container) = containers.get(container_entity) {
            for (index, &item_entity) in container.items.iter().enumerate() {
                crossterm::execute!(stdout(), crossterm::cursor::MoveTo(x, y + 2 + index as u16))?;
                
                if index == self.selected_container_index && self.active_panel == ContainerPanel::Container {
                    crossterm::execute!(stdout(), SetBackgroundColor(Color::DarkGrey))?;
                }
                
                let name = get_item_display_name(world, item_entity).unwrap_or("Unknown".to_string());
                println!("{:2}. {}", index + 1, name);
                
                crossterm::execute!(stdout(), ResetColor)?;
            }
        }
        
        Ok(())
    }

    fn render_inventory_panel(&self, world: &World, player_entity: Entity, x: u16, y: u16, width: u16, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(x, y))?;
        
        if self.active_panel == ContainerPanel::Inventory {
            crossterm::execute!(stdout(), SetBackgroundColor(Color::DarkBlue))?;
        }
        
        println!("INVENTORY");
        crossterm::execute!(stdout(), ResetColor)?;
        
        // Render inventory contents
        let inventories = world.read_storage::<AdvancedInventory>();
        if let Some(inventory) = inventories.get(player_entity) {
            for (index, slot) in inventory.items.iter().enumerate() {
                crossterm::execute!(stdout(), crossterm::cursor::MoveTo(x, y + 2 + index as u16))?;
                
                if index == self.selected_inventory_index && self.active_panel == ContainerPanel::Inventory {
                    crossterm::execute!(stdout(), SetBackgroundColor(Color::DarkGrey))?;
                }
                
                let name = get_item_display_name(world, slot.entity).unwrap_or("Unknown".to_string());
                if slot.quantity > 1 {
                    println!("{:2}. {} x{}", index + 1, name, slot.quantity);
                } else {
                    println!("{:2}. {}", index + 1, name);
                }
                
                crossterm::execute!(stdout(), ResetColor)?;
            }
        }
        
        Ok(())
    }

    fn render_container_controls(&self, height: u16) -> Result<(), Box<dyn std::error::Error>> {
        crossterm::execute!(stdout(), crossterm::cursor::MoveTo(0, height - 2))?;
        crossterm::execute!(stdout(), SetForegroundColor(Color::DarkGrey))?;
        println!("Controls: ↑↓ Navigate | Tab Switch Panel | Enter Transfer | T Transfer All | ESC Close");
        crossterm::execute!(stdout(), ResetColor)?;
        Ok(())
    }

    pub fn handle_input(&mut self, key: KeyEvent) -> ContainerAction {
        match key.code {
            KeyCode::Up => {
                match self.active_panel {
                    ContainerPanel::Container => {
                        if self.selected_container_index > 0 {
                            self.selected_container_index -= 1;
                        }
                    },
                    ContainerPanel::Inventory => {
                        if self.selected_inventory_index > 0 {
                            self.selected_inventory_index -= 1;
                        }
                    },
                }
                ContainerAction::None
            },
            KeyCode::Down => {
                // Would need to check actual container/inventory sizes
                match self.active_panel {
                    ContainerPanel::Container => {
                        self.selected_container_index += 1;
                    },
                    ContainerPanel::Inventory => {
                        self.selected_inventory_index += 1;
                    },
                }
                ContainerAction::None
            },
            KeyCode::Tab => {
                self.active_panel = match self.active_panel {
                    ContainerPanel::Container => ContainerPanel::Inventory,
                    ContainerPanel::Inventory => ContainerPanel::Container,
                };
                ContainerAction::None
            },
            KeyCode::Enter => {
                match self.active_panel {
                    ContainerPanel::Container => ContainerAction::TakeFromContainer(self.selected_container_index),
                    ContainerPanel::Inventory => ContainerAction::PutInContainer(self.selected_inventory_index),
                }
            },
            KeyCode::Char('t') | KeyCode::Char('T') => {
                match self.active_panel {
                    ContainerPanel::Container => ContainerAction::TakeAll,
                    ContainerPanel::Inventory => ContainerAction::PutAll,
                }
            },
            KeyCode::Esc => ContainerAction::Close,
            _ => ContainerAction::None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContainerPanel {
    Container,
    Inventory,
}

#[derive(Debug, Clone)]
pub enum ContainerAction {
    None,
    TakeFromContainer(usize),
    PutInContainer(usize),
    TakeAll,
    PutAll,
    Close,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_ui_creation() {
        let ui = InventoryUI::new();
        assert_eq!(ui.selected_index, 0);
        assert_eq!(ui.scroll_offset, 0);
        assert!(!ui.show_details);
        assert_eq!(ui.filter_mode, InventoryFilter::All);
    }

    #[test]
    fn test_filter_cycling() {
        let mut ui = InventoryUI::new();
        assert_eq!(ui.filter_mode, InventoryFilter::All);
        
        ui.cycle_filter_mode();
        assert_eq!(ui.filter_mode, InventoryFilter::Weapons);
        
        ui.cycle_filter_mode();
        assert_eq!(ui.filter_mode, InventoryFilter::Armor);
    }

    #[test]
    fn test_sort_cycling() {
        let mut ui = InventoryUI::new();
        assert_eq!(ui.sort_mode, InventorySortMode::None);
        
        ui.cycle_sort_mode();
        assert_eq!(ui.sort_mode, InventorySortMode::Name);
        
        ui.cycle_sort_mode();
        assert_eq!(ui.sort_mode, InventorySortMode::Type);
    }

    #[test]
    fn test_container_ui_creation() {
        let ui = ContainerUI::new();
        assert_eq!(ui.selected_container_index, 0);
        assert_eq!(ui.selected_inventory_index, 0);
        assert_eq!(ui.active_panel, ContainerPanel::Container);
        assert!(!ui.transfer_mode);
    }
}