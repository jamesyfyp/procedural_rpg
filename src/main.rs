use avian3d::prelude::*;
use bevy::{color::palettes::css, input::mouse::MouseMotion, prelude::*};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_skein::SkeinPlugin;
use bevy_tnua::{
    builtins::{TnuaBuiltinCrouch, TnuaBuiltinDash},
    prelude::*,
};
use bevy_tnua_avian3d::*;

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
        ))
        .register_type::<Player>()
        .add_systems(
            Startup,
            (setup_camera_and_lights, setup_level, setup_player),
        )
        .add_systems(
            FixedUpdate,
            (apply_controls, cam_follow_and_face, always_orbit_camera)
                .chain()
                .in_set(TnuaUserControlsSystemSet),
        )
        .add_systems(Update, (draw_player_gizmo,))
        .run();
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Player;

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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("Untitled.glb")),
    ));

    // Spawn a little platform for the player to jump on.
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(4.0, 0.5, 4.0))),
        MeshMaterial3d(materials.add(Color::from(css::GRAY))),
        Transform::from_xyz(-6.0, 2.5, 0.0),
        RigidBody::Static,
        Collider::cuboid(4.0, 0.5, 4.0),
    ));
}

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Capsule3d {
            radius: 0.5,
            half_length: 0.5,
        })),
        MeshMaterial3d(materials.add(Color::from(css::DARK_GOLDENROD))),
        Transform::from_xyz(0.0, 2.0, 0.0),
        // The player character needs to be configured as a dynamic rigid body of the physics
        // engine.
        RigidBody::Dynamic,
        Collider::capsule(0.5, 1.0),
        // This is Tnua's interface component.
        TnuaController::default(),
        // A sensor shape is not strictly necessary, but without it we'll get weird results.
        TnuaAvian3dSensorShape(Collider::cylinder(0.49, 0.0)),
        // Tnua can fix the rotation, but the character will still get rotated before it can do so.
        // By locking the rotation we can prevent this.
        LockedAxes::ROTATION_LOCKED,
        Player,
    ));
}

fn apply_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut TnuaController, With<Player>>,
    tfm_q: Query<&Transform, With<Player>>,
) {
    let Ok(mut controller) = query.single_mut() else {
        return;
    };
    let Ok(transform) = tfm_q.single() else {
        return;
    };

    // --- JUMP ---
    if keyboard.pressed(KeyCode::Space) {
        controller.action(TnuaBuiltinJump {
            height: 4.0,
            ..Default::default()
        });
    }

    // --- CROUCH ---
    if keyboard.pressed(KeyCode::ControlLeft) {
        controller.action(TnuaBuiltinCrouch {
            float_offset: -0.5,
            ..Default::default()
        });
    }

    // --- WALK/RUN ---
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyS) {
        direction += transform.rotation * Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyW) {
        direction -= transform.rotation * Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction -= transform.rotation * Vec3::X;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += transform.rotation * Vec3::X;
    }

    //---DASH
    if keyboard.pressed(KeyCode::ShiftLeft) {
        // Dash in the facing direction
        controller.action(TnuaBuiltinDash {
            //desired_forward: dash_dir,
            displacement: direction.normalize_or_zero() * 20.0,
            ..Default::default()
        });
    }

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: direction.normalize_or_zero() * 10.0,
        float_height: 1.5,
        ..Default::default()
    });
}

fn cam_follow_and_face(
    mut pan_orbit_q: Query<&mut PanOrbitCamera>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    if let (Ok(mut pan_orbit), Ok(mut player_tfm)) =
        (pan_orbit_q.single_mut(), player_q.single_mut())
    {
        // Make camera follow player
        pan_orbit.target_focus = player_tfm.translation;
        pan_orbit.force_update = true;

        // Rotate player to match camera yaw (around Y axis)
        if let Some(yaw) = pan_orbit.yaw {
            player_tfm.rotation = Quat::from_rotation_y(yaw);
        } else {
            player_tfm.rotation = Quat::from_rotation_y(pan_orbit.target_yaw);
        }
    }
}

fn draw_player_gizmo(mut gizmos: Gizmos, query: Query<&Transform, With<Player>>) {
    if let Ok(transform) = query.single() {
        let start = transform.translation + Vec3::Y * 1.0;
        let forward = transform.forward();
        let end = start + forward * 1.5;
        gizmos.arrow(start, end, Color::from(css::DARK_CYAN));
    }
}

fn always_orbit_camera(
    mut panorbit_query: Query<&mut PanOrbitCamera>,
    mut mouse_motion_events: EventReader<MouseMotion>,
) {
    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }
    if delta != Vec2::ZERO {
        for mut cam in &mut panorbit_query {
            cam.target_yaw -= delta.x * 0.005;
            cam.target_pitch += delta.y * 0.005;

            // Clamp pitch (in radians). Example: -1.5 to 1.5 (~-86 to +86 degrees)
            let min_pitch = 0.0;
            let max_pitch = 1.5;
            cam.target_pitch = cam.target_pitch.clamp(min_pitch, max_pitch);

            cam.force_update = true;
        }
    }
}
