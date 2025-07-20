use crossterm::{event::KeyCode, style::Color};
use specs::{World, Entity, Join, ReadStorage, WorldExt};
use std::collections::HashMap;
use crate::components::{Player, Name, Position};
use crate::items::{
    ItemProperties, ItemType, ItemRarity, WeaponType, ArmorType, ConsumableType,
    AdvancedInventory, InventorySlot, Equipment, Equippable, ItemBonuses
};
use crate::ui::{
    ui_components::{UIComponent, UIRenderCommand, UIPanel, UIText, TextAlignment},
    menu_system::{MenuRenderer, MenuInput},
};

/// Inventory UI state
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryUIState {
    ItemList,
    ItemDetails,
    ItemComparison,
    FilterMenu,
    SortMenu,
    ActionMenu,
    Closed,
}

/// Inventory actions that can be performed on items
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryAction {
    Use,
    Equip,
    Unequip,
    Drop,
    Examine,
    Compare,
    Split,
    Combine,
    Repair,
    Enchant,
    Sell,
    Cancel,
}

impl InventoryAction {
    pub fn to_string(&self) -> String {
        match self {
            InventoryAction::Use => "Use".to_string(),
            InventoryAction::Equip => "Equip".to_string(),
            InventoryAction::Unequip => "Unequip".to_string(),
            InventoryAction::Drop => "Drop".to_string(),
            InventoryAction::Examine => "Examine".to_string(),
            InventoryAction::Compare => "Compare".to_string(),
            InventoryAction::Split => "Split Stack".to_string(),
            InventoryAction::Combine => "Combine".to_string(),
            InventoryAction::Repair => "Repair".to_string(),
            InventoryAction::Enchant => "Enchant".to_string(),
            InventoryAction::Sell => "Sell".to_string(),
            InventoryAction::Cancel => "Cancel".to_string(),
        }
    }

    pub fn is_available_for_item(&self, item_type: &ItemType, is_equipped: bool) -> bool {
        match self {
            InventoryAction::Use => matches!(item_type, ItemType::Consumable(_)),
            InventoryAction::Equip => !is_equipped && matches!(item_type, ItemType::Weapon(_) | ItemType::Armor(_)),
            InventoryAction::Unequip => is_equipped,
            InventoryAction::Drop => true,
            InventoryAction::Examine => true,
            InventoryAction::Compare => matches!(item_type, ItemType::Weapon(_) | ItemType::Armor(_)),
            InventoryAction::Split => false, // TODO: Implement stack splitting
            InventoryAction::Combine => false, // TODO: Implement item combining
            InventoryAction::Repair => matches!(item_type, ItemType::Weapon(_) | ItemType::Armor(_)),
            InventoryAction::Enchant => matches!(item_type, ItemType::Weapon(_) | ItemType::Armor(_)),
            InventoryAction::Sell => true,
            InventoryAction::Cancel => true,
        }
    }
}

/// Inventory sorting options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InventorySortMode {
    Name,
    Type,
    Rarity,
    Value,
    Weight,
    Recent,
}

impl InventorySortMode {
    pub fn to_string(&self) -> String {
        match self {
            InventorySortMode::Name => "Name".to_string(),
            InventorySortMode::Type => "Type".to_string(),
            InventorySortMode::Rarity => "Rarity".to_string(),
            InventorySortMode::Value => "Value".to_string(),
            InventorySortMode::Weight => "Weight".to_string(),
            InventorySortMode::Recent => "Recently Added".to_string(),
        }
    }

    pub fn all_modes() -> Vec<InventorySortMode> {
        vec![
            InventorySortMode::Name,
            InventorySortMode::Type,
            InventorySortMode::Rarity,
            InventorySortMode::Value,
            InventorySortMode::Weight,
            InventorySortMode::Recent,
        ]
    }
}

/// Inventory filtering options
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryFilter {
    All,
    Weapons,
    Armor,
    Consumables,
    Tools,
    Materials,
    Quest,
    Equipped,
    Unequipped,
    Rarity(ItemRarity),
}

impl InventoryFilter {
    pub fn to_string(&self) -> String {
        match self {
            InventoryFilter::All => "All Items".to_string(),
            InventoryFilter::Weapons => "Weapons".to_string(),
            InventoryFilter::Armor => "Armor".to_string(),
            InventoryFilter::Consumables => "Consumables".to_string(),
            InventoryFilter::Tools => "Tools".to_string(),
            InventoryFilter::Materials => "Materials".to_string(),
            InventoryFilter::Quest => "Quest Items".to_string(),
            InventoryFilter::Equipped => "Equipped".to_string(),
            InventoryFilter::Unequipped => "Unequipped".to_string(),
            InventoryFilter::Rarity(rarity) => format!("{} Items", rarity.name()),
        }
    }

