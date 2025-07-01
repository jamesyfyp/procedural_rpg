use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};

use crate::GameState;

pub struct DevUtilsPlugin;

// use crate::player::Health;
use crate::player::Player;

impl Plugin for DevUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsDebugPlugin::default())
            .add_systems(Update, draw_player_gizmo);
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

// uncomment this and the health import to sanity check the player health/ui
// fn damage_player(mut health_query: Query<&mut Health, With<Player>>) {
//     if let Ok(mut health) = health_query.single_mut() {
//         health.0 = (health.0 - 0.005).max(0.0);
//     }
// }

pub fn debug_print_game_state(state: Res<State<GameState>>) {
    println!("Current GameState: {:?}", state.get());
}
