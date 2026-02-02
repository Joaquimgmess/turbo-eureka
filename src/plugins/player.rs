use bevy::prelude::*;

use crate::components::*;
use crate::systems::player::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_movement,
                player_attack,
                player_skills,
                update_dash,
                update_invulnerability,
                regen_health,
                check_player_death,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}
