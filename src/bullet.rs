use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::common::{AppState, Direction, GameSounds, *};
use crate::config::GameConfig;
use crate::enemy::Enemy;
use crate::level::{CollisionEvent, HomeDyingEvent, LevelItem};
use crate::player::{PlayerLives, PlayerNo, Shield};

/// Bullet type component
/// Identifies whether a bullet belongs to a player or enemy
#[derive(Component, PartialEq, Eq, Debug)]
pub enum Bullet {
    /// Player bullet
    Player,
    /// Enemy bullet
    Enemy,
}

/// Component marking an explosion entity
#[derive(Debug, Component)]
pub struct Explosion;

/// Event emitted when an explosion should be spawned
#[derive(Debug, Message)]
pub struct ExplosionEvent {
    /// Position where the explosion should occur
    pub pos: Vec3,
    /// Type of explosion to spawn
    pub explosion_type: ExplosionType,
}

/// Type of explosion effect
#[derive(Debug, PartialEq, Eq)]
pub enum ExplosionType {
    /// Large explosion (for tanks, home base)
    BigExplosion,
    /// Small explosion (for bullets hitting walls)
    BulletExplosion,
}

/// Resource containing explosion texture assets
#[derive(Debug, Resource)]
pub struct ExplosionAssets {
    /// Handles for big explosion animation frames
    pub big_explosion: Vec<Handle<Image>>,
    /// Handles for bullet explosion animation frames
    pub bullet_explosion: Vec<Handle<Image>>,
}

pub fn setup_explosion_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let big_explosion: Vec<Handle<Image>> = vec![
        asset_server.load("textures/big_explosion_1.png"),
        asset_server.load("textures/big_explosion_2.png"),
        asset_server.load("textures/big_explosion_3.png"),
        asset_server.load("textures/big_explosion_4.png"),
        asset_server.load("textures/big_explosion_5.png"),
    ];

    let bullet_explosion: Vec<Handle<Image>> = vec![
        asset_server.load("textures/bullet_explosion_1.png"),
        asset_server.load("textures/bullet_explosion_2.png"),
        asset_server.load("textures/bullet_explosion_3.png"),
    ];

    commands.insert_resource(ExplosionAssets {
        big_explosion,
        bullet_explosion,
    });
}

/// Update bullet positions based on their direction and speed
/// Moves bullets in a straight line until they hit something or leave the map
pub fn move_bullet(
    mut q_bullet: Query<(&mut Transform, &Direction), With<Bullet>>,
    time: Res<Time>,
    game_config: Res<GameConfig>,
) {
    let bullet_speed = game_config.bullet.speed;
    for (mut bullet_transform, direction) in &mut q_bullet {
        match direction {
            Direction::Left => bullet_transform.translation.x -= bullet_speed * time.delta_secs(),
            Direction::Right => bullet_transform.translation.x += bullet_speed * time.delta_secs(),
            Direction::Up => bullet_transform.translation.y += bullet_speed * time.delta_secs(),
            Direction::Down => bullet_transform.translation.y -= bullet_speed * time.delta_secs(),
        }
    }
}

