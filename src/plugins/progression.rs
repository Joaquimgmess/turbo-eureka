use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::progression::*;

pub struct ProgressionPlugin;

impl Plugin for ProgressionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProgressionEventTimer>()
            .init_resource::<HordeWaveActive>()
            .insert_resource(MiniBossTimer::new())
            .init_resource::<BossEntranceActive>()
            .init_resource::<BossDeathActive>()
            .add_systems(
                Update,
                (
                    tick_progression_event_timer,
                    spawn_progression_event,
                    update_horde_wave,
                    treasure_goblin_ai,
                    update_elite_aura,
                    follow_elite_aura,
                    interact_with_shrine,
                    apply_shrine_buffs,
                    update_challenge_zones,
                    cleanup_used_shrines,
                    spawn_mini_boss,
                    boss_entrance_effect,
                    boss_death_effect,
                    spawn_boss_with_entrance,
                    handle_boss_death,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
