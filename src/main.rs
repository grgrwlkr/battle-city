mod area;
mod bullet;
mod common;
mod config;
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

/// Core plugin for game initialization
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<HomeDyingEvent>()
            .add_message::<CollisionEvent>()
            .init_state::<AppState>()
            .insert_resource(ClearColor(BACKGROUND_COLOR))
            .insert_resource(MultiplayerMode::SinglePlayer)
            .insert_resource(LevelSelection::index(0))
            .insert_resource(PlayerLives::default())
            .init_resource::<ScheduledDespawn>()
            .init_resource::<config::GameConfig>()
            .add_systems(
                Startup,
                (
                    setup_camera,
                    setup_rapier,
                    setup_wall,
                    setup_explosion_assets,
                    setup_game_sounds,
                ),
            );
    }
}

// TODO: Tank collision causes forced movement
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        // .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugins(LdtkPlugin)
        .add_plugins(CorePlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(EnemyPlugin)
        .add_plugins(BulletPlugin)
        .add_plugins(LevelPlugin)
        .add_plugins(UiPlugin)
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
