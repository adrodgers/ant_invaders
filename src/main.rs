#![allow(unused)]

use bevy::{prelude::*, render::camera::ScalingMode};
use components::{Movable, Velocity, SpriteSize, FromPlayer, Laser, Enemy};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

mod components;
mod enemy;
mod player;

// region: --- Asset Constants
const PLAYER_SIZE: (f32, f32) = (144., 75.);
const PLAYER_SPRITE: &str = "player_a_01.png";
const PLAYER_LASER_SPRITE: &str = "laser_a_01.png";
const PLAYER_LASER_SIZE: (f32, f32) = (9., 54.);

const ENEMY_SIZE: (f32, f32) = (144., 75.);
const ENEMY_SPRITE: &str = "enemy_a_01.png";
const ENEMY_LASER_SPRITE: &str = "enemy_a_01.png";
const ENEMY_LASER_SIZE: (f32, f32) = (9., 54.);

const SPRITE_SCALE: f32 = 0.5;

// endregion --- Asset Constants

// region: --- Game Constants
const TIME_STEP: f32 = 1. / 60.;
const BASE_SPEED: f32 = 500.;
// endregion: --- Game Constants

// region: --- Resources
pub struct WinSize {
    pub w: f32,
    pub h: f32,
}
struct GameTextures {
    player: Handle<Image>,
    player_laser: Handle<Image>,
    enemy: Handle<Image>,
    enemy_laser: Handle<Image>,
}
// endregion: --- Resources

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "Ant Invaders".to_string(),
            width: 598.0,
            height: 676.0,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup_system)
        .add_system(movable_system)
        .run();
}

fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut windows: ResMut<Windows>,
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // capture window size
    let window = windows.get_primary_mut().unwrap();
    let (win_w, win_h) = (window.width(), window.height());

    // position window
    window.set_position(IVec2::new(2780, 4900));

    // add winsize
    let win_size = WinSize { w: win_w, h: win_h };
    commands.insert_resource(win_size);

    // add game textures
    let game_textures: GameTextures = GameTextures {
        player: asset_server.load(PLAYER_SPRITE),
        player_laser: asset_server.load(PLAYER_LASER_SPRITE),
        enemy: asset_server.load(ENEMY_SPRITE),
        enemy_laser: asset_server.load(ENEMY_LASER_SPRITE),
    };
    commands.insert_resource(game_textures);
}

fn movable_system(
    mut commands: Commands,
    win_size: Res<WinSize>,
    mut query: Query<(Entity, &Velocity, &mut Transform, &Movable)>,
) {
    for (entity, velocity, mut transform, movable) in query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.x * BASE_SPEED * TIME_STEP;
        translation.y += velocity.y * BASE_SPEED * TIME_STEP;

        if movable.auto_despawn {
            const MARGIN: f32 = 200.;
            if translation.y > win_size.h / 2. + MARGIN
                || translation.y < -win_size.h / 2. - MARGIN
                || translation.x > win_size.w / 2. + MARGIN
                || translation.x < -win_size.w / 2. - MARGIN
            {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn player_laser_hit_enemy_system(
    mut commands: Commands,
    laser_query: Query<(Entity, &Transform, &SpriteSize), (With<FromPlayer>, With<Laser>)>,
    enemy_query: Query<(Entity, &Transform, &SpriteSize), With<Enemy>>
) {

}