/// Handle bullet collision events
/// Processes collisions between bullets and other entities (walls, tanks, base)
/// Triggers appropriate responses: destroy wall, damage tank, game over, etc.
///
/// # Collision Types Handled
/// - Bullet vs Level Items (walls, home)
/// - Bullet vs Area Walls (boundaries)
/// - Player Bullet vs Enemy
/// - Enemy Bullet vs Player (with shield protection)
#[allow(clippy::too_many_arguments)]
pub fn handle_bullet_collision(
    mut commands: Commands,
    q_bullets: Query<(Entity, &Bullet, &Transform)>,
    q_level_items: Query<(&LevelItem, &GlobalTransform, &mut Sprite)>,
    q_area_wall: Query<(), With<AreaWall>>,
    q_players: Query<(Entity, &PlayerNo, &Transform, &Children), With<PlayerNo>>,
    q_shields: Query<Entity, With<Shield>>,
    q_enemies: Query<&Transform, With<Enemy>>,
    mut collision_er: MessageReader<CollisionEvent>,
    mut explosion_ew: MessageWriter<ExplosionEvent>,
    mut home_dying_ew: MessageWriter<HomeDyingEvent>,
    mut player_lives: ResMut<PlayerLives>,
    multiplayer_mode: Res<MultiplayerMode>,
    mut app_state: ResMut<NextState<AppState>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    for event in collision_er.read() {
        match event {
            CollisionEvent::Started(entity1, entity2, _flags)
            | CollisionEvent::Stopped(entity1, entity2, _flags) => {
                let bullet_entity = if q_bullets.contains(*entity1) {
                    *entity1
                } else if q_bullets.contains(*entity2) {
                    *entity2
                } else {
                    continue;
                };
                let other_entity = if bullet_entity == *entity1 {
                    *entity2
                } else {
                    *entity1
                };

                let Ok((_, bullet, bullet_transform)) = q_bullets.get(bullet_entity) else {
                    continue;
                };
                // Other object
                if q_level_items.contains(other_entity) {
                    let Ok((level_item, level_item_transform, _)) = q_level_items.get(other_entity)
                    else {
                        continue;
                    };
                    match level_item {
                        LevelItem::Home => {
                            // Game Over
                            if scheduled.0.insert(bullet_entity) {
                                commands.entity(bullet_entity).despawn();
                            }
                            explosion_ew.write(ExplosionEvent {
                                pos: Vec3::new(
                                    level_item_transform.translation().x,
                                    level_item_transform.translation().y,
                                    level_item_transform.translation().z,
                                ),
                                explosion_type: ExplosionType::BigExplosion,
                            });
                            home_dying_ew.write_default();
                        }
                        LevelItem::StoneWall => {
                            if scheduled.0.insert(bullet_entity) {
                                commands.entity(bullet_entity).despawn();
                            }
                            if scheduled.0.insert(other_entity) {
                                commands.entity(other_entity).despawn();
                            }
                            explosion_ew.write(ExplosionEvent {
                                pos: Vec3::new(
                                    bullet_transform.translation.x,
                                    bullet_transform.translation.y,
                                    bullet_transform.translation.z,
                                ),
                                explosion_type: ExplosionType::BulletExplosion,
                            });
                        }
                        LevelItem::Tree => {
                            // Trees provide cover - bullets stop when hitting trees
                            if scheduled.0.insert(bullet_entity) {
                                commands.entity(bullet_entity).despawn();
                            }
                            explosion_ew.write(ExplosionEvent {
                                pos: Vec3::new(
                                    bullet_transform.translation.x,
                                    bullet_transform.translation.y,
                                    bullet_transform.translation.z,
                                ),
                                explosion_type: ExplosionType::BulletExplosion,
                            });
                        }
                        LevelItem::IronWall => {
                            if scheduled.0.insert(bullet_entity) {
                                commands.entity(bullet_entity).despawn();
                            }
                            explosion_ew.write(ExplosionEvent {
                                pos: Vec3::new(
                                    bullet_transform.translation.x,
                                    bullet_transform.translation.y,
                                    bullet_transform.translation.z,
                                ),
                                explosion_type: ExplosionType::BulletExplosion,
                            });
                        }
                        _ => {}
                    }
                }

                if q_area_wall.contains(other_entity) {
                    if scheduled.0.insert(bullet_entity) {
                        commands.entity(bullet_entity).despawn();
                    }
                    explosion_ew.write(ExplosionEvent {
                        pos: Vec3::new(
                            bullet_transform.translation.x,
                            bullet_transform.translation.y,
                            bullet_transform.translation.z,
                        ),
                        explosion_type: ExplosionType::BulletExplosion,
                    });
                }

                if *bullet == Bullet::Player && q_enemies.contains(other_entity) {
                    let Ok(enemy_transform) = q_enemies.get(other_entity) else {
                        continue;
                    };
                    if scheduled.0.insert(bullet_entity) {
                        commands.entity(bullet_entity).despawn();
                    }
                    if scheduled.0.insert(other_entity) {
                        commands.entity(other_entity).despawn();
                    }
                    explosion_ew.write(ExplosionEvent {
                        pos: Vec3::new(
                            enemy_transform.translation.x,
                            enemy_transform.translation.y,
                            enemy_transform.translation.z,
                        ),
                        explosion_type: ExplosionType::BigExplosion,
                    });
                }

                // Check if enemy bullet hit player (directly or through shield)
                if *bullet == Bullet::Enemy {
                    // Find player that was hit (either directly or through shield child)
                    if let Some((player_entity, player_no, player_transform, player_children)) =
                        q_players.iter().find_map(
                            |(player_entity, player_no, player_transform, player_children)| {
                                // Check if bullet hit player directly or through shield (child entity)
                                if player_entity == other_entity
                                    || player_children.contains(&other_entity)
                                {
                                    warn!("SOME");
                                    Some((
                                        player_entity,
                                        player_no,
                                        player_transform,
                                        player_children,
                                    ))
                                } else {
                                    warn!("NONE");
                                    None
                                }
                            },
                        )
                    {
                        let mut player_has_shield = false;
                        for child in player_children.iter() {
                            if q_shields.contains(child) {
                                player_has_shield = true;
                                break;
                            }
                        }

                        if scheduled.0.insert(bullet_entity) {
                            commands.entity(bullet_entity).despawn();
                        }

                        if player_has_shield {
                            explosion_ew.write(ExplosionEvent {
                                pos: Vec3::new(
                                    player_transform.translation.x,
                                    player_transform.translation.y,
                                    player_transform.translation.z,
                                ),
                                explosion_type: ExplosionType::BulletExplosion,
                            });
                        } else {
                            // Decrease player lives
                            if player_no.0 == 1 {
                                player_lives.player1 -= 1;
                            } else if player_no.0 == 2 {
                                player_lives.player2 -= 1;
                            }
                            warn!(
                                "Enemy bullet hit player {} at position ({:.1}, {:.1}), destroying player. Lives remaining: P1={}, P2={}",
                                player_no.0,
                                player_transform.translation.x, player_transform.translation.y,
                                player_lives.player1, player_lives.player2
                            );
                            if scheduled.0.insert(player_entity) {
                                commands.entity(player_entity).despawn();
                            }
                            explosion_ew.write(ExplosionEvent {
                                pos: Vec3::new(
                                    player_transform.translation.x,
                                    player_transform.translation.y,
                                    player_transform.translation.z,
                                ),
                                explosion_type: ExplosionType::BigExplosion,
                            });
                            if player_lives.player1 <= 0 && player_lives.player2 <= 0 {
                                app_state.set(AppState::GameOver);
                            }
                            if player_lives.player1 <= 0
                                && *multiplayer_mode == MultiplayerMode::SinglePlayer
                            {
                                app_state.set(AppState::GameOver);
                            }
                        }
                        // Skip other collision checks for this bullet
                        continue;
                    }
                }
            }
        }
    }
}

