use crate::components::*;
use crate::constants::*;
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
            Lifetime(Timer::from_seconds(XP_ORB_LIFETIME, TimerMode::Once)),
        ));
    }
}

pub fn collect_xp(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(&Transform, Entity), With<Player>>,
    mut xp_orbs: Query<(Entity, &mut Transform, &XpOrb, &mut Lifetime), Without<Player>>,
    mut levels: Query<(&mut Level, &mut Stats, &mut Health, &mut PlayerPassives)>,
    mut camera_shake: Query<&mut CameraShake, With<Camera2d>>,
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

        if distance < XP_ATTRACT_RADIUS {
            let direction = (player_pos - orb_pos).normalize_or_zero();
            let speed = 250.0 * (1.0 - distance / XP_ATTRACT_RADIUS) + 80.0;
            orb_transform.translation += (direction * speed * time.delta_seconds()).extend(0.0);
        }

        if distance < XP_PICKUP_RADIUS {
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

                if let Ok(mut shake) = camera_shake.get_single_mut() {
                    crate::plugins::game_feel::add_trauma(&mut shake, 0.3);
                }

                commands.spawn((
                    Text2dBundle {
                        text: Text::from_section(
                            format!("LEVEL {}!", level.level),
                            TextStyle {
                                font_size: 32.0,
                                color: Color::srgb(1.0, 0.9, 0.2),
                                ..default()
                            },
                        ),
                        transform: Transform::from_translation(
                            player_transform.translation + Vec3::new(0.0, 60.0, 100.0),
                        ),
                        ..default()
                    },
                    DamageNumber {
                        velocity: Vec2::new(0.0, 80.0),
                        lifetime: Timer::from_seconds(1.5, TimerMode::Once),
                    },
                ));

                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(1.0, 0.9, 0.2, 0.6),
                            custom_size: Some(Vec2::splat(50.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(player_transform.translation),
                        ..default()
                    },
                    LevelUpRing {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                ));
            }

            commands.entity(orb_entity).despawn();
        }
    }
}

pub fn spawn_boss(
    mut commands: Commands,
    time: Res<Time>,
    game_stats: Res<GameStats>,
    map_tier: Res<MapTier>,
    sprites: Res<CharacterSprites>,
    player_query: Query<&Transform, With<Player>>,
    boss_query: Query<&Boss>,
) {
    let spawn_interval = BOSS_SPAWN_INTERVAL;
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

pub fn generate_map(
    mut commands: Commands,
    map_tier: Res<MapTier>,
    mut map_data: ResMut<MapData>,
    walls: Query<Entity, With<Wall>>,
    obstacles: Query<Entity, With<Obstacle>>,
) {
    if map_data.seed == map_tier.0 as u64 {
        return;
    }
    map_data.seed = map_tier.0 as u64;

    for entity in walls.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in obstacles.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let bounds = map_data.bounds;
    let mut rng = rand::thread_rng();

    for x in (-bounds as i32..=bounds as i32).step_by(TILE_SIZE as usize) {
        for y in (-bounds as i32..=bounds as i32).step_by(TILE_SIZE as usize) {
            let pos = Vec2::new(x as f32, y as f32);
            commands.spawn((SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.15, 0.18),
                    custom_size: Some(Vec2::splat(TILE_SIZE - 5.0)),
                    ..default()
                },
                transform: Transform::from_translation(pos.extend(-1.0)),
                ..default()
            },));
        }
    }

    for _ in 0..20 {
        let x = rng.gen_range(-bounds..bounds);
        let y = rng.gen_range(-bounds..bounds);
        let size = rng.gen_range(40.0..100.0);

        commands.spawn((
            Obstacle,
            Wall,
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.3, 0.3, 0.35),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(x, y, 2.0)),
                ..default()
            },
        ));
    }
}

pub fn setup_minimap(mut commands: Commands, query: Query<Entity, With<MinimapUi>>) {
    if query.get_single().is_ok() {
        return;
    }
    commands
        .spawn((
            MinimapUi,
            NodeBundle {
                style: Style {
                    width: Val::Px(150.0),
                    height: Val::Px(150.0),
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.0),
                    right: Val::Px(150.0),
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                border_color: Color::srgb(0.4, 0.4, 0.4).into(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                MinimapPlayerIcon,
                NodeBundle {
                    style: Style {
                        width: Val::Px(4.0),
                        height: Val::Px(4.0),
                        position_type: PositionType::Absolute,
                        left: Val::Px(73.0),
                        bottom: Val::Px(73.0),
                        ..default()
                    },
                    background_color: Color::srgb(0.0, 1.0, 0.0).into(),
                    ..default()
                },
            ));
        });
}

pub fn update_minimap(
    mut commands: Commands,
    minimap_query: Query<Entity, With<MinimapUi>>,
    player_query: Query<&Transform, With<Player>>,
    enemies_query: Query<(Entity, &Transform, Option<&Boss>), With<Enemy>>,
    mut player_icon: Query<&mut Style, With<MinimapPlayerIcon>>,
    enemy_icons: Query<(Entity, &MinimapEnemyIcon)>,
) {
    let Ok(minimap_entity) = minimap_query.get_single() else {
        return;
    };
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let map_scale = 150.0 / (MAP_BOUNDS * 2.0);

    if let Ok(mut style) = player_icon.get_single_mut() {
        style.left = Val::Px(75.0 + player_pos.x * map_scale - 2.0);
        style.bottom = Val::Px(75.0 + player_pos.y * map_scale - 2.0);
    }

    let mut existing: std::collections::HashSet<Entity> =
        enemy_icons.iter().map(|(_, icon)| icon.0).collect();

    for (enemy_entity, transform, boss) in enemies_query.iter() {
        let pos = transform.translation.truncate();

        if existing.remove(&enemy_entity) {
        } else {
            let (color, size) = if boss.is_some() {
                (Color::srgb(1.0, 1.0, 0.0), 6.0)
            } else {
                (Color::srgb(1.0, 0.0, 0.0), 3.0)
            };

            commands.entity(minimap_entity).with_children(|parent| {
                parent.spawn((
                    MinimapEnemyIcon(enemy_entity),
                    NodeBundle {
                        style: Style {
                            width: Val::Px(size),
                            height: Val::Px(size),
                            position_type: PositionType::Absolute,
                            left: Val::Px(75.0 + pos.x * map_scale - size / 2.0),
                            bottom: Val::Px(75.0 + pos.y * map_scale - size / 2.0),
                            ..default()
                        },
                        background_color: color.into(),
                        ..default()
                    },
                ));
            });
        }
    }

    for (icon_entity, icon) in enemy_icons.iter() {
        if existing.contains(&icon.0) {
            commands.entity(icon_entity).despawn();
        }
    }
}
