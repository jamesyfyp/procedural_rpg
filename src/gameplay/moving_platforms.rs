use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_tnua::math::{AdjustPrecision, Float, Vector3};

use crate::GameState;
//use crate::dev_utils::debug_print_game_state;

pub struct MovingPlatformPlugin;

impl Plugin for MovingPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MovingPlatform>()
            .register_type::<PlatformWaypoint>()
            .register_type::<PlatformGroup>()
            .add_systems(
                OnEnter(GameState::InGame),
                (
                    setup_platform_paths,
                    //debug_platform_waypoints,
                    //debug_print_game_state,
                ),
            )
            .add_systems(
                Update,
                moving_platform_system.run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MovingPlatform {
    pub current_leg: usize,
    pub speed: Float,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
pub struct PlatformWaypoint {
    pub index: usize,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct PlatformGroup(pub String);

#[derive(Component)]
pub struct PlatformPath {
    pub locations: Vec<Vector3>,
}

fn setup_platform_paths(
    mut commands: Commands,
    platform_query: Query<(Entity, &PlatformGroup, &GlobalTransform), With<MovingPlatform>>,
    waypoint_query: Query<(&PlatformGroup, &PlatformWaypoint, &GlobalTransform)>,
) {
    // Collect all waypoints by group name
    let mut group_to_waypoints: std::collections::HashMap<String, Vec<(usize, Vector3)>> =
        std::collections::HashMap::new();

    for (group, waypoint, transform) in &waypoint_query {
        group_to_waypoints
            .entry(group.0.clone())
            .or_default()
            .push((waypoint.index, transform.translation().adjust_precision()));
    }

    // Assign sorted waypoints to each platform by group name, with initial position as index 0
    for (platform_entity, group, platform_transform) in &platform_query {
        let mut locations = Vec::new();
        // Insert the platform's initial position as the first location
        locations.push(platform_transform.translation().adjust_precision());

        if let Some(mut waypoints) = group_to_waypoints.get(&group.0).cloned() {
            // Sort by index
            waypoints.sort_by_key(|(index, _)| *index);
            // Append the sorted waypoints
            locations.extend(waypoints.iter().map(|(_, pos)| *pos));
            println!(
                "  -> Added {} waypoints for group '{}': {:?}",
                waypoints.len(),
                group.0,
                locations
            );
        } else {
            warn!("No waypoints found for platform group '{}'", group.0);
        }

        println!(
            "Setting up platform path for group '{}': locations={:?}",
            group.0, locations,
        );

        commands
            .entity(platform_entity)
            .insert(PlatformPath { locations });
    }
}

fn moving_platform_system(
    time: Res<Time>,
    mut query: Query<(
        &mut MovingPlatform,
        &Transform,
        &PlatformPath,
        &mut LinearVelocity,
    )>,
) {
    for (mut moving_platform, transform, path, mut linear_velocity) in query.iter_mut() {
        if path.locations.is_empty() {
            linear_velocity.0 = Vec3::ZERO;
            continue;
        }
        let current = transform.translation;
        let target = path.locations[moving_platform.current_leg];
        let vec_to = target - current;
        let speed = moving_platform.speed;
        let step = vec_to.normalize_or_zero() * speed;

        // If we're close enough, snap to the target and advance
        if vec_to.length() <= step.length() * time.delta_secs() {
            // Snap to target by zeroing velocity, will be updated next frame
            linear_velocity.0 = Vec3::ZERO;
            moving_platform.current_leg = (moving_platform.current_leg + 1) % path.locations.len();
        } else {
            // Set velocity toward the target
            linear_velocity.0 = step;
        }
    }
}
