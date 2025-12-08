use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::Rng;

use crate::{
    bullet::{spawn_bullet, Bullet},
    common::{self, AnimationIndices, AnimationTimer, AppState, TankRefreshBulletTimer, TILE_SIZE},
    config::GameConfig,
    level::{EnemiesMarker, LevelItem},
    player::PlayerNo,
};

// Number of enemies spawned in the current level
#[derive(Resource, Default)]
pub struct LevelSpawnedEnemies(pub i32);

#[derive(Component)]
pub struct Enemy;

// Direction change timer
#[derive(Component)]
pub struct EnemyChangeDirectionTimer(pub Timer);

/// Automatically spawn enemies when conditions are met
/// Checks enemy limits (max alive, per level) and spawns at random marker positions
/// Ensures enemies don't spawn too close to existing tanks
#[allow(clippy::too_many_arguments)]
pub fn auto_spawn_enemies(
    mut commands: Commands,
    mut level_spawned_enemies: ResMut<LevelSpawnedEnemies>,
    q_enemies: Query<&Transform, With<Enemy>>,
    q_enemies_marker: Query<&GlobalTransform, With<EnemiesMarker>>,
    q_players: Query<&Transform, With<PlayerNo>>,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    game_config: Res<GameConfig>,
) {
    let max_live_enemies = game_config.enemy.max_live_enemies;
    let enemies_per_level = game_config.enemy.enemies_per_level;
    let current_enemy_count = q_enemies.iter().len();
    if current_enemy_count >= max_live_enemies as usize {
        // Maximum number of alive enemies on the battlefield has been reached
        trace!(
            "Cannot spawn enemy: maximum alive enemies reached ({}/{})",
            current_enemy_count,
            max_live_enemies
        );
        return;
    }
    if level_spawned_enemies.0 >= enemies_per_level {
        // Maximum number of enemies spawned for this level has been reached
        trace!(
            "Cannot spawn enemy: level enemy limit reached ({}/{})",
            level_spawned_enemies.0,
            enemies_per_level
        );
        return;
    }
    let mut marker_positions = Vec::new();
    for enemy_marker in &q_enemies_marker {
        // Prevent enemy_marker from being used before initialization
        if enemy_marker.translation() == Vec3::ZERO {
            continue;
        }
        marker_positions.push(*enemy_marker);
    }

    if !marker_positions.is_empty() {
        // Random location
        let mut rng = rand::thread_rng();
        let idx = rng.gen_range(0..marker_positions.len());
        let choosed_pos = marker_positions[idx].translation();

        // Cannot be too close to tanks on the battlefield
        for enemy_pos in &q_enemies {
            if choosed_pos.distance(enemy_pos.translation) < 2. * TILE_SIZE {
                trace!(
                    "Enemy spawn cancelled: too close to existing enemy at ({:.1}, {:.1})",
                    enemy_pos.translation.x,
                    enemy_pos.translation.y
                );
                return;
            }
        }
        for player_pos in &q_players {
            if choosed_pos.distance(player_pos.translation) < 2. * TILE_SIZE {
                trace!(
                    "Enemy spawn cancelled: too close to player at ({:.1}, {:.1})",
                    player_pos.translation.x,
                    player_pos.translation.y
                );
                return;
            }
        }
        info!(
            "Spawning enemy at position ({:.1}, {:.1}), level progress: {}/{}, alive enemies: {}/{}",
            choosed_pos.x, choosed_pos.y, level_spawned_enemies.0 + 1, enemies_per_level,
            current_enemy_count + 1, max_live_enemies
        );
        spawn_enemy(
            choosed_pos,
            &mut commands,
            &asset_server,
            &mut atlas_layouts,
            &game_config,
        );
        level_spawned_enemies.0 += 1;
    }
}

