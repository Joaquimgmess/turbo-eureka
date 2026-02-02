use bevy::prelude::*;

use crate::components::*;
use crate::systems::selection::*;

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::CharacterSelection),
            setup_class_selection,
        )
        .add_systems(
            Update,
            handle_class_selection.run_if(in_state(GameState::CharacterSelection)),
        )
        .add_systems(OnExit(GameState::CharacterSelection), despawn_selection_ui)
        .add_systems(OnEnter(GameState::PetSelection), setup_pet_selection)
        .add_systems(
            Update,
            handle_pet_selection.run_if(in_state(GameState::PetSelection)),
        )
        .add_systems(OnExit(GameState::PetSelection), despawn_selection_ui)
        .add_systems(OnEnter(GameState::Playing), start_game)
        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .add_systems(Update, restart_game);
    }
}
