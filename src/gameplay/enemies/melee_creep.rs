use crate::GameState;
use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    // Add more fields as needed for Skein or your systems
}

// Optionally, you can add a marker for melee creeps:
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct MeleeCreep;

pub struct MeleeCreepPlugin;

impl Plugin for MeleeCreepPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .register_type::<MeleeCreep>()
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
        (Entity, &Enemy, &GlobalTransform, &mut LinearVelocity),
        With<MeleeCreep>,
    >,
) {
    if let Some(player_transform) = player_query.iter().next() {
        let mut player_pos = player_transform.translation();
        player_pos.y = 0.0;

        // Collect creep positions for repulsion
        let creep_positions: Vec<_> = creep_query
            .iter()
            .map(|(e, _, t, _)| (e, t.translation()))
            .collect();

        for (entity, enemy, creep_transform, mut velocity) in &mut creep_query {
            let mut creep_pos = creep_transform.translation();
            creep_pos.y = 0.0;

            // Move toward player
            let mut direction = (player_pos - creep_pos).normalize_or_zero();

            // Boids-style repulsion from other creeps (2m cube)
            let mut repulsion = Vec3::ZERO;
            let min_separation = 3.5;
            for (other_entity, other_pos) in &creep_positions {
                if *other_entity != entity {
                    let mut other_pos = *other_pos;
                    other_pos.y = 0.0;
                    let dist = creep_pos.distance(other_pos);
                    if dist < min_separation && dist > 0.0 {
                        repulsion += (creep_pos - other_pos).normalize() / dist;
                    }
                }
            }
            direction += repulsion * 0.5; // Tune repulsion strength

            velocity.0 = direction.normalize_or_zero() * enemy.speed;
        }
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
        println!("Player hit by melee creep! Health: {}", player_health.0);

        // Knockback: push player away from creep
        let mut knockback_dir = player_transform.translation() - creep_transform.translation();
        knockback_dir.y = 0.0; // Only knock back on XZ plane
        let knockback_strength = 8.0; // Tune this value as needed

        let knockback = knockback_dir.normalize_or_zero() * knockback_strength;
        player_velocity.0 += knockback;
    }
}
