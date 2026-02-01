//! ARPG Minimalista
//!
//! Controles:
//! - WASD: Mover
//! - Mouse: Mirar
//! - Click Esquerdo: Projétil
//! - Click Direito: Ataque melee (área)
//! - Q: Dash
//! - Space: Fire Nova
//! - Tab: Mostrar/esconder stats
//! - R: Reiniciar

use bevy::{prelude::*, window::PrimaryWindow};
use rand::Rng;
use std::collections::HashSet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "ARPG Minimal".into(),
                resolution: (1280., 720.).into(),
                ..default()
            }),
            ..default()
        }))
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::srgb(0.1, 0.1, 0.12)))
        .insert_resource(CursorWorldPos(Vec2::ZERO))
        .insert_resource(GameStats::default())
        .insert_resource(PendingSelection::default())
        .add_event::<DamageEvent>()
        .add_event::<SpawnXpOrbEvent>()
        .add_systems(Startup, setup)
        // Sistemas de Seleção
        .add_systems(
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
        .add_systems(
            Update,
            (
                update_cursor_world_pos,
                player_movement,
                player_attack,
                player_skills,
                update_dash,
                update_invulnerability,
                regen_health,
                update_projectiles,
                update_melee_attacks,
                update_aoe_effects,
                enemy_ai,
                enemy_attack,
                process_damage,
                check_player_death,
                check_enemy_death,
                update_xp_orbs,
                collect_xp,
                spawn_enemies,
                update_pets,
                pet_actions,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            Update,
            (
                update_health_bars,
                update_cooldown_ui,
                update_stats_ui,
                update_damage_numbers,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(Update, restart_game)
        .add_systems(OnEnter(GameState::GameOver), show_game_over)
        .run();
}

// ============================================================================
// ESTADOS
// ============================================================================

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
enum GameState {
    #[default]
    CharacterSelection,
    PetSelection,
    Playing,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlayerClass {
    Tank,
    Archer,
    Mage,
    Tamer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PetType {
    Healer,
    Damager,
    Buffer,
    Tanker,
}

// ============================================================================
// COMPONENTES
// ============================================================================

#[derive(Component)]
struct Player {
    class: PlayerClass,
}

#[derive(Component)]
struct TamerData {
    selected_pets: Vec<PetType>,
}

#[derive(Component)]
struct Enemy {
    damage: f32,
    xp_value: u32,
    attack_cooldown: Timer,
    speed: f32,
}

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component, Clone)]
struct Stats {
    speed: f32,
    damage: f32,
    attack_speed: f32,
    crit_chance: f32,
    crit_multiplier: f32,
    life_regen: f32,
    armor: f32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            speed: 200.0,
            damage: 25.0,
            attack_speed: 1.0,
            crit_chance: 0.1,
            crit_multiplier: 2.0,
            life_regen: 2.0,
            armor: 0.0,
        }
    }
}

#[derive(Component)]
struct Level {
    level: u32,
    xp: u32,
    xp_to_next: u32,
}

impl Level {
    fn new() -> Self {
        Self {
            level: 1,
            xp: 0,
            xp_to_next: 100,
        }
    }

    fn add_xp(&mut self, amount: u32) -> bool {
        self.xp += amount;
        if self.xp >= self.xp_to_next {
            self.xp -= self.xp_to_next;
            self.level += 1;
            self.xp_to_next = (self.xp_to_next as f32 * 1.4) as u32;
            true
        } else {
            false
        }
    }
}

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Projectile {
    damage: f32,
    owner: Entity,
    pierce: u32,
    hit_entities: HashSet<Entity>,
    is_crit: bool,
}

#[derive(Component)]
struct MeleeAttack {
    damage: f32,
    owner: Entity,
    duration: Timer,
    hit_entities: HashSet<Entity>,
    is_crit: bool,
}

#[derive(Component)]
struct AoeEffect {
    damage: f32,
    owner: Entity,
    tick_timer: Timer,
    duration: Timer,
    hit_this_tick: HashSet<Entity>,
}

#[derive(Component)]
struct Lifetime(Timer);

#[derive(Component)]
struct XpOrb {
    value: u32,
}

#[derive(Component)]
struct HealthBar;

#[derive(Component)]
struct HealthBarFill(f32);

#[derive(Component)]
struct Dash {
    direction: Vec2,
    speed: f32,
    duration: Timer,
}

#[derive(Component)]
struct Invulnerable(Timer);

#[derive(Component)]
struct AttackCooldown(Timer);

#[derive(Component)]
struct SkillCooldowns {
    dash: Timer,
    nova: Timer,
}

#[derive(Component)]
struct DamageNumber {
    velocity: Vec2,
    lifetime: Timer,
}

#[derive(Component)]
struct CooldownUi;

#[derive(Component)]
struct StatsUi;

#[derive(Component)]
struct GameOverUi;

#[derive(Resource, Default)]
struct PendingSelection {
    class: Option<PlayerClass>,
    pets: Vec<PetType>,
}

#[derive(Component)]
struct SelectionUi;

#[derive(Component)]
struct ClassButton(PlayerClass);

#[derive(Component)]
struct PetButton(PetType);

#[derive(Component)]
struct Pet {
    owner: Entity,
    pet_type: PetType,
    action_timer: Timer,
}

#[derive(Component)]
struct Taunt;

// ============================================================================
// RECURSOS
// ============================================================================

#[derive(Resource, Default)]
struct CursorWorldPos(Vec2);

#[derive(Resource)]
struct GameStats {
    enemies_killed: u32,
    damage_dealt: f32,
    time_survived: f32,
    show_stats: bool,
}

impl Default for GameStats {
    fn default() -> Self {
        Self {
            enemies_killed: 0,
            damage_dealt: 0.0,
            time_survived: 0.0,
            show_stats: false,
        }
    }
}

// ============================================================================
// EVENTOS
// ============================================================================

#[derive(Event)]
struct DamageEvent {
    target: Entity,
    amount: f32,
    is_crit: bool,
}

#[derive(Event)]
struct SpawnXpOrbEvent {
    position: Vec3,
    value: u32,
}

// ============================================================================
// SETUP
// ============================================================================

fn setup(mut commands: Commands) {
    // Camera
    commands.spawn(Camera2dBundle::default());

    // Player
    // Retirado do setup pois agora o player nasce após a seleção
    // spawn_player(&mut commands, Vec3::ZERO, PlayerClass::Tank);

    // UI - Cooldowns
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Skills:\n",
                TextStyle {
                    font_size: 20.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "[Q] Dash: Ready\n",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(0.5, 0.8, 1.0),
                    ..default()
                },
            ),
            TextSection::new(
                "[Space] Nova: Ready\n",
                TextStyle {
                    font_size: 16.0,
                    color: Color::srgb(1.0, 0.5, 0.3),
                    ..default()
                },
            ),
            TextSection::new(
                "\nLevel: 1\n",
                TextStyle {
                    font_size: 18.0,
                    color: Color::srgb(1.0, 0.9, 0.3),
                    ..default()
                },
            ),
            TextSection::new(
                "XP: 0/100",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.8, 0.7, 0.3),
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        CooldownUi,
    ));

    // UI - Stats
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Stats (Tab):\n",
                TextStyle {
                    font_size: 18.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
            TextSection::new(
                "",
                TextStyle {
                    font_size: 14.0,
                    color: Color::srgb(0.7, 0.7, 0.7),
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            ..default()
        }),
        StatsUi,
    ));

    // Instruções
    commands.spawn(
        TextBundle::from_section(
            "WASD: Move | LMB: Shoot | RMB: Melee | Q: Dash | Space: Nova | Tab: Stats",
            TextStyle {
                font_size: 14.0,
                color: Color::srgb(0.5, 0.5, 0.5),
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
    );
}

fn spawn_player(commands: &mut Commands, position: Vec3, class: PlayerClass) -> Entity {
    let mut stats = Stats::default();
    let mut health = Health {
        current: 100.0,
        max: 100.0,
    };
    let mut attack_cooldown = Timer::from_seconds(0.3, TimerMode::Once);
    let mut skill_cooldowns = SkillCooldowns {
        dash: Timer::from_seconds(2.0, TimerMode::Once),
        nova: Timer::from_seconds(5.0, TimerMode::Once),
    };

    match class {
        PlayerClass::Tank => {
            health.max = 200.0;
            health.current = 200.0;
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

    let player_entity = commands
        .spawn((
            Player { class },
            health,
            stats,
            Level::new(),
            Velocity(Vec2::ZERO),
            AttackCooldown(attack_cooldown),
            skill_cooldowns,
            SpatialBundle::from_transform(Transform::from_translation(
                position.truncate().extend(10.0),
            )),
        ))
        .id();

    let body_color = match class {
        PlayerClass::Tank => Color::srgb(0.2, 0.4, 0.8),
        PlayerClass::Archer => Color::srgb(0.8, 0.7, 0.2),
        PlayerClass::Mage => Color::srgb(0.6, 0.2, 0.8),
        PlayerClass::Tamer => Color::srgb(0.2, 0.8, 0.3),
    };

    commands.entity(player_entity).with_children(|parent| {
        // Corpo
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: body_color,
                custom_size: Some(Vec2::new(28.0, 36.0)),
                ..default()
            },
            ..default()
        });

        // Indicador de direção
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.5, 1.0, 0.6),
                custom_size: Some(Vec2::new(8.0, 14.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 22.0, 0.1),
            ..default()
        });

        // Health bar background
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(42.0, 7.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 32.0, 0.1),
                ..default()
            },
            HealthBar,
        ));

        // Health bar fill
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.1, 0.9, 0.1),
                    custom_size: Some(Vec2::new(40.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 32.0, 0.2),
                ..default()
            },
            HealthBarFill(40.0),
        ));
    });

    player_entity
}

