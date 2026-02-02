use bevy::prelude::*;
use rand::Rng;

use crate::components::*;
use crate::constants::*;
use crate::resources::*;

pub struct GameFeelPlugin;

impl Plugin for GameFeelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HitStop>()
            .init_resource::<KillFeedback>()
            .init_resource::<SlowMotion>()
            .add_systems(Startup, setup_camera_shake)
            .add_systems(
                Update,
                (
                    camera_follow,
                    update_screen_shake.after(camera_follow),
                    update_hit_stop,
                    flash_on_damage,
                    update_level_up_rings,
                    update_projectile_trails,
                    update_trail_particles,
                    update_muzzle_flash,
                    update_attack_recoil,
                    update_death_particles,
                    update_slow_motion,
                    update_kill_feedback,
                    update_knockback,
                    update_camera_zoom_punch,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

fn setup_camera_shake(mut commands: Commands, camera: Query<Entity, With<Camera2d>>) {
    if let Ok(entity) = camera.get_single() {
        commands.entity(entity).insert((
            CameraShake {
                trauma: 0.0,
                direction: None,
            },
            CameraTarget::default(),
        ));
    }
}

const CAMERA_LERP_FACTOR: f32 = 5.0;
const CAMERA_DEAD_ZONE: f32 = 20.0;
const CAMERA_LOOK_AHEAD_FACTOR: f32 = 50.0;

fn camera_follow(
    player: Query<(&Transform, &Velocity), With<Player>>,
    mut camera: Query<&mut CameraTarget, With<Camera2d>>,
    time: Res<Time>,
) {
    let Ok((player_transform, velocity)) = player.get_single() else {
        return;
    };
    let Ok(mut camera_target) = camera.get_single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let look_ahead = if velocity.0.length_squared() > 0.1 {
        velocity.0.normalize() * CAMERA_LOOK_AHEAD_FACTOR
    } else {
        Vec2::ZERO
    };

    let target_pos = player_pos + look_ahead;
    let diff = target_pos - camera_target.position;

    if diff.length() > CAMERA_DEAD_ZONE {
        let lerp_factor = CAMERA_LERP_FACTOR * time.delta_seconds();
        camera_target.position = camera_target
            .position
            .lerp(target_pos, lerp_factor.min(1.0));
    }
}

fn update_screen_shake(
    time: Res<Time>,
    mut rng: ResMut<GameRng>,
    mut camera: Query<(&mut Transform, &mut CameraShake, &CameraTarget), With<Camera2d>>,
) {
    let Ok((mut transform, mut shake, camera_target)) = camera.get_single_mut() else {
        return;
    };

    let base_x = camera_target.position.x;
    let base_y = camera_target.position.y;

    if shake.trauma > 0.0 {
        let shake_amount = shake.trauma * shake.trauma;

        let (dir_bias_x, dir_bias_y) = if let Some(dir) = shake.direction {
            let normalized = dir.normalize_or_zero();
            (normalized.x * 0.6, normalized.y * 0.6)
        } else {
            (0.0, 0.0)
        };

        let offset_x = SCREEN_SHAKE_MAX
            * shake_amount
            * (dir_bias_x + rng.0.gen_range(-0.5..0.5) * (1.0 - dir_bias_x.abs()));
        let offset_y = SCREEN_SHAKE_MAX
            * shake_amount
            * (dir_bias_y + rng.0.gen_range(-0.5..0.5) * (1.0 - dir_bias_y.abs()));
        let rotation = 0.06 * shake_amount * rng.0.gen_range(-1.0..1.0);

        transform.translation.x = base_x + offset_x;
        transform.translation.y = base_y + offset_y;
        transform.rotation = Quat::from_rotation_z(rotation);

        shake.trauma = (shake.trauma - 3.5 * time.delta_seconds()).max(0.0);

        if shake.trauma <= 0.0 {
            shake.direction = None;
        }
    } else {
        transform.translation.x = base_x;
        transform.translation.y = base_y;
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
            let intensity = 1.0 - (t * t);
            sprite.color = Color::srgb(1.0, 1.0, 1.0).mix(&flash.original_color, 1.0 - intensity);
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
    commands.entity(entity).try_insert(DamageFlash {
        timer: Timer::from_seconds(DAMAGE_FLASH_DURATION, TimerMode::Once),
        original_color: current_color,
    });
}

fn update_projectile_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(&Transform, &mut ProjectileTrail, &Velocity)>,
) {
    for (transform, mut trail, velocity) in projectiles.iter_mut() {
        trail.spawn_timer.tick(time.delta());
        if trail.spawn_timer.just_finished() {
            let speed = velocity.0.length();
            let size = 12.0 + (speed / 100.0).min(8.0);
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: trail.color,
                        custom_size: Some(Vec2::splat(size)),
                        ..default()
                    },
                    transform: Transform::from_translation(
                        transform.translation - velocity.0.normalize().extend(0.0) * 10.0,
                    ),
                    ..default()
                },
                TrailParticle {
                    lifetime: Timer::from_seconds(0.15, TimerMode::Once),
                    initial_size: size,
                },
            ));
        }
    }
}

