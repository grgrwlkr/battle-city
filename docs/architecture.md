# Battle City - Architecture Documentation

**Version:** 1.0  
**Last Updated:** 2024

---

## Overview

Battle City is a top-down tank battle game built with Bevy game engine using ECS (Entity Component System) architecture. The game follows best practices for Bevy development including modular plugins, organized system sets, and structured configuration.

---

## Project Structure

```
src/
├── main.rs           # Application entry point, plugin registration
├── common.rs         # Shared types, constants, system sets, utilities
├── config.rs         # Game configuration resources
├── player.rs         # Player systems, components, plugin
├── enemy.rs          # Enemy systems, AI, plugin
├── bullet.rs         # Bullet systems, collisions, explosions, plugin
├── level.rs          # Level management, LDTK integration, plugin
├── ui.rs             # UI systems, menus, plugin
└── area.rs           # Game area boundaries
```

---

## Architecture Principles

### 1. Modular Plugins

Each major game system is encapsulated in its own plugin:

- **CorePlugin**: Initializes core resources, messages, and states
- **PlayerPlugin**: Manages player entities, spawning, movement, combat
- **EnemyPlugin**: Manages enemy entities, AI, spawning
- **BulletPlugin**: Handles bullet physics, collisions, explosions
- **LevelPlugin**: Manages level loading, transitions, LDTK entities
- **UiPlugin**: Handles UI, menus, state transitions

This modular approach provides:
- Clear separation of concerns
- Easy testing and maintenance
- Simplified `main.rs` (reduced from ~224 to ~66 lines)

### 2. System Sets and Execution Order

Systems are organized into logical `GameplaySet`s with explicit dependencies:

```rust
GameplaySet::Input
  ↓
GameplaySet::Spawning
  ↓
GameplaySet::Movement
  ↓
GameplaySet::Combat
  ↓
GameplaySet::BulletMovement
  ↓
GameplaySet::Collision
  ↓
GameplaySet::Effects
  ↓
GameplaySet::Animation
```

This ensures:
- Predictable execution order
- No race conditions
- Clear dependencies between systems

### 3. Configuration Resources

All game parameters are configurable through `GameConfig` resource:

- `PlayerConfig`: Speed, bullet cooldown, lives, tank properties
- `EnemyConfig`: Speed, cooldown, spawn limits, tank properties
- `LevelConfig`: Dimensions, tile size, max levels
- `BulletConfig`: Speed
- `SpriteOrderConfig`: Z-ordering for rendering

Benefits:
- Easy balancing without code changes
- Potential for runtime configuration loading
- Type-safe configuration

### 4. State Management

Game flow is controlled through `AppState`:

- **StartMenu**: Initial menu screen
- **Playing**: Active gameplay
- **Paused**: Paused gameplay (can resume)
- **GameOver**: Game over screen (transitions back to menu)

State transitions trigger appropriate systems:
- `OnEnter` systems: Setup and initialization
- `OnExit` systems: Cleanup and despawn
- State-specific `Update` systems: Gameplay logic

---

## Key Systems

### Player Systems

- `auto_spawn_players`: Monitors and spawns players when needed
- `players_move`: Handles keyboard input and movement
- `players_attack`: Manages shooting and bullet spawning
- `animate_players`: Updates player sprite animations
- `animate_shield`: Animates spawn protection shield
- `animate_born`: Handles spawn animation sequence

### Enemy Systems

- `auto_spawn_enemies`: Spawns enemies based on limits and conditions
- `enemies_move`: AI pathfinding and movement
- `enemies_attack`: Automatic shooting behavior
- `animate_enemies`: Enemy sprite animations
- `handle_enemy_collision`: Collision-based direction changes

### Bullet Systems

- `move_bullet`: Updates bullet positions
- `handle_bullet_collision`: Processes all bullet collision events
- `spawn_explosion`: Creates explosion effects
- `animate_explosion`: Explosion animation loop

### Level Systems

- `setup_levels`: Loads LDTK level files
- `spawn_ldtk_entity`: Spawns entities from level data
- `auto_switch_level`: Manages level progression
- `animate_water`: Water animation effects
- `animate_home`: Home base destruction animation

