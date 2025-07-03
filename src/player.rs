use avian3d::prelude::*;
use bevy::{color::palettes::css, input::mouse::MouseMotion, prelude::*};
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_tnua::{
    builtins::{TnuaBuiltinCrouch, TnuaBuiltinDash},
    prelude::*,
};
use bevy_tnua_avian3d::*;

use crate::GameState;

use crate::SpikeDamageCooldown;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Player;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Health(pub f32);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<(Player, Health)>()
            .add_systems(OnEnter(GameState::InGame), setup_player)
            .add_systems(
                FixedUpdate,
                (apply_controls, cam_follow_and_face, always_orbit_camera)
                    .chain()
                    .in_set(TnuaUserControlsSystemSet)
                    .run_if(in_state(GameState::InGame)),
            );
    }
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
        //TODO: FIX HEIGHT WHEN YOU SET UP LOADING
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
        CollisionEventsEnabled,
        Player,
        Health(100.0),
        SpikeDamageCooldown(Timer::from_seconds(1.0, TimerMode::Once)),
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
            let min_pitch = -0.25;
            let max_pitch = 1.5;
            cam.target_pitch = cam.target_pitch.clamp(min_pitch, max_pitch);

            cam.force_update = true;
        }
    }
}

fn cam_follow_and_face(
    mut pan_orbit_q: Query<&mut PanOrbitCamera>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    if let (Ok(mut pan_orbit), Ok(mut player_tfm)) =
        (pan_orbit_q.single_mut(), player_q.single_mut())
    {
        // Smoothly follow player
        let smoothing = 0.15;
        pan_orbit.target_focus = pan_orbit
            .target_focus
            .lerp(player_tfm.translation, smoothing);
        pan_orbit.force_update = true;

        // Rotate player to match camera yaw (around Y axis)
        if let Some(yaw) = pan_orbit.yaw {
            player_tfm.rotation = Quat::from_rotation_y(yaw);
        } else {
            player_tfm.rotation = Quat::from_rotation_y(pan_orbit.target_yaw);
        }
    }
}
