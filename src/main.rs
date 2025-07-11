use bevy::prelude::*;
use procedural_rpg::AppPlugin;

fn main() {
    App::new().add_plugins(AppPlugin).run();
}

// at some point when we implement spikes again use this code but put it in a separate file
// #[derive(Component, Reflect)]
// #[reflect(Component)]
// struct Spikes {
//     damage: f32,
// }

// #[derive(Component, Default, Reflect)]
// #[reflect(Component)]
// pub struct SpikeDamageCooldown(Timer);

// fn spike_damage_system(
//     time: Res<Time>,
//     mut health_query: Query<
//         (
//             &mut Health,
//             &Transform,
//             &mut SpikeDamageCooldown,
//             &mut TnuaController,
//         ),
//         With<Player>,
//     >,
//     spike_query: Query<(&Spikes, &Transform)>,
// ) {
//     if let Ok((mut health, player_transform, mut cooldown, mut tnua_controller)) =
//         health_query.single_mut()
//     {
//         cooldown.0.tick(time.delta());

//         for (spike, spike_transform) in &spike_query {
//             let player_pos = player_transform.translation;
//             let spike_pos = spike_transform.translation;
//             let distance = player_pos.distance(spike_pos);

//             if distance < 3.0 && cooldown.0.finished() {
//                 // Damage
//                 health.0 = (health.0 - spike.damage).max(0.0);

//                 // Knockback direction using Tnua impulse
//                 let knock_dir = (player_pos - spike_pos).normalize_or_zero();
//                 tnua_controller.action(TnuaBuiltinDash {
//                     displacement: knock_dir * 5.0, // Adjust strength as needed
//                     ..Default::default()
//                 });
//                 // Reset cooldown
//                 cooldown.0.reset();
//             }
//         }
//     }
// }
