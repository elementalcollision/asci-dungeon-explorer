use crossterm::style::Color;
use serde::{Serialize, Deserialize};

/// Base trait for UI components
pub trait UIComponent {
    fn render(&self, x: i32, y: i32, width: i32, height: i32) -> Vec<UIRenderCommand>;
    fn handle_input(&mut self, input: char) -> bool;
    fn is_focused(&self) -> bool;
    fn set_focus(&mut self, focused: bool);
}

/// Commands for rendering UI elements
#[derive(Debug, Clone)]
pub enum UIRenderCommand {
    DrawText {
        x: i32,
        y: i32,
        text: String,
        fg: Color,
        bg: Color,
    },
    DrawBox {
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border_color: Color,
        fill_color: Color,
    },
    DrawLine {
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        color: Color,
        character: char,
    },
    SetCursor {
        x: i32,
        y: i32,
        visible: bool,
    },
}

/// A panel that can contain other UI components
#[derive(Debug, Clone)]
pub struct UIPanel {
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub border_color: Color,
    pub background_color: Color,
    pub title_color: Color,
    pub visible: bool,
    pub has_border: bool,
}

impl UIPanel {
    pub fn new(title: String, x: i32, y: i32, width: i32, height: i32) -> Self {
        UIPanel {
            title,
            x,
            y,
            width,
            height,
            border_color: Color::White,
            background_color: Color::Black,
            title_color: Color::Yellow,
            visible: true,
            has_border: true,
        }
    }

    pub fn with_colors(mut self, border: Color, background: Color, title: Color) -> Self {
        self.border_color = border;
        self.background_color = background;
        self.title_color = title;
        self
    }

    pub fn without_border(mut self) -> Self {
        self.has_border = false;
        self
    }

    pub fn render(&self) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if !self.visible {
            return commands;
        }

        // Draw background
        commands.push(UIRenderCommand::DrawBox {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            border_color: if self.has_border { self.border_color } else { self.background_color },
            fill_color: self.background_color,
        });

        // Draw title if not empty
        if !self.title.is_empty() && self.has_border {
            let title_x = self.x + (self.width - self.title.len() as i32) / 2;
            commands.push(UIRenderCommand::DrawText {
                x: title_x,
                y: self.y,
                text: format!(" {} ", self.title),
                fg: self.title_color,
                bg: self.background_color,
            });
        }

        commands
    }

    pub fn inner_bounds(&self) -> (i32, i32, i32, i32) {
        if self.has_border {
            (self.x + 1, self.y + 1, self.width - 2, self.height - 2)
        } else {
            (self.x, self.y, self.width, self.height)
        }
    }
}

/// A clickable button
#[derive(Debug, Clone)]
pub struct UIButton {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub normal_fg: Color,
    pub normal_bg: Color,
    pub hover_fg: Color,
    pub hover_bg: Color,
    pub pressed_fg: Color,
    pub pressed_bg: Color,
    pub focused: bool,
    pub pressed: bool,
    pub enabled: bool,
}

impl UIButton {
    pub fn new(text: String, x: i32, y: i32, width: i32) -> Self {
        UIButton {
            text,
            x,
            y,
            width,
            height: 1,
            normal_fg: Color::White,
            normal_bg: Color::DarkGrey,
            hover_fg: Color::Black,
            hover_bg: Color::White,
            pressed_fg: Color::White,
            pressed_bg: Color::DarkBlue,
            focused: false,
            pressed: false,
            enabled: true,
        }
    }

    pub fn with_colors(mut self, normal_fg: Color, normal_bg: Color, hover_fg: Color, hover_bg: Color) -> Self {
        self.normal_fg = normal_fg;
        self.normal_bg = normal_bg;
        self.hover_fg = hover_fg;
        self.hover_bg = hover_bg;
        self
    }
}

impl UIComponent for UIButton {
    fn render(&self, _x: i32, _y: i32, _width: i32, _height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();

        if !self.enabled {
            commands.push(UIRenderCommand::DrawText {
                x: self.x,
                y: self.y,
                text: format!("{:width$}", self.text, width = self.width as usize),
                fg: Color::DarkGrey,
                bg: Color::Black,
            });
            return commands;
        }

        let (fg, bg) = if self.pressed {
            (self.pressed_fg, self.pressed_bg)
        } else if self.focused {
            (self.hover_fg, self.hover_bg)
        } else {
            (self.normal_fg, self.normal_bg)
        };

        // Draw button background
        commands.push(UIRenderCommand::DrawBox {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            border_color: fg,
            fill_color: bg,
        });

        // Draw button text (centered)
        let text_x = self.x + (self.width - self.text.len() as i32) / 2;
        commands.push(UIRenderCommand::DrawText {
            x: text_x,
            y: self.y,
            text: self.text.clone(),
            fg,
            bg,
        });

        commands
    }

