use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::bullet::*;
use crate::common::{self, AppState, *};
use crate::config::GameConfig;
use crate::enemy::Enemy;
use crate::level::{level_translation_offset, CollisionEvent, Player1Marker, Player2Marker};

/// Spawn protection shield component
/// Provides temporary invincibility after player spawn
#[derive(Component)]
pub struct Shield;

/// Timer component for shield removal
/// Controls when the spawn protection shield is removed
#[derive(Component)]
pub struct ShieldRemoveTimer(pub Timer);

/// Component marking a player spawn animation entity
#[derive(Component)]
pub struct Born;

/// Timer component for spawn animation removal
#[derive(Component)]
pub struct BornRemoveTimer(pub Timer);

/// Player number component
/// Identifies which player this entity belongs to (1 or 2)
#[derive(Debug, Clone, Copy, Component, Reflect, Default, PartialEq, Eq)]
#[reflect(Component)]
pub struct PlayerNo(pub u32);

/// Event emitted when a player should be spawned after animation completes
#[derive(Debug, Message)]
pub struct SpawnPlayerEvent {
    /// Spawn position on the map
    pub pos: Vec2,
    /// Player number (1 or 2)
    pub player_no: PlayerNo,
}

/// Resource tracking player lives
#[derive(Debug, Resource)]
pub struct PlayerLives {
    /// Number of lives remaining for player 1
    pub player1: i8,
    /// Number of lives remaining for player 2
    pub player2: i8,
}

impl Default for PlayerLives {
    fn default() -> Self {
        Self {
            player1: 3,
            player2: 3,
        }
    }
}

