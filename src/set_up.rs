use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;

use crate::GameState;
use crate::gameplay::moving_platforms::PlatformWaypoint; // Import your GameState

//use crate::dev_utils::debug_print_game_state;

#[derive(Resource)]
pub struct SceneHandle(pub Handle<Scene>);

pub struct SetupPlugin;
impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera_and_lights)
            .add_systems(
                OnEnter(GameState::Loading),
                (load_scene, spawn_scene).chain(), // Ensures load_scene runs before spawn_scene
            )
            .add_systems(
                Update,
                check_scene_loaded.run_if(in_state(GameState::Loading)),
            );
    }
}

fn setup_camera_and_lights(mut commands: Commands) {
    commands.spawn((
        Transform::from_translation(Vec3::new(0.0, 1.5, 5.0)),
        PanOrbitCamera {
            pan_sensitivity: 0.0,
            pan_smoothness: 0.0,
            orbit_sensitivity: 0.0,
            zoom_sensitivity: 0.0,
            ..default()
        },
    ));

    commands.spawn((PointLight::default(), Transform::from_xyz(5.0, 5.0, 5.0)));

    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::default().looking_at(-Vec3::Y, Vec3::Z),
    ));
}

fn load_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset("Untitled.glb"));
    commands.insert_resource(SceneHandle(handle));
}

fn spawn_scene(mut commands: Commands, scene_handle: Res<SceneHandle>) {
    commands.spawn(SceneRoot(scene_handle.0.clone()));
}

fn check_scene_loaded(
    mut next_state: ResMut<NextState<GameState>>,
    scene_handle: Option<Res<SceneHandle>>,
    asset_server: Res<AssetServer>,
    waypoint_query: Query<&PlatformWaypoint>,
) {
    if let Some(handle) = scene_handle {
        if asset_server.is_loaded(&handle.0) && !waypoint_query.is_empty() {
            println!("Scene loaded and waypoints present! Transitioning to InGame.");
            next_state.set(GameState::InGame);
        } else {
            println!(
                "Waiting: loaded={} waypoints={}",
                asset_server.is_loaded(&handle.0),
                waypoint_query.iter().count()
            );
        }
    }
}
