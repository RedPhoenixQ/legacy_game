use bevy::prelude::*;
use bevy_ggrs::*;

use crate::{cursor::CursorWorldCords, Packet};

const INPUT_UP: u32 = 1 << 0;
const INPUT_DOWN: u32 = 1 << 1;
const INPUT_LEFT: u32 = 1 << 2;
const INPUT_RIGHT: u32 = 1 << 3;
const INPUT_FIRE: u32 = 1 << 4;

pub fn input(
    _: In<ggrs::PlayerHandle>,
    keys: Res<Input<KeyCode>>,
    cursor: Res<CursorWorldCords>,
) -> Packet {
    let mut input = Packet::default();

    if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
        input.input |= INPUT_UP;
    }
    if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
        input.input |= INPUT_DOWN;
    }
    if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
        input.input |= INPUT_LEFT
    }
    if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
        input.input |= INPUT_RIGHT;
    }
    if keys.any_pressed([KeyCode::Space, KeyCode::Return]) {
        input.input |= INPUT_FIRE;
        input.angle = cursor.0;
    }

    input
}

pub fn direction(input: u32) -> Vec2 {
    let mut direction = Vec2::ZERO;
    if input & INPUT_UP != 0 {
        direction.y += 1.;
    }
    if input & INPUT_DOWN != 0 {
        direction.y -= 1.;
    }
    if input & INPUT_RIGHT != 0 {
        direction.x += 1.;
    }
    if input & INPUT_LEFT != 0 {
        direction.x -= 1.;
    }
    direction
}

pub fn has_fired(input: u32) -> bool {
    input & INPUT_FIRE != 0
}