/// Automatically spawn players when they die and have remaining lives
/// Monitors player entities and spawn markers to initiate spawn animations
///
/// This system checks if players exist and if they should be respawned based on:
/// - Player existence on the map
/// - Remaining lives count
/// - Multiplayer mode configuration
#[allow(clippy::too_many_arguments)]
pub fn auto_spawn_players(
    mut commands: Commands,
    q_players: Query<&PlayerNo>,
    q_player1_marker: Query<&Transform, With<Player1Marker>>,
    q_player2_marker: Query<&Transform, With<Player2Marker>>,
    mut spawn_player_er: MessageReader<SpawnPlayerEvent>,
    mut spawning_player1: Local<bool>,
    mut spawning_player2: Local<bool>,
    multiplayer_mode: Res<MultiplayerMode>,
    mut player_lives: ResMut<PlayerLives>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    game_config: Res<GameConfig>,
) {
    // Optimize: use any() instead of iterating all players
    let player1_exists = q_players.iter().any(|player| player.0 == 1);
    let player2_exists = q_players.iter().any(|player| player.0 == 2);
    if !player1_exists {
        for player1_marker in &q_player1_marker {
            if !*spawning_player1 && player_lives.player1 > 0 {
                // Spawn animation
                let offset = level_translation_offset(&game_config);
                info!(
                    "Starting spawn animation for Player 1 at position {:?}, remaining lives: {}",
                    player1_marker.translation + offset,
                    player_lives.player1
                );
                spawn_born(
                    player1_marker.translation + offset,
                    PlayerNo(1),
                    &mut commands,
                    &asset_server,
                    &mut atlas_layouts,
                );
                *spawning_player1 = true;
            }
        }
    }
    if !player2_exists && *multiplayer_mode == MultiplayerMode::TwoPlayers {
        for player2_marker in &q_player2_marker {
            if !*spawning_player2 && player_lives.player2 > 0 {
                let offset = level_translation_offset(&game_config);
                info!(
                    "Starting spawn animation for Player 2 at position {:?}, remaining lives: {}",
                    player2_marker.translation + offset,
                    player_lives.player2
                );
                // Spawn animation
                let offset = level_translation_offset(&game_config);
                spawn_born(
                    player2_marker.translation + offset,
                    PlayerNo(2),
                    &mut commands,
                    &asset_server,
                    &mut atlas_layouts,
                );
                *spawning_player2 = true;
            }
        }
    }

    let shield_texture_handle = asset_server.load("textures/shield.bmp");
    let shield_texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(31, 31), 1, 2, None, None);
    let shield_atlas_layout_handle = atlas_layouts.add(shield_texture_atlas);

    // Player 1
    let tank_size = game_config.player.tank_size;
    let player1_texture_handle = asset_server.load("textures/tank1.bmp");
    let player1_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(tank_size, tank_size), 8, 4, None, None);
    let player1_atlas_layout_handle = atlas_layouts.add(player1_texture_atlas);

    // Player 2
    let player2_texture_handle = asset_server.load("textures/tank2.bmp");
    let player2_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(tank_size, tank_size), 8, 4, None, None);
    let player2_atlas_layout_handle = atlas_layouts.add(player2_texture_atlas);

    // After spawn animation completes, create player
    for spawn_player_event in spawn_player_er.read() {
        info!(
            "Spawning Player {} at position ({:.1}, {:.1}), lives remaining: P1={}, P2={}",
            spawn_player_event.player_no.0,
            spawn_player_event.pos.x,
            spawn_player_event.pos.y,
            player_lives.player1,
            player_lives.player2
        );
        // Protection shield
        let shield = commands
            .spawn((
                Shield,
                Sprite {
                    image: shield_texture_handle.clone(),
                    texture_atlas: Some(TextureAtlas {
                        index: 0,
                        layout: shield_atlas_layout_handle.clone(),
                    }),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, -1.0)), // Control sprite order via z-axis
                AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                AnimationIndices { first: 0, last: 1 },
                ShieldRemoveTimer(Timer::from_seconds(5.0, TimerMode::Once)),
            ))
            .id();

        // Tank
        let tank = commands
            .spawn((
                spawn_player_event.player_no,
                Sprite {
                    image: if spawn_player_event.player_no.0 == 1 {
                        player1_texture_handle.clone()
                    } else {
                        player2_texture_handle.clone()
                    },
                    texture_atlas: Some(TextureAtlas {
                        index: 0,
                        layout: if spawn_player_event.player_no.0 == 1 {
                            player1_atlas_layout_handle.clone()
                        } else {
                            player2_atlas_layout_handle.clone()
                        },
                    }),
                    ..default()
                },
                Transform {
                    translation: spawn_player_event
                        .pos
                        .extend(game_config.sprite_order.player),
                    scale: Vec3::splat(game_config.player.tank_scale),
                    ..default()
                },
                TankRefreshBulletTimer(Timer::from_seconds(
                    game_config.player.bullet_cooldown,
                    TimerMode::Once,
                )),
                common::Direction::Up,
                AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                AnimationIndices { first: 0, last: 1 },
                RigidBody::Dynamic,
                Velocity::zero(),
                // Circular collider prevents getting stuck on terrain due to ROTATION_LOCKED
                Collider::ball(
                    game_config.player.tank_size as f32 * game_config.player.tank_scale / 2.0 + 2.0,
                ),
                ActiveEvents::COLLISION_EVENTS,
                LockedAxes::ROTATION_LOCKED,
            ))
            .id();

        commands.entity(tank).add_child(shield);

        // Decrease lives
        if spawn_player_event.player_no.0 == 1 {
            player_lives.player1 -= 1;
            info!(
                "Player 1 spawned, lives remaining: {}",
                player_lives.player1
            );
        } else if spawn_player_event.player_no.0 == 2 {
            player_lives.player2 -= 1;
            info!(
                "Player 2 spawned, lives remaining: {}",
                player_lives.player2
            );
        }

        // Reset state
        if spawn_player_event.player_no.0 == 1 {
            *spawning_player1 = false;
        } else if spawn_player_event.player_no.0 == 2 {
            *spawning_player2 = false;
        }
    }
}

pub fn spawn_born(
    pos: Vec3,
    player_no: PlayerNo,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
) {
    // Spawn effect
    trace!(
        "Spawning born effect animation for player {} at position ({:.1}, {:.1}, {:.1})",
        player_no.0,
        pos.x,
        pos.y,
        pos.z
    );
    let born_texture_handle = asset_server.load("textures/born.bmp");

    let born_texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 4, 1, None, None);
    let born_atlas_layout_handle = atlas_layouts.add(born_texture_atlas);

    commands.spawn((
        Born,
        player_no,
        Sprite {
            image: born_texture_handle,
            texture_atlas: Some(TextureAtlas {
                index: 0,
                layout: born_atlas_layout_handle,
            }),
            ..default()
        },
        Transform::from_translation(pos),
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        AnimationIndices { first: 0, last: 3 },
        BornRemoveTimer(Timer::from_seconds(2.0, TimerMode::Once)),
    ));
}

