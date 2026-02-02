use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

pub fn enemy_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    taunt_query: Query<&Transform, (With<Taunt>, Without<Enemy>, Without<Player>)>,
    mut enemies: Query<(&mut Transform, &Enemy, &mut CharacterState, &mut Sprite), Without<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut transform, enemy, mut state, mut sprite) in enemies.iter_mut() {
        let enemy_pos = transform.translation.truncate();

        let mut target_pos = player_pos;
        let mut min_dist = enemy_pos.distance(player_pos);

        for taunt_transform in taunt_query.iter() {
            let dist = enemy_pos.distance(taunt_transform.translation.truncate());
            if dist < 300.0 && dist < min_dist {
                min_dist = dist;
                target_pos = taunt_transform.translation.truncate();
            }
        }

        let to_target = target_pos - enemy_pos;
        let distance = to_target.length();

        if distance > 80.0 {
            let direction = to_target.normalize();
            transform.translation += (direction * enemy.speed * time.delta_seconds()).extend(0.0);

            if *state != CharacterState::Attacking {
                *state = CharacterState::Walking;
            }

            if direction.x < 0.0 {
                sprite.flip_x = true;
            } else if direction.x > 0.0 {
                sprite.flip_x = false;
            }
        } else if *state != CharacterState::Attacking {
            *state = CharacterState::Idle;
        }
    }
}

pub fn enemy_attack(
    time: Res<Time>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy, &mut CharacterState)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (enemy_entity, transform, mut enemy, mut state) in enemies.iter_mut() {
        enemy.attack_cooldown.tick(time.delta());

        if enemy.attack_cooldown.finished() && *state == CharacterState::Attacking {
            *state = CharacterState::Idle;
        }

        let enemy_pos = transform.translation.truncate();
        let distance = enemy_pos.distance(player_pos);

        if distance < 85.0 && enemy.attack_cooldown.finished() {
            enemy.attack_cooldown = Timer::from_seconds(1.0, TimerMode::Once);
            *state = CharacterState::Attacking;

            damage_events.send(DamageEvent {
                target: player_entity,
                attacker: Some(enemy_entity),
                amount: enemy.damage,
                is_crit: false,
            });
        }
    }
}

pub fn spawn_enemies(
    mut commands: Commands,
    sprites: Res<CharacterSprites>,
    time: Res<Time>,
    mut game_stats: ResMut<GameStats>,
    player_query: Query<(&Transform, &Level), With<Player>>,
    enemies: Query<&Enemy>,
) {
    let Ok((player_transform, player_level)) = player_query.get_single() else {
        return;
    };

    game_stats.time_survived += time.delta_seconds();

    let max_enemies = (8 + player_level.level * 2).min(30) as usize;
    if enemies.iter().count() >= max_enemies {
        return;
    }

    let spawn_chance = 0.025 + (game_stats.time_survived / 200.0).min(0.06);

    if rand::thread_rng().r#gen::<f32>() > spawn_chance {
        return;
    }

    let player_pos = player_transform.translation.truncate();

    let mut rng = rand::thread_rng();
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let distance = rng.gen_range(350.0..550.0);
    let spawn_pos = player_pos + Vec2::from_angle(angle) * distance;

    let health_scale = 1.0 + (player_level.level as f32 - 1.0) * 0.25;
    let damage_scale = 1.0 + (player_level.level as f32 - 1.0) * 0.15;

    let enemy_type = rng.gen_range(0..3);
    let (size, color, health, damage, xp, speed) = match enemy_type {
        0 => (Vec2::new(170.0, 170.0), Color::WHITE, 35.0, 10.0, 12, 85.0),
        1 => (
            Vec2::new(220.0, 220.0),
            Color::srgb(1.0, 0.6, 0.6),
            70.0,
            15.0,
            30,
            60.0,
        ),
        _ => (
            Vec2::new(140.0, 140.0),
            Color::srgb(0.6, 0.4, 0.9),
            22.0,
            18.0,
            18,
            130.0,
        ),
    };

    let enemy_entity = commands
        .spawn((
            Enemy {
                damage: damage * damage_scale,
                xp_value: xp,
                attack_cooldown: Timer::from_seconds(1.0, TimerMode::Once),
                speed,
            },
            ElementalStatus::default(),
            Health {
                current: health * health_scale,
                max: health * health_scale,
            },
            Velocity(Vec2::ZERO),
            CharacterState::Idle,
            SpriteBundle {
                texture: sprites.orc_idle.clone(),
                sprite: Sprite {
                    color,
                    custom_size: Some(size),
                    ..default()
                },
                transform: Transform::from_translation(spawn_pos.extend(5.0)),
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
        ))
        .id();

    commands.entity(enemy_entity).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(size.x + 8.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.1),
                ..default()
            },
            HealthBar,
        ));

        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.85, 0.1, 0.1),
                    custom_size: Some(Vec2::new(size.x + 6.0, 3.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 15.0, 0.2),
                ..default()
            },
            HealthBarFill(size.x + 6.0),
        ));
    });
}