// ============================================================================
// SISTEMAS - INPUT E MOVIMENTO
// ============================================================================
// SISTEMAS - INPUT E MOVIMENTO
// ============================================================================

fn update_cursor_world_pos(
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

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cursor_pos: Res<CursorWorldPos>,
    mut query: Query<(&mut Transform, &Stats, Option<&Dash>), With<Player>>,
) {
    let Ok((mut transform, stats, dash)) = query.get_single_mut() else {
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
        transform.translation += (direction * stats.speed * time.delta_seconds()).extend(0.0);
    }

    // Rotação para cursor
    let to_cursor = cursor_pos.0 - transform.translation.truncate();
    let angle = to_cursor.y.atan2(to_cursor.x) - std::f32::consts::FRAC_PI_2;
    transform.rotation = Quat::from_rotation_z(angle);
}

fn update_dash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut Dash)>,
) {
    for (entity, mut transform, mut dash) in query.iter_mut() {
        dash.duration.tick(time.delta());

        if dash.duration.finished() {
            commands.entity(entity).remove::<Dash>();
        } else {
            let movement = dash.direction * dash.speed * time.delta_seconds();
            transform.translation += movement.extend(0.0);
        }
    }
}

fn update_invulnerability(
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

fn regen_health(time: Res<Time>, mut query: Query<(&mut Health, &Stats), With<Player>>) {
    for (mut health, stats) in query.iter_mut() {
        if health.current < health.max {
            health.current =
                (health.current + stats.life_regen * time.delta_seconds()).min(health.max);
        }
    }
}

// ============================================================================
// SISTEMAS - COMBATE
// ============================================================================

fn player_attack(
    mut commands: Commands,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    cursor_pos: Res<CursorWorldPos>,
    mut query: Query<(Entity, &Transform, &Stats, &Player, &mut AttackCooldown), With<Player>>,
) {
    let Ok((player_entity, transform, stats, player, mut cooldown)) = query.get_single_mut() else {
        return;
    };

    cooldown.0.tick(time.delta());

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

    // LMB - Ataque (Projetil ou Melee para Tank)
    if mouse.pressed(MouseButton::Left) {
        if player.class == PlayerClass::Tank {
            // Tank usa o ataque melee no LMB também, já que não tem projétil
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

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        color: if is_crit {
                            Color::srgb(1.0, 1.0, 0.2)
                        } else {
                            proj_color
                        },
                        custom_size: Some(Vec2::new(14.0, 14.0)),
                        ..default()
                    },
                    transform: Transform::from_translation(spawn_pos.extend(5.0)),
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
                    hit_entities: HashSet::new(),
                    is_crit,
                },
                Velocity(direction * 550.0),
                Lifetime(Timer::from_seconds(2.0, TimerMode::Once)),
            ));
        }
    }

    // RMB - Melee (Sempre disponível, mas Tank é o foco)
    if mouse.pressed(MouseButton::Right) {
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

fn spawn_melee_attack(
    commands: &mut Commands,
    player_entity: Entity,
    player_pos: Vec2,
    direction: Vec2,
    damage: f32,
    _attack_speed: f32,
    is_tank: bool,
) {
    let mut rng = rand::thread_rng();
    let is_crit = rng.r#gen::<f32>() < 0.1; // Simplificado para o helper, mas idealmente passaria crit_chance
    let melee_damage = damage * 1.8;
    let spawn_pos = player_pos + direction * 45.0;

    let (size, color) = if is_tank {
        (Vec2::new(120.0, 100.0), Color::srgba(0.2, 0.5, 1.0, 0.7))
    } else {
        (Vec2::new(90.0, 70.0), Color::srgba(0.9, 0.4, 0.1, 0.7))
    };

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: if is_crit {
                    Color::srgba(1.0, 0.6, 0.0, 0.85)
                } else {
                    color
                },
                custom_size: Some(size),
                ..default()
            },
            transform: Transform::from_translation(spawn_pos.extend(4.0))
                .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x))),
            ..default()
        },
        MeleeAttack {
            damage: melee_damage,
            owner: player_entity,
            duration: Timer::from_seconds(0.12, TimerMode::Once),
            hit_entities: HashSet::new(),
            is_crit,
        },
    ));
}

