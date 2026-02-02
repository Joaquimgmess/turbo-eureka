use crate::components::*;
use crate::resources::*;
use crate::systems::combat::spawn_melee_attack;
use bevy::prelude::*;
use rand::Rng;
use std::collections::HashSet;

pub fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cursor_pos: Res<CursorWorldPos>,
    mut query: Query<(&mut Transform, &Stats, Option<&Dash>, &mut CharacterState), With<Player>>,
    obstacles: Query<(&Transform, &Sprite), (With<Obstacle>, Without<Player>, Without<Dash>)>,
) {
    let Ok((mut transform, stats, dash, mut state)) = query.get_single_mut() else {
        return;
    };
    if dash.is_some() {
        return;
    }
    let mut direction = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
        let movement = direction * stats.speed * time.delta_seconds();
        let mut new_pos = transform.translation + movement.extend(0.0);

        let player_radius = 25.0;
        let mut collision = false;
        for (obs_transform, obs_sprite) in obstacles.iter() {
            let obs_pos = obs_transform.translation.truncate();
            let obs_size = obs_sprite.custom_size.unwrap_or(Vec2::splat(50.0)) / 2.0;

            let dist_x = (new_pos.x - obs_pos.x).abs();
            let dist_y = (new_pos.y - obs_pos.y).abs();

            if dist_x < (obs_size.x + player_radius) && dist_y < (obs_size.y + player_radius) {
                collision = true;
                break;
            }
        }

        if !collision {
            new_pos.x = new_pos.x.clamp(-1200.0, 1200.0);
            new_pos.y = new_pos.y.clamp(-1200.0, 1200.0);
            transform.translation = new_pos;
        }

        if *state != CharacterState::Attacking {
            *state = CharacterState::Walking;
        }
    } else if *state != CharacterState::Attacking {
        *state = CharacterState::Idle;
    }
    let to_cursor = cursor_pos.0 - transform.translation.truncate();
    let angle = to_cursor.y.atan2(to_cursor.x) - std::f32::consts::FRAC_PI_2;
    transform.rotation = Quat::from_rotation_z(angle);
}

pub fn update_dash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Dash), (With<Dash>, Without<Obstacle>)>,
    obstacles: Query<(&Transform, &Sprite), (With<Obstacle>, Without<Dash>)>,
) {
    for (entity, mut transform, mut dash) in query.iter_mut() {
        dash.duration.tick(time.delta());
        if dash.duration.finished() {
            commands.entity(entity).remove::<Dash>();
        } else {
            let movement = dash.direction * dash.speed * time.delta_seconds();
            let mut new_pos = transform.translation + movement.extend(0.0);

            let mut collision = false;
            for (obs_transform, obs_sprite) in obstacles.iter() {
                let obs_pos = obs_transform.translation.truncate();
                let obs_size = obs_sprite.custom_size.unwrap_or(Vec2::splat(50.0)) / 2.0;
                let dist_x = (new_pos.x - obs_pos.x).abs();
                let dist_y = (new_pos.y - obs_pos.y).abs();
                if dist_x < obs_size.x + 20.0 && dist_y < obs_size.y + 20.0 {
                    collision = true;
                    break;
                }
            }

            if !collision {
                new_pos.x = new_pos.x.clamp(-1200.0, 1200.0);
                new_pos.y = new_pos.y.clamp(-1200.0, 1200.0);
                transform.translation = new_pos;
            }
        }
    }
}