pub fn spawn_bullet(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    bullet: Bullet,
    translation: Vec3,
    direction: Direction,
) {
    let bullet_texture_handle = asset_server.load("textures/bullet.bmp");
    let bullet_texture_layout = TextureAtlasLayout::from_grid(UVec2::new(7, 8), 4, 1, None, None);
    commands.spawn((
        bullet,
        direction,
        Sprite {
            image: bullet_texture_handle.clone(),
            texture_atlas: Some(TextureAtlas {
                index: match direction {
                    Direction::Up => 0,
                    Direction::Right => 1,
                    Direction::Down => 2,
                    Direction::Left => 3,
                },
                layout: atlas_layouts.add(bullet_texture_layout),
            }),
            ..default()
        },
        Transform {
            translation: Vec3::new(translation.x, translation.y, translation.z),
            ..default()
        },
        Collider::cuboid(2.0, 2.0),
        Sensor,
        RigidBody::Dynamic,
        ActiveEvents::COLLISION_EVENTS,
    ));
}

/// Spawn explosion entities based on explosion events
/// Creates animated explosion sprites at specified positions
/// Plays appropriate sound effects based on explosion type
pub fn spawn_explosion(
    mut commands: Commands,
    mut explosion_er: MessageReader<ExplosionEvent>,
    explosion_assets: Res<ExplosionAssets>,
    mut textures: ResMut<Assets<Image>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    game_sounds: Res<GameSounds>,
) {
    let mut big_explosion_texture_atlas_builder = TextureAtlasBuilder::default();
    for handle in &explosion_assets.big_explosion {
        let Some(texture) = textures.get(handle.id()) else {
            continue;
        };
        big_explosion_texture_atlas_builder.add_texture(Some(handle.id()), texture);
    }
    let big_explosion_texture_atlas = match big_explosion_texture_atlas_builder.build() {
        Ok(atlas) => atlas,
        Err(_e) => {
            return;
        }
    };
    let big_explosion_atlas_layout_handle = atlas_layouts.add(big_explosion_texture_atlas.0);
    let big_explosion_texture_handle = textures.add(big_explosion_texture_atlas.2);

    let mut bullet_explosion_texture_atlas_builder = TextureAtlasBuilder::default();
    for handle in &explosion_assets.bullet_explosion {
        let Some(texture) = textures.get(handle.id()) else {
            continue;
        };
        bullet_explosion_texture_atlas_builder.add_texture(Some(handle.id()), texture);
    }
    let bullet_explosion_texture_atlas = match bullet_explosion_texture_atlas_builder.build() {
        Ok(atlas) => atlas,
        Err(_e) => {
            return;
        }
    };
    let bullet_explosion_atlas_layout_handle = atlas_layouts.add(bullet_explosion_texture_atlas.0);
    let bullet_explosion_texture_handle = textures.add(bullet_explosion_texture_atlas.2);

    for explosion in explosion_er.read() {
        commands.spawn((
            Explosion,
            Sprite {
                image: if explosion.explosion_type == ExplosionType::BigExplosion {
                    big_explosion_texture_handle.clone()
                } else {
                    bullet_explosion_texture_handle.clone()
                },
                texture_atlas: Some(TextureAtlas {
                    layout: if explosion.explosion_type == ExplosionType::BigExplosion {
                        big_explosion_atlas_layout_handle.clone()
                    } else {
                        bullet_explosion_atlas_layout_handle.clone()
                    },
                    index: 0,
                }),
                ..default()
            },
            Transform::from_translation(explosion.pos),
            AnimationTimer(Timer::from_seconds(0.05, TimerMode::Repeating)),
            AnimationIndices {
                first: 0,
                last: if explosion.explosion_type == ExplosionType::BigExplosion {
                    4
                } else {
                    2
                },
            },
        ));
        if explosion.explosion_type == ExplosionType::BigExplosion {
            commands.spawn((
                AudioPlayer(game_sounds.big_explosion.clone()),
                PlaybackSettings::DESPAWN,
            ));
        } else if explosion.explosion_type == ExplosionType::BulletExplosion {
            commands.spawn((
                AudioPlayer(game_sounds.bullet_explosion.clone()),
                PlaybackSettings::DESPAWN,
            ));
        }
    }
}