fn player_skills(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    cursor_pos: Res<CursorWorldPos>,
    mut query: Query<(Entity, &mut Transform, &Stats, &Player, &mut SkillCooldowns), With<Player>>,
) {
    let Ok((player_entity, mut transform, stats, player, mut cooldowns)) = query.get_single_mut()
    else {
        return;
    };

    cooldowns.dash.tick(time.delta());
    cooldowns.nova.tick(time.delta());

    let player_pos = transform.translation.truncate();

    // Q - Dash
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

    // Space - Nova (Skill Especial)
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
                // Iron Skin
                commands
                    .entity(player_entity)
                    .insert(Invulnerable(Timer::from_seconds(2.0, TimerMode::Once)));
                // Visual effect for Iron Skin
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
                // Vault: Dash backwards and shoot
                let direction = (player_pos - cursor_pos.0).normalize_or_zero();
                commands.entity(player_entity).insert(Dash {
                    direction,
                    speed: 1200.0,
                    duration: Timer::from_seconds(0.15, TimerMode::Once),
                });
                // Shoot 3 arrows
                for i in -1..=1 {
                    let angle = (i as f32) * 0.2;
                    let shoot_dir = Quat::from_rotation_z(angle) * (-direction).extend(0.0);
                    commands.spawn((
                        SpriteBundle {
                            sprite: Sprite {
                                color: Color::srgb(1.0, 1.0, 0.5),
                                custom_size: Some(Vec2::new(10.0, 10.0)),
                                ..default()
                            },
                            transform: Transform::from_translation(player_pos.extend(5.0)),
                            ..default()
                        },
                        Projectile {
                            damage: stats.damage * 0.8,
                            owner: player_entity,
                            pierce: 0,
                            hit_entities: HashSet::new(),
                            is_crit: false,
                        },
                        Velocity(shoot_dir.truncate() * 700.0),
                        Lifetime(Timer::from_seconds(1.0, TimerMode::Once)),
                    ));
                }
            }
            PlayerClass::Mage => {
                // Teleport
                let direction = (cursor_pos.0 - player_pos).normalize_or_zero();
                let target = player_pos + direction * 200.0;
                transform.translation = target.extend(10.0);

                // Visual effect
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
                // Command: No specific logic here yet, but could buff pets
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

fn update_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &Velocity,
        &mut Lifetime,
        &mut Projectile,
    )>,
    enemies: Query<(Entity, &Transform), (With<Enemy>, Without<Projectile>)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (proj_entity, mut transform, velocity, mut lifetime, mut projectile) in
        projectiles.iter_mut()
    {
        transform.translation += (velocity.0 * time.delta_seconds()).extend(0.0);

        lifetime.0.tick(time.delta());
        if lifetime.0.finished() {
            commands.entity(proj_entity).despawn();
            continue;
        }

        let proj_pos = transform.translation.truncate();

        for (enemy_entity, enemy_transform) in enemies.iter() {
            if projectile.hit_entities.contains(&enemy_entity) {
                continue;
            }

            let enemy_pos = enemy_transform.translation.truncate();
            let distance = proj_pos.distance(enemy_pos);

            if distance < 28.0 {
                projectile.hit_entities.insert(enemy_entity);

                damage_events.send(DamageEvent {
                    target: enemy_entity,
                    amount: projectile.damage,
                    is_crit: projectile.is_crit,
                });

                if projectile.pierce == 0 {
                    commands.entity(proj_entity).despawn();
                    break;
                } else {
                    projectile.pierce -= 1;
                }
            }
        }
    }
}

