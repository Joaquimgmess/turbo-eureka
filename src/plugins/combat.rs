use bevy::prelude::*;

use crate::components::*;
use crate::events::*;
use crate::systems::combat::*;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<SpawnXpOrbEvent>()
            .add_event::<ApplyStatusEvent>()
            .add_systems(
                Update,
                (
                    update_projectiles,
                    update_melee_attacks,
                    update_aoe_effects,
                    process_damage,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
