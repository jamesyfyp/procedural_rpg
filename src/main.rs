use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_skein::SkeinPlugin;
use bevy_tnua::{builtins::TnuaBuiltinDash, prelude::*};
use bevy_tnua_avian3d::*;

mod player;
use player::{Health, Player, PlayerPlugin};

mod dev_utils;
use dev_utils::DevUtilsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes_override: Some(true),
            ..default()
        }))
        .add_plugins((
            PhysicsPlugins::default(),
            SkeinPlugin::default(),
            PanOrbitCameraPlugin,
            // We need both Tnua's main controller plugin, and the plugin to connect to the physics
            // backend (in this case Avian 3D)
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
            PlayerPlugin,
            // remove dev utils for final build
            DevUtilsPlugin,
        ))
        .register_type::<(FloatingPlatform, Spikes)>()
        .add_systems(
            Startup,
            (setup_camera_and_lights, setup_level, spawn_health_bar),
        )
        .add_systems(Update, (spike_damage_system, update_health_bar))
        .run();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct FloatingPlatform;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Spikes {
    damage: f32,
}

#[derive(Component, Default)]
pub struct SpikeDamageCooldown(Timer);

#[derive(Component)]
struct HealthBarFill;

#[derive(Component)]
struct HealthBarText;

// No Tnua-related setup here - this is just normal Bevy stuff.
fn setup_camera_and_lights(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera {
            // Panning the camera changes the focus, and so you most likely want to disable
            // panning when setting the focus manually
            pan_sensitivity: 0.0,
            // If you want to fully control the camera's focus, set smoothness to 0 so it
            // immediately snaps to that location. If you want the 'follow' to be smoothed,
            // leave this at default or set it to something between 0 and 1.
            pan_smoothness: 0.0,
            orbit_sensitivity: 0.0,
            zoom_sensitivity: 0.0,
            ..default()
        },
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(5.0, 5.0, 5.0)));

    // A directly-down light to tell where the player is going to land.
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::default().looking_at(-Vec3::Y, Vec3::Z),
    ));
}

// No Tnua-related setup here - this is just normal Bevy (and Avian) stuff.
fn setup_level(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("Untitled.glb")),
    ));
}

fn spawn_health_bar(mut commands: Commands) {
    // Parent node (background)
    let parent = commands
        .spawn((
            Node {
                width: Val::Px(200.0),
                height: Val::Px(24.0),
                position_type: PositionType::Absolute,
                left: Val::Px(200.0), // Move bar horizontally
                top: Val::Px(200.0),  // Move bar vertically
                ..default()
            },
            BackgroundColor(Color::from(css::DARK_GRAY)),
        ))
        .id();

    // Fill node (foreground)
    let fill = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::from(css::GREEN)),
            HealthBarFill,
        ))
        .id();

    // Text node (health value)
    let text = commands
        .spawn((
            Text::new("100"),
            TextFont {
                font_size: 100.0,
                ..default()
            },
            TextColor(Color::from(css::WHITE)),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
            HealthBarText,
        ))
        .id();

    // Add children to parent
    commands.entity(parent).add_children(&[fill, text]);
}

fn update_health_bar(
    health_query: Query<&Health, With<Player>>,
    mut fill_query: Query<&mut Node, With<HealthBarFill>>,
    mut text_query: Query<&mut Text, With<HealthBarText>>,
) {
    if let Ok(health) = health_query.single() {
        let health_percent = (health.0 / 100.0).clamp(0.0, 1.0);

        // Update fill width
        if let Ok(mut style) = fill_query.single_mut() {
            style.width = Val::Percent(health_percent * 100.0);
        }

        // Update text
        if let Ok(mut text) = text_query.single_mut() {
            text.0 = format!("{:.0}", health.0);
        }
    }
}

// this is for refference
// fn damage_player(mut health_query: Query<&mut Health, With<Player>>) {
//     if let Ok(mut health) = health_query.single_mut() {
//         health.0 = (health.0 - 0.005).max(0.0);
//     }
// }

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