    fn handle_input(&mut self, input: char) -> bool {
        if !self.enabled {
            return false;
        }

        match input {
            '\n' | ' ' => {
                self.pressed = true;
                true
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

/// A text display component
#[derive(Debug, Clone)]
pub struct UIText {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub fg: Color,
    pub bg: Color,
    pub alignment: TextAlignment,
    pub word_wrap: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
}

impl UIText {
    pub fn new(text: String, x: i32, y: i32, width: i32) -> Self {
        UIText {
            text,
            x,
            y,
            width,
            height: 1,
            fg: Color::White,
            bg: Color::Black,
            alignment: TextAlignment::Left,
            word_wrap: false,
        }
    }

    pub fn with_colors(mut self, fg: Color, bg: Color) -> Self {
        self.fg = fg;
        self.bg = bg;
        self
    }

    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_word_wrap(mut self) -> Self {
        self.word_wrap = true;
        self
    }

    fn wrap_text(&self, text: &str, width: usize) -> Vec<String> {
        if !self.word_wrap || width == 0 {
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

        lines
    }

    fn align_text(&self, text: &str, width: usize) -> String {
        match self.alignment {
            TextAlignment::Left => format!("{:<width$}", text, width = width),
            TextAlignment::Center => {
                let padding = (width.saturating_sub(text.len())) / 2;
                format!("{:>pad$}{}", "", text, pad = padding)
            }
            TextAlignment::Right => format!("{:>width$}", text, width = width),
        }
    }
}

impl UIComponent for UIText {
    fn render(&self, _x: i32, _y: i32, _width: i32, _height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let lines = self.wrap_text(&self.text, self.width as usize);

        for (i, line) in lines.iter().enumerate() {
            if i >= self.height as usize {
                break;
            }

            let aligned_text = self.align_text(line, self.width as usize);
            commands.push(UIRenderCommand::DrawText {
                x: self.x,
                y: self.y + i as i32,
                text: aligned_text,
                fg: self.fg,
                bg: self.bg,
            });
        }

        commands
    }

    fn handle_input(&mut self, _input: char) -> bool {
        false // Text components don't handle input
    }

    fn is_focused(&self) -> bool {
        false // Text components can't be focused
    }

    fn set_focus(&mut self, _focused: bool) {
        // Text components can't be focused
    }
}

/// A list component for displaying selectable items
#[derive(Debug, Clone)]
pub struct UIList {
    pub items: Vec<String>,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub selected_index: usize,
    pub scroll_offset: usize,
    pub normal_fg: Color,
    pub normal_bg: Color,
    pub selected_fg: Color,
    pub selected_bg: Color,
    pub focused: bool,
}

impl UIList {
    pub fn new(items: Vec<String>, x: i32, y: i32, width: i32, height: i32) -> Self {
        UIList {
            items,
            x,
            y,
            width,
            height,
            selected_index: 0,
            scroll_offset: 0,
            normal_fg: Color::White,
            normal_bg: Color::Black,
            selected_fg: Color::Black,
            selected_bg: Color::White,
            focused: false,
        }
    }

    pub fn with_colors(mut self, normal_fg: Color, normal_bg: Color, selected_fg: Color, selected_bg: Color) -> Self {
        self.normal_fg = normal_fg;
        self.normal_bg = normal_bg;
        self.selected_fg = selected_fg;
        self.selected_bg = selected_bg;
        self
    }

    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }

    pub fn remove_item(&mut self, index: usize) {
        if index < self.items.len() {
            self.items.remove(index);
            if self.selected_index >= self.items.len() && !self.items.is_empty() {
                self.selected_index = self.items.len() - 1;
            }
        }
    }

    pub fn get_selected_item(&self) -> Option<&String> {
        self.items.get(self.selected_index)
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
            self.ensure_visible();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.items.len() - 1
            } else {
                self.selected_index - 1
            };
            self.ensure_visible();
        }
    }

    fn ensure_visible(&mut self) {
        let visible_height = self.height as usize;
        
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected_index - visible_height + 1;
        }
    }
}

impl UIComponent for UIList {
    fn render(&self, _x: i32, _y: i32, _width: i32, _height: i32) -> Vec<UIRenderCommand> {
        let mut commands = Vec::new();
        let visible_height = self.height as usize;

        for i in 0..visible_height {
            let item_index = self.scroll_offset + i;
            let y_pos = self.y + i as i32;

            if item_index < self.items.len() {
                let item = &self.items[item_index];
                let is_selected = item_index == self.selected_index && self.focused;
                
                let (fg, bg) = if is_selected {
                    (self.selected_fg, self.selected_bg)
                } else {
                    (self.normal_fg, self.normal_bg)
                };

                // Truncate item text if too long
                let display_text = if item.len() > self.width as usize {
                    format!("{}...", &item[..self.width as usize - 3])
                } else {
                    format!("{:<width$}", item, width = self.width as usize)
                };

                commands.push(UIRenderCommand::DrawText {
                    x: self.x,
                    y: y_pos,
                    text: display_text,
                    fg,
                    bg,
                });
            } else {
                // Fill empty space
                commands.push(UIRenderCommand::DrawText {
                    x: self.x,
                    y: y_pos,
                    text: " ".repeat(self.width as usize),
                    fg: self.normal_fg,
                    bg: self.normal_bg,
                });
            }
        }

        // Draw scrollbar if needed
        if self.items.len() > visible_height {
            let scrollbar_x = self.x + self.width - 1;
            let scrollbar_height = self.height;
            let thumb_size = (visible_height * scrollbar_height as usize / self.items.len()).max(1);
            let thumb_pos = self.scroll_offset * scrollbar_height as usize / self.items.len();

            for i in 0..scrollbar_height {
                let y_pos = self.y + i;
                let is_thumb = i >= thumb_pos as i32 && i < (thumb_pos + thumb_size) as i32;
                
                commands.push(UIRenderCommand::DrawText {
                    x: scrollbar_x,
                    y: y_pos,
                    text: if is_thumb { "█" } else { "░" }.to_string(),
                    fg: Color::Grey,
                    bg: self.normal_bg,
                });
            }
        }

        commands
    }

