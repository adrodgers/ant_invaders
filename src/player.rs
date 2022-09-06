use bevy::{prelude::*, time::FixedTimestep};

use crate::{
    components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity, Health, Damage},
    GameTextures, WinSize, BASE_SPEED, PLAYER_LASER_SIZE, PLAYER_SIZE, PLAYER_SPRITE, SPRITE_SCALE,
    TIME_STEP, PlayerState, PLAYER_RESPAWN_DELAY, player,
};

use std::f32::consts::PI;
const BASE_ROTATION_ANGLE_RAD: f32 = PI/2.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(PlayerState::default())
        .add_system(player_spawn_system)
        .add_system(player_keyboard_event_system)
        .add_system(player_fire_system);
    }
}

fn player_spawn_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    game_textures: Res<GameTextures>,
    win_size: Res<WinSize>,
) {

    if !player_state.on && player_state.spawn_cooldown.tick(time.delta()).finished() {
        player_state.spawned();
        // add player
        let bottom = -win_size.h / 2.;
        commands
            .spawn_bundle(SpriteBundle {
                texture: game_textures.player.clone(),
                transform: Transform {
                    translation: Vec3::new(0., bottom + PLAYER_SIZE.1 / 2. * SPRITE_SCALE + 5., 10.),
                    scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                    ..Default::default()
                },
                // sprite: Sprite {
                //     color: Color::rgb(0.25,0.25,0.9),
                //     custom_size: Some(Vec2::new(150.,150.)),
                //     ..Default::default()
                // },
                ..Default::default()
            })
            .insert(Player)
            // .insert(Health{hp: 3.,extra:0.})
            .insert(SpriteSize::from(PLAYER_SIZE)) // 
            .insert(Movable {
                auto_despawn: false,
            })
            .insert(Velocity { x: 0., y: 0. });
        // player_state.spawned(time.seconds_since_startup());
        player_state.health = Health {hp: 3., multiplier: 1.};
    }
    
}

fn player_keyboard_event_system(
    kb: Res<Input<KeyCode>>,
    win_size: Res<WinSize>,
    mut player_state: ResMut<PlayerState>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Player>>,
    time: Res<Time>
) {
        if let Ok((mut velocity, mut transform)) = query.get_single_mut() {
            if kb.pressed(KeyCode::Left) { 
                // println!("{:?}{:?}",transform.translation.x,win_size.w);
                if transform.translation.x > -win_size.w/2. {
                    if velocity.x > 0. {
                        velocity.x = 0.
                    }
                    if velocity.x > -0.5 {
                        velocity.x -= 0.25
                    } else if velocity.x > -1. {
                        velocity.x -= 0.05
                    }
                } else {
                    velocity.x = 0.
                }
            } else if kb.pressed(KeyCode::Right) {
                // println!("{:?}{:?}",transform.translation.x,win_size.w);
                if transform.translation.x < win_size.w/2. {
                    if velocity.x < 0. {
                        velocity.x = 0.
                    }
                    if velocity.x < 0.5 {
                        velocity.x += 0.25
                    } else if velocity.x < 1. {
                        velocity.x += 0.05
                    }
                } else {
                    velocity.x = 0.
                }
            } else if kb.pressed(KeyCode::Down) {
                // println!("{:?}{:?}",transform.translation.x,win_size.w);
                if transform.translation.y > -win_size.h/2. {
                    if velocity.y > 0. {
                        velocity.y = 0.
                    }
                    if velocity.y > -0.5 {
                        velocity.y -= 0.25
                    } else if velocity.y > -1. {
                        velocity.y -= 0.05
                    }
                } else {
                    velocity.y = 0.
                }
            } else if kb.pressed(KeyCode::Up) {
                // println!("{:?}{:?}",transform.translation.x,win_size.w);
                if transform.translation.y < win_size.h/2. {
                    if velocity.y < 0. {
                        velocity.y = 0.
                    }
                    if velocity.y < 0.5 {
                        velocity.y += 0.25
                    } else if velocity.y < 1. {
                        velocity.y += 0.05
                    }
                } else {
                    velocity.y = 0.
                }
            } else {
                velocity.x = 0.;
                velocity.y = 0.;
            };
            if kb.pressed(KeyCode::D) {
                // if player_state.angle.to_radians() < PI/4. {
                    player_state.angle += 5.;
                    transform.rotate_z(-5_f32.to_radians()); //Quat::from_rotation_z(
                // }
            } else if kb.pressed(KeyCode::A) {
                // if player_state.angle.to_radians() > -PI/4. {
                    player_state.angle -= 5.;
                    transform.rotate_z(5_f32.to_radians()); //Quat::from_rotation_z(
                // }
            }
            
        }
}

fn player_fire_system(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    kb: Res<Input<KeyCode>>,
    game_textures: Res<GameTextures>,
    query: Query<(&Transform, &Velocity), With<Player>>,
    time: Res<Time>,
) {
    // let mut fired = false;
    if let Ok((player_tf, vel)) = query.get_single() {
        if player_state.fire_cooldown.tick(time.delta()).finished() {
            if kb.pressed(KeyCode::Space) || kb.just_pressed(KeyCode::Space) {
                let (x, y) = (player_tf.translation.x, player_tf.translation.y);
                let x_offset: f32 = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

                let mut spawn_laser = |x_offset: f32| {
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: game_textures.player_laser.clone(),
                            transform: Transform {
                                translation: Vec3::new(x + x_offset, y, 0.),
                                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                                rotation: Quat::from_rotation_z(-player_state.angle.to_radians()),
                                // ..Default::default(),
                                
                            },
                            ..Default::default()
                        })
                        .insert(FromPlayer)
                        .insert(Laser)
                        .insert(Damage{dmg:10.,multiplier:1.,limit:5.})
                        .insert(SpriteSize::from(PLAYER_LASER_SIZE))
                        .insert(Movable { auto_despawn: true })
                        .insert(Velocity { x: player_state.angle.to_radians().sin(), y: player_state.angle.to_radians().cos() });
                };

                spawn_laser(0.);
                // spawn_laser(-x_offset);
                // fired = true;
                player_state.fire_cooldown.reset();
                // spawn_laser(0.);
                // player_state.fire_cooldown.reset();
            }
            // if fired {
                
            //     fired = false;
            // }
        }
        
    }
}
