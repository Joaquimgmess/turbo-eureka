use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::resources::*;

pub struct GameFeelPlugin;

impl Plugin for GameFeelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HitStop>()
            .add_systems(Startup, setup_camera_shake)
            .add_systems(
                Update,
                (
                    update_screen_shake,
                    update_hit_stop,
                    flash_on_damage,
                    update_level_up_rings,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_camera_shake(mut commands: Commands, camera: Query<Entity, With<Camera2d>>) {
    if let Ok(entity) = camera.get_single() {
        commands.entity(entity).insert(CameraShake { trauma: 0.0 });
    }
}

fn update_screen_shake(
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    mut camera: Query<(&mut Transform, &mut CameraShake), With<Camera2d>>,
) {
    let Ok((mut transform, mut shake)) = camera.get_single_mut() else {
        return;
    };

    if shake.trauma > 0.0 {
        let shake_amount = shake.trauma * shake.trauma;

        let offset_x = 12.0 * shake_amount * rng.0.gen_range(-1.0..1.0);
        let offset_y = 12.0 * shake_amount * rng.0.gen_range(-1.0..1.0);
        let rotation = 0.05 * shake_amount * rng.0.gen_range(-1.0..1.0);

        transform.translation.x = offset_x;
        transform.translation.y = offset_y;
        transform.rotation = Quat::from_rotation_z(rotation);

        shake.trauma = (shake.trauma - 3.0 * time.delta_seconds()).max(0.0);
    } else {
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
        transform.rotation = Quat::IDENTITY;
    }
}

fn update_hit_stop(time: Res<Time>, mut hit_stop: ResMut<HitStop>) {
    if hit_stop.active {
        hit_stop.timer.tick(time.delta());
        if hit_stop.timer.finished() {
            hit_stop.active = false;
        }
    }
}

fn flash_on_damage(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut DamageFlash)>,
) {
    for (entity, mut sprite, mut flash) in query.iter_mut() {
        flash.timer.tick(time.delta());

        if flash.timer.finished() {
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<DamageFlash>();
        } else {
            let t = flash.timer.fraction();
            sprite.color = Color::WHITE.mix(&flash.original_color, t);
        }
    }
}

fn update_level_up_rings(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut LevelUpRing)>,
) {
    for (entity, mut sprite, mut ring) in query.iter_mut() {
        ring.timer.tick(time.delta());

        if ring.timer.finished() {
            commands.entity(entity).despawn();
        } else {
            let t = ring.timer.fraction();
            let size = 50.0 + 200.0 * t;
            let alpha = 0.6 * (1.0 - t);

            sprite.custom_size = Some(Vec2::splat(size));
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

pub fn add_trauma(shake: &mut CameraShake, amount: f32) {
    shake.trauma = (shake.trauma + amount).min(1.0);
}

pub fn trigger_hit_stop(hit_stop: &mut HitStop, duration_ms: u64) {
    hit_stop.timer = Timer::from_seconds(duration_ms as f32 / 1000.0, TimerMode::Once);
    hit_stop.active = true;
}

pub fn trigger_damage_flash(commands: &mut Commands, entity: Entity, current_color: Color) {
    commands.entity(entity).insert(DamageFlash {
        timer: Timer::from_seconds(0.12, TimerMode::Once),
        original_color: current_color,
    });
}
