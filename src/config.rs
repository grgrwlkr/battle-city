use bevy::prelude::*;

/// Game configuration resources
/// This module contains all configurable game parameters that can be
/// easily modified or loaded from external files.
/// Player configuration
#[derive(Resource, Debug, Clone)]
pub struct PlayerConfig {
    /// Player movement speed in pixels per second
    pub speed: f32,
    /// Bullet cooldown interval in seconds
    pub bullet_cooldown: f32,
    /// Initial number of lives
    pub initial_lives: i8,
    /// Tank size in pixels
    pub tank_size: u32,
    /// Tank scale factor
    pub tank_scale: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            speed: 150.0,
            bullet_cooldown: 0.5,
            initial_lives: 3,
            tank_size: 28,
            tank_scale: 0.8,
        }
    }
}

/// Enemy configuration
#[derive(Resource, Debug, Clone)]
pub struct EnemyConfig {
    /// Enemy movement speed in pixels per second
    pub speed: f32,
    /// Bullet cooldown interval in seconds
    pub bullet_cooldown: f32,
    /// Maximum number of enemies that can coexist simultaneously
    pub max_live_enemies: i32,
    /// Number of enemies per level
    pub enemies_per_level: i32,
    /// Tank size in pixels
    pub tank_size: u32,
    /// Tank scale factor
    pub tank_scale: f32,
}

impl Default for EnemyConfig {
    fn default() -> Self {
        Self {
            speed: 100.0,
            bullet_cooldown: 2.0,
            max_live_enemies: 5,
            enemies_per_level: 12,
            tank_size: 28,
            tank_scale: 0.8,
        }
    }
}

/// Level configuration
#[derive(Resource, Debug, Clone)]
pub struct LevelConfig {
    /// Number of rows in the level map
    pub rows: i32,
    /// Number of columns in the level map
    pub columns: i32,
    /// Tile size in pixels
    pub tile_size: f32,
    /// Maximum number of levels
    pub max_levels: i32,
}

impl Default for LevelConfig {
    fn default() -> Self {
        Self {
            rows: 18,
            columns: 27,
            tile_size: 32.0,
            max_levels: 2,
        }
    }
}

/// Bullet configuration
#[derive(Resource, Debug, Clone)]
pub struct BulletConfig {
    /// Bullet movement speed in pixels per second
    pub speed: f32,
}

impl Default for BulletConfig {
    fn default() -> Self {
        Self { speed: 300.0 }
    }
}

/// Sprite ordering configuration (z-axis)
#[derive(Resource, Debug, Clone)]
pub struct SpriteOrderConfig {
    /// Game over screen z-order
    pub game_over: f32,
    /// Tree sprite z-order
    pub tree: f32,
    /// Player sprite z-order
    pub player: f32,
}

impl Default for SpriteOrderConfig {
    fn default() -> Self {
        Self {
            game_over: 4.0,
            tree: 3.0,
            player: 2.0,
        }
    }
}

/// Main game configuration resource
/// Contains all sub-configurations for different game systems
#[derive(Resource, Debug, Clone, Default)]
pub struct GameConfig {
    pub player: PlayerConfig,
    pub enemy: EnemyConfig,
    pub level: LevelConfig,
    pub bullet: BulletConfig,
    pub sprite_order: SpriteOrderConfig,
}
