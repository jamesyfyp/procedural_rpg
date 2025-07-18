//TODO: refactor this
//TODO: do some research on animation graphs, see how if avain does anything to help here

use crate::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;

use crate::gameplay::enemies::enemy::Enemy;
use crate::gameplay::enemies::melee_creeps::goopers::GooperPlugin;

// Optionally, you can add a marker for melee creeps:
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MeleeCreep;

pub struct MeleeCreepPlugin;

impl Plugin for MeleeCreepPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<MeleeCreep>()
            .add_plugins(GooperPlugin)
            .add_systems(
                Update,
                (melee_creep_movement_system, melee_creep_damage_system)
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn melee_creep_movement_system(
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    mut creep_query: Query<
        (
            Entity,
            &Enemy,
            &GlobalTransform,
            &mut LinearVelocity,
            &mut Rotation,
        ),
        With<MeleeCreep>,
    >,
) {
    let player_transform = if let Some(t) = player_query.iter().next() {
        t
    } else {
        return;
    };
    let mut player_pos = player_transform.translation();
    player_pos.y = 0.0;

    let creep_positions: Vec<_> = creep_query
        .iter()
        .map(|(e, _, t, _, _)| (e, t.translation()))
        .collect();

    for (_e, enemy, creep_transform, mut velocity, mut rotation) in &mut creep_query {
        let mut creep_pos = creep_transform.translation();
        creep_pos.y = 0.0;

        // Movement direction with repulsion
        let mut dir = (player_pos - creep_pos).normalize_or_zero();
        let mut rep = Vec3::ZERO;
        let min_sep = 3.5;
        for (other_e, other_pos) in &creep_positions {
            if *other_e != _e {
                let mut op = *other_pos;
                op.y = 0.0;
                let dist = creep_pos.distance(op);
                if dist < min_sep && dist > 0.0 {
                    rep += (creep_pos - op).normalize() / dist;
                }
            }
        }
        dir += rep * 1.5;
        let dir = dir.normalize_or_zero();
        velocity.0 = dir * enemy.speed;
    }
}

fn melee_creep_damage_system(
    mut collision_events: EventReader<CollisionStarted>,
    mut player_query: Query<
        (
            &mut crate::player::Health,
            &mut LinearVelocity,
            &GlobalTransform,
        ),
        With<crate::player::Player>,
    >,
    creep_query: Query<(&Enemy, &GlobalTransform), With<MeleeCreep>>,
) {
    for CollisionStarted(e1, e2) in collision_events.read() {
        let (player_entity, creep_entity) =
            if player_query.get(*e1).is_ok() && creep_query.get(*e2).is_ok() {
                (*e1, *e2)
            } else if player_query.get(*e2).is_ok() && creep_query.get(*e1).is_ok() {
                (*e2, *e1)
            } else {
                continue;
            };

        let (mut player_health, mut player_velocity, player_transform) =
            player_query.get_mut(player_entity).unwrap();
        let (creep, creep_transform) = creep_query.get(creep_entity).unwrap();

        // Damage the player
        player_health.0 -= creep.damage;

        // Knockback: push player away from creep
        let mut knockback_dir = player_transform.translation() - creep_transform.translation();
        knockback_dir.y = 0.0; // Only knock back on XZ plane
        let knockback_strength = 8.0; // Tune this value as needed

        let knockback = knockback_dir.normalize_or_zero() * knockback_strength;
        player_velocity.0 += knockback;
    }
}
