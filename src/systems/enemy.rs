use crate::components::*;
use crate::events::*;
use crate::resources::*;
use bevy::prelude::*;
use rand::Rng;

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

        // Prioriza alvos com Taunt se estiverem próximos
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

        if distance > 35.0 {
            let direction = to_target.normalize();
            transform.translation += (direction * enemy.speed * time.delta_seconds()).extend(0.0);

            if *state != CharacterState::Attacking {
                *state = CharacterState::Walking;
            }

            // Flip sprite based on direction
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
    mut enemies: Query<(&Transform, &mut Enemy, &mut CharacterState)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut enemy, mut state) in enemies.iter_mut() {
        enemy.attack_cooldown.tick(time.delta());

        if enemy.attack_cooldown.finished() && *state == CharacterState::Attacking {
            *state = CharacterState::Idle;
        }

        let enemy_pos = transform.translation.truncate();
        let distance = enemy_pos.distance(player_pos);

        if distance < 45.0 && enemy.attack_cooldown.finished() {
            enemy.attack_cooldown = Timer::from_seconds(1.0, TimerMode::Once);
            *state = CharacterState::Attacking;

            damage_events.send(DamageEvent {
                target: player_entity,
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
        0 => (Vec2::new(100.0, 100.0), Color::WHITE, 35.0, 10.0, 12, 85.0),
        1 => (
            Vec2::new(130.0, 130.0),
            Color::srgb(1.0, 0.6, 0.6), // Reddish
            70.0,
            15.0,
            30,
            60.0,
        ),
        _ => (
            Vec2::new(80.0, 80.0),
            Color::srgb(0.6, 0.4, 0.9), // Purpleish
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
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 10.0, 0.1),
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
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 10.0, 0.2),
                ..default()
            },
            HealthBarFill(size.x + 6.0),
        ));
    });
}

pub fn check_enemy_death(
    mut commands: Commands,
    enemies: Query<(Entity, &Health, &Transform, &Enemy)>,
    mut game_stats: ResMut<GameStats>,
    mut xp_events: EventWriter<SpawnXpOrbEvent>,
) {
    for (entity, health, transform, enemy) in enemies.iter() {
        if health.current <= 0.0 {
            game_stats.enemies_killed += 1;

            xp_events.send(SpawnXpOrbEvent {
                position: transform.translation,
                value: enemy.xp_value,
            });

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
