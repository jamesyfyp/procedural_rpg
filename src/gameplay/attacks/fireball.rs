use avian3d::prelude::*;
use bevy::{color::palettes::css, prelude::*};

use crate::GameState;
use crate::gameplay::enemies::melee_creep::Enemy;

pub struct FireballPlugin;

impl Plugin for FireballPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            fireball_collision_system.run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component)]
pub struct Fireball {
    pub damage: f32,
}

pub fn spawn_fireball(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    spawn_pos: Vec3,
    direction: Dir3,
    damage: f32,
    speed: f32,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere { radius: 0.3 })),
        MeshMaterial3d(materials.add(Color::from(css::DARK_ORANGE))),
        Transform::from_translation(spawn_pos),
        RigidBody::Dynamic,
        Collider::sphere(0.3),
        LinearVelocity(direction * speed),
        Fireball { damage },
        CollisionEventsEnabled,
    ));
}

fn fireball_collision_system(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionStarted>,
    mut enemy_query: Query<(&mut Enemy, Entity)>,
    fireball_query: Query<&Fireball>,
) {
    for CollisionStarted(e1, e2) in collision_events.read() {
        let (fireball_entity, enemy_entity) =
            if fireball_query.get(*e1).is_ok() && enemy_query.get(*e2).is_ok() {
                (*e1, *e2)
            } else if fireball_query.get(*e2).is_ok() && enemy_query.get(*e1).is_ok() {
                (*e2, *e1)
            } else {
                continue;
            };

        let fireball = fireball_query.get(fireball_entity).unwrap();
        if let Ok((mut enemy, enemy_entity)) = enemy_query.get_mut(enemy_entity) {
            enemy.health -= fireball.damage;
            println!("Enemy hit! Health: {}", enemy.health);
            commands.entity(fireball_entity).despawn();
        }
    }
}
