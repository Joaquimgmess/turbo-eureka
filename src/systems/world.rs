use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;

pub fn update_cursor_world_pos(
    mut cursor_pos: ResMut<CursorWorldPos>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
) {
    let Ok(window) = window_query.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor))
    {
        cursor_pos.0 = world_position;
    }
}

pub fn update_xp_orbs(mut commands: Commands, mut xp_events: EventReader<SpawnXpOrbEvent>) {
    for event in xp_events.read() {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.3, 0.85, 1.0),
                    custom_size: Some(Vec2::splat(12.0)),
                    ..default()
                },
                transform: Transform::from_translation(event.position),
                ..default()
            },
            XpOrb { value: event.value },
            Lifetime(Timer::from_seconds(12.0, TimerMode::Once)),
        ));
    }
}

pub fn collect_xp(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(&Transform, Entity), With<Player>>,
    mut xp_orbs: Query<(Entity, &mut Transform, &XpOrb, &mut Lifetime), Without<Player>>,
    mut levels: Query<(&mut Level, &mut Stats, &mut Health, &mut PlayerPassives)>,
) {
    let Ok((player_transform, player_entity)) = player_query.get_single() else {
        return;
    };
    let Ok((mut level, mut stats, mut health, mut passives)) = levels.get_mut(player_entity) else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (orb_entity, mut orb_transform, xp_orb, mut lifetime) in xp_orbs.iter_mut() {
        lifetime.0.tick(time.delta());

        if lifetime.0.finished() {
            commands.entity(orb_entity).despawn();
            continue;
        }

        let orb_pos = orb_transform.translation.truncate();
        let distance = orb_pos.distance(player_pos);

        if distance < 120.0 {
            let direction = (player_pos - orb_pos).normalize_or_zero();
            let speed = 250.0 * (1.0 - distance / 120.0) + 80.0;
            orb_transform.translation += (direction * speed * time.delta_seconds()).extend(0.0);
        }

        if distance < 22.0 {
            if level.add_xp(xp_orb.value) {
                stats.damage *= 1.12;
                stats.speed *= 1.02;
                stats.attack_speed *= 1.03;
                stats.crit_chance = (stats.crit_chance + 0.015).min(0.5);
                stats.life_regen += 0.4;
                stats.armor += 4.0;

                health.max *= 1.12;
                health.current = health.max;

                passives.points += 1;
            }

            commands.entity(orb_entity).despawn();
        }
    }
}

pub fn spawn_boss(
    mut commands: Commands,
    time: Res<Time>,
    mut game_stats: ResMut<GameStats>,
    map_tier: Res<MapTier>,
    sprites: Res<CharacterSprites>,
    player_query: Query<&Transform, With<Player>>,
    boss_query: Query<&Boss>,
) {
    let spawn_interval = 120.0;
    let current_time = game_stats.time_survived;
    let last_time = current_time - time.delta_seconds();

    if (current_time / spawn_interval).floor() > (last_time / spawn_interval).floor()
        && boss_query.iter().count() == 0
    {
        let Ok(player_transform) = player_query.get_single() else {
            return;
        };
        let spawn_pos = player_transform.translation.truncate() + Vec2::new(0.0, 400.0);

        let tier_scale = 1.0 + (map_tier.0 as f32 - 1.0) * 0.5;

        commands.spawn((
            Boss,
            Enemy {
                damage: 45.0 * tier_scale,
                xp_value: 500 * map_tier.0,
                attack_cooldown: Timer::from_seconds(0.8, TimerMode::Once),
                speed: 120.0 + (map_tier.0 as f32 * 5.0),
            },
            ElementalStatus::default(),
            Health {
                current: 2000.0 * tier_scale,
                max: 2000.0 * tier_scale,
            },
            SpriteBundle {
                texture: sprites.orc_idle.clone(),
                sprite: Sprite {
                    color: Color::srgb(0.3, 0.1, 0.4),
                    custom_size: Some(Vec2::splat(350.0)),
                    ..default()
                },
                transform: Transform::from_translation(spawn_pos.extend(6.0)),
                ..default()
            },
            TextureAtlas {
                layout: sprites.layout.clone(),
                index: 0,
            },
            AnimationConfig {
                timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                frame_count: 6,
                state: CharacterState::Idle,
            },
        ));
    }
}

pub fn update_hazards(
    _commands: Commands,
    _time: Res<Time>,
    _player_query: Query<(Entity, &Transform), (With<Player>, Without<Hazard>)>,
    _hazards: Query<
        (Entity, &mut Transform, &Hazard, &mut Sprite),
        (With<Hazard>, Without<Player>),
    >,
    _damage_events: EventWriter<DamageEvent>,
    _status_events: EventWriter<ApplyStatusEvent>,
) {
}

pub fn handle_loot(
    mut commands: Commands,
    player_query: Query<&Transform, With<Player>>,
    loot_query: Query<(Entity, &Transform), With<Loot>>,
    mut stats_query: Query<&mut Stats, With<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (entity, transform) in loot_query.iter() {
        if transform.translation.truncate().distance(player_pos) < 50.0 {
            if let Ok(mut stats) = stats_query.get_single_mut() {
                stats.damage += 2.0;
                stats.crit_chance += 0.01;
                commands.entity(entity).despawn();
            }
        }
    }
}
