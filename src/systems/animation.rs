use crate::components::*;
use crate::resources::*;
use bevy::prelude::*;

pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas)>,
) {
    for (mut config, mut atlas) in query.iter_mut() {
        config.timer.tick(time.delta());
        if config.timer.just_finished() {
            atlas.index = (atlas.index + 1) % config.frame_count;
        }
    }
}

pub fn update_character_animation_texture(
    sprites: Res<CharacterSprites>,
    mut query: Query<
        (
            &CharacterState,
            Option<&Player>,
            Option<&Enemy>,
            &mut Handle<Image>,
            &mut TextureAtlas,
        ),
        Changed<CharacterState>,
    >,
) {
    for (state, player, enemy, mut image, mut atlas) in query.iter_mut() {
        if player.is_some() {
            *image = match state {
                CharacterState::Idle => sprites.soldier_idle.clone(),
                CharacterState::Walking => sprites.soldier_walk.clone(),
                CharacterState::Attacking => sprites.soldier_attack.clone(),
                _ => sprites.soldier_idle.clone(),
            };
        } else if enemy.is_some() {
            *image = match state {
                CharacterState::Idle => sprites.orc_idle.clone(),
                CharacterState::Walking => sprites.orc_walk.clone(),
                CharacterState::Attacking => sprites.orc_attack.clone(),
                _ => sprites.orc_idle.clone(),
            };
        }
        atlas.index = 0; // Reset animation when state changes
    }
}