pub fn update_invulnerability(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Invulnerable)>,
) {
    for (entity, mut invuln) in query.iter_mut() {
        invuln.0.tick(time.delta());
        if invuln.0.finished() {
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

pub fn regen_health(
    time: Res<Time>,
    mut query: Query<(&mut Health, &mut Shield, &Stats, &PlayerPassives), With<Player>>,
) {
    for (mut health, mut shield, stats, passives) in query.iter_mut() {
        if health.current < health.max {
            health.current =
                (health.current + stats.life_regen * time.delta_seconds()).min(health.max);
        }
        let mut shield_regen = 0.0;
        for &node_id in &passives.unlocked_nodes {
            if node_id == 102 {
                shield_regen += 8.0;
            }
        }
        if shield.amount < (health.max * 0.5) {
            shield.amount =
                (shield.amount + shield_regen * time.delta_seconds()).min(health.max * 0.5);
        }
    }
}

pub fn player_attack(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    cursor_pos: Res<CursorWorldPos>,
    mut query: Query<
        (
            Entity,
            &Transform,
            &Stats,
            &Player,
            &PlayerPassives,
            &mut AttackCooldown,
            &mut CharacterState,
        ),
        With<Player>,
    >,
) {
    let Ok((player_entity, transform, stats, player, passives, mut cooldown, mut state)) =
        query.get_single_mut()
    else {
        return;
    };
    cooldown.0.tick(time.delta());
    if cooldown.0.finished() && *state == CharacterState::Attacking {
        *state = CharacterState::Idle;
    }
    if !cooldown.0.finished() {
        return;
    }
    let player_pos = transform.translation.truncate();
    let direction = (cursor_pos.0 - player_pos).normalize_or_zero();
    let mut rng = rand::thread_rng();
    let is_crit = rng.r#gen::<f32>() < stats.crit_chance;
    let damage = if is_crit {
        stats.damage * stats.crit_multiplier
    } else {
        stats.damage
    };
    if mouse.pressed(MouseButton::Left) {
        *state = CharacterState::Attacking;
        if player.class == PlayerClass::Tank {
            spawn_melee_attack(
                &mut commands,
                player_entity,
                player_pos,
                direction,
                damage,
                stats.attack_speed,
                true,
            );
            cooldown.0 = Timer::from_seconds(0.4 / stats.attack_speed, TimerMode::Once);
        } else {
            cooldown.0 = Timer::from_seconds(0.25 / stats.attack_speed, TimerMode::Once);
            let spawn_pos = player_pos + direction * 30.0;
            let proj_color = match player.class {
                PlayerClass::Mage => Color::srgb(0.6, 0.3, 1.0),
                PlayerClass::Archer => Color::srgb(0.9, 0.9, 0.4),
                _ => Color::srgb(1.0, 0.7, 0.1),
            };
            let texture = if player.class == PlayerClass::Archer {
                Some(asset_server.load("sprites/projectiles/arrow.png"))
            } else {
                None
            };
            commands.spawn((
                SpriteBundle {
                    texture: texture.unwrap_or_default(),
                    sprite: Sprite {
                        color: if is_crit {
                            Color::srgb(1.0, 1.0, 0.2)
                        } else {
                            proj_color
                        },
                        custom_size: Some(if player.class == PlayerClass::Archer {
                            Vec2::new(32.0, 32.0)
                        } else {
                            Vec2::new(14.0, 14.0)
                        }),
                        ..default()
                    },
                    transform: Transform::from_translation(spawn_pos.extend(5.0))
                        .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
                    ..default()
                },
                Projectile {
                    damage,
                    owner: player_entity,
                    pierce: if player.class == PlayerClass::Archer {
                        1
                    } else {
                        0
                    },
                    chain_count: if passives.unlocked_nodes.contains(&10) {
                        1
                    } else {
                        0
                    },
                    hit_entities: HashSet::new(),
                    is_crit,
                },
                Velocity(direction * 550.0),
                Lifetime(Timer::from_seconds(2.0, TimerMode::Once)),
            ));
        }
    }
    if mouse.pressed(MouseButton::Right) {
        *state = CharacterState::Attacking;
        spawn_melee_attack(
            &mut commands,
            player_entity,
            player_pos,
            direction,
            damage,
            stats.attack_speed,
            player.class == PlayerClass::Tank,
        );
        cooldown.0 = Timer::from_seconds(0.4 / stats.attack_speed, TimerMode::Once);
    }
}

pub fn player_skills(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    cursor_pos: Res<CursorWorldPos>,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            &Stats,
            &Player,
            &PlayerPassives,
            &mut SkillCooldowns,
            &mut Shield,
        ),
        With<Player>,
    >,
) {
    let Ok((player_entity, mut transform, stats, player, passives, mut cooldowns, mut shield)) =
        query.get_single_mut()
    else {
        return;
    };
    cooldowns.dash.tick(time.delta());
    cooldowns.nova.tick(time.delta());
    let player_pos = transform.translation.truncate();
    if keyboard.just_pressed(KeyCode::KeyQ) && cooldowns.dash.finished() {
        cooldowns.dash = Timer::from_seconds(2.0, TimerMode::Once);
        let direction = (cursor_pos.0 - player_pos).normalize_or_zero();
        commands.entity(player_entity).insert((
            Dash {
                direction,
                speed: 900.0,
                duration: Timer::from_seconds(0.12, TimerMode::Once),
            },
            Invulnerable(Timer::from_seconds(0.12, TimerMode::Once)),
        ));
    }
    if keyboard.just_pressed(KeyCode::Space) && cooldowns.nova.finished() {
        cooldowns.nova = Timer::from_seconds(
            match player.class {
                PlayerClass::Mage => 3.0,
                PlayerClass::Tank => 8.0,
                _ => 5.0,
            },
            TimerMode::Once,
        );
        match player.class {
            PlayerClass::Tank => {
                commands
                    .entity(player_entity)
                    .insert(Invulnerable(Timer::from_seconds(2.0, TimerMode::Once)));
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.5, 0.5, 1.0, 0.4),
                            custom_size: Some(Vec2::splat(60.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(player_pos.extend(3.0)),
                        ..default()
                    },
                    Lifetime(Timer::from_seconds(2.0, TimerMode::Once)),
                ));
            }
            PlayerClass::Archer => {
                let direction = (player_pos - cursor_pos.0).normalize_or_zero();
                commands.entity(player_entity).insert(Dash {
                    direction,
                    speed: 1200.0,
                    duration: Timer::from_seconds(0.15, TimerMode::Once),
                });
                let arrow_texture = asset_server.load("sprites/projectiles/arrow.png");
                for i in -1..=1 {
                    let angle = (i as f32) * 0.2;
                    let shoot_dir = Quat::from_rotation_z(angle) * (-direction).extend(0.0);
                    let shoot_dir_2d = shoot_dir.truncate();
                    commands.spawn((
                        SpriteBundle {
                            texture: arrow_texture.clone(),
                            sprite: Sprite {
                                color: Color::srgb(1.0, 1.0, 0.5),
                                custom_size: Some(Vec2::new(24.0, 24.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(player_pos.extend(5.0))
                                .with_rotation(Quat::from_rotation_z(
                                    shoot_dir_2d.y.atan2(shoot_dir_2d.x),
                                )),
                            ..default()
                        },
                        Projectile {
                            damage: stats.damage * 0.8,
                            owner: player_entity,
                            pierce: 0,
                            chain_count: if passives.unlocked_nodes.contains(&10) {
                                1
                            } else {
                                0
                            },
                            hit_entities: HashSet::new(),
                            is_crit: false,
                        },
                        Velocity(shoot_dir_2d * 700.0),
                        Lifetime(Timer::from_seconds(1.0, TimerMode::Once)),
                    ));
                }
            }
            PlayerClass::Mage => {
                let direction = (cursor_pos.0 - player_pos).normalize_or_zero();
                let mut target = player_pos + direction * 200.0;

                target.x = target.x.clamp(-1200.0, 1200.0);
                target.y = target.y.clamp(-1200.0, 1200.0);

                transform.translation = target.extend(10.0);
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.8, 0.2, 1.0, 0.6),
                            custom_size: Some(Vec2::splat(100.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(player_pos.extend(3.0)),
                        ..default()
                    },
                    Lifetime(Timer::from_seconds(0.3, TimerMode::Once)),
                ));
            }
            PlayerClass::Tamer => {
                shield.amount += 30.0;
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::srgba(0.2, 1.0, 0.3, 0.3),
                            custom_size: Some(Vec2::splat(300.0)),
                            ..default()
                        },
                        transform: Transform::from_translation(player_pos.extend(3.0)),
                        ..default()
                    },
                    Lifetime(Timer::from_seconds(0.5, TimerMode::Once)),
                ));
            }
        }
    }
}

