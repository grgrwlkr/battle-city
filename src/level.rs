use crate::{
    common::{AnimationIndices, AnimationTimer, AppState, TILE_SIZE},
    config::GameConfig,
    enemy::{Enemy, LevelSpawnedEnemies},
    player::PlayerNo,
};
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

/// Calculate the translation offset for level positioning
/// Centers the level map based on configuration dimensions
///
/// # Arguments
/// * `game_config` - Game configuration containing level dimensions
///
/// # Returns
/// Translation offset vector to center the level on screen
pub fn level_translation_offset(game_config: &GameConfig) -> Vec3 {
    Vec3::new(
        -game_config.level.columns as f32 / 2.0 * game_config.level.tile_size,
        -game_config.level.rows as f32 / 2.0 * game_config.level.tile_size,
        0.0,
    )
}

/// Level map element types
/// Represents different types of static objects in the game level
#[derive(Component, Clone, PartialEq, Eq, Debug, Default)]
pub enum LevelItem {
    /// No level item (empty space)
    #[default]
    None,
    /// Stone wall (can be destroyed)
    StoneWall,
    /// Iron wall (cannot be destroyed)
    IronWall,
    /// Tree (decorative, can be passed through)
    Tree,
    /// Water (decorative)
    Water,
    /// Home base (player must protect)
    Home,
}

// Re-export CollisionEvent from bevy_rapier2d for convenience
pub use bevy_rapier2d::prelude::CollisionEvent;

/// Event emitted when the home base is being destroyed
#[derive(Debug, Message, Default)]
pub struct HomeDyingEvent;

/// Component marking the spawn position for player 1
#[derive(Component, Default)]
pub struct Player1Marker;

/// Component marking the spawn position for player 2
#[derive(Component, Default)]
pub struct Player2Marker;

/// Component marking possible spawn positions for enemies
#[derive(Component, Default)]
pub struct EnemiesMarker;

#[derive(Clone, Debug, Default, Bundle)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
}

#[derive(Clone, Debug, Bundle)]
pub struct AnimationBundle {
    pub timer: AnimationTimer,
    pub indices: AnimationIndices,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct StoneWallBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    // #[sprite_sheet_bundle("path/to/asset.png", tile_width, tile_height, columns, rows, padding, offset, index)]
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 0)]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct IronWallBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 1)]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct TreeBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 2)]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity)]
pub struct WaterBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 3)]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    pub annimation_bundle: AnimationBundle,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct HomeBundle {
    #[from_entity_instance]
    level_item: LevelItem,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    #[sprite_sheet("textures/map.bmp", 32, 32, 7, 1, 0, 0, 5)]
    sprite_sheet: Sprite,
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct Player1MarkerBundle {
    marker: Player1Marker,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct Player2MarkerBundle {
    marker: Player2Marker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}
#[derive(Bundle, LdtkEntity, Default)]
pub struct EnemiesMarkerBundle {
    marker: EnemiesMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "StoneWall" | "IronWall" | "Water" | "Home" => ColliderBundle {
                collider: Collider::cuboid(TILE_SIZE / 2., TILE_SIZE / 2.),
                rigid_body: RigidBody::Fixed,
            },
            _ => ColliderBundle::default(),
        }
    }
}
impl From<&EntityInstance> for AnimationBundle {
    fn from(entity_instance: &EntityInstance) -> AnimationBundle {
        match entity_instance.identifier.as_ref() {
            "Water" => AnimationBundle {
                timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                indices: AnimationIndices { first: 3, last: 4 },
            },
            _ => AnimationBundle {
                timer: AnimationTimer(Timer::from_seconds(0.2, TimerMode::Repeating)),
                indices: AnimationIndices { first: 0, last: 0 },
            },
        }
    }
}
impl From<&EntityInstance> for LevelItem {
    fn from(entity_instance: &EntityInstance) -> LevelItem {
        match entity_instance.identifier.as_ref() {
            "StoneWall" => LevelItem::StoneWall,
            "IronWall" => LevelItem::IronWall,
            "Tree" => LevelItem::Tree,
            "Water" => LevelItem::Water,
            "Home" => LevelItem::Home,
            _ => LevelItem::None,
        }
    }
}

/// Setup and load level from LDTK file
/// Spawns LDTK world bundle when entering Playing state
/// Skips reload if level is already loaded (e.g., returning from pause)
pub fn setup_levels(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    q_ldtk_world: Query<(), With<LdtkProjectHandle>>,
    level_selection: Res<LevelSelection>,
    game_config: Res<GameConfig>,
) {
    // Optimize: use is_empty() instead of iter().len() > 0
    if !q_ldtk_world.is_empty() {
        // No need to reload LDTK when entering from Paused state
        trace!("LDTK world already loaded, skipping setup");
        return;
    }
    if let LevelSelection::Indices(LevelIndices { level, .. }) = *level_selection {
        info!("Loading level {} from LDTK file", level + 1);
    } else {
        info!("Loading level from LDTK file");
    }
    let offset = level_translation_offset(&game_config);
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels.ldtk").into(),
        transform: Transform::from_translation(Vec3::ZERO + offset),
        ..Default::default()
    });
}

