#![allow(unused)]

use bevy::prelude::*;

// region: --- Asset Constants
const PLAYER_SIZE: (f32,f32) = (144.,75.);
const PLAYER_SPRITE: &str = "player_a_01.png";
// endregion --- Asset Constants

fn main() {
    App::new()
    .insert_resource(ClearColor(Color::rgb(0.04,0.04,0.04)))
    .insert_resource(WindowDescriptor {
        title: "Ant Invaders".to_string(),
        width: 598.0,
        height: 676.0,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup_system)
    .run();
}

fn setup_system(
    mut commands: Commands,
    asser_server: Res<AssetServer>,
    mut windows: ResMut<Windows>,
) {
    // camera
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // capture window size
    let window = windows.get_primary_mut().unwrap();
    let (win_w, win_h) = (window.width(),window.height());

    // position window
    window.set_position(IVec2::new(2780,4900));
    // add player
    commands.spawn_bundle(SpriteBundle {
        texture: asser_server.load(PLAYER_SPRITE),
        // sprite: Sprite {
        //     color: Color::rgb(0.25,0.25,0.9),
        //     custom_size: Some(Vec2::new(150.,150.)),
        //     ..Default::default()
        // },
        ..Default::default()
    });
}
