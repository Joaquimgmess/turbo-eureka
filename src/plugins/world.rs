use bevy::prelude::*;

use crate::components::*;
use crate::resources::*;
use crate::systems::world::*;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapData::default())
            .insert_resource(MapTier(1))
            .add_systems(OnEnter(GameState::Playing), setup_minimap)
            .add_systems(
                Update,
                (
                    update_cursor_world_pos,
                    generate_map,
                    update_minimap,
                    update_xp_orbs,
                    collect_xp,
                    update_hazards,
                    handle_loot,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
