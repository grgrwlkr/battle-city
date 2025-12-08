use bevy::prelude::*;
use std::collections::HashSet;

// Number of rows and columns in the level map
pub const LEVEL_ROWS: i32 = 18;
pub const LEVEL_COLUMNS: i32 = 27;
pub const TILE_SIZE: f32 = 32.0;

/// Application states for game flow management
#[derive(Debug, Clone, Eq, PartialEq, Hash, States, Default)]
pub enum AppState {
    #[default]
    /// Initial menu screen where players can start the game
    StartMenu,
    /// Main gameplay state
    Playing,
    /// Paused state (can be resumed)
    Paused,
    /// Game over state
    GameOver,
}

/// Multiplayer mode configuration
#[derive(Resource, Debug, PartialEq, Eq, Clone, Copy)]
pub enum MultiplayerMode {
    /// Single player mode
    SinglePlayer,
    /// Two players mode
    TwoPlayers,
}

/// Direction enum for entity movement and orientation
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Left direction
    Left,
    /// Right direction
    Right,
    /// Up direction
    Up,
    /// Down direction
    Down,
}

/// Animation timer component for sprite sheet animations
#[derive(Component, Debug, Clone)]
pub struct AnimationTimer(pub Timer);

/// Animation indices for sprite sheet animation range
#[derive(Component, Debug, Clone, Copy)]
pub struct AnimationIndices {
    /// First frame index in the animation sequence
    pub first: usize,
    /// Last frame index in the animation sequence
    pub last: usize,
}

/// Tank refresh bullet timer component
/// Controls the cooldown between bullet shots
#[derive(Component, Debug, Deref, DerefMut)]
pub struct TankRefreshBulletTimer(pub Timer);

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

/// Resource to deduplicate despawn requests within a frame.
/// This prevents multiple systems from trying to despawn the same entity
/// in a single frame, which would cause panics.
#[derive(Default, Resource)]
pub struct ScheduledDespawn(pub HashSet<Entity>);

/// Game sounds resource containing all audio handles
#[derive(Debug, Resource)]
pub struct GameSounds {
    /// Mode switch sound
    pub mode_switch: Handle<AudioSource>,
    /// Bullet explosion sound
    pub bullet_explosion: Handle<AudioSource>,
    /// Big explosion sound
    pub big_explosion: Handle<AudioSource>,
    /// Player fire sound
    pub player_fire: Handle<AudioSource>,
    /// Game over sound
    pub game_over: Handle<AudioSource>,
    /// Game pause sound
    pub game_pause: Handle<AudioSource>,
}

/// Setup game sounds resource
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

// Re-exports for convenience
pub use bevy_ecs_ldtk::LevelSelection;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state, AppState::StartMenu);
    }

    #[test]
    fn test_app_state_equality() {
        assert_eq!(AppState::StartMenu, AppState::StartMenu);
        assert_ne!(AppState::StartMenu, AppState::Playing);
        assert_ne!(AppState::Playing, AppState::Paused);
        assert_ne!(AppState::Paused, AppState::GameOver);
    }

    #[test]
    fn test_multiplayer_mode() {
        assert_eq!(MultiplayerMode::SinglePlayer, MultiplayerMode::SinglePlayer);
        assert_ne!(MultiplayerMode::SinglePlayer, MultiplayerMode::TwoPlayers);
    }

    #[test]
    fn test_direction_equality() {
        assert_eq!(Direction::Up, Direction::Up);
        assert_ne!(Direction::Up, Direction::Down);
        assert_ne!(Direction::Left, Direction::Right);
    }

    #[test]
    fn test_scheduled_despawn_default() {
        let despawn = ScheduledDespawn::default();
        assert!(despawn.0.is_empty());
    }

    #[test]
    fn test_animation_indices() {
        let indices = AnimationIndices { first: 0, last: 3 };
        assert_eq!(indices.first, 0);
        assert_eq!(indices.last, 3);
    }
}
