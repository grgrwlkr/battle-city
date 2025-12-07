use bevy::prelude::*;
use std::collections::HashSet;

// Number of rows and columns in the level map
pub const LEVEL_ROWS: i32 = 18;
pub const LEVEL_COLUMNS: i32 = 27;
pub const TILE_SIZE: f32 = 32.0;
// Number of levels
pub const MAX_LEVELS: i32 = 2;
// Maximum number of enemies that can coexist simultaneously
pub const MAX_LIVE_ENEMIES: i32 = 5;
// Number of enemies per level
pub const ENEMIES_PER_LEVEL: i32 = 12;
// Tank bullet refresh interval (seconds)
pub const PLAYER_REFRESH_BULLET_INTERVAL: f32 = 0.5;
pub const ENEMY_REFRESH_BULLET_INTERVAL: f32 = 2.0;
// Tank speed, size and scale
pub const PLAYER_SPEED: f32 = 150.0;
pub const ENEMY_SPEED: f32 = 100.0;
pub const TANK_SIZE: u32 = 28;
pub const TANK_SCALE: f32 = 0.8;

// Sprite z-axis ordering
pub const SPRITE_GAME_OVER_ORDER: f32 = 4.0;
pub const SPRITE_TREE_ORDER: f32 = 3.0;
pub const SPRITE_PLAYER_ORDER: f32 = 2.0;

#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    #[default]
    StartMenu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Resource, Debug, PartialEq, Eq)]
pub enum MultiplayerMode {
    SinglePlayer,
    TwoPlayers,
}

// Direction
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Component, Clone, Default, Debug)]
pub struct AnimationTimer(pub Timer);

#[derive(Component, Clone, Default, Debug)]
pub struct AnimationIndices {
    pub first: usize,
    pub last: usize,
}

// Tank bullet refresh timer
#[derive(Component, Deref, DerefMut)]
pub struct TankRefreshBulletTimer(pub Timer);

#[derive(Default, Message)]
pub struct HomeDyingEvent;

#[derive(Debug, Resource)]
pub struct GameSounds {
    pub mode_switch: Handle<AudioSource>,
    pub bullet_explosion: Handle<AudioSource>,
    pub big_explosion: Handle<AudioSource>,
    pub player_fire: Handle<AudioSource>,
    pub game_over: Handle<AudioSource>,
    pub game_pause: Handle<AudioSource>,
}

pub fn setup_game_sounds(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameSounds {
        mode_switch: asset_server.load("sounds/mode_switch.ogg"),
        bullet_explosion: asset_server.load("sounds/bullet_explosion.ogg"),
        big_explosion: asset_server.load("sounds/big_explosion.ogg"),
        player_fire: asset_server.load("sounds/player_fire.ogg"),
        game_over: asset_server.load("sounds/game_over.ogg"),
        game_pause: asset_server.load("sounds/game_pause.ogg"),
    });
}

// Resource to deduplicate despawn requests within a frame.
#[derive(Default, Resource)]
pub struct ScheduledDespawn(pub HashSet<Entity>);

/// System sets for organizing gameplay systems execution order
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameplaySet {
    /// Input handling (keyboard, gamepad, etc.)
    Input,
    /// Entity spawning (players, enemies, entities from LDTK)
    Spawning,
    /// Entity movement and AI
    Movement,
    /// Combat actions (shooting, attacks)
    Combat,
    /// Bullet movement
    BulletMovement,
    /// Collision detection and handling
    Collision,
    /// Post-collision effects (explosions, spawning after events)
    Effects,
    /// Animations
    Animation,
    /// Level management (switching levels, cleanup)
    LevelManagement,
    /// UI and game state updates (pause, game over)
    Ui,
}

/// Generic sprite sheet animation system
/// Animates sprites with AnimationTimer and AnimationIndices components
/// This function is used to avoid code duplication in animation systems
pub fn animate_sprite_sheet<T: Component>(
    time: Res<Time>,
    mut query: Query<(&mut AnimationTimer, &AnimationIndices, &mut Sprite), With<T>>,
) {
    for (mut timer, indices, mut sprite) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            // Switch to next sprite
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}