    pub fn all_filters() -> Vec<InventoryFilter> {
        vec![
            InventoryFilter::All,
            InventoryFilter::Weapons,
            InventoryFilter::Armor,
            InventoryFilter::Consumables,
            InventoryFilter::Tools,
            InventoryFilter::Materials,
            InventoryFilter::Quest,
            InventoryFilter::Equipped,
            InventoryFilter::Unequipped,
            InventoryFilter::Rarity(ItemRarity::Common),
            InventoryFilter::Rarity(ItemRarity::Uncommon),
            InventoryFilter::Rarity(ItemRarity::Rare),
            InventoryFilter::Rarity(ItemRarity::Epic),
            InventoryFilter::Rarity(ItemRarity::Legendary),
        ]
    }

    pub fn matches_item(&self, item_props: &ItemProperties, is_equipped: bool) -> bool {
        match self {
            InventoryFilter::All => true,
            InventoryFilter::Weapons => matches!(item_props.item_type, ItemType::Weapon(_)),
            InventoryFilter::Armor => matches!(item_props.item_type, ItemType::Armor(_)),
            InventoryFilter::Consumables => matches!(item_props.item_type, ItemType::Consumable(_)),
            InventoryFilter::Tools => matches!(item_props.item_type, ItemType::Tool(_)),
            InventoryFilter::Materials => matches!(item_props.item_type, ItemType::Material(_)),
            InventoryFilter::Quest => item_props.tags.contains(&crate::items::ItemTag::Quest),
            InventoryFilter::Equipped => is_equipped,
            InventoryFilter::Unequipped => !is_equipped,
            InventoryFilter::Rarity(rarity) => item_props.rarity == *rarity,
        }
    }
}

// Main inventory UI component
pub struct InventoryUI {
    pub state: InventoryUIState,
    pub player_entity: Option<Entity>,
    pub selected_item_index: usize,
    pub selected_action_index: usize,
    pub selected_filter_index: usize,
    pub selected_sort_index: usize,
    pub current_filter: InventoryFilter,
    pub current_sort: InventorySortMode,
    pub sort_ascending: bool,
    pub filtered_items: Vec<(Entity, InventorySlot)>,
    pub comparison_item: Option<Entity>,
    pub scroll_offset: usize,
    pub items_per_page: usize,
    pub show_item_icons: bool,
    pub show_tooltips: bool,
}

impl InventoryUI {
    pub fn new() -> Self {
        InventoryUI {
            state: InventoryUIState::Closed,
            player_entity: None,
            selected_item_index: 0,
            selected_action_index: 0,
            selected_filter_index: 0,
            selected_sort_index: 0,
            current_filter: InventoryFilter::All,
            current_sort: InventorySortMode::Type,
            sort_ascending: true,
            filtered_items: Vec::new(),
            comparison_item: None,
            scroll_offset: 0,
            items_per_page: 20,
            show_item_icons: true,
            show_tooltips: true,
        }
    }

    pub fn open(&mut self, player_entity: Entity) {
        self.player_entity = Some(player_entity);
        self.state = InventoryUIState::ItemList;
        self.selected_item_index = 0;
        self.scroll_offset = 0;
    }

    pub fn close(&mut self) {
        self.state = InventoryUIState::Closed;
        self.comparison_item = None;
    }

    pub fn is_open(&self) -> bool {
        self.state != InventoryUIState::Closed
    }

    pub fn update_filtered_items(&mut self, world: &World) {
        self.filtered_items.clear();

        if let Some(player_entity) = self.player_entity {
            let inventories = world.read_storage::<AdvancedInventory>();
            let item_properties = world.read_storage::<ItemProperties>();
            let equippables = world.read_storage::<Equippable>();

            if let Some(inventory) = inventories.get(player_entity) {
                for slot in &inventory.slots {
                    if let Some(item_entity) = slot.item {
                        if let Some(props) = item_properties.get(item_entity) {
                            let is_equipped = equippables.get(item_entity)
                                .map(|e| e.equipped)
                                .unwrap_or(false);

                            if self.current_filter.matches_item(props, is_equipped) {
                                self.filtered_items.push((item_entity, slot.clone()));
                            }
                        }
                    }
                }

                // Sort items
                self.sort_items(world);
            }
        }
    }

