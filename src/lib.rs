use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_skein::SkeinPlugin;
use bevy_tnua::prelude::*;
use bevy_tnua_avian3d::*;

mod gameplay {
    pub mod attacks;
    pub mod enemies;
    pub mod moving_platforms;
}

use gameplay::attacks::fireball::FireballPlugin;
use gameplay::enemies::enemy::EnemyPlugin;
use gameplay::moving_platforms::MovingPlatformPlugin;

mod set_up;
use set_up::SetupPlugin;

mod player;
use player::PlayerPlugin;

mod dev_utils;
use dev_utils::DevUtilsPlugin;

mod ui;
use ui::UiPlugin;

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .init_state::<GameState>()
        .add_plugins((
            PhysicsPlugins::default(),
            SkeinPlugin::default(),
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            PanOrbitCameraPlugin,
            SetupPlugin,
            PlayerPlugin,
            UiPlugin,
            MovingPlatformPlugin,
            EnemyPlugin,
            FireballPlugin,
            // DevUtilsPlugin,
        ));
    }
}
