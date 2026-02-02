use bevy::prelude::*;

use crate::components::*;
use crate::resources::EnemyCount;
use crate::systems::enemy::*;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EnemyCount>().add_systems(
            Update,
            (
                track_enemy_count,
                enemy_ai,
                enemy_attack,
                spawn_enemies,
                check_enemy_death,
                handle_status_applications,
                update_elemental_statuses,
                handle_mastery_effects,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

fn track_enemy_count(enemies: Query<(), With<Enemy>>, mut count: ResMut<EnemyCount>) {
    count.0 = enemies.iter().len();
}