fn update_melee_attacks(
    mut commands: Commands,
    time: Res<Time>,
    mut melee_attacks: Query<(Entity, &Transform, &mut MeleeAttack)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (melee_entity, transform, mut melee) in melee_attacks.iter_mut() {
        melee.duration.tick(time.delta());

        if melee.duration.finished() {
            commands.entity(melee_entity).despawn();
            continue;
        }

        let melee_pos = transform.translation.truncate();

        for (enemy_entity, enemy_transform) in enemies.iter() {
            if melee.hit_entities.contains(&enemy_entity) {
                continue;
            }

            let enemy_pos = enemy_transform.translation.truncate();
            let distance = melee_pos.distance(enemy_pos);

            if distance < 65.0 {
                melee.hit_entities.insert(enemy_entity);

                damage_events.send(DamageEvent {
                    target: enemy_entity,
                    amount: melee.damage,
                    is_crit: melee.is_crit,
                });
            }
        }
    }
}

fn update_aoe_effects(
    mut commands: Commands,
    time: Res<Time>,
    mut aoe_effects: Query<(Entity, &Transform, &mut AoeEffect, &mut Sprite)>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (aoe_entity, transform, mut aoe, mut sprite) in aoe_effects.iter_mut() {
        aoe.duration.tick(time.delta());
        aoe.tick_timer.tick(time.delta());

        if aoe.duration.finished() {
            commands.entity(aoe_entity).despawn();
            continue;
        }

        let alpha = 0.45 * (1.0 - aoe.duration.fraction());
        sprite.color = sprite.color.with_alpha(alpha);

        if aoe.tick_timer.just_finished() {
            aoe.hit_this_tick.clear();

            let aoe_pos = transform.translation.truncate();
            let radius = sprite.custom_size.unwrap_or(Vec2::splat(100.0)).x / 2.0;

            for (enemy_entity, enemy_transform) in enemies.iter() {
                if aoe.hit_this_tick.contains(&enemy_entity) {
                    continue;
                }

                let enemy_pos = enemy_transform.translation.truncate();
                let distance = aoe_pos.distance(enemy_pos);

                if distance < radius {
                    aoe.hit_this_tick.insert(enemy_entity);

                    damage_events.send(DamageEvent {
                        target: enemy_entity,
                        amount: aoe.damage,
                        is_crit: false,
                    });
                }
            }
        }
    }
}