    fn sort_items(&mut self, world: &World) {
        let item_properties = world.read_storage::<ItemProperties>();

        self.filtered_items.sort_by(|a, b| {
            let props_a = item_properties.get(a.0);
            let props_b = item_properties.get(b.0);

            if let (Some(props_a), Some(props_b)) = (props_a, props_b) {
                let comparison = match self.current_sort {
                    InventorySortMode::Name => props_a.name.cmp(&props_b.name),
                    InventorySortMode::Type => {
                        let type_order_a = self.get_type_sort_order(&props_a.item_type);
                        let type_order_b = self.get_type_sort_order(&props_b.item_type);
                        type_order_a.cmp(&type_order_b)
                    }
                    InventorySortMode::Rarity => props_a.rarity.cmp(&props_b.rarity),
                    InventorySortMode::Value => props_a.value.cmp(&props_b.value),
                    InventorySortMode::Weight => props_a.weight.partial_cmp(&props_b.weight).unwrap_or(std::cmp::Ordering::Equal),
                    InventorySortMode::Recent => a.1.added_time.cmp(&b.1.added_time),
                };

                if self.sort_ascending {
                    comparison
                } else {
                    comparison.reverse()
                }
            } else {
                std::cmp::Ordering::Equal
            }
        });
    }

    fn get_type_sort_order(&self, item_type: &ItemType) -> u8 {
        match item_type {
            ItemType::Weapon(_) => 0,
            ItemType::Armor(_) => 1,
            ItemType::Consumable(_) => 2,
            ItemType::Tool(_) => 3,
            ItemType::Material(_) => 4,
            ItemType::Quest => 5,
            ItemType::Misc => 6,
        }
    }

    pub fn handle_key(&mut self, key: KeyCode, world: &World) -> Option<InventoryAction> {
        match self.state {
            InventoryUIState::ItemList => self.handle_item_list_key(key, world),
            InventoryUIState::ItemDetails => self.handle_item_details_key(key),
            InventoryUIState::ItemComparison => self.handle_comparison_key(key),
            InventoryUIState::FilterMenu => self.handle_filter_menu_key(key),
            InventoryUIState::SortMenu => self.handle_sort_menu_key(key),
            InventoryUIState::ActionMenu => self.handle_action_menu_key(key, world),
            InventoryUIState::Closed => None,
        }
    }