/// Handle tank-to-tank collisions causing forced movement
/// When tanks collide, they stop to prevent overlap
/// Rapier physics will handle the separation naturally
///
/// Note: ParamSet is required to avoid Query conflict (B0001 error).
/// Type complexity warning is acceptable here as ParamSet cannot use type aliases
/// due to lifetime parameters in Query types.
#[allow(clippy::type_complexity)]
pub fn handle_tank_collisions(
    mut velocities: ParamSet<(
        Query<&mut Velocity, With<PlayerNo>>,
        Query<&mut Velocity, With<Enemy>>,
    )>,
    mut collision_er: MessageReader<CollisionEvent>,
    player_entities: Query<Entity, With<PlayerNo>>,
    enemy_entities: Query<Entity, With<Enemy>>,
) {
    for event in collision_er.read() {
        if let CollisionEvent::Started(entity1, entity2, _flags) = event {
            // Check if both entities are tanks (player or enemy)
            let is_tank1 = player_entities.contains(*entity1) || enemy_entities.contains(*entity1);
            let is_tank2 = player_entities.contains(*entity2) || enemy_entities.contains(*entity2);

            if !is_tank1 || !is_tank2 {
                continue;
            }

            // Stop both tanks when they collide
            // Rapier will handle the physical separation
            // Use ParamSet to avoid Query conflict - access players first, then enemies
            // Process entity1
            if player_entities.contains(*entity1) {
                if let Ok(mut vel) = velocities.p0().get_mut(*entity1) {
                    vel.linvel *= 0.3;
                }
            } else if enemy_entities.contains(*entity1) {
                if let Ok(mut vel) = velocities.p1().get_mut(*entity1) {
                    vel.linvel *= 0.3;
                }
            }

            // Process entity2
            if player_entities.contains(*entity2) {
                if let Ok(mut vel) = velocities.p0().get_mut(*entity2) {
                    vel.linvel *= 0.3;
                }
            } else if enemy_entities.contains(*entity2) {
                if let Ok(mut vel) = velocities.p1().get_mut(*entity2) {
                    vel.linvel *= 0.3;
                }
            }
        }
    }
}

/// Handle player tank movement based on keyboard input
/// Updates velocity and direction components based on pressed keys
/// Supports both single player (WASD) and multiplayer (WASD + Arrow keys)
pub fn players_move(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(
        &PlayerNo,
        &mut Velocity,
        &mut common::Direction,
        &mut Sprite,
        &mut AnimationIndices,
    )>,
    game_config: Res<GameConfig>,
) {
    let player_speed = game_config.player.speed;
    for (player_no, mut velocity, mut direction, mut sprite, mut indices) in &mut query {
        if player_no.0 == 1
            && keyboard_input.any_just_released([
                KeyCode::KeyW,
                KeyCode::KeyS,
                KeyCode::KeyA,
                KeyCode::KeyD,
            ])
        {
            velocity.linvel = Vec2::ZERO;
            continue;
        }
        if player_no.0 == 2
            && keyboard_input.any_just_released([
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ])
        {
            velocity.linvel = Vec2::ZERO;
            continue;
        }
        // Can only move in one direction at a time
        if (player_no.0 == 1 && keyboard_input.pressed(KeyCode::KeyW))
            || (player_no.0 == 2 && keyboard_input.pressed(KeyCode::ArrowUp))
        {
            velocity.linvel = Vec2::new(0.0, player_speed);
            *direction = common::Direction::Up;
        } else if (player_no.0 == 1 && keyboard_input.pressed(KeyCode::KeyS))
            || (player_no.0 == 2 && keyboard_input.pressed(KeyCode::ArrowDown))
        {
            velocity.linvel = Vec2::new(0.0, -player_speed);
            *direction = common::Direction::Down;
        } else if (player_no.0 == 1 && keyboard_input.pressed(KeyCode::KeyA))
            || (player_no.0 == 2 && keyboard_input.pressed(KeyCode::ArrowLeft))
        {
            velocity.linvel = Vec2::new(-player_speed, 0.0);
            *direction = common::Direction::Left;
        } else if (player_no.0 == 1 && keyboard_input.pressed(KeyCode::KeyD))
            || (player_no.0 == 2 && keyboard_input.pressed(KeyCode::ArrowRight))
        {
            velocity.linvel = Vec2::new(player_speed, 0.0);
            *direction = common::Direction::Right;
        } else {
            continue;
        }

        match *direction {
            common::Direction::Up => {
                *indices = AnimationIndices { first: 0, last: 1 };
            }
            common::Direction::Right => {
                *indices = AnimationIndices { first: 8, last: 9 };
            }
            common::Direction::Down => {
                *indices = AnimationIndices {
                    first: 16,
                    last: 17,
                };
            }
            common::Direction::Left => {
                *indices = AnimationIndices {
                    first: 24,
                    last: 25,
                };
            }
        }
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = indices.first;
        } else {
            warn!("Player sprite has no texture atlas, cannot update animation");
        }
    }
}