fn process_damage(
    mut commands: Commands,
    mut damage_events: EventReader<DamageEvent>,
    mut targets: Query<(
        &mut Health,
        &Transform,
        Option<&Stats>,
        Option<&Invulnerable>,
    )>,
    mut game_stats: ResMut<GameStats>,
) {
    for event in damage_events.read() {
        let Ok((mut health, transform, stats, invuln)) = targets.get_mut(event.target) else {
            continue;
        };

        if invuln.is_some() {
            continue;
        }

        let armor = stats.map(|s| s.armor).unwrap_or(0.0);
        let damage_reduction = armor / (armor + 100.0);
        let final_damage = event.amount * (1.0 - damage_reduction);

        health.current -= final_damage;
        game_stats.damage_dealt += final_damage;

        // Damage number
        let color = if event.is_crit {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            Color::srgb(1.0, 0.3, 0.3)
        };

        let font_size = if event.is_crit { 26.0 } else { 18.0 };

        commands.spawn((
            Text2dBundle {
                text: Text::from_section(
                    format!("{:.0}", final_damage),
                    TextStyle {
                        font_size,
                        color,
                        ..default()
                    },
                ),
                transform: Transform::from_translation(
                    transform.translation
                        + Vec3::new(rand::thread_rng().gen_range(-15.0..15.0), 20.0, 100.0),
                ),
                ..default()
            },
            DamageNumber {
                velocity: Vec2::new(rand::thread_rng().gen_range(-25.0..25.0), 60.0),
                lifetime: Timer::from_seconds(0.7, TimerMode::Once),
            },
        ));
    }
}

fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut numbers: Query<(Entity, &mut Transform, &mut DamageNumber, &mut Text)>,
) {
    for (entity, mut transform, mut number, mut text) in numbers.iter_mut() {
        number.lifetime.tick(time.delta());

        if number.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        number.velocity.y -= 120.0 * time.delta_seconds();
        transform.translation += (number.velocity * time.delta_seconds()).extend(0.0);

        let alpha = 1.0 - number.lifetime.fraction();
        for section in text.sections.iter_mut() {
            section.style.color = section.style.color.with_alpha(alpha);
        }
    }
}

// ============================================================================
// SISTEMAS - INIMIGOS
// ============================================================================

fn enemy_ai(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    taunt_query: Query<&Transform, (With<Taunt>, Without<Enemy>, Without<Player>)>,
    mut enemies: Query<(&mut Transform, &Enemy), Without<Player>>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (mut transform, enemy) in enemies.iter_mut() {
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
        }
    }
}

