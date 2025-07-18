use bevy::prelude::*;

use crate::GameState;

use crate::gameplay::enemies::melee_creeps::melee_creep::MeleeCreepPlugin;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Enemy {
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    // Add more fields as needed for Skein or your systems
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Enemy>()
            .add_plugins(MeleeCreepPlugin)
            .add_systems(
                Update,
                (enemy_death_system,).run_if(in_state(GameState::InGame)),
            );
    }
}

fn enemy_death_system(mut commands: Commands, query: Query<(Entity, &Enemy)>) {
    for (entity, enemy) in &query {
        if enemy.health <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
