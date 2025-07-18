use avian3d::prelude::*;
use bevy::{prelude::*, scene};

use crate::GameState;
use crate::gameplay::enemies::enemy::Enemy;
use crate::gameplay::enemies::melee_creeps::melee_creep::MeleeCreep;
#[derive(Resource)]
struct Animations {
    node_indices: Vec<AnimationNodeIndex>,
    graph: Handle<AnimationGraph>,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Gooper;

pub struct GooperPlugin;

impl Plugin for GooperPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), set_up);
        app.add_systems(
            Update,
            (setup_scene_once_loaded, melee_creep_animation_switch_system)
                .run_if(in_state(GameState::InGame)),
        );
    }
}

fn set_up(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
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

    for i in 0..20 {
        let x = (i % 10) as f32 * 2.0;
        let z = (i / 10) as f32 * 2.0;
        commands.spawn((
            SceneRoot(gooper_handle.clone()),
            Transform::from_xyz(x, 0.5, z).with_scale(Vec3::splat(0.5)),
            RigidBody::Kinematic,
            Enemy {
                health: 100.0,
                speed: 2.0,
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
    mut commands: Commands,
    mut animation_player: Query<(Entity, &mut AnimationPlayer)>,
    mut done: Local<bool>,
) {
    if !*done && animation_player.iter().len() == 20 {
        for (entity, mut animation_player) in &mut animation_player {
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(animations.graph.clone()))
                .insert(AnimationTransitions::new());
        }
        *done = true;
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