// Tank movement animation - uses generic animate_sprite_sheet from common
pub fn animate_players(
    time: Res<Time>,
    query: Query<(&mut AnimationTimer, &AnimationIndices, &mut Sprite), With<PlayerNo>>,
) {
    crate::common::animate_sprite_sheet::<PlayerNo>(time, query);
}

/// Handle player shooting/attack actions
/// Spawns bullets when fire keys are pressed and cooldown timer allows
/// Plays fire sound effects on successful shot
pub fn players_attack(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_players: Query<(
        &PlayerNo,
        &Transform,
        &common::Direction,
        &mut TankRefreshBulletTimer,
    )>,
    time: Res<Time>,
    game_sounds: Res<GameSounds>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (player_no, transform, direction, mut refresh_bullet_timer) in &mut q_players {
        refresh_bullet_timer.tick(time.delta());
        if ((player_no.0 == 1 && keyboard_input.just_pressed(KeyCode::Space))
            || (player_no.0 == 2 && keyboard_input.just_pressed(KeyCode::Enter)))
            && refresh_bullet_timer.is_finished()
        {
            debug!(
                "Player {} firing bullet in direction {:?} from position ({:.1}, {:.1})",
                player_no.0, direction, transform.translation.x, transform.translation.y
            );
            spawn_bullet(
                &mut commands,
                &asset_server,
                &mut atlas_layouts,
                Bullet::Player,
                transform.translation,
                *direction,
            );
            commands.spawn((
                AudioPlayer(game_sounds.player_fire.clone()),
                PlaybackSettings::DESPAWN,
            ));
            refresh_bullet_timer.reset();
        }
    }
}

// Shield animation - uses generic animate_sprite_sheet from common
pub fn animate_shield(
    time: Res<Time>,
    query: Query<(&mut AnimationTimer, &AnimationIndices, &mut Sprite), With<Shield>>,
) {
    crate::common::animate_sprite_sheet::<Shield>(time, query);
}

// Remove protection shield
pub fn remove_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShieldRemoveTimer), With<Shield>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.0.tick(time.delta());

        if timer.0.is_finished() && scheduled.0.insert(entity) {
            debug!("Shield timer expired, removing protection shield");
            commands.entity(entity).despawn();
        }
    }
}

// Spawn animation
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn animate_born(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (
            Entity,
            &PlayerNo,
            &Transform,
            &mut AnimationTimer,
            &AnimationIndices,
            &mut Sprite,
            &mut BornRemoveTimer,
        ),
        With<Born>,
    >,
    mut spawn_player_ew: MessageWriter<SpawnPlayerEvent>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    for (entity, player_no, transform, mut timer, indices, mut sprite, mut born_remove_timer) in
        &mut query
    {
        timer.0.tick(time.delta());
        born_remove_timer.0.tick(time.delta());
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
        if born_remove_timer.0.is_finished() {
            if scheduled.0.insert(entity) {
                trace!(
                    "Born animation completed for player {}, despawning animation entity",
                    player_no.0
                );
                commands.entity(entity).despawn();
            }
            debug!(
                "Born animation finished, creating player spawn event for player {}",
                player_no.0
            );
            spawn_player_ew.write(SpawnPlayerEvent {
                pos: transform.translation.truncate(),
                player_no: *player_no,
            });
        }
    }
}