    fn handle_item_list_key(&mut self, key: KeyCode, world: &World) -> Option<InventoryAction> {
        match key {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                if self.selected_item_index > 0 {
                    self.selected_item_index -= 1;
                    self.ensure_item_visible();
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                if self.selected_item_index < self.filtered_items.len().saturating_sub(1) {
                    self.selected_item_index += 1;
                    self.ensure_item_visible();
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if !self.filtered_items.is_empty() {
                    self.state = InventoryUIState::ActionMenu;
                    self.selected_action_index = 0;
                }
                None
            }
            KeyCode::Char('e') => {
                // Quick equip/use
                if let Some((item_entity, _)) = self.get_selected_item() {
                    let item_properties = world.read_storage::<ItemProperties>();
                    if let Some(props) = item_properties.get(item_entity) {
                        match props.item_type {
                            ItemType::Weapon(_) | ItemType::Armor(_) => Some(InventoryAction::Equip),
                            ItemType::Consumable(_) => Some(InventoryAction::Use),
                            _ => None,
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            KeyCode::Char('d') => {
                // Quick drop
                Some(InventoryAction::Drop)
            }
            KeyCode::Char('x') => {
                // Examine item
                if !self.filtered_items.is_empty() {
                    self.state = InventoryUIState::ItemDetails;
                }
                None
            }
            KeyCode::Char('c') => {
                // Compare item
                if let Some((item_entity, _)) = self.get_selected_item() {
                    self.comparison_item = Some(item_entity);
                    self.state = InventoryUIState::ItemComparison;
                }
                None
            }
            KeyCode::Char('f') => {
                // Filter menu
                self.state = InventoryUIState::FilterMenu;
                None
            }
            KeyCode::Char('o') => {
                // Sort menu
                self.state = InventoryUIState::SortMenu;
                None
            }
            KeyCode::Char('r') => {
                // Reverse sort order
                self.sort_ascending = !self.sort_ascending;
                self.update_filtered_items(world);
                None
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close();
                None
            }
            _ => None,
        }
    }

    fn handle_item_details_key(&mut self, key: KeyCode) -> Option<InventoryAction> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.state = InventoryUIState::ItemList;
                None
            }
            _ => None,
        }
    }

    fn handle_comparison_key(&mut self, key: KeyCode) -> Option<InventoryAction> {
        match key {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.state = InventoryUIState::ItemList;
                self.comparison_item = None;
                None
            }
            _ => None,
        }
    }

    fn handle_filter_menu_key(&mut self, key: KeyCode) -> Option<InventoryAction> {
        let filters = InventoryFilter::all_filters();
        
        match key {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                if self.selected_filter_index > 0 {
                    self.selected_filter_index -= 1;
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                if self.selected_filter_index < filters.len() - 1 {
                    self.selected_filter_index += 1;
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.current_filter = filters[self.selected_filter_index].clone();
                self.state = InventoryUIState::ItemList;
                None
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.state = InventoryUIState::ItemList;
                None
            }
            _ => None,
        }
    }

    fn handle_sort_menu_key(&mut self, key: KeyCode) -> Option<InventoryAction> {
        let sort_modes = InventorySortMode::all_modes();
        
        match key {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                if self.selected_sort_index > 0 {
                    self.selected_sort_index -= 1;
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                if self.selected_sort_index < sort_modes.len() - 1 {
                    self.selected_sort_index += 1;
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                self.current_sort = sort_modes[self.selected_sort_index];
                self.state = InventoryUIState::ItemList;
                None
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.state = InventoryUIState::ItemList;
                None
            }
            _ => None,
        }
    }

    fn handle_action_menu_key(&mut self, key: KeyCode, world: &World) -> Option<InventoryAction> {
        let available_actions = self.get_available_actions(world);
        
        match key {
            KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('w') => {
                if self.selected_action_index > 0 {
                    self.selected_action_index -= 1;
                }
                None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('s') => {
                if self.selected_action_index < available_actions.len().saturating_sub(1) {
                    self.selected_action_index += 1;
                }
                None
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if let Some(action) = available_actions.get(self.selected_action_index) {
                    self.state = InventoryUIState::ItemList;
                    Some(action.clone())
                } else {
                    None
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Backspace => {
                self.state = InventoryUIState::ItemList;
                None
            }
            _ => None,
        }
    }

    fn get_available_actions(&self, world: &World) -> Vec<InventoryAction> {
        let mut actions = Vec::new();

        if let Some((item_entity, _)) = self.get_selected_item() {
            let item_properties = world.read_storage::<ItemProperties>();
            let equippables = world.read_storage::<Equippable>();

            if let Some(props) = item_properties.get(item_entity) {
                let is_equipped = equippables.get(item_entity)
                    .map(|e| e.equipped)
                    .unwrap_or(false);

                for action in &[
                    InventoryAction::Use,
                    InventoryAction::Equip,
                    InventoryAction::Unequip,
                    InventoryAction::Drop,
                    InventoryAction::Examine,
                    InventoryAction::Compare,
                    InventoryAction::Repair,
                    InventoryAction::Enchant,
                    InventoryAction::Sell,
                ] {
                    if action.is_available_for_item(&props.item_type, is_equipped) {
                        actions.push(action.clone());
                    }
                }
            }
        }

        actions.push(InventoryAction::Cancel);
        actions
    }

    fn get_selected_item(&self) -> Option<(Entity, InventorySlot)> {
        self.filtered_items.get(self.selected_item_index).cloned()
    }

    fn ensure_item_visible(&mut self) {
        if self.selected_item_index < self.scroll_offset {
            self.scroll_offset = self.selected_item_index;
        } else if self.selected_item_index >= self.scroll_offset + self.items_per_page {
            self.scroll_offset = self.selected_item_index - self.items_per_page + 1;
        }
    }

    pub fn render(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        match self.state {
            InventoryUIState::ItemList => self.render_item_list(world, screen_width, screen_height),
            InventoryUIState::ItemDetails => self.render_item_details(world, screen_width, screen_height),
            InventoryUIState::ItemComparison => self.render_item_comparison(world, screen_width, screen_height),
            InventoryUIState::FilterMenu => self.render_filter_menu(screen_width, screen_height),
            InventoryUIState::SortMenu => self.render_sort_menu(screen_width, screen_height),
            InventoryUIState::ActionMenu => self.render_action_menu(world, screen_width, screen_height),
            InventoryUIState::Closed => Vec::new(),
        }
    }

    fn render_item_list(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // Main inventory panel
        let panel_width = screen_width - 4;
        let panel_height = screen_height - 4;
        let panel = UIPanel::new(
            "Inventory".to_string(),
            2,
            2,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        // Header with filter and sort info
        let header_y = 3;
        let header_text = format!("Filter: {} | Sort: {} {} | Items: {}/{}",
            self.current_filter.to_string(),
            self.current_sort.to_string(),
            if self.sort_ascending { "↑" } else { "↓" },
            self.filtered_items.len(),
            // TODO: Get max inventory size
            100
        );

        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y: header_y,
            text: header_text,
            fg: Color::Cyan,
            bg: Color::Black,
        });

        // Item list
        let list_start_y = header_y + 2;
        let list_height = panel_height - 6;
        let visible_items = self.filtered_items.iter()
            .skip(self.scroll_offset)
            .take(list_height as usize);

        for (i, (item_entity, slot)) in visible_items.enumerate() {
            let y = list_start_y + i as i32;
            let is_selected = (i + self.scroll_offset) == self.selected_item_index;

            let item_text = self.format_item_text(world, *item_entity, slot);
            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else {
                (Color::White, Color::Black)
            };

            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y,
                text: format!("{:<width$}", item_text, width = (panel_width - 4) as usize),
                fg,
                bg,
            });
        }

        // Controls help
        let controls_y = panel_height - 2;
        let controls_text = "E:Equip/Use D:Drop X:Examine C:Compare F:Filter O:Sort R:Reverse ESC:Close";
        commands.push(UIRenderCommand::DrawText {
            x: 4,
            y: controls_y,
            text: controls_text.to_string(),
            fg: Color::DarkGrey,
            bg: Color::Black,
        });

        commands
    }

    fn format_item_text(&self, world: &World, item_entity: Entity, slot: &InventorySlot) -> String {
        let item_properties = world.read_storage::<ItemProperties>();
        let equippables = world.read_storage::<Equippable>();

        if let Some(props) = item_properties.get(item_entity) {
            let mut text = String::new();

            // Item icon/glyph
            if self.show_item_icons {
                let icon = match props.item_type {
                    ItemType::Weapon(WeaponType::Sword) => "⚔",
                    ItemType::Weapon(WeaponType::Bow) => "🏹",
                    ItemType::Weapon(_) => "🗡",
                    ItemType::Armor(ArmorType::Helmet) => "⛑",
                    ItemType::Armor(ArmorType::Chest) => "🛡",
                    ItemType::Armor(_) => "👕",
                    ItemType::Consumable(ConsumableType::Potion) => "🧪",
                    ItemType::Consumable(ConsumableType::Food) => "🍖",
                    ItemType::Consumable(_) => "📜",
                    _ => "📦",
                };
                text.push_str(&format!("{} ", icon));
            }

            // Item name with rarity color coding
            text.push_str(&props.name);

            // Quantity if stacked
            if slot.quantity > 1 {
                text.push_str(&format!(" ({})", slot.quantity));
            }

            // Equipped indicator
            if equippables.get(item_entity).map(|e| e.equipped).unwrap_or(false) {
                text.push_str(" [E]");
            }

            // Condition indicator
            if let Some(durability) = &props.durability {
                let condition_percent = (durability.current as f32 / durability.max as f32 * 100.0) as i32;
                if condition_percent < 100 {
                    text.push_str(&format!(" ({}%)", condition_percent));
                }
            }

            text
        } else {
            "Unknown Item".to_string()
        }
    }

    fn render_item_details(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if let Some((item_entity, _)) = self.get_selected_item() {
            let panel_width = screen_width - 4;
            let panel_height = screen_height - 4;
            let panel = UIPanel::new(
                "Item Details".to_string(),
                2,
                2,
                panel_width,
                panel_height,
            ).with_colors(Color::White, Color::Black, Color::Yellow);

            commands.extend(panel.render());

            // Get item information
            let item_properties = world.read_storage::<ItemProperties>();
            let item_bonuses = world.read_storage::<ItemBonuses>();

            if let Some(props) = item_properties.get(item_entity) {
                let mut y = 4;

                // Item name and type
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: format!("{} ({})", props.name, self.format_item_type(&props.item_type)),
                    fg: self.get_rarity_color(&props.rarity),
                    bg: Color::Black,
                });
                y += 2;

                // Description
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: props.description.clone(),
                    fg: Color::White,
                    bg: Color::Black,
                });
                y += 2;

                // Stats
                commands.push(UIRenderCommand::DrawText {
                    x: 4,
                    y,
                    text: format!("Value: {} gold  Weight: {:.1} lbs", props.value, props.weight),
                    fg: Color::Cyan,
                    bg: Color::Black,
                });
                y += 1;

                // Durability
                if let Some(durability) = &props.durability {
                    commands.push(UIRenderCommand::DrawText {
                        x: 4,
                        y,
                        text: format!("Condition: {}/{} ({}%)", 
                            durability.current, 
                            durability.max,
                            (durability.current as f32 / durability.max as f32 * 100.0) as i32),
                        fg: Color::Yellow,
                        bg: Color::Black,
                    });
                    y += 1;
                }

                // Bonuses
                if let Some(bonuses) = item_bonuses.get(item_entity) {
                    if bonuses.combat_bonuses.attack_bonus != 0 ||
                       bonuses.combat_bonuses.damage_bonus != 0 ||
                       bonuses.combat_bonuses.defense_bonus != 0 {
                        y += 1;
                        commands.push(UIRenderCommand::DrawText {
                            x: 4,
                            y,
                            text: "Combat Bonuses:".to_string(),
                            fg: Color::Green,
                            bg: Color::Black,
                        });
                        y += 1;

                        if bonuses.combat_bonuses.attack_bonus != 0 {
                            commands.push(UIRenderCommand::DrawText {
                                x: 6,
                                y,
                                text: format!("Attack: +{}", bonuses.combat_bonuses.attack_bonus),
                                fg: Color::White,
                                bg: Color::Black,
                            });
                            y += 1;
                        }

                        if bonuses.combat_bonuses.damage_bonus != 0 {
                            commands.push(UIRenderCommand::DrawText {
                                x: 6,
                                y,
                                text: format!("Damage: +{}", bonuses.combat_bonuses.damage_bonus),
                                fg: Color::White,
                                bg: Color::Black,
                            });
                            y += 1;
                        }

                        if bonuses.combat_bonuses.defense_bonus != 0 {
                            commands.push(UIRenderCommand::DrawText {
                                x: 6,
                                y,
                                text: format!("Defense: +{}", bonuses.combat_bonuses.defense_bonus),
                                fg: Color::White,
                                bg: Color::Black,
                            });
                            y += 1;
                        }
                    }
                }
            }

            // Controls
            let controls_y = panel_height - 2;
            commands.push(UIRenderCommand::DrawText {
                x: 4,
                y: controls_y,
                text: "ESC/Q:Back".to_string(),
                fg: Color::DarkGrey,
                bg: Color::Black,
            });
        }