fn update_trail_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Sprite, &mut TrailParticle)>,
) {
    for (entity, mut sprite, mut particle) in particles.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            let t = particle.lifetime.fraction();
            let size = particle.initial_size * (1.0 - t);
            let alpha = 0.6 * (1.0 - t);
            sprite.custom_size = Some(Vec2::splat(size));
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

fn update_muzzle_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut Sprite, &mut MuzzleFlash, &mut Transform)>,
) {
    for (entity, mut sprite, mut flash, mut transform) in flashes.iter_mut() {
        flash.lifetime.tick(time.delta());
        if flash.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            let t = flash.lifetime.fraction();
            let scale = 1.0 + t * 0.5;
            let alpha = 0.9 * (1.0 - t);
            transform.scale = Vec3::splat(scale);
            sprite.color = sprite.color.with_alpha(alpha);
        }
    }
}

fn update_attack_recoil(
    mut commands: Commands,
    time: Res<Time>,
    mut players: Query<(Entity, &mut Transform, &mut AttackRecoil), With<Player>>,
) {
    for (entity, mut transform, mut recoil) in players.iter_mut() {
        recoil.timer.tick(time.delta());
        if recoil.timer.finished() {
            commands.entity(entity).remove::<AttackRecoil>();
        } else {
            let t = recoil.timer.fraction();
            let ease = if t < 0.5 { t * 2.0 } else { 2.0 - t * 2.0 };
            let offset = recoil.direction * recoil.recoil_amount * ease;
            transform.translation.x += offset.x * time.delta_seconds() * 60.0;
            transform.translation.y += offset.y * time.delta_seconds() * 60.0;
        }
    }
}

fn update_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Transform, &mut Sprite, &mut DeathParticle)>,
) {
    for (entity, mut transform, mut sprite, mut particle) in particles.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
        } else {
            transform.translation += (particle.velocity * time.delta_seconds()).extend(0.0);
            particle.velocity.y -= 400.0 * time.delta_seconds();
            transform.rotate_z(particle.rotation_speed * time.delta_seconds());
            let t = particle.lifetime.fraction();
            let alpha = 1.0 - t;
            sprite.color = sprite.color.with_alpha(alpha);
            let scale = 1.0 - t * 0.5;
            transform.scale = Vec3::splat(scale);
        }
    }
}

fn update_slow_motion(time: Res<Time>, mut slow_mo: ResMut<SlowMotion>) {
    if slow_mo.active {
        slow_mo.timer.tick(time.delta());
        if slow_mo.timer.finished() {
            slow_mo.active = false;
            slow_mo.time_scale = 1.0;
        }
    }
}

fn update_kill_feedback(time: Res<Time>, mut kill_feedback: ResMut<KillFeedback>) {
    kill_feedback.kill_timer.tick(time.delta());
    if kill_feedback.kill_timer.finished() {
        kill_feedback.recent_kills = 0;
        kill_feedback.last_kill_was_crit = false;
    }
}

pub fn trigger_slow_motion(slow_mo: &mut SlowMotion, duration: f32, time_scale: f32) {
    slow_mo.timer = Timer::from_seconds(duration, TimerMode::Once);
    slow_mo.time_scale = time_scale;
    slow_mo.active = true;
}