pub fn spawn_player(
    commands: &mut Commands,
    sprites: &Res<CharacterSprites>,
    position: Vec3,
    class: PlayerClass,
) -> Entity {
    let mut stats = Stats::default();
    let mut health = Health {
        current: 100.0,
        max: 100.0,
    };
    let mut shield = Shield::default();
    let mut attack_cooldown = Timer::from_seconds(0.3, TimerMode::Once);
    let mut skill_cooldowns = SkillCooldowns {
        dash: Timer::from_seconds(2.0, TimerMode::Once),
        nova: Timer::from_seconds(5.0, TimerMode::Once),
    };
    match class {
        PlayerClass::Tank => {
            health.max = 200.0;
            health.current = 200.0;
            shield.amount = 50.0;
            stats.armor = 50.0;
            stats.speed = 170.0;
            stats.damage = 30.0;
            stats.life_regen = 5.0;
        }
        PlayerClass::Archer => {
            health.max = 80.0;
            health.current = 80.0;
            stats.speed = 240.0;
            stats.attack_speed = 1.6;
            stats.damage = 18.0;
            attack_cooldown = Timer::from_seconds(0.15, TimerMode::Once);
        }
        PlayerClass::Mage => {
            health.max = 70.0;
            health.current = 70.0;
            stats.damage = 65.0;
            stats.attack_speed = 0.7;
            skill_cooldowns.nova = Timer::from_seconds(3.5, TimerMode::Once);
            attack_cooldown = Timer::from_seconds(0.6, TimerMode::Once);
        }
        PlayerClass::Tamer => {
            health.max = 110.0;
            health.current = 110.0;
            stats.speed = 210.0;
        }
    }
    let body_color = match class {
        PlayerClass::Tank => Color::srgb(1.0, 1.0, 1.0),
        PlayerClass::Archer => Color::srgb(0.8, 0.7, 0.2),
        PlayerClass::Mage => Color::srgb(0.6, 0.2, 0.8),
        PlayerClass::Tamer => Color::srgb(0.2, 0.8, 0.3),
    };
    let player_entity = commands
        .spawn((
            Player { class },
            health,
            shield,
            stats,
            Level::new(),
            Velocity(Vec2::ZERO),
            AttackCooldown(attack_cooldown),
            skill_cooldowns,
            PlayerPassives::default(),
            CharacterState::Idle,
            SpriteBundle {
                texture: sprites.soldier_idle.clone(),
                sprite: Sprite {
                    color: body_color,
                    custom_size: Some(Vec2::new(180.0, 180.0)),
                    ..default()
                },
                transform: Transform::from_translation(position.truncate().extend(10.0)),
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
    commands.entity(player_entity).with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(50.0, 8.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 95.0, 0.1),
                ..default()
            },
            HealthBar,
        ));
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.1, 0.9, 0.1),
                    custom_size: Some(Vec2::new(48.0, 6.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 95.0, 0.2),
                ..default()
            },
            HealthBarFill(48.0),
        ));
    });
    player_entity
}