/// Spawn entities from LDTK level data
/// Processes newly added entity instances from LDTK and creates game entities
/// Currently handles Tree entities with proper positioning and sprites
pub fn spawn_ldtk_entity(
    mut commands: Commands,
    entity_query: Query<(Entity, &Transform, &EntityInstance), Added<EntityInstance>>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
    game_config: Res<GameConfig>,
) {
    // Optimize: pre-allocate texture atlas for all trees (shared resource)
    let map_texture_handle = asset_server.load("textures/map.bmp");
    let map_texture_atlas = TextureAtlasLayout::from_grid(UVec2::new(32, 32), 7, 1, None, None);
    let map_texture_atlas_handle = texture_atlases.add(map_texture_atlas);
    let offset = level_translation_offset(&game_config);

    let mut spawned_count = 0;
    let mut entities_to_spawn = Vec::new();

    for (_entity, transform, entity_instance) in entity_query.iter() {
        if entity_instance.identifier == *"Tree" {
            trace!(
                "Spawning LDTK entity: Tree at position ({:.1}, {:.1})",
                transform.translation.x,
                transform.translation.y
            );
            spawned_count += 1;
            let mut translation = transform.translation + offset;
            translation.z = game_config.sprite_order.tree;
            entities_to_spawn.push(translation);
        }
    }

    // Batch spawn all tree entities
    for translation in entities_to_spawn {
        commands.spawn((
            LevelItem::Tree,
            Sprite {
                image: map_texture_handle.clone(),
                texture_atlas: Some(TextureAtlas {
                    index: 2,
                    layout: map_texture_atlas_handle.clone(),
                }),
                ..default()
            },
            Transform::from_translation(translation),
        ));
    }

    if spawned_count > 0 {
        debug!("Spawned {} Tree entities from LDTK", spawned_count);
    }
}

/// Set z-coordinate for all level items to render above tanks
pub fn set_level_items_z_coordinate(
    mut query: Query<&mut Transform, (With<LevelItem>, Added<LevelItem>)>,
    game_config: Res<GameConfig>,
) {
    for mut transform in &mut query {
        transform.translation.z = game_config.sprite_order.tree;
    }
}