fn enemy_attack(
    time: Res<Time>,
    player_query: Query<(Entity, &Transform), (With<Player>, Without<Enemy>)>,
    mut enemies: Query<(&Transform, &mut Enemy)>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    let Ok((player_entity, player_transform)) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    for (transform, mut enemy) in enemies.iter_mut() {
        enemy.attack_cooldown.tick(time.delta());

        let enemy_pos = transform.translation.truncate();
        let distance = enemy_pos.distance(player_pos);

        if distance < 40.0 && enemy.attack_cooldown.finished() {
            enemy.attack_cooldown = Timer::from_seconds(1.0, TimerMode::Once);

            damage_events.send(DamageEvent {
                target: player_entity,
                amount: enemy.damage,
                is_crit: false,
            });
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
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
        0 => (
            Vec2::new(24.0, 24.0),
            Color::srgb(0.85, 0.2, 0.2),
            35.0,
            10.0,
            12,
            85.0,
        ),
        1 => (
            Vec2::new(36.0, 36.0),
            Color::srgb(0.9, 0.55, 0.15),
            70.0,
            15.0,
            30,
            60.0,
        ),
        _ => (
            Vec2::new(18.0, 18.0),
            Color::srgb(0.6, 0.2, 0.7),
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
            SpatialBundle::from_transform(Transform::from_translation(spawn_pos.extend(5.0))),
        ))
        .id();

    commands.entity(enemy_entity).with_children(|parent| {
        parent.spawn(SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(size),
                ..default()
            },
            ..default()
        });

        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::srgb(0.15, 0.0, 0.0),
                    custom_size: Some(Vec2::new(size.x + 8.0, 5.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 6.0, 0.1),
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
                transform: Transform::from_xyz(0.0, size.y / 2.0 + 6.0, 0.2),
                ..default()
            },
            HealthBarFill(size.x + 6.0),
        ));
    });
}

fn check_enemy_death(
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

fn check_player_death(
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

// ============================================================================
// SISTEMAS - XP
// ============================================================================

fn update_xp_orbs(mut commands: Commands, mut xp_events: EventReader<SpawnXpOrbEvent>) {
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

fn collect_xp(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<(&Transform, Entity), With<Player>>,
    mut xp_orbs: Query<(Entity, &mut Transform, &XpOrb, &mut Lifetime), Without<Player>>,
    mut levels: Query<(&mut Level, &mut Stats, &mut Health)>,
) {
    let Ok((player_transform, player_entity)) = player_query.get_single() else {
        return;
    };
    let Ok((mut level, mut stats, mut health)) = levels.get_mut(player_entity) else {
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
            }

            commands.entity(orb_entity).despawn();
        }
    }
}

// ============================================================================
// UI
// ============================================================================

fn update_health_bars(
    parents: Query<&Health>,
    mut health_bars: Query<(&Parent, &mut Sprite, &mut Transform, &HealthBarFill)>,
) {
    for (parent, mut sprite, mut transform, fill) in health_bars.iter_mut() {
        let Ok(health) = parents.get(parent.get()) else {
            continue;
        };

        let percent = (health.current / health.max).clamp(0.0, 1.0);

        if let Some(ref mut size) = sprite.custom_size {
            let full_width = fill.0;
            let new_width = full_width * percent;
            let offset = (full_width - new_width) / 2.0;
            transform.translation.x = -offset;
            size.x = new_width.max(0.1);
        }
    }
}

fn update_cooldown_ui(
    player: Query<(&SkillCooldowns, &Level, &Health), With<Player>>,
    mut ui: Query<&mut Text, With<CooldownUi>>,
    mut game_stats: ResMut<GameStats>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let Ok((cooldowns, level, health)) = player.get_single() else {
        return;
    };
    let Ok(mut text) = ui.get_single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::Tab) {
        game_stats.show_stats = !game_stats.show_stats;
    }

    text.sections[1].value = if cooldowns.dash.finished() {
        "[Q] Dash: Ready\n".to_string()
    } else {
        format!("[Q] Dash: {:.1}s\n", cooldowns.dash.remaining_secs())
    };

    text.sections[2].value = if cooldowns.nova.finished() {
        "[Space] Nova: Ready\n".to_string()
    } else {
        format!("[Space] Nova: {:.1}s\n", cooldowns.nova.remaining_secs())
    };

    text.sections[3].value = format!(
        "\nLevel: {} | HP: {:.0}/{:.0}\n",
        level.level, health.current, health.max
    );
    text.sections[4].value = format!("XP: {}/{}", level.xp, level.xp_to_next);
}

fn update_stats_ui(
    player: Query<&Stats, With<Player>>,
    game_stats: Res<GameStats>,
    mut ui: Query<&mut Text, With<StatsUi>>,
) {
    let Ok(stats) = player.get_single() else {
        return;
    };
    let Ok(mut text) = ui.get_single_mut() else {
        return;
    };

    if game_stats.show_stats {
        text.sections[1].value = format!(
            "Damage: {:.1}\n\
             Atk Speed: {:.2}x\n\
             Crit: {:.0}%\n\
             Crit Multi: {:.1}x\n\
             Speed: {:.0}\n\
             Armor: {:.0}\n\
             Regen: {:.1}/s\n\
             \n-- Session --\n\
             Kills: {}\n\
             Dmg: {:.0}\n\
             Time: {:.0}s",
            stats.damage,
            stats.attack_speed,
            stats.crit_chance * 100.0,
            stats.crit_multiplier,
            stats.speed,
            stats.armor,
            stats.life_regen,
            game_stats.enemies_killed,
            game_stats.damage_dealt,
            game_stats.time_survived,
        );
    } else {
        text.sections[1].value = "(Tab)".to_string();
    }
}

