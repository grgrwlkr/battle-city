use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::area::*;
use crate::common::{Direction, *};
use crate::enemy::Enemy;
use crate::level::LevelItem;
use crate::player::{PlayerLives, PlayerNo, Shield};

pub const BULLET_SPEED: f32 = 300.0;

#[derive(Component, PartialEq, Eq, Debug)]
pub enum Bullet {
    Player,
    Enemy,
}

#[derive(Debug, Component)]
pub struct Explosion;

#[derive(Debug, Message)]
pub struct ExplosionEvent {
    pos: Vec3,
    explosion_type: ExplosionType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ExplosionType {
    BigExplosion,
    BulletExplosion,
}

#[derive(Debug, Resource)]
pub struct ExplosionAssets {
    pub big_explosion: Vec<Handle<Image>>,
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

// Bullet movement
pub fn move_bullet(
    mut q_bullet: Query<(&mut Transform, &Direction), With<Bullet>>,
    time: Res<Time>,
) {
    for (mut bullet_transform, direction) in &mut q_bullet {
        match direction {
            Direction::Left => bullet_transform.translation.x -= BULLET_SPEED * time.delta_secs(),
            Direction::Right => bullet_transform.translation.x += BULLET_SPEED * time.delta_secs(),
            Direction::Up => bullet_transform.translation.y += BULLET_SPEED * time.delta_secs(),
            Direction::Down => bullet_transform.translation.y -= BULLET_SPEED * time.delta_secs(),
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn handle_bullet_collision(
    mut commands: Commands,
    q_bullets: Query<(Entity, &Bullet, &Transform)>,
    q_level_items: Query<(&LevelItem, &GlobalTransform, &mut Sprite)>,
    q_area_wall: Query<(), With<AreaWall>>,
    q_players: Query<(&Transform, &Children), With<PlayerNo>>,
    q_shields: Query<Entity, With<Shield>>,
    q_enemies: Query<&Transform, With<Enemy>>,
    mut collision_er: MessageReader<CollisionEvent>,
    mut explosion_ew: MessageWriter<ExplosionEvent>,
    mut home_dying_ew: MessageWriter<HomeDyingEvent>,
    player_lives: Res<PlayerLives>,
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

                debug!(
                    "Bullet collision: bullet={:?}, entity1={:?}, entity2={:?}",
                    bullet_entity, entity1, entity2
                );

                let (_, bullet, bullet_transform) = q_bullets.get(bullet_entity).unwrap();

                info!(
                    "{:?} bullet hit entity {:?} at position ({:.1}, {:.1})",
                    bullet, other_entity, bullet_transform.translation.x, bullet_transform.translation.y
                );
                // Other object
                if q_level_items.contains(other_entity) {
                    let (level_item, level_item_transform, _) =
                        q_level_items.get(other_entity).unwrap();
                    info!("Bullet hit level item: {:?} at position ({:.1}, {:.1})", 
                          level_item, level_item_transform.translation().x, level_item_transform.translation().y);
                    match level_item {
                        LevelItem::Home => {
                            // Game Over
                            error!("Game over: Home destroyed by bullet at position ({:.1}, {:.1})", 
                                   level_item_transform.translation().x, level_item_transform.translation().y);
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
                            info!("Stone wall destroyed at position ({:.1}, {:.1})", 
                                  bullet_transform.translation.x, bullet_transform.translation.y);
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
                        LevelItem::IronWall => {
                            debug!("Bullet bounced off iron wall at position ({:.1}, {:.1})", 
                                   bullet_transform.translation.x, bullet_transform.translation.y);
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
                    debug!("Bullet hit area boundary wall at position ({:.1}, {:.1}), destroying bullet", 
                           bullet_transform.translation.x, bullet_transform.translation.y);
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
                    let enemy_transform = q_enemies.get(other_entity).unwrap();
                    info!(
                        "Player bullet destroyed enemy at position ({:.1}, {:.1})",
                        enemy_transform.translation.x, enemy_transform.translation.y
                    );
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

                if *bullet == Bullet::Enemy && q_players.contains(other_entity) {
                    let (player_transform, player_children) = q_players.get(other_entity).unwrap();
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
                        info!(
                            "Enemy bullet blocked by player shield at position ({:.1}, {:.1})",
                            player_transform.translation.x, player_transform.translation.y
                        );
                        explosion_ew.write(ExplosionEvent {
                            pos: Vec3::new(
                                player_transform.translation.x,
                                player_transform.translation.y,
                                player_transform.translation.z,
                            ),
                            explosion_type: ExplosionType::BulletExplosion,
                        });
                    } else {
                        warn!(
                            "Enemy bullet hit player at position ({:.1}, {:.1}), destroying player. Lives remaining: P1={}, P2={}",
                            player_transform.translation.x, player_transform.translation.y,
                            player_lives.player1, player_lives.player2
                        );
                        if scheduled.0.insert(other_entity) {
                            commands.entity(other_entity).despawn();
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
                            error!("Game over: All players have no lives remaining");
                            app_state.set(AppState::GameOver);
                        }
                        if player_lives.player1 <= 0
                            && *multiplayer_mode == MultiplayerMode::SinglePlayer
                        {
                            error!("Game over: Single player has no lives remaining");
                            app_state.set(AppState::GameOver);
                        }
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

pub fn spawn_explosion(
    mut commands: Commands,
    mut explosion_er: MessageReader<ExplosionEvent>,
    explosion_assets: Res<ExplosionAssets>,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<Image>>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    game_sounds: Res<GameSounds>,
) {
    let mut big_explosion_texture_atlas_builder = TextureAtlasBuilder::default();
    for handle in &explosion_assets.big_explosion {
        let Some(texture) = textures.get(handle.id()) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                asset_server.get_path(handle.id())
            );
            continue;
        };
        big_explosion_texture_atlas_builder.add_texture(Some(handle.id()), texture);
    }
    let big_explosion_texture_atlas = big_explosion_texture_atlas_builder.build().unwrap();
    let big_explosion_atlas_layout_handle = atlas_layouts.add(big_explosion_texture_atlas.0);
    let big_explosion_texture_handle = textures.add(big_explosion_texture_atlas.2);

    let mut bullet_explosion_texture_atlas_builder = TextureAtlasBuilder::default();
    for handle in &explosion_assets.bullet_explosion {
        let Some(texture) = textures.get(handle.id()) else {
            warn!(
                "{:?} did not resolve to an `Image` asset.",
                asset_server.get_path(handle.id())
            );
            continue;
        };
        bullet_explosion_texture_atlas_builder.add_texture(Some(handle.id()), texture);
    }
    let bullet_explosion_texture_atlas = bullet_explosion_texture_atlas_builder.build().unwrap();
    let bullet_explosion_atlas_layout_handle = atlas_layouts.add(bullet_explosion_texture_atlas.0);
    let bullet_explosion_texture_handle = textures.add(bullet_explosion_texture_atlas.2);

    for explosion in explosion_er.read() {
        trace!(
            "Spawning {:?} explosion at position ({:.1}, {:.1}, {:.1})",
            explosion.explosion_type, explosion.pos.x, explosion.pos.y, explosion.pos.z
        );
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
                    trace!("Explosion animation completed, despawning explosion entity {:?}", entity);
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
    let bullet_count = q_bullets.iter().count();
    if bullet_count > 0 {
        debug!("Cleaning up {} bullet entities", bullet_count);
    }
    for entity in &q_bullets {
        if scheduled.0.insert(entity) {
            trace!("Despawning bullet entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_explosions(
    mut commands: Commands,
    q_explosions: Query<Entity, With<Explosion>>,
    mut scheduled: ResMut<ScheduledDespawn>,
) {
    let explosion_count = q_explosions.iter().count();
    if explosion_count > 0 {
        debug!("Cleaning up {} explosion entities", explosion_count);
    }
    for entity in &q_explosions {
        if scheduled.0.insert(entity) {
            trace!("Despawning explosion entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}
