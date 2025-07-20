# ASCII Dungeon Explorer

An ASCII-based roguelike game with procedurally generated dungeons, loot, and character development, built in Rust.

## Features

- Procedurally generated dungeons for endless exploration
- ASCII-based visualization for a classic roguelike experience
- Character development and progression system
- Turn-based tactical combat
- Diverse item and loot system
- Guild system with agent-based characters for autonomous exploration
- Optional language model integration for dynamic dialogue (requires the `language_model` feature)

## Building and Running

### Prerequisites

- Rust and Cargo (latest stable version recommended)

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/ascii-dungeon-explorer.git
cd ascii-dungeon-explorer

# Build the game
cargo build --release

# Run the game
cargo run --release
```

### Building with Language Model Support

```bash
# Build with language model support
cargo build --release --features language_model

# Run with language model support
cargo run --release --features language_model
```

## Controls

- Arrow keys or HJKL (vi keys): Move character
- YUBN: Move diagonally
- Space or .: Wait a turn
- G: Pick up item
- I: Open inventory
- C: Open character sheet
- >: Use stairs
- Q: Quit game
- Ctrl+S: Save game

## License

This project is licensed under the MIT License - see the LICENSE file for details.