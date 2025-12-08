# battle-city

A classic top-down tank battle game built with Bevy game engine.

## Features

- [x] Design levels (Ldtk software)
- [x] Load levels
- [x] Switch levels
- [x] Random resurrection positions
- [x] Collision detection using physics engine
- [x] Resurrection, shield, explosion and water etc sprite animations
- [x] game ui
- [x] game sounds
- [x] pause game
- [x] enemies ai
- [x] local multiplayer
- [x] WASM support

## Architecture

The project follows Bevy best practices and clean architecture principles:

- **Modular Plugins**: Each game system is encapsulated in its own plugin
- **System Sets**: Organized system execution with explicit dependencies
- **Configuration Resources**: All game parameters are configurable
- **ECS Architecture**: Entity Component System for game logic
- **Structured Logging**: Comprehensive logging for debugging

For detailed architecture documentation, see [docs/architecture.md](docs/architecture.md).

## Project Structure

```
src/
├── main.rs       # Application entry, plugin registration
├── common.rs     # Shared types, system sets
├── config.rs     # Game configuration
├── player.rs     # Player systems
├── enemy.rs      # Enemy systems and AI
├── bullet.rs     # Bullet and collision systems
├── level.rs      # Level management
├── ui.rs         # UI and menus
└── area.rs       # Game boundaries
```

Play Online: [Click here](https://nightswatchgames.github.io/games/battle-city/) (Open with PC Chrome/Firefox/Edge)

## Get started
1. Native
```
cargo run
```
2. WASM
```
rustup target install wasm32-unknown-unknown
cargo install wasm-server-runner
cargo run --target wasm32-unknown-unknown
```
```
cargo install wasm-bindgen-cli
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/battle-city.wasm
```

## Screenshots
Game video: [YouTube](https://www.youtube.com/watch?v=54Z2WBFZfzA)

![start_menu](screenshots/start_menu.png)
![game_playing](screenshots/game_playing.png)
![game_over](screenshots/game_over.png)

## References
- [bevy-cheatbook](https://github.com/bevy-cheatbook/bevy-cheatbook) ([Chinese translation](https://yiviv.com/bevy-cheatbook/))
- [Battle City - Wikipedia](https://en.wikipedia.org/wiki/Battle_City)
- [Unity Tank Battle Tutorial](https://www.bilibili.com/video/BV1PW41197Su)

## Questions

**1. When changing direction in a 2D game, is it better to switch sprites or rotate the sprite?**

**2. Understanding the difference between Sprite Sheet and Texture Atlas:**
A Texture Atlas is a large image containing multiple textures. A Sprite Sheet usually refers to a large image containing each frame of frame animation (texture), which is essentially the same as a Texture Atlas. In Bevy 0.10, frame animation (SpriteSheetBundle) also directly uses `struct TextureAtlas` to store the atlas, and does not use a separate `struct SpriteSheet`.

References:
- https://forum.unity.com/threads/sprite-atlas-vs-manual-sprite-sheet.1229424/
- https://gamedev.stackexchange.com/questions/69895/what-is-the-difference-between-a-sprite-sheet-and-a-texture-atlas
- https://docs.rs/bevy/latest/bevy/sprite/struct.TextureAtlas.html
- https://docs.rs/bevy/latest/bevy/sprite/struct.SpriteSheetBundle.html
