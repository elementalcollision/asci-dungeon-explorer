use ascii_dungeon_explorer::rendering::terminal::{Terminal, with_terminal};
use crossterm::style::Color;
use std::{thread, time::Duration};

fn main() -> crossterm::Result<()> {
    // Run the test with the terminal helper
    with_terminal(|terminal| {
        // Get terminal size
        let (width, height) = terminal.size();
        let center_x = width / 2;
        let center_y = height / 2;
        
        // Clear the screen
        terminal.clear()?;
        
        // Draw a box in the center
        terminal.draw_box(
            center_x - 15,
            center_y - 5,
            30,
            10,
            Color::White,
            Color::Black
        )?;
        
        // Draw a title
        terminal.draw_text_centered(
            center_y - 4,
            "Terminal Test",
            Color::Yellow,
            Color::Black
        )?;
        
        // Draw some text
        terminal.draw_text(
            center_x - 13,
            center_y - 2,
            "This is a test of the terminal",
            Color::White,
            Color::Black
        )?;
        
        terminal.draw_text(
            center_x - 13,
            center_y,
            "Press any key to continue...",
            Color::Green,
            Color::Black
        )?;
        
        // Flush the output
        terminal.flush()?;
        
        // Wait for a key press
        loop {
            if let Some(_) = terminal.poll_key(100)? {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        // Test colors
        terminal.clear()?;
        
        terminal.draw_text_centered(
            2,
            "Color Test",
            Color::White,
            Color::Black
        )?;
        
        // Draw color samples
        let colors = [
            (Color::Black, "Black"),
            (Color::DarkGrey, "DarkGrey"),
            (Color::Grey, "Grey"),
            (Color::White, "White"),
            (Color::Red, "Red"),
            (Color::DarkRed, "DarkRed"),
            (Color::Green, "Green"),
            (Color::DarkGreen, "DarkGreen"),
            (Color::Blue, "Blue"),
            (Color::DarkBlue, "DarkBlue"),
            (Color::Yellow, "Yellow"),
            (Color::Magenta, "Magenta"),
            (Color::Cyan, "Cyan"),
        ];
        
        for (i, (color, name)) in colors.iter().enumerate() {
            terminal.draw_text(
                5,
                4 + i as u16,
                name,
                *color,
                Color::Black
            )?;
        }
        
        // Draw RGB colors
        for i in 0..10 {
            let r = (i * 25) as u8;
            let g = 255 - (i * 25) as u8;
            let b = 128;
            
            terminal.draw_text(
                20,
                4 + i as u16,
                &format!("RGB({}, {}, {})", r, g, b),
                Color::Rgb { r, g, b },
                Color::Black
            )?;
        }
        
        terminal.draw_text_centered(
            height - 2,
            "Press any key to exit...",
            Color::White,
            Color::Black
        )?;
        
        terminal.flush()?;
        
        // Wait for a key press
        loop {
            if let Some(_) = terminal.poll_key(100)? {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        Ok(())
    })
}