### UI Systems

- `setup_start_menu`: Creates menu UI
- `start_game`: Handles game start input
- `switch_multiplayer_mode`: Toggles between 1P and 2P modes
- `pause_game`: Pauses gameplay
- `unpause_game`: Resumes gameplay
- `setup_game_over`: Creates game over screen
- `animate_game_over`: Game over animation sequence

---

## ECS Patterns

### Components

Components are data that can be attached to entities:

- **Player Components**: `PlayerNo`, `Shield`, `TankRefreshBulletTimer`
- **Enemy Components**: `Enemy`, `EnemyChangeDirectionTimer`
- **Animation Components**: `AnimationTimer`, `AnimationIndices`
- **UI Components**: `OnStartMenuScreen`, `OnGameOverScreen`
- **Level Components**: `LevelItem`, `Player1Marker`, `Player2Marker`, `EnemiesMarker`

### Resources

Resources are singleton data accessible by systems:

- **Configuration**: `GameConfig`, `PlayerLives`, `LevelSpawnedEnemies`
- **Game State**: `AppState`, `MultiplayerMode`, `LevelSelection`
- **Assets**: `GameSounds`, `ExplosionAssets`
- **Utilities**: `ScheduledDespawn`

### Messages/Events

Messages provide inter-system communication:

- `SpawnPlayerEvent`: Player spawn trigger
- `ExplosionEvent`: Explosion spawn trigger
- `HomeDyingEvent`: Home base destruction trigger
- `CollisionEvent`: Physics collision events (from Rapier)

---

## Collision Detection

The game uses Bevy Rapier2D for physics and collision detection:

- **RigidBody**: Dynamic for tanks/bullets, Fixed for walls
- **Collider**: Circle for tanks, Cuboid for walls/bullets
- **Sensor**: Bullets use sensors for collision detection
- **ActiveEvents::COLLISION_EVENTS**: Enables collision event generation

Collision handling flow:
1. Rapier detects collision → `CollisionEvent`
2. `handle_bullet_collision` processes bullet collisions
3. Appropriate responses (destroy, damage, game over)
4. Explosion events spawned if needed

---

## Animation System

The game uses a generic `animate_sprite_sheet` function to avoid code duplication:

```rust
pub fn animate_sprite_sheet<T: Component>(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &AnimationIndices, &mut Sprite), With<T>>,
)
```

All animation systems (`animate_players`, `animate_enemies`, `animate_shield`) internally use this function, ensuring consistent animation behavior.

---

## Testing

### Unit Tests

Unit tests cover:
- Configuration defaults and cloning
- Component equality and behavior
- Resource initialization
- Enum variants and state transitions

Located in `#[cfg(test)]` modules within each source file.

### Running Tests

```bash
cargo test              # Run all tests
cargo test --lib        # Run library tests only
cargo test config::tests # Run specific module tests
```

---

## Build Configuration

The project uses standard Rust/Cargo configuration:

- **Edition**: 2021
- **Bevy Version**: 0.17
- **Physics**: bevy_rapier2d (custom git revision)
- **Level Editor**: bevy_ecs_ldtk 0.13

---

## Future Improvements

Potential enhancements (from TODO list):

1. **Tank collision forces movement** - Currently not implemented
2. **Game victory screen** - Placeholder exists
3. **Active enemy attacks** - Enemies should attack when player detected
4. **Tree cover mechanics** - Trees could provide cover
5. **Configuration file loading** - Load config from external files
6. **More levels** - Currently 2 levels
7. **Achievement system** - Track player achievements
8. **Statistics system** - Gameplay statistics

---

## Code Style

The project follows:
- Rust standard formatting (rustfmt)
- Clippy linting recommendations
- Bevy naming conventions
- English comments and documentation
- Clear, descriptive function and variable names

---

**See also:**
- [Bevy Documentation](https://bevyengine.org/learn/)
- [Bevy Cheatbook](https://bevy-cheatbook.github.io/)
- [Rapier Physics Documentation](https://rapier.rs/docs/)