fn show_game_over(mut commands: Commands, game_stats: Res<GameStats>) {
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "GAME OVER\n\n",
                TextStyle {
                    font_size: 64.0,
                    color: Color::srgb(0.95, 0.15, 0.15),
                    ..default()
                },
            ),
            TextSection::new(
                format!(
                    "Enemies Killed: {}\n\
                     Damage Dealt: {:.0}\n\
                     Time Survived: {:.0}s\n\n\
                     Press R to restart",
                    game_stats.enemies_killed, game_stats.damage_dealt, game_stats.time_survived,
                ),
                TextStyle {
                    font_size: 26.0,
                    color: Color::WHITE,
                    ..default()
                },
            ),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(28.0),
            left: Val::Percent(32.0),
            ..default()
        }),
        GameOverUi,
    ));
}

fn restart_game(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    despawn_query: Query<
        Entity,
        Or<(
            With<Player>,
            With<Enemy>,
            With<Projectile>,
            With<XpOrb>,
            With<AoeEffect>,
            With<MeleeAttack>,
            With<DamageNumber>,
            With<GameOverUi>,
        )>,
    >,
    mut game_stats: ResMut<GameStats>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        for entity in despawn_query.iter() {
            commands.entity(entity).despawn_recursive();
        }

        *game_stats = GameStats::default();

        if *current_state.get() == GameState::GameOver {
            next_state.set(GameState::CharacterSelection);
        } else {
            next_state.set(GameState::CharacterSelection);
        }
    }
}

// ============================================================================
// SISTEMAS - SELEÇÃO
// ============================================================================

fn setup_class_selection(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            SelectionUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "ESCOLHA SUA CLASSE",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            let classes = [
                (PlayerClass::Tank, "TANK (Defesa/AoE)"),
                (PlayerClass::Archer, "ARCHER (Velocidade/Range)"),
                (PlayerClass::Mage, "MAGE (Alto Dano/CD)"),
                (PlayerClass::Tamer, "TAMER (Domador de Pets)"),
            ];

            for (class, label) in classes {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(300.0),
                                height: Val::Px(50.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                            ..default()
                        },
                        ClassButton(class),
                    ))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn handle_class_selection(
    mut next_state: ResMut<NextState<GameState>>,
    mut pending: ResMut<PendingSelection>,
    mut interaction_query: Query<
        (&Interaction, &ClassButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, class_btn, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                pending.class = Some(class_btn.0);
                if class_btn.0 == PlayerClass::Tamer {
                    next_state.set(GameState::PetSelection);
                } else {
                    next_state.set(GameState::Playing);
                }
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.4, 0.4, 0.4).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}