/// Spawn a single enemy entity at the specified position
/// Creates enemy with random sprite variant, physics, and AI components
///
/// # Arguments
/// * `pos` - Position where the enemy should spawn
/// * `commands` - Commands to spawn entities
/// * `asset_server` - Asset server for loading textures
/// * `atlas_layouts` - Texture atlas layouts resource
/// * `game_config` - Game configuration for enemy properties
pub fn spawn_enemy(
    pos: Vec3,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    atlas_layouts: &mut ResMut<Assets<TextureAtlasLayout>>,
    game_config: &GameConfig,
) {
    let tank_size = game_config.enemy.tank_size;
    let enemies_texture_handle = asset_server.load("textures/enemies.bmp");
    let enemies_texture_atlas =
        TextureAtlasLayout::from_grid(UVec2::new(tank_size, tank_size), 8, 8, None, None);
    let enemies_atlas_layout_handle = atlas_layouts.add(enemies_texture_atlas);

    // Random color
    let indexes: Vec<i32> = enemies_sprite_index_sets().iter().map(|v| v[0]).collect();
    let mut rng = rand::thread_rng();
    let choosed_index = indexes[rng.gen_range(0..indexes.len())];
    trace!("Enemy spawned with sprite index {}", choosed_index);

    commands.spawn((
        Enemy,
        Sprite {
            image: enemies_texture_handle,
            texture_atlas: Some(TextureAtlas {
                layout: enemies_atlas_layout_handle,
                index: choosed_index as usize,
            }),
            ..default()
        },
        Transform {
            translation: pos,
            scale: Vec3::splat(game_config.enemy.tank_scale),
            ..default()
        },
        TankRefreshBulletTimer(Timer::from_seconds(
            game_config.enemy.bullet_cooldown,
            TimerMode::Repeating,
        )),
        EnemyChangeDirectionTimer(Timer::from_seconds(1.0, TimerMode::Once)),
        AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
        AnimationIndices {
            first: choosed_index as usize,
            last: choosed_index as usize + 1,
        },
        common::Direction::Up,
        RigidBody::Dynamic,
        Collider::cuboid(
            tank_size as f32 * game_config.enemy.tank_scale / 2.0,
            tank_size as f32 * game_config.enemy.tank_scale / 2.0,
        ),
        ActiveEvents::COLLISION_EVENTS,
        LockedAxes::ROTATION_LOCKED,
    ));
}