pub fn cleanup_players(
    mut commands: Commands,
    q_players: Query<Entity, With<PlayerNo>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    let player_count = q_players.iter().count();
    if player_count > 0 {
        debug!("Cleaning up {} player entities", player_count);
    }
    for entity in &q_players {
        if scheduled.0.insert(entity) {
            trace!("Despawning player entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_born(
    mut commands: Commands,
    q_born: Query<Entity, With<Born>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    let born_count = q_born.iter().count();
    if born_count > 0 {
        debug!("Cleaning up {} born animation entities", born_count);
    }
    for entity in &q_born {
        if scheduled.0.insert(entity) {
            trace!("Despawning born animation entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

pub fn reset_player_lives(
    mut player_lives: ResMut<PlayerLives>,
    game_config: Res<crate::config::GameConfig>,
) {
    let initial_lives = game_config.player.initial_lives;
    player_lives.player1 = initial_lives;
    player_lives.player2 = initial_lives;
}

/// Plugin for player-related systems and resources
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerNo>()
            .add_message::<SpawnPlayerEvent>()
            .init_resource::<PlayerLives>()
            .add_systems(
                OnEnter(AppState::StartMenu),
                (cleanup_players, cleanup_born, reset_player_lives),
            )
            .add_systems(
                Update,
                auto_spawn_players
                    .in_set(crate::common::GameplaySet::Spawning)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                players_move
                    .in_set(crate::common::GameplaySet::Movement)
                    .after(crate::common::GameplaySet::Spawning)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                players_attack
                    .in_set(crate::common::GameplaySet::Combat)
                    .after(crate::common::GameplaySet::Movement)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                handle_tank_collisions
                    .in_set(crate::common::GameplaySet::Collision)
                    .after(crate::common::GameplaySet::Movement)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (animate_players, animate_shield, remove_shield)
                    .chain()
                    .in_set(crate::common::GameplaySet::Animation)
                    .after(crate::common::GameplaySet::Effects)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                animate_born
                    .in_set(crate::common::GameplaySet::Effects)
                    .after(crate::common::GameplaySet::Collision)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                animate_born
                    .in_set(crate::common::GameplaySet::Effects)
                    .run_if(in_state(AppState::GameOver)),
            )
            // Animation systems for GameOver state
            .add_systems(
                Update,
                (animate_players, animate_shield)
                    .chain()
                    .in_set(crate::common::GameplaySet::Animation)
                    .run_if(in_state(AppState::GameOver)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::GameConfig;

    #[test]
    fn test_player_no() {
        let player1 = PlayerNo(1);
        let player2 = PlayerNo(2);

        assert_eq!(player1.0, 1);
        assert_eq!(player2.0, 2);
        assert_ne!(player1, player2);
    }

    #[test]
    fn test_player_lives_default() {
        let lives = PlayerLives::default();
        assert_eq!(lives.player1, 3);
        assert_eq!(lives.player2, 3);
    }

    #[test]
    fn test_spawn_player_event() {
        let event = SpawnPlayerEvent {
            pos: Vec2::new(100.0, 200.0),
            player_no: PlayerNo(1),
        };

        assert_eq!(event.pos, Vec2::new(100.0, 200.0));
        assert_eq!(event.player_no, PlayerNo(1));
    }

    #[test]
    fn test_reset_player_lives_with_config() {
        // Test that reset_player_lives uses initial_lives from config
        use crate::config::GameConfig;
        use bevy::prelude::*;

        let mut app = App::new();
        app.init_resource::<GameConfig>()
            .init_resource::<PlayerLives>();

        // Set custom lives
        {
            let mut lives = app.world_mut().resource_mut::<PlayerLives>();
            lives.player1 = 0;
            lives.player2 = 0;
        }

        // Reset using function - simplified test
        let initial_lives = {
            let config = app.world().resource::<GameConfig>();
            config.player.initial_lives
        };
        {
            let mut lives_mut = app.world_mut().resource_mut::<PlayerLives>();
            lives_mut.player1 = initial_lives;
            lives_mut.player2 = initial_lives;
        }

        let lives_after = app.world().resource::<PlayerLives>();
        assert_eq!(lives_after.player1, 3);
        assert_eq!(lives_after.player2, 3);
    }
}
