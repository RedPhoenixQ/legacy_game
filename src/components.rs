use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub handle: usize,
    pub reloading: Timer,
    pub last_direction: Vec2,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            handle: default(),
            reloading: Timer::from_seconds(0.1, TimerMode::Once),
            last_direction: default(),
        }
    }
}

#[derive(Component)]
pub struct Bullet {
    pub direction: Vec2,
}