        commands
    }

    fn render_item_comparison(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        // TODO: Implement item comparison view
        // This would show two items side by side with their stats compared

        commands
    }

    fn render_filter_menu(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let panel_width = 40;
        let panel_height = 20;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        let panel = UIPanel::new(
            "Filter Items".to_string(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        let filters = InventoryFilter::all_filters();
        let list_start_y = panel_y + 2;

        for (i, filter) in filters.iter().enumerate() {
            let y = list_start_y + i as i32;
            let is_selected = i == self.selected_filter_index;
            let is_current = *filter == self.current_filter;

            let mut text = filter.to_string();
            if is_current {
                text.push_str(" ✓");
            }

            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else if is_current {
                (Color::Green, Color::Black)
            } else {
                (Color::White, Color::Black)
            };

            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y,
                text: format!("{:<width$}", text, width = (panel_width - 4) as usize),
                fg,
                bg,
            });
        }

        commands
    }

    fn render_sort_menu(&self, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let panel_width = 30;
        let panel_height = 12;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        let panel = UIPanel::new(
            "Sort Items".to_string(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        let sort_modes = InventorySortMode::all_modes();
        let list_start_y = panel_y + 2;

        for (i, sort_mode) in sort_modes.iter().enumerate() {
            let y = list_start_y + i as i32;
            let is_selected = i == self.selected_sort_index;
            let is_current = *sort_mode == self.current_sort;

            let mut text = sort_mode.to_string();
            if is_current {
                text.push_str(if self.sort_ascending { " ↑" } else { " ↓" });
            }

            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else if is_current {
                (Color::Green, Color::Black)
            } else {
                (Color::White, Color::Black)
            };

            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y,
                text: format!("{:<width$}", text, width = (panel_width - 4) as usize),
                fg,
                bg,
            });
        }

        commands
    }

    fn render_action_menu(&self, world: &World, screen_width: i32, screen_height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        let available_actions = self.get_available_actions(world);
        let panel_width = 25;
        let panel_height = available_actions.len() as i32 + 4;
        let panel_x = (screen_width - panel_width) / 2;
        let panel_y = (screen_height - panel_height) / 2;

        let panel = UIPanel::new(
            "Item Actions".to_string(),
            panel_x,
            panel_y,
            panel_width,
            panel_height,
        ).with_colors(Color::White, Color::Black, Color::Yellow);

        commands.extend(panel.render());

        let list_start_y = panel_y + 2;

        for (i, action) in available_actions.iter().enumerate() {
            let y = list_start_y + i as i32;
            let is_selected = i == self.selected_action_index;

            let (fg, bg) = if is_selected {
                (Color::Black, Color::White)
            } else {
                (Color::White, Color::Black)
            };

            commands.push(UIRenderCommand::DrawText {
                x: panel_x + 2,
                y,
                text: format!("{:<width$}", action.to_string(), width = (panel_width - 4) as usize),
                fg,
                bg,
            });
        }

        commands
    }

    fn format_item_type(&self, item_type: &ItemType) -> String {
        match item_type {
            ItemType::Weapon(weapon_type) => format!("Weapon ({})", self.format_weapon_type(weapon_type)),
            ItemType::Armor(armor_type) => format!("Armor ({})", self.format_armor_type(armor_type)),
            ItemType::Consumable(consumable_type) => format!("Consumable ({})", self.format_consumable_type(consumable_type)),
            ItemType::Tool(tool_type) => format!("Tool ({})", self.format_tool_type(tool_type)),
            ItemType::Material(material_type) => format!("Material ({})", self.format_material_type(material_type)),
            ItemType::Quest => "Quest Item".to_string(),
            ItemType::Misc => "Miscellaneous".to_string(),
        }
    }

    fn format_weapon_type(&self, weapon_type: &WeaponType) -> &'static str {
        match weapon_type {
            WeaponType::Sword => "Sword",
            WeaponType::Axe => "Axe",
            WeaponType::Mace => "Mace",
            WeaponType::Dagger => "Dagger",
            WeaponType::Spear => "Spear",
            WeaponType::Bow => "Bow",
            WeaponType::Crossbow => "Crossbow",
            WeaponType::Staff => "Staff",
            WeaponType::Wand => "Wand",
            WeaponType::Thrown => "Thrown",
        }
    }

    fn format_armor_type(&self, armor_type: &ArmorType) -> &'static str {
        match armor_type {
            ArmorType::Helmet => "Helmet",
            ArmorType::Chest => "Chest",
            ArmorType::Legs => "Legs",
            ArmorType::Boots => "Boots",
            ArmorType::Gloves => "Gloves",
            ArmorType::Shield => "Shield",
            ArmorType::Cloak => "Cloak",
            ArmorType::Ring => "Ring",
            ArmorType::Amulet => "Amulet",
        }
    }

    fn format_consumable_type(&self, consumable_type: &ConsumableType) -> &'static str {
        match consumable_type {
            ConsumableType::Potion => "Potion",
            ConsumableType::Food => "Food",
            ConsumableType::Scroll => "Scroll",
            ConsumableType::Wand => "Wand",
            ConsumableType::Misc => "Miscellaneous",
        }
    }

    fn format_tool_type(&self, tool_type: &crate::items::ToolType) -> &'static str {
        match tool_type {
            crate::items::ToolType::Lockpick => "Lockpick",
            crate::items::ToolType::Rope => "Rope",
            crate::items::ToolType::Torch => "Torch",
            crate::items::ToolType::Key => "Key",
            crate::items::ToolType::Misc => "Miscellaneous",
        }
    }

    fn format_material_type(&self, material_type: &crate::items::MaterialType) -> &'static str {
        match material_type {
            crate::items::MaterialType::Metal => "Metal",
            crate::items::MaterialType::Wood => "Wood",
            crate::items::MaterialType::Cloth => "Cloth",
            crate::items::MaterialType::Leather => "Leather",
            crate::items::MaterialType::Gem => "Gem",
            crate::items::MaterialType::Herb => "Herb",
            crate::items::MaterialType::Misc => "Miscellaneous",
        }
    }

    fn get_rarity_color(&self, rarity: &ItemRarity) -> Color {
        match rarity {
            ItemRarity::Common => Color::White,
            ItemRarity::Uncommon => Color::Green,
            ItemRarity::Rare => Color::Blue,
            ItemRarity::Epic => Color::Magenta,
            ItemRarity::Legendary => Color::Yellow,
        }
    }
}