pub fn check_enemy_death(
    mut commands: Commands,
    enemies: Query<(Entity, &Health, &Transform, &Enemy, Option<&Boss>)>,
    player_query: Query<(Entity, &PlayerPassives), With<Player>>,
    mut game_stats: ResMut<GameStats>,
    mut xp_events: EventWriter<SpawnXpOrbEvent>,
) {
    let Ok((player_entity, passives)) = player_query.get_single() else {
        return;
    };

    for (entity, health, transform, enemy, boss) in enemies.iter() {
        if health.current <= 0.0 {
            game_stats.enemies_killed += 1;

            xp_events.send(SpawnXpOrbEvent {
                position: transform.translation,
                value: enemy.xp_value,
            });

            if boss.is_some() {
                // Drop a relic
                commands.spawn((
                    Loot,
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgb(1.0, 0.8, 0.0),
                            custom_size: Some(Vec2::splat(25.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(transform.translation),
                        ..default()
                    },
                ));
            }

            if passives.unlocked_nodes.contains(&9) {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(1.0, 0.4, 0.0, 0.6),
                            custom_size: Some(Vec2::splat(150.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(transform.translation),
                        ..default()
                    },
                    AoeEffect {
                        damage: 40.0,
                        owner: player_entity,
                        tick_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                        duration: Timer::from_seconds(0.3, TimerMode::Once),
                        hit_this_tick: HashSet::new(),
                    },
                ));
            }

            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn check_player_death(
    player: Query<&Health, With<Player>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok(health) = player.get_single() else {
        return;
    };

    if health.current <= 0.0 {
        next_state.set(GameState::GameOver);
    }
}

pub fn handle_status_applications(
    mut events: EventReader<ApplyStatusEvent>,
    mut query: Query<&mut ElementalStatus>,
) {
    for event in events.read() {
        if let Ok(mut status) = query.get_mut(event.target) {
            match event.effect {
                PassiveEffect::ChanceFire(_) => {
                    status.fire_stacks = (status.fire_stacks + 1).min(10)
                }
                PassiveEffect::ChanceIce(_) => status.ice_stacks = (status.ice_stacks + 1).min(10),
                PassiveEffect::ChanceLightning(_) => {
                    status.lightning_stacks = (status.lightning_stacks + 1).min(10)
                }
                _ => {}
            }
        }
    }
}

pub fn update_elemental_statuses(
    time: Res<Time>,
    mut query: Query<(Entity, &mut ElementalStatus, &mut Sprite, &mut Enemy)>,
    mut damage_events: EventWriter<DamageEvent>,
    player_query: Query<Entity, With<Player>>,
) {
    let player_entity = player_query.get_single().ok();

    for (entity, mut status, mut sprite, mut enemy) in query.iter_mut() {
        // Visual feedback based on dominant status
        if status.fire_stacks > status.ice_stacks && status.fire_stacks > status.lightning_stacks {
            sprite.color = Color::srgb(1.0, 0.5, 0.5);
        } else if status.ice_stacks > status.fire_stacks
            && status.ice_stacks > status.lightning_stacks
        {
            sprite.color = Color::srgb(0.5, 0.8, 1.0);
        } else if status.lightning_stacks > status.fire_stacks
            && status.lightning_stacks > status.ice_stacks
        {
            sprite.color = Color::srgb(1.0, 1.0, 0.5);
        }

        // Fire effect: Ignite (DoT)
        if status.fire_stacks >= 10 {
            damage_events.send(DamageEvent {
                target: entity,
                attacker: player_entity,
                amount: 5.0 * time.delta_seconds() * 60.0,
                is_crit: false,
            });
        }
    }
}

pub fn handle_mastery_effects(
    mut commands: Commands,
    player_query: Query<(Entity, &PlayerPassives), With<Player>>,
    mut enemies: Query<(Entity, &mut ElementalStatus, &Transform), With<Enemy>>,
) {
    let Ok((player_entity, passives)) = player_query.get_single() else {
        return;
    };

    for (entity, mut status, transform) in enemies.iter_mut() {
        // Fire Mastery: Combustion
        if status.fire_stacks >= 10 && passives.unlocked_nodes.contains(&12) && !status.is_ignited {
            status.is_ignited = true;
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(1.0, 0.2, 0.0, 0.8),
                        custom_size: Some(Vec2::splat(200.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                },
                AoeEffect {
                    damage: 60.0,
                    owner: player_entity,
                    tick_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    duration: Timer::from_seconds(0.4, TimerMode::Once),
                    hit_this_tick: HashSet::new(),
                },
            ));
        }

        // Ice Mastery: Shatter
        if status.ice_stacks >= 10 && passives.unlocked_nodes.contains(&15) && !status.is_frozen {
            status.is_frozen = true;
            // Freeze logic could go here (stopping enemy AI)
            // For now, big damage spike
            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::srgba(0.0, 0.8, 1.0, 0.8),
                        custom_size: Some(Vec2::splat(150.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(transform.translation),
                    ..default()
                },
                AoeEffect {
                    damage: 80.0,
                    owner: player_entity,
                    tick_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    duration: Timer::from_seconds(0.2, TimerMode::Once),
                    hit_this_tick: HashSet::new(),
                },
            ));
        }
    }
}
