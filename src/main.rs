mod components;
mod constants;
mod events;
mod helpers;
mod plugins;
mod resources;
mod systems;

use bevy::prelude::*;
use components::*;
use plugins::*;
use resources::*;

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
        .init_resource::<GameRng>()
        .add_plugins((
            UIPlugin,
            CombatPlugin,
            EnemyPlugin,
            PlayerPlugin,
            WorldPlugin,
            PassiveTreePlugin,
            SelectionPlugin,
            GameFeelPlugin,
            ProgressionPlugin,
        ))
        .add_systems(Startup, setup_camera_and_sprites)
        .add_systems(Update, systems::animation::animate_sprite)
        .add_systems(
            Update,
            systems::animation::update_character_animation_texture,
        )
        .run();
}

fn setup_camera_and_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());

    let layout = TextureAtlasLayout::from_grid(UVec2::splat(100), 6, 1, None, None);
    let layout_handle = texture_atlases.add(layout);

    commands.insert_resource(CharacterSprites {
        orc_idle: asset_server.load("sprites/orc/idle.png"),
        orc_walk: asset_server.load("sprites/orc/walk.png"),
        orc_attack: asset_server.load("sprites/orc/attack1.png"),
        soldier_idle: asset_server.load("sprites/soldier/idle.png"),
        soldier_walk: asset_server.load("sprites/soldier/walk.png"),
        soldier_attack: asset_server.load("sprites/soldier/attack1.png"),
        layout: layout_handle,
    });
}