impl UIComponent for InventoryUI {
    fn render(&self, _x: i32, _y: i32, width: i32, height: i32) -> Vec<UIRenderCommand> {
        // This method signature doesn't provide access to World, so we use the other render method
        vec![]
    }

    fn handle_input(&mut self, input: char) -> bool {
        // Convert char to KeyCode for consistency
        let key = match input {
            '\n' => KeyCode::Enter,
            '\x1b' => KeyCode::Esc,
            'k' | 'w' => KeyCode::Up,
            'j' | 's' => KeyCode::Down,
            c => KeyCode::Char(c),
        };

        // We need World access for proper handling, so this is a simplified version
        match key {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.close();
                true
            }
            _ => false,
        }
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
    use specs::{World, WorldExt, Builder};
    use crate::components::{Player, Name};

    fn setup_test_world() -> (World, Entity) {
        let mut world = World::new();
        world.register::<Player>();
        world.register::<Name>();
        world.register::<AdvancedInventory>();
        world.register::<ItemProperties>();
        world.register::<Equippable>();
        world.register::<ItemBonuses>();

        let player = world.create_entity()
            .with(Player)
            .with(Name { name: "Hero".to_string() })
            .with(AdvancedInventory::new(30))
            .build();

        (world, player)
    }

    #[test]
    fn test_inventory_ui_creation() {
        let ui = InventoryUI::new();
        
        assert_eq!(ui.state, InventoryUIState::Closed);
        assert!(ui.player_entity.is_none());
        assert_eq!(ui.current_filter, InventoryFilter::All);
        assert_eq!(ui.current_sort, InventorySortMode::Type);
        assert!(ui.sort_ascending);
    }