    fn handle_input(&mut self, input: char) -> bool {
        match input {
            'j' | 's' => {
                self.select_next();
                true
            }
            'k' | 'w' => {
                self.select_previous();
                true
            }
            '\n' | ' ' => {
                // Item selected
                true
            }
            _ => false,
        }
    }

    fn is_focused(&self) -> bool {
        self.focused
    }

    fn set_focus(&mut self, focused: bool) {
        self.focused = focused;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_panel_creation() {
        let panel = UIPanel::new("Test Panel".to_string(), 10, 5, 20, 10);
        
        assert_eq!(panel.title, "Test Panel");
        assert_eq!(panel.x, 10);
        assert_eq!(panel.y, 5);
        assert_eq!(panel.width, 20);
        assert_eq!(panel.height, 10);
        assert!(panel.visible);
        assert!(panel.has_border);
    }

    #[test]
    fn test_ui_panel_inner_bounds() {
        let panel = UIPanel::new("Test".to_string(), 0, 0, 10, 5);
        let (x, y, w, h) = panel.inner_bounds();
        
        assert_eq!(x, 1);
        assert_eq!(y, 1);
        assert_eq!(w, 8);
        assert_eq!(h, 3);
    }

    #[test]
    fn test_ui_panel_inner_bounds_no_border() {
        let panel = UIPanel::new("Test".to_string(), 0, 0, 10, 5).without_border();
        let (x, y, w, h) = panel.inner_bounds();
        
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(w, 10);
        assert_eq!(h, 5);
    }

    #[test]
    fn test_ui_button_creation() {
        let button = UIButton::new("Click Me".to_string(), 5, 3, 15);
        
        assert_eq!(button.text, "Click Me");
        assert_eq!(button.x, 5);
        assert_eq!(button.y, 3);
        assert_eq!(button.width, 15);
        assert!(!button.focused);
        assert!(!button.pressed);
        assert!(button.enabled);
    }

    #[test]
    fn test_ui_button_input() {
        let mut button = UIButton::new("Test".to_string(), 0, 0, 10);
        
        assert!(!button.handle_input('a'));
        assert!(button.handle_input('\n'));
        assert!(button.pressed);
        
        button.pressed = false;
        assert!(button.handle_input(' '));
        assert!(button.pressed);
    }

    #[test]
    fn test_ui_list_creation() {
        let items = vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()];
        let list = UIList::new(items.clone(), 0, 0, 20, 5);
        
        assert_eq!(list.items, items);
        assert_eq!(list.selected_index, 0);
        assert_eq!(list.scroll_offset, 0);
        assert!(!list.focused);
    }

    #[test]
    fn test_ui_list_navigation() {
        let items = vec!["Item 1".to_string(), "Item 2".to_string(), "Item 3".to_string()];
        let mut list = UIList::new(items, 0, 0, 20, 5);
        
        assert_eq!(list.selected_index, 0);
        
        list.select_next();
        assert_eq!(list.selected_index, 1);
        
        list.select_next();
        assert_eq!(list.selected_index, 2);
        
        list.select_next(); // Should wrap to 0
        assert_eq!(list.selected_index, 0);
        
        list.select_previous(); // Should wrap to 2
        assert_eq!(list.selected_index, 2);
    }

    #[test]
    fn test_ui_text_word_wrap() {
        let text = UIText::new("This is a long text that should wrap".to_string(), 0, 0, 10)
            .with_word_wrap();
        
        let wrapped = text.wrap_text("This is a long text that should wrap", 10);
        
        assert!(wrapped.len() > 1);
        for line in &wrapped {
            assert!(line.len() <= 10);
        }
    }

    #[test]
    fn test_ui_text_alignment() {
        let text = UIText::new("Test".to_string(), 0, 0, 10)
            .with_alignment(TextAlignment::Center);
        
        let aligned = text.align_text("Test", 10);
        assert!(aligned.starts_with("   ")); // Should have padding for centering
    }
}