/// Handle enemy movement and AI pathfinding
/// Enemies move in their current direction until hitting obstacles
/// When timer expires or collision occurs, enemies choose a new random direction
/// Implements obstacle avoidance for level items
///
/// # TODO
/// - Actively attack when player is detected
/// - Trees can provide cover
pub fn enemies_move(
    mut q_enemies: Query<
        (
            &mut Transform,
            &mut common::Direction,
            &mut Sprite,
            &mut AnimationIndices,
            &mut EnemyChangeDirectionTimer,
        ),
        With<Enemy>,
    >,
    q_level_items: Query<(&LevelItem, &GlobalTransform)>,
    time: Res<Time>,
    game_config: Res<GameConfig>,
) {
    let enemy_speed = game_config.enemy.speed;
    for (mut transform, mut direction, mut sprite, mut indices, mut timer) in &mut q_enemies {
        timer.0.tick(time.delta());
        if !timer.0.is_finished() {
            match *direction {
                common::Direction::Up => {
                    transform.translation.y += enemy_speed * time.delta_secs();
                }
                common::Direction::Right => {
                    transform.translation.x += enemy_speed * time.delta_secs();
                }
                common::Direction::Down => {
                    transform.translation.y -= enemy_speed * time.delta_secs();
                }
                common::Direction::Left => {
                    transform.translation.x -= enemy_speed * time.delta_secs();
                }
            }
            continue;
        }

        // Choose a new direction
        let mut can_left = true;
        let mut can_right = true;
        let mut can_up = true;
        let mut can_down = true;

        // Current available paths
        let tank_size = game_config.enemy.tank_size as f32;
        for (level_item, level_item_transform) in &q_level_items {
            if *level_item == LevelItem::Tree {
                continue;
            }
            if (level_item_transform.translation().x - transform.translation.x).abs()
                < (tank_size + TILE_SIZE) / 2.0 - 5.0
            {
                if level_item_transform.translation().y > transform.translation.y
                    && level_item_transform.translation().y - transform.translation.y < TILE_SIZE
                {
                    can_up = false;
                }
                if level_item_transform.translation().y < transform.translation.y
                    && transform.translation.y - level_item_transform.translation().y < TILE_SIZE
                {
                    can_down = false;
                }
            }
            if (level_item_transform.translation().y - transform.translation.y).abs()
                < (tank_size + TILE_SIZE) / 2. - 5.0
            {
                if level_item_transform.translation().x > transform.translation.x
                    && level_item_transform.translation().x - transform.translation.x < TILE_SIZE
                {
                    can_right = false;
                }
                if level_item_transform.translation().x < transform.translation.x
                    && transform.translation.x - level_item_transform.translation().x < TILE_SIZE
                {
                    can_left = false;
                }
            }
        }
        if !can_left && !can_right && !can_up && !can_down {
            continue;
        }

        // Randomly choose a direction based on weights
        let mut rng = rand::thread_rng();
        let choosed_direction = loop {
            let rand = rng.gen_range(0..9);
            match rand {
                0 => {
                    if can_up {
                        break common::Direction::Up;
                    }
                }
                1 | 2 => {
                    if can_left {
                        break common::Direction::Left;
                    }
                }
                3 | 4 => {
                    if can_right {
                        break common::Direction::Right;
                    }
                }
                5..=8 => {
                    if can_down {
                        break common::Direction::Down;
                    }
                }
                _ => {}
            }
        };

        // Set direction and sprite
        *direction = choosed_direction;
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = new_sprite_index(atlas.index as i32, *direction) as usize;
            *indices = AnimationIndices {
                first: atlas.index,
                last: atlas.index + 1,
            };
        }

        // Reset direction change timer
        timer.0.reset();
        trace!("Enemy changed direction to {:?}", choosed_direction);
    }
}

/// Handle enemy shooting behavior
/// Enemies automatically fire bullets based on their cooldown timer
/// Bullets are spawned in the direction the enemy is facing
pub fn enemies_attack(
    mut q_players: Query<
        (&Transform, &common::Direction, &mut TankRefreshBulletTimer),
        With<Enemy>,
    >,
    time: Res<Time>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (transform, direction, mut refresh_bullet_timer) in &mut q_players {
        refresh_bullet_timer.tick(time.delta());
        if refresh_bullet_timer.just_finished() {
            debug!(
                "Enemy firing bullet in direction {:?} from position ({:.1}, {:.1})",
                direction, transform.translation.x, transform.translation.y
            );
            spawn_bullet(
                &mut commands,
                &asset_server,
                &mut atlas_layouts,
                Bullet::Enemy,
                transform.translation,
                *direction,
            );
        }
    }
}

pub fn handle_enemy_collision(
    mut q_enemies: Query<&mut EnemyChangeDirectionTimer, With<Enemy>>,
    mut collision_er: MessageReader<CollisionEvent>,
) {
    for event in collision_er.read() {
        match event {
            CollisionEvent::Started(entity1, entity2, _flags)
            | CollisionEvent::Stopped(entity1, entity2, _flags) => {
                let enemy_entity = if q_enemies.contains(*entity1) {
                    *entity1
                } else if q_enemies.contains(*entity2) {
                    *entity2
                } else {
                    continue;
                };

                // Reset direction change timer
                trace!("Enemy collision detected, resetting direction change timer for enemy entity {:?}", enemy_entity);
                if let Ok(mut change_direction_timer) = q_enemies.get_mut(enemy_entity) {
                    change_direction_timer.0.reset();
                } else {
                    warn!(
                        "Enemy entity {:?} not found in query when trying to reset timer",
                        enemy_entity
                    );
                }
            }
        }
    }
}

