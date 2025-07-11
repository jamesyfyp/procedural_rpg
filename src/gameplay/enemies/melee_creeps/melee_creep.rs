//TODO: refactor this
//TODO: do some research on animation graphs, see how if avain does anything to help here

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

#[derive(Resource)]
struct MeleeCreeps {
    count: usize,
    speed: f32,
    moving: bool,
    sync: bool,
}

#[derive(Resource)]
struct Animations {
    node_indices: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

pub struct MeleeCreepPlugin;

impl Plugin for MeleeCreepPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .register_type::<MeleeCreep>()
            .insert_resource(MeleeCreeps {
                count: 2,
                speed: 2.0,
                moving: true,
                sync: false,
            })
            .add_systems(OnEnter(GameState::InGame), set_up)
            .add_systems(
                Update,
                (
                    setup_scene_once_loaded,
                    melee_creep_movement_system,
                    melee_creep_damage_system,
                    enemy_death_system,
                    melee_creep_animation_switch_system,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn set_up(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    melee_creeps: Res<MeleeCreeps>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    let animation_clips = [
        asset_server
            .load(GltfAssetLabel::Animation(0).from_asset("enemies/gooper/gooper_test.gltf")),
        asset_server
            .load(GltfAssetLabel::Animation(1).from_asset("enemies/gooper/gooper_test.gltf")),
    ];
    let mut animation_graph = AnimationGraph::new();
    let node_indices = animation_graph
        .add_clips(animation_clips.iter().cloned(), 1.0, animation_graph.root)
        .collect();
    commands.insert_resource(Animations {
        node_indices,
        graph: animation_graphs.add(animation_graph),
    });

    let gooper_handle =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("enemies/gooper/gooper_test.gltf"));

    for i in 0..melee_creeps.count {
        let x = (i % 10) as f32 * 2.0;
        let z = (i / 10) as f32 * 2.0;
        commands.spawn((
            SceneRoot(gooper_handle.clone()),
            Transform::from_xyz(x, 0.5, z).with_scale(Vec3::splat(0.5)),
            RigidBody::Kinematic,
            Enemy {
                health: 100.0,
                speed: melee_creeps.speed,
                damage: 10.0,
            },
            MeleeCreep,
            ColliderConstructor::Capsule {
                radius: 1.,
                height: 1.5,
            },
            CollisionEventsEnabled,
        ));
    }
}

fn setup_scene_once_loaded(
    animations: Res<Animations>,
    melee_creeps: Res<MeleeCreeps>,
    mut commands: Commands,
    mut animation_player: Query<(Entity, &mut AnimationPlayer)>,
    mut done: Local<bool>,
) {
    if !*done && animation_player.iter().len() == melee_creeps.count {
        for (entity, mut animation_player) in &mut animation_player {
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(animations.graph.clone()))
                .insert(AnimationTransitions::new());
        }
        *done = true;
    }
}

//gameplay logic for spawned creeps
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

        // Rotate via Rotation component
        if dir.length_squared() > 0.0001 {
            let angle = dir.x.atan2(-dir.z);
            rotation.0 = Quat::from_rotation_y(angle);
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

        // Knockback: push player away from creep
        let mut knockback_dir = player_transform.translation() - creep_transform.translation();
        knockback_dir.y = 0.0; // Only knock back on XZ plane
        let knockback_strength = 8.0; // Tune this value as needed

        let knockback = knockback_dir.normalize_or_zero() * knockback_strength;
        player_velocity.0 += knockback;
    }
}

fn enemy_death_system(mut commands: Commands, query: Query<(Entity, &Enemy)>) {
    for (entity, enemy) in &query {
        if enemy.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn melee_creep_animation_switch_system(
    player_query: Query<&GlobalTransform, With<crate::player::Player>>,
    transforms: Query<&GlobalTransform>,
    mut anim_players: Query<(Entity, &mut AnimationPlayer)>,
    animations: Res<Animations>,
) {
    let Some(player_transform) = player_query.iter().next() else {
        return;
    };
    let player_pos = player_transform.translation();
    let attack_range = 3.0;

    println!(
        "Found {} entities with AnimationPlayer",
        anim_players.iter().len()
    );

    for (entity, mut animation_player) in &mut anim_players {
        if let Ok(creep_transform) = transforms.get(entity) {
            let creep_pos = creep_transform.translation();
            let distance = player_pos.distance(creep_pos);

            let desired_animation = if distance <= attack_range {
                animations.node_indices[0] // Attack
            } else {
                animations.node_indices[1] // Walk
            };

            animation_player.play(desired_animation).repeat();
        }
    }
}

// fn melee_creep_animation_switch_system(
//     player_query: Query<&GlobalTransform, With<crate::player::Player>>,
//     mut query: Query<
//         (
//             &GlobalTransform,
//             &mut AnimationTransitions,
//             &mut AnimationPlayer,
//         ),
//         With<MeleeCreep>,
//     >,
//     animations: Res<Animations>,
// ) {
//     let Some(player_tf) = player_query.iter().next() else {
//         return;
//     };
//     let player_pos = player_tf.translation();
//     let attack_range = 3.0;

//     for (creep_tf, mut transitions, mut anim_player) in &mut query {
//         let creep_pos = creep_tf.translation();
//         let distance = player_pos.distance(creep_pos);

//         let desired_animation = if distance <= attack_range {
//             animations.node_indices[0] // Attack
//         } else {
//             animations.node_indices[1] // Walk
//         };

//         // Pass mutable AnimationPlayer and Duration
//         // if anim_player != Some(desired_animation) {
//         //     transitions.play(
//         //         &mut anim_player,
//         //         desired_animation,
//         //         Duration::from_secs_f32(0.2),
//         //     );
//         // }
//     }
// }