    #[test]
    fn test_inventory_ui_open_close() {
        let (world, player) = setup_test_world();
        let mut ui = InventoryUI::new();
        
        assert!(!ui.is_open());
        
        ui.open(player);
        assert!(ui.is_open());
        assert_eq!(ui.player_entity, Some(player));
        assert_eq!(ui.state, InventoryUIState::ItemList);
        
        ui.close();
        assert!(!ui.is_open());
        assert_eq!(ui.state, InventoryUIState::Closed);
    }

    #[test]
    fn test_inventory_action_availability() {
        let weapon_type = ItemType::Weapon(WeaponType::Sword);
        let consumable_type = ItemType::Consumable(ConsumableType::Potion);
        
        assert!(InventoryAction::Equip.is_available_for_item(&weapon_type, false));
        assert!(!InventoryAction::Equip.is_available_for_item(&weapon_type, true));
        assert!(InventoryAction::Unequip.is_available_for_item(&weapon_type, true));
        assert!(!InventoryAction::Unequip.is_available_for_item(&weapon_type, false));
        
        assert!(InventoryAction::Use.is_available_for_item(&consumable_type, false));
        assert!(!InventoryAction::Use.is_available_for_item(&weapon_type, false));
        
        assert!(InventoryAction::Drop.is_available_for_item(&weapon_type, false));
        assert!(InventoryAction::Examine.is_available_for_item(&weapon_type, false));
    }