// Tank movement animation - uses generic animate_sprite_sheet from common
pub fn animate_enemies(
    time: Res<Time>,
    query: Query<(&mut AnimationTimer, &AnimationIndices, &mut Sprite), With<Enemy>>,
) {
    crate::common::animate_sprite_sheet::<Enemy>(time, query);
}

pub fn cleanup_enemies(
    mut commands: Commands,
    q_enemies: Query<Entity, With<Enemy>>,
    mut scheduled: ResMut<crate::common::ScheduledDespawn>,
) {
    let enemy_count = q_enemies.iter().count();
    if enemy_count > 0 {
        info!("Cleaning up {} enemy entities", enemy_count);
    }
    for entity in &q_enemies {
        if scheduled.0.insert(entity) {
            trace!("Despawning enemy entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

/// Plugin for enemy-related systems and resources
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LevelSpawnedEnemies>()
            .add_systems(
                OnEnter(AppState::StartMenu),
                (cleanup_enemies, reset_level_spawned_enemies),
            )
            .add_systems(
                Update,
                auto_spawn_enemies
                    .in_set(crate::common::GameplaySet::Spawning)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                enemies_move
                    .in_set(crate::common::GameplaySet::Movement)
                    .after(crate::common::GameplaySet::Spawning)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                enemies_attack
                    .in_set(crate::common::GameplaySet::Combat)
                    .after(crate::common::GameplaySet::Movement)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                animate_enemies
                    .in_set(crate::common::GameplaySet::Animation)
                    .after(crate::common::GameplaySet::Effects)
                    .run_if(in_state(AppState::Playing)),
            )
            // Animation systems for GameOver state
            .add_systems(
                Update,
                animate_enemies
                    .in_set(crate::common::GameplaySet::Animation)
                    .run_if(in_state(AppState::GameOver)),
            )
            .add_systems(
                Update,
                handle_enemy_collision
                    .in_set(crate::common::GameplaySet::Collision)
                    .after(crate::common::GameplaySet::BulletMovement)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_spawned_enemies_default() {
        let spawned = LevelSpawnedEnemies::default();
        assert_eq!(spawned.0, 0);
    }

    #[test]
    fn test_level_spawned_enemies_increment() {
        let mut spawned = LevelSpawnedEnemies(5);
        spawned.0 += 1;
        assert_eq!(spawned.0, 6);
    }

    #[test]
    fn test_enemy_sprite_index_sets() {
        let sets = enemies_sprite_index_sets();
        assert!(!sets.is_empty());

        // Each set should have 8 elements (Up, Right, Down, Left + variations)
        for set in &sets {
            assert!(!set.is_empty());
        }
    }
}

pub fn reset_level_spawned_enemies(mut level_spawned_enemies: ResMut<LevelSpawnedEnemies>) {
    level_spawned_enemies.0 = 0;
}

pub fn enemies_sprite_index_sets() -> Vec<Vec<i32>> {
    vec![
        // Up, Right, Down, Left + other possible indices
        vec![0, 8, 16, 24, 1, 9, 17, 25],
        vec![2, 10, 18, 26, 3, 11, 19, 27],
        vec![4, 12, 20, 28, 5, 13, 21, 29],
        vec![6, 14, 22, 30, 7, 15, 23, 31],
        vec![32, 40, 48, 56, 33, 41, 49, 57],
        vec![34, 42, 50, 58, 35, 43, 51, 59],
        vec![36, 44, 52, 60, 37, 45, 53, 61],
        vec![38, 46, 54, 62, 39, 47, 55, 63],
    ]
}
pub fn new_sprite_index(current_index: i32, direction: common::Direction) -> i32 {
    let index_sets = enemies_sprite_index_sets();
    for index_set in index_sets {
        if index_set.contains(&current_index) {
            info!("found index_set");
            match direction {
                common::Direction::Up => {
                    return index_set[0];
                }
                common::Direction::Right => {
                    return index_set[1];
                }
                common::Direction::Down => {
                    return index_set[2];
                }
                common::Direction::Left => {
                    return index_set[3];
                }
            }
        }
    }
    0
}