pub fn animate_explosion(
    mut commands: Commands,
    mut q_explosion: Query<
        (Entity, &mut AnimationTimer, &AnimationIndices, &mut Sprite),
        With<Explosion>,
    >,
    time: Res<Time>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    for (entity, mut timer, indices, mut sprite) in &mut q_explosion {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                atlas.index += 1;
                if atlas.index > indices.last && scheduled.0.insert(entity) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

pub fn cleanup_bullets(
    mut commands: Commands,
    q_bullets: Query<Entity, With<Bullet>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    for entity in &q_bullets {
        if scheduled.0.insert(entity) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_explosions(
    mut commands: Commands,
    q_explosions: Query<Entity, With<Explosion>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    for entity in &q_explosions {
        if scheduled.0.insert(entity) {
            commands.entity(entity).despawn();
        }
    }
}

/// Plugin for bullet-related systems and resources
pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ExplosionEvent>()
            .add_systems(
                OnEnter(AppState::StartMenu),
                (cleanup_bullets, cleanup_explosions),
            )
            .add_systems(
                Update,
                move_bullet
                    .in_set(crate::common::GameplaySet::BulletMovement)
                    .after(crate::common::GameplaySet::Combat)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                handle_bullet_collision
                    .in_set(crate::common::GameplaySet::Collision)
                    .after(crate::common::GameplaySet::BulletMovement)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                spawn_explosion
                    .in_set(crate::common::GameplaySet::Effects)
                    .after(crate::common::GameplaySet::Collision)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                spawn_explosion
                    .in_set(crate::common::GameplaySet::Effects)
                    .run_if(in_state(AppState::GameOver)),
            )
            .add_systems(
                Update,
                animate_explosion
                    .in_set(crate::common::GameplaySet::Animation)
                    .after(crate::common::GameplaySet::Effects)
                    .run_if(in_state(AppState::Playing)),
            )
            // Animation systems for GameOver state
            .add_systems(
                Update,
                animate_explosion
                    .in_set(crate::common::GameplaySet::Animation)
                    .run_if(in_state(AppState::GameOver)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bullet_variants() {
        assert_eq!(Bullet::Player, Bullet::Player);
        assert_eq!(Bullet::Enemy, Bullet::Enemy);
        assert_ne!(Bullet::Player, Bullet::Enemy);
    }

    #[test]
    fn test_explosion_type_variants() {
        assert_eq!(ExplosionType::BigExplosion, ExplosionType::BigExplosion);
        assert_eq!(
            ExplosionType::BulletExplosion,
            ExplosionType::BulletExplosion
        );
        assert_ne!(ExplosionType::BigExplosion, ExplosionType::BulletExplosion);
    }

    #[test]
    fn test_explosion_event() {
        let event = ExplosionEvent {
            pos: Vec3::new(100.0, 200.0, 0.0),
            explosion_type: ExplosionType::BigExplosion,
        };

        assert_eq!(event.pos, Vec3::new(100.0, 200.0, 0.0));
        assert_eq!(event.explosion_type, ExplosionType::BigExplosion);
    }
}