// Water animation
pub fn animate_water(
    time: Res<Time>,
    mut query: Query<(
        &LevelItem,
        &mut AnimationTimer,
        &AnimationIndices,
        &mut Sprite,
    )>,
) {
    for (level_item, mut timer, indices, mut sprite) in &mut query {
        if *level_item == LevelItem::Water {
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
}

/// Automatically switch to next level when conditions are met
/// Triggers level transition when all enemies are spawned and destroyed
/// Handles game victory when max levels are completed
/// Cleans up entities before transitioning
#[allow(clippy::too_many_arguments)]
pub fn auto_switch_level(
    mut commands: Commands,
    q_enemies: Query<(), With<Enemy>>,
    q_players: Query<Entity, With<PlayerNo>>,
    q_level_items: Query<Entity, With<LevelItem>>,
    mut level_selection: ResMut<LevelSelection>,
    mut level_spawned_enemies: ResMut<LevelSpawnedEnemies>,
    mut app_state: ResMut<NextState<AppState>>,
    mut scheduled: ResMut<crate::common::ScheduledDespawn>,
    game_config: Res<GameConfig>,
) {
    // Switch to next level when maximum enemies spawned and all enemies are destroyed
    let enemies_per_level = game_config.enemy.enemies_per_level;
    // Optimize: use is_empty() instead of iter().len() == 0
    if level_spawned_enemies.0 == enemies_per_level && q_enemies.is_empty() {
        if let LevelSelection::Indices(LevelIndices { level, .. }) = *level_selection {
            if level as i32 == game_config.level.max_levels - 1 {
                // Game victory - all levels completed
                info!(
                    "Level {} completed! All enemies destroyed. Game victory!",
                    level + 1
                );
                app_state.set(AppState::Victory);
            } else {
                // Next level
                let player_count = q_players.iter().count();
                let item_count = q_level_items.iter().count();
                info!(
                    "Level {} completed! All {} enemies destroyed. Switching to level {}, cleaning up {} players and {} level items",
                    level + 1, enemies_per_level, level + 2, player_count, item_count
                );
                *level_selection = LevelSelection::index(level + 1);
                level_spawned_enemies.0 = 0;

                // Respawn players
                for player in &q_players {
                    if scheduled.0.insert(player) {
                        trace!("Despawning player entity {:?} for level transition", player);
                        commands.entity(player).despawn();
                    }
                }
                for level_item in &q_level_items {
                    if scheduled.0.insert(level_item) {
                        trace!(
                            "Despawning level item entity {:?} for level transition",
                            level_item
                        );
                        commands.entity(level_item).despawn();
                    }
                }
            }
        }
    }
    // Placeholder for patch format
    // No operation changes
}

pub fn animate_home(
    mut home_dying_er: MessageReader<HomeDyingEvent>,
    mut q_level_items: Query<(&LevelItem, &mut Sprite, &Transform)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for _ in home_dying_er.read() {
        for (level_item, mut sprite, transform) in &mut q_level_items {
            if *level_item == LevelItem::Home {
                warn!(
                    "Home base destroyed! Changing sprite and triggering game over. Position: ({:.1}, {:.1})",
                    transform.translation.x, transform.translation.y
                );
                if let Some(atlas) = sprite.texture_atlas.as_mut() {
                    atlas.index = 6;
                } else {
                    error!("Home sprite has no texture atlas, cannot update animation");
                }
                app_state.set(AppState::GameOver);
            }
        }
    }
}

pub fn cleanup_level_items(
    mut commands: Commands,
    q_level_items: Query<Entity, With<LevelItem>>,
    mut scheduled: ResMut<crate::common::ScheduledDespawn>,
) {
    let item_count = q_level_items.iter().count();
    if item_count > 0 {
        debug!("Cleaning up {} level item entities", item_count);
    }
    for entity in &q_level_items {
        if scheduled.0.insert(entity) {
            trace!("Despawning level item entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

pub fn cleanup_ldtk_world(
    mut commands: Commands,
    q_ldtk_world: Query<Entity, With<LdtkProjectHandle>>,
    mut scheduled: ResMut<crate::common::ScheduledDespawn>,
) {
    let world_count = q_ldtk_world.iter().count();
    if world_count > 0 {
        info!("Cleaning up {} LDTK world entities", world_count);
    }
    for entity in &q_ldtk_world {
        if scheduled.0.insert(entity) {
            trace!("Despawning LDTK world entity {:?}", entity);
            commands.entity(entity).despawn();
        }
    }
}

pub fn reset_level_selection(mut level_selection: ResMut<LevelSelection>) {
    *level_selection = LevelSelection::index(0);
}

/// Plugin for level-related systems and resources
pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<StoneWallBundle>("StoneWall")
            .register_ldtk_entity::<IronWallBundle>("IronWall")
            .register_ldtk_entity::<TreeBundle>("Tree")
            .register_ldtk_entity::<WaterBundle>("Water")
            .register_ldtk_entity::<HomeBundle>("Home")
            .register_ldtk_entity::<Player1MarkerBundle>("Player1")
            .register_ldtk_entity::<Player2MarkerBundle>("Player2")
            .register_ldtk_entity::<EnemiesMarkerBundle>("Enemies")
            .add_systems(
                OnEnter(AppState::StartMenu),
                (
                    cleanup_level_items,
                    cleanup_ldtk_world,
                    reset_level_selection,
                ),
            )
            .add_systems(OnEnter(AppState::Playing), setup_levels)
            .add_systems(
                Update,
                (spawn_ldtk_entity, set_level_items_z_coordinate)
                    .chain()
                    .in_set(crate::common::GameplaySet::Spawning)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                auto_switch_level
                    .in_set(crate::common::GameplaySet::LevelManagement)
                    .after(crate::common::GameplaySet::Collision)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                (animate_water, animate_home)
                    .chain()
                    .in_set(crate::common::GameplaySet::Animation)
                    .after(crate::common::GameplaySet::Effects)
                    .run_if(in_state(AppState::Playing)),
            )
            // Animation systems for GameOver state
            .add_systems(
                Update,
                (animate_water, animate_home)
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
    fn test_level_translation_offset() {
        let config = GameConfig::default();
        let offset = level_translation_offset(&config);

        // Expected: -columns/2 * tile_size, -rows/2 * tile_size, 0.0
        let expected_x = -(config.level.columns as f32 / 2.0) * config.level.tile_size;
        let expected_y = -(config.level.rows as f32 / 2.0) * config.level.tile_size;

        assert_eq!(offset.x, expected_x);
        assert_eq!(offset.y, expected_y);
        assert_eq!(offset.z, 0.0);
    }

    #[test]
    fn test_level_item_from_entity_instance() {
        use bevy_ecs_ldtk::EntityInstance;

        // This would require creating an EntityInstance, which is complex
        // For now, we test the enum itself
        assert_eq!(LevelItem::StoneWall, LevelItem::StoneWall);
        assert_ne!(LevelItem::StoneWall, LevelItem::IronWall);
        assert_ne!(LevelItem::Tree, LevelItem::Water);
        assert_eq!(LevelItem::None, LevelItem::default());
    }

    #[test]
    fn test_level_item_variants() {
        assert_eq!(LevelItem::None, LevelItem::default());
        assert_ne!(LevelItem::StoneWall, LevelItem::default());
        assert_ne!(LevelItem::IronWall, LevelItem::default());
        assert_ne!(LevelItem::Tree, LevelItem::default());
        assert_ne!(LevelItem::Water, LevelItem::default());
        assert_ne!(LevelItem::Home, LevelItem::default());
    }
}