pub fn spawn_death_particles(commands: &mut Commands, position: Vec3, color: Color, count: usize) {
    let mut rng = rand::thread_rng();
    for _ in 0..count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let speed = rng.gen_range(150.0..350.0);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed + 100.0);
        let size = rng.gen_range(8.0..18.0);
        let rotation_speed = rng.gen_range(-10.0..10.0);
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                transform: Transform::from_translation(position),
                ..default()
            },
            DeathParticle {
                velocity,
                lifetime: Timer::from_seconds(rng.gen_range(0.4..0.8), TimerMode::Once),
                rotation_speed,
            },
        ));
    }
}

fn update_knockback(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut KnockbackEffect)>,
) {
    for (entity, mut transform, mut knockback) in query.iter_mut() {
        knockback.timer.tick(time.delta());

        if knockback.timer.finished() {
            commands.entity(entity).remove::<KnockbackEffect>();
        } else {
            let t = 1.0 - knockback.timer.fraction();
            let ease = t * t;
            let offset = knockback.direction * knockback.force * ease * time.delta_seconds();
            transform.translation.x += offset.x;
            transform.translation.y += offset.y;
        }
    }
}

fn update_camera_zoom_punch(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut OrthographicProjection, &mut CameraZoomPunch), With<Camera2d>>,
) {
    for (entity, mut projection, mut zoom_punch) in query.iter_mut() {
        zoom_punch.timer.tick(time.delta());

        if zoom_punch.timer.finished() {
            if zoom_punch.returning {
                projection.scale = zoom_punch.base_scale;
                commands.entity(entity).remove::<CameraZoomPunch>();
            } else {
                zoom_punch.returning = true;
                zoom_punch.timer = Timer::from_seconds(ZOOM_RETURN_DURATION, TimerMode::Once);
            }
        } else {
            let t = zoom_punch.timer.fraction();
            let zoom_scale = 1.0 - ZOOM_PUNCH_AMOUNT;
            if zoom_punch.returning {
                let ease = t;
                projection.scale =
                    (zoom_punch.base_scale * zoom_scale).lerp(zoom_punch.base_scale, ease);
            } else {
                let ease = 1.0 - (1.0 - t).powi(2);
                projection.scale = zoom_punch
                    .base_scale
                    .lerp(zoom_punch.base_scale * zoom_scale, ease);
            }
        }
    }
}

pub fn add_directional_trauma(shake: &mut CameraShake, amount: f32, direction: Vec2) {
    shake.trauma = (shake.trauma + amount).min(1.0);
    shake.direction = Some(direction);
}

pub fn trigger_hit_stop_by_type(hit_stop: &mut HitStop, hit_type: HitType) {
    let frames = match hit_type {
        HitType::Normal => HITSTOP_NORMAL_FRAMES,
        HitType::Crit => HITSTOP_CRIT_FRAMES,
        HitType::Kill => HITSTOP_KILL_FRAMES,
    };
    trigger_hit_stop(hit_stop, frames * FRAME_MS);
}

pub fn apply_knockback(
    commands: &mut Commands,
    target: Entity,
    direction: Vec2,
    force: f32,
    size: EnemySize,
) {
    let size_mult = match size {
        EnemySize::Small => KNOCKBACK_SMALL_MULT,
        EnemySize::Medium => KNOCKBACK_MEDIUM_MULT,
        EnemySize::Large => KNOCKBACK_LARGE_MULT,
        EnemySize::Boss => KNOCKBACK_BOSS_MULT,
    };

    commands.entity(target).try_insert(KnockbackEffect {
        direction: direction.normalize_or_zero(),
        force: force * size_mult,
        timer: Timer::from_seconds(KNOCKBACK_DURATION, TimerMode::Once),
    });
}

pub fn trigger_zoom_punch(commands: &mut Commands, camera_entity: Entity, current_scale: f32) {
    commands.entity(camera_entity).try_insert(CameraZoomPunch {
        timer: Timer::from_seconds(ZOOM_PUNCH_DURATION, TimerMode::Once),
        returning: false,
        base_scale: current_scale,
    });
}