    #[test]
    fn test_inventory_filter_matching() {
        let weapon_props = ItemProperties::new("Sword".to_string(), ItemType::Weapon(WeaponType::Sword));
        let armor_props = ItemProperties::new("Helmet".to_string(), ItemType::Armor(ArmorType::Helmet));
        let rare_props = ItemProperties::new("Rare Item".to_string(), ItemType::Misc)
            .with_rarity(ItemRarity::Rare);
        
        assert!(InventoryFilter::All.matches_item(&weapon_props, false));
        assert!(InventoryFilter::Weapons.matches_item(&weapon_props, false));
        assert!(!InventoryFilter::Armor.matches_item(&weapon_props, false));
        
        assert!(InventoryFilter::Armor.matches_item(&armor_props, false));
        assert!(!InventoryFilter::Weapons.matches_item(&armor_props, false));
        
        assert!(InventoryFilter::Rarity(ItemRarity::Rare).matches_item(&rare_props, false));
        assert!(!InventoryFilter::Rarity(ItemRarity::Common).matches_item(&rare_props, false));
        
        assert!(InventoryFilter::Equipped.matches_item(&weapon_props, true));
        assert!(!InventoryFilter::Equipped.matches_item(&weapon_props, false));
        assert!(InventoryFilter::Unequipped.matches_item(&weapon_props, false));
        assert!(!InventoryFilter::Unequipped.matches_item(&weapon_props, true));
    }

    #[test]
    fn test_sort_mode_strings() {
        assert_eq!(InventorySortMode::Name.to_string(), "Name");
        assert_eq!(InventorySortMode::Type.to_string(), "Type");
        assert_eq!(InventorySortMode::Rarity.to_string(), "Rarity");
        assert_eq!(InventorySortMode::Value.to_string(), "Value");
        assert_eq!(InventorySortMode::Weight.to_string(), "Weight");
        assert_eq!(InventorySortMode::Recent.to_string(), "Recently Added");
    }

    #[test]
    fn test_inventory_action_strings() {
        assert_eq!(InventoryAction::Use.to_string(), "Use");
        assert_eq!(InventoryAction::Equip.to_string(), "Equip");
        assert_eq!(InventoryAction::Drop.to_string(), "Drop");
        assert_eq!(InventoryAction::Examine.to_string(), "Examine");
        assert_eq!(InventoryAction::Compare.to_string(), "Compare");
        assert_eq!(InventoryAction::Cancel.to_string(), "Cancel");
    }
}