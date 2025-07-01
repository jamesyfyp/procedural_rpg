use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_skein::SkeinPlugin;
use bevy_tnua::{builtins::TnuaBuiltinDash, prelude::*};
use bevy_tnua_avian3d::*;

mod gameplay {
    pub mod moving_platforms;
}
use gameplay::moving_platforms::MovingPlatformPlugin;

mod set_up;
use set_up::SetupPlugin;

mod player;
use player::{Health, Player, PlayerPlugin};

mod dev_utils;
use dev_utils::DevUtilsPlugin;

mod ui;
use ui::UiPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
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
            // remove dev utils for final build
            DevUtilsPlugin,
        ))
        .add_systems(Update, spike_damage_system)
        .run();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Spikes {
    damage: f32,
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct SpikeDamageCooldown(Timer);

#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Loading,
    InGame,
}

fn spike_damage_system(
    time: Res<Time>,
    mut health_query: Query<
        (
            &mut Health,
            &Transform,
            &mut SpikeDamageCooldown,
            &mut TnuaController,
        ),
        With<Player>,
    >,
    spike_query: Query<(&Spikes, &Transform)>,
) {
    if let Ok((mut health, player_transform, mut cooldown, mut tnua_controller)) =
        health_query.single_mut()
    {
        cooldown.0.tick(time.delta());

        for (spike, spike_transform) in &spike_query {
            let player_pos = player_transform.translation;
            let spike_pos = spike_transform.translation;
            let distance = player_pos.distance(spike_pos);

            if distance < 3.0 && cooldown.0.finished() {
                // Damage
                health.0 = (health.0 - spike.damage).max(0.0);

                // Knockback direction using Tnua impulse
                let knock_dir = (player_pos - spike_pos).normalize_or_zero();
                tnua_controller.action(TnuaBuiltinDash {
                    displacement: knock_dir * 5.0, // Adjust strength as needed
                    ..Default::default()
                });
                // Reset cooldown
                cooldown.0.reset();
            }
        }
    }
}