fn setup_pet_selection(mut commands: Commands) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            SelectionUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "ESCOLHA 2 PETS",
                TextStyle {
                    font_size: 40.0,
                    color: Color::WHITE,
                    ..default()
                },
            ));

            let pets = [
                (PetType::Healer, "CURA (Regen HP)"),
                (PetType::Damager, "DANO (Ataca Inimigos)"),
                (PetType::Buffer, "BUFF (Escudo/Dano)"),
                (PetType::Tanker, "TANK (Atrai Agro)"),
            ];

            for (pet, label) in pets {
                parent
                    .spawn((
                        ButtonBundle {
                            style: Style {
                                width: Val::Px(300.0),
                                height: Val::Px(50.0),
                                margin: UiRect::all(Val::Px(10.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            background_color: Color::srgb(0.2, 0.2, 0.2).into(),
                            ..default()
                        },
                        PetButton(pet),
                    ))
                    .with_children(|p| {
                        p.spawn(TextBundle::from_section(
                            label,
                            TextStyle {
                                font_size: 20.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn handle_pet_selection(
    mut next_state: ResMut<NextState<GameState>>,
    mut pending: ResMut<PendingSelection>,
    mut interaction_query: Query<
        (&Interaction, &PetButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, pet_btn, mut color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                if !pending.pets.contains(&pet_btn.0) {
                    pending.pets.push(pet_btn.0);
                    *color = Color::srgb(0.2, 0.6, 0.2).into();
                }

                if pending.pets.len() >= 2 {
                    next_state.set(GameState::Playing);
                }
            }
            Interaction::Hovered => {
                if !pending.pets.contains(&pet_btn.0) {
                    *color = Color::srgb(0.4, 0.4, 0.4).into();
                }
            }
            Interaction::None => {
                if !pending.pets.contains(&pet_btn.0) {
                    *color = Color::srgb(0.2, 0.2, 0.2).into();
                }
            }
        }
    }
}

fn start_game(mut commands: Commands, mut pending: ResMut<PendingSelection>) {
    let class = pending.class.unwrap_or(PlayerClass::Tank);
    let player_entity = spawn_player(&mut commands, Vec3::ZERO, class);

    if class == PlayerClass::Tamer {
        for pet_type in pending.pets.iter() {
            spawn_pet(&mut commands, player_entity, *pet_type);
        }
    }

    // Limpar seleção para o próximo restart
    *pending = PendingSelection::default();
}

fn spawn_pet(commands: &mut Commands, owner: Entity, pet_type: PetType) {
    let color = match pet_type {
        PetType::Healer => Color::srgb(0.2, 1.0, 0.5),
        PetType::Damager => Color::srgb(1.0, 0.2, 0.2),
        PetType::Buffer => Color::srgb(0.2, 0.5, 1.0),
        PetType::Tanker => Color::srgb(0.7, 0.7, 0.7),
    };

    let mut entity = commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 8.0),
            ..default()
        },
        Pet {
            owner,
            pet_type,
            action_timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        },
    ));

    if pet_type == PetType::Tanker {
        entity.insert(Taunt);
    }
}

fn update_pets(
    time: Res<Time>,
    owner_query: Query<&Transform, With<Player>>,
    mut pet_query: Query<(&mut Transform, &Pet), Without<Player>>,
) {
    for (mut transform, pet) in pet_query.iter_mut() {
        if let Ok(owner_transform) = owner_query.get(pet.owner) {
            let target = owner_transform.translation.truncate();
            let current = transform.translation.truncate();

            // Offset baseado no tipo de pet para não ficarem encavalados
            let offset = match pet.pet_type {
                PetType::Healer => Vec2::new(-40.0, 20.0),
                PetType::Damager => Vec2::new(40.0, 20.0),
                PetType::Buffer => Vec2::new(-40.0, -20.0),
                PetType::Tanker => Vec2::new(40.0, -20.0),
            };

            let dest = target + offset;
            let dist = current.distance(dest);

            if dist > 5.0 {
                let dir = (dest - current).normalize();
                let speed = 250.0;
                transform.translation += (dir * speed * time.delta_seconds()).extend(0.0);
            }
        }
    }
}

fn pet_actions(
    time: Res<Time>,
    mut pet_query: Query<(&Transform, &mut Pet)>,
    mut owner_query: Query<(&mut Health, &mut Stats), With<Player>>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    mut damage_events: EventWriter<DamageEvent>,
) {
    for (transform, mut pet) in pet_query.iter_mut() {
        pet.action_timer.tick(time.delta());

        if pet.action_timer.just_finished() {
            match pet.pet_type {
                PetType::Healer => {
                    if let Ok((mut health, _)) = owner_query.get_mut(pet.owner) {
                        health.current = (health.current + health.max * 0.01).min(health.max);
                    }
                }
                PetType::Damager => {
                    // Ataca o inimigo mais próximo
                    let pet_pos = transform.translation.truncate();
                    let mut nearest: Option<(Entity, f32)> = None;

                    for (enemy_entity, enemy_transform) in enemy_query.iter() {
                        let dist = pet_pos.distance(enemy_transform.translation.truncate());
                        if dist < 250.0 {
                            if nearest.is_none() || dist < nearest.unwrap().1 {
                                nearest = Some((enemy_entity, dist));
                            }
                        }
                    }

                    if let Some((target, _)) = nearest {
                        damage_events.send(DamageEvent {
                            target,
                            amount: 10.0,
                            is_crit: false,
                        });
                    }
                }
                PetType::Buffer => {
                    if let Ok((_, mut stats)) = owner_query.get_mut(pet.owner) {
                        // Aplica bônus temporário (como o sistema roda sempre, mantemos o bônus)
                        // Para ser robusto deveríamos resetar os stats base antes, mas ARPGs simples
                        // costumam ter buffs que expiram. Aqui, enquanto o pet viver, o regen/armor é maior.
                        stats.armor = (stats.armor).max(25.0);
                        stats.damage = (stats.damage).max(35.0);
                    }
                }
                PetType::Tanker => {
                    // Tanker pet just exists and is handled by Taunt component in enemy_ai
                }
            }
        }
    }
}

fn despawn_selection_ui(mut commands: Commands, query: Query<Entity, With<SelectionUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
