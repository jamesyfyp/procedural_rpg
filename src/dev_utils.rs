use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

pub struct DevUtilsPlugin;

// use crate::player::Health;
use crate::player::Player;

impl Plugin for DevUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InspectorActive(false));
        app.add_plugins(PhysicsDebugPlugin::default())
            .add_systems(Update, draw_player_gizmo);
        app.add_plugins((
            EguiPlugin {
                enable_multipass_for_primary_context: true,
            },
            WorldInspectorPlugin::new().run_if(is_inspector_active),
        ));
        // Uncomment this to see the player health decrease over tim
    }
}

#[derive(Resource, Debug, Default, Eq, PartialEq)]
struct InspectorActive(bool);

fn is_inspector_active(inspector_active: Res<InspectorActive>) -> bool {
    inspector_active.0
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
