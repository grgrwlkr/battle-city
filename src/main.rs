mod area;
mod bullet;
mod common;
mod enemy;
mod level;
mod player;
mod ui;

use area::*;
use bullet::*;
use common::*;
use enemy::*;
use level::*;
use player::*;
use ui::*;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

const BACKGROUND_COLOR: Color = Color::BLACK;

// TODO: Tank collision causes forced movement
fn main() {
    App::new()
        .register_type::<PlayerNo>()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugins(LdtkPlugin)
        .add_message::<ExplosionEvent>()
        .add_message::<SpawnPlayerEvent>()
        .add_message::<HomeDyingEvent>()
        .add_message::<CollisionEvent>()
        .init_state::<AppState>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(MultiplayerMode::SinglePlayer)
        .insert_resource(LevelSelection::index(0))
        .insert_resource(LevelSpawnedEnemies(0))
        .insert_resource(PlayerLives {
            player1: 3,
            player2: 3,
        })
        .init_resource::<ScheduledDespawn>()
        .register_ldtk_entity::<level::StoneWallBundle>("StoneWall")
        .register_ldtk_entity::<level::IronWallBundle>("IronWall")
        .register_ldtk_entity::<level::WaterBundle>("Water")
        .register_ldtk_entity::<level::HomeBundle>("Home")
        .register_ldtk_entity::<level::Player1MarkerBundle>("Player1")
        .register_ldtk_entity::<level::Player2MarkerBundle>("Player2")
        .register_ldtk_entity::<level::EnemiesMarkerBundle>("Enemies")
        .add_systems(
            Startup,
            (
                setup_camera,
                setup_rapier,
                setup_wall,
                setup_explosion_assets,
                setup_game_sounds,
            ),
        )
        .add_systems(
            OnEnter(AppState::StartMenu),
            (
                setup_start_menu,
                cleanup_level_items,
                cleanup_ldtk_world,
                cleanup_players,
                cleanup_born,
                cleanup_bullets,
                cleanup_explosions,
                cleanup_enemies,
                reset_player_lives,
                reset_level_selection,
                reset_level_spawned_enemies,
                reset_multiplayer_mode,
                |state: Res<State<AppState>>| {
                    info!("Entered state: {:?}", state.get());
                },
            ),
        )
        .add_systems(
            OnExit(AppState::StartMenu),
            (despawn_screen::<OnStartMenuScreen>,),
        )
        .add_systems(
            OnEnter(AppState::Playing),
            (
                setup_levels,
                |state: Res<State<AppState>>| {
                    info!("Entered state: {:?}", state.get());
                },
            ),
        )
        // Input handling
        .add_systems(
            Update,
            (start_game, switch_multiplayer_mode)
                .in_set(GameplaySet::Input)
                .run_if(in_state(AppState::StartMenu)),
        )
        .add_systems(
            Update,
            pause_game
                .in_set(GameplaySet::Ui)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            Update,
            unpause_game
                .in_set(GameplaySet::Ui)
                .run_if(in_state(AppState::Paused)),
        )
        // Spawning entities
        .add_systems(
            Update,
            (spawn_ldtk_entity, auto_spawn_players, auto_spawn_enemies)
                .in_set(GameplaySet::Spawning)
                .run_if(in_state(AppState::Playing)),
        )
        // Movement systems
        .add_systems(
            Update,
            (players_move, enemies_move)
                .in_set(GameplaySet::Movement)
                .after(GameplaySet::Spawning)
                .run_if(in_state(AppState::Playing)),
        )
        // Combat systems (shooting)
        .add_systems(
            Update,
            (players_attack, enemies_attack)
                .in_set(GameplaySet::Combat)
                .after(GameplaySet::Movement)
                .run_if(in_state(AppState::Playing)),
        )
        // Bullet movement
        .add_systems(
            Update,
            move_bullet
                .in_set(GameplaySet::BulletMovement)
                .after(GameplaySet::Combat)
                .run_if(in_state(AppState::Playing)),
        )
        // Collision handling
        .add_systems(
            Update,
            (handle_bullet_collision, handle_enemy_collision)
                .in_set(GameplaySet::Collision)
                .after(GameplaySet::BulletMovement)
                .run_if(in_state(AppState::Playing)),
        )
        // Post-collision effects
        // spawn_explosion and animate_born need to work in Playing and GameOver states
        // to allow animations to complete
        .add_systems(
            Update,
            (spawn_explosion, animate_born)
                .in_set(GameplaySet::Effects)
                .after(GameplaySet::Collision)
                .run_if(in_state(AppState::Playing)),
        )
        // Same systems for GameOver state (without collision dependency as collisions don't happen in GameOver)
        .add_systems(
            Update,
            (spawn_explosion, animate_born)
                .in_set(GameplaySet::Effects)
                .run_if(in_state(AppState::GameOver)),
        )
        // Animations
        // Note: animate_players, animate_enemies, animate_shield now use the generic
        // animate_sprite_sheet function from common.rs to avoid code duplication
        .add_systems(
            Update,
            (
                animate_players,
                animate_enemies,
                animate_shield,
                animate_water,
                animate_home,
                animate_explosion,
                remove_shield,
            )
                .in_set(GameplaySet::Animation)
                .after(GameplaySet::Effects)
                .run_if(in_state(AppState::Playing)),
        )
        // Level management
        .add_systems(
            Update,
            auto_switch_level
                .in_set(GameplaySet::LevelManagement)
                .after(GameplaySet::Collision)
                .run_if(in_state(AppState::Playing)),
        )
        .add_systems(
            OnEnter(AppState::GameOver),
            (
                setup_game_over,
                |state: Res<State<AppState>>| {
                    warn!("Entered state: {:?} - Game Over!", state.get());
                },
            ),
        )
        .add_systems(
            Update,
            (
                animate_game_over,
                animate_players,
                animate_enemies,
                animate_shield,
                animate_water,
                animate_home,
                animate_explosion,
            )
                .in_set(GameplaySet::Animation)
                .run_if(in_state(AppState::GameOver)),
        )
        .add_systems(
            OnExit(AppState::GameOver),
            (despawn_screen::<OnGameOverScreen>,),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    info!("Setting up 2D camera");
    commands.spawn(Camera2d);
}

fn setup_rapier(mut rapier_config: Single<&mut RapierConfiguration>) {
    info!("Configuring Rapier physics: gravity disabled (top-down game)");
    rapier_config.gravity = Vec2::ZERO;
}
