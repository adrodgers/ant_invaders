use bevy::{
    time::Timer,
    math::{Vec2, Vec3},
    prelude::{Component, Entity},
};

// region: --- Common Components
#[derive(Component,Clone, Copy)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Movable {
    pub auto_despawn: bool,
}

#[derive(Component)]
pub struct Laser;

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct Health {
    pub hp: f32,
    pub multiplier: f32,
}

#[derive(Component)]
pub struct NumberOfHits {
    pub hits: i32
}

#[derive(Component)]
pub struct Damage {
    pub dmg: f32,
    pub limit: f32,
    pub multiplier: f32
}

impl Damage {
    pub fn damage_dealt(&self) -> f32 {
        if self.dmg* self.multiplier > self.limit {self.limit} else {self.dmg*self.multiplier}
    }
}

#[derive(Component)]
pub struct SpriteSize(pub Vec2);

impl From<(f32, f32)> for SpriteSize {
    fn from(val: (f32, f32)) -> Self {
        SpriteSize(Vec2::new(val.0, val.1))
    }
}
// endregion: --- Common Components

// region: --- Player Components
#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct FromPlayer;
// endregion: --- Player Components

// region: --- Enemy Components
#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct FromEnemy;

#[derive(Component)]
pub struct ParentEntity {
    pub entity: Entity
}
// endregion: --- Enemy Components

// region: --- Explosion Components
#[derive(Component)]
pub struct Explosion;

#[derive(Component)]
pub struct ExplosionToSpawn(pub Vec3);

#[derive(Component)]
pub struct ExplosionTimer(pub Timer);

impl Default for ExplosionTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.05, true))
    }
}
// endregion: --- Explosion Components
