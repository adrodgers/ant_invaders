use bevy::{prelude::*, time::FixedTimestep};

use crate::{
    components::{FromPlayer, Laser, Movable, Player, SpriteSize, Velocity, Health, Damage},
    GameTextures, WinSize, BASE_SPEED, PLAYER_LASER_SIZE, PLAYER_SIZE, PLAYER_SPRITE, SPRITE_SCALE,
    TIME_STEP, PlayerState, PLAYER_RESPAWN_DELAY, player,
};

use std::f32::consts::PI;
// const BASE_ROTATION_ANGLE_RAD: f32 = PI/2.;
const ACCELERATION: f32 = 1.0;
const MAX_VELOCITY: f32 = 15.0;
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
                ..Default::default()
            })
            .insert(Player)
            .insert(SpriteSize::from(PLAYER_SIZE)) // 
            .insert(Movable {
                auto_despawn: false,
            })
            .insert(Velocity { x: 0., y: 0. });
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
        if kb.pressed(KeyCode::A) { 
            player_state.delta_x -= ACCELERATION;
        } 
        if kb.pressed(KeyCode::D) {
            player_state.delta_x += ACCELERATION;
        } 
        if kb.pressed(KeyCode::S) {
            player_state.delta_y -= ACCELERATION;
        } 
        if kb.pressed(KeyCode::W) {
            player_state.delta_y += ACCELERATION;
        } 

        player_state.delta_x = player_state.delta_x.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        transform.translation.x += player_state.delta_x;
        player_state.delta_y = player_state.delta_y.clamp(-MAX_VELOCITY, MAX_VELOCITY);
        transform.translation.y += player_state.delta_y;

        // transform.translation.x = transform.translation.x.clamp(-320.0, 320.0);
        // transform.translation.y = transform.translation.y.clamp(-320.0, 320.0);

        // Decelerate
        player_state.delta_x *= 0.9;
        player_state.delta_y *= 0.9;
        // Fire angle
        let curr_angle = player_state.angle;
        if kb.pressed(KeyCode::Up) {
            player_state.angle = 0.;
            // angle = 0.;
        }
        if kb.pressed(KeyCode::Down) {
            player_state.angle = PI;
            // angle = PI;
        }
        if kb.pressed(KeyCode::Left) {
            player_state.angle = -PI/2.;
            // angle = -PI/2.;
        }
        if kb.pressed(KeyCode::Right) {
            player_state.angle = PI/2.;
            // angle = PI/2.;
        } 
        if (kb.any_pressed(vec![KeyCode::Up,KeyCode::Down,KeyCode::Left,KeyCode::Right])) {
            player_state.firing = true;
        } else {player_state.firing = false}

        if curr_angle != player_state.angle {
            transform.rotate_z(curr_angle-player_state.angle);
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
            if player_state.firing { //|| kb.just_pressed(KeyCode::Space)
                let (x, y) = (player_tf.translation.x, player_tf.translation.y);
                let x_offset: f32 = PLAYER_SIZE.0 / 2. * SPRITE_SCALE - 5.;

                let mut spawn_laser = |x_offset: f32| {
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: game_textures.player_laser.clone(),
                            transform: Transform {
                                translation: Vec3::new(x + x_offset, y, 0.),
                                scale: Vec3::new(SPRITE_SCALE, SPRITE_SCALE, 1.),
                                // rotation: Quat::from_rotation_z(player_tf.rotation.z.to_radians()), 
                                rotation: player_tf.rotation                              
                            },
                            ..Default::default()
                        })
                        .insert(FromPlayer)
                        .insert(Laser)
                        .insert(Damage{dmg:10.,multiplier:1.,limit:5.})
                        .insert(SpriteSize::from(PLAYER_LASER_SIZE))
                        .insert(Movable { auto_despawn: true })
                        .insert(Velocity { x: player_state.angle.sin() + player_state.delta_x/50., y: player_state.angle.cos() + player_state.delta_y/50. });
                };
                spawn_laser(0.);
                player_state.fire_cooldown.reset();
            }
        }
        
    }
}
