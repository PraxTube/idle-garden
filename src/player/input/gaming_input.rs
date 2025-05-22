use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::input::InputSystem;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use crate::player::Player;
use crate::world::MainCamera;
use crate::GameState;

use super::{GamingInput, InputControllerSystem, InputDevice};

fn fetch_mouse_world_coords(
    mut gaming_input: ResMut<GamingInput>,
    window: Single<&Window, With<PrimaryWindow>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
) {
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.0.viewport_to_world(camera.1, cursor).ok())
        .map(|ray| ray.origin.xy())
    {
        gaming_input.mouse_world_coords = world_position;
    }
}

fn update_aim_direction(
    mut gaming_input: ResMut<GamingInput>,
    q_players: Query<&Transform, With<Player>>,
    input_device: Res<InputDevice>,
) {
    if *input_device != InputDevice::Mouse && *input_device != InputDevice::Keyboard {
        return;
    }

    for transform in &q_players {
        let dir = gaming_input.mouse_world_coords - transform.translation.xy();

        if dir != Vec2::ZERO {
            gaming_input.aim_direction = dir.normalize_or_zero();
        }
    }
}

fn handle_keyboard_inputs(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut gaming_input: ResMut<GamingInput>,
    mut input_device: ResMut<InputDevice>,
    mut scroll_evr: EventReader<MouseWheel>,
) {
    let mut input = GamingInput::default();

    input.toggle_debug = keys.just_pressed(KeyCode::F3);

    input.light_attack =
        keys.just_pressed(KeyCode::KeyL) || mouse_buttons.just_pressed(MouseButton::Left);
    input.heavy_attack =
        keys.just_pressed(KeyCode::KeyN) || mouse_buttons.just_pressed(MouseButton::Right);
    input.parry = keys.just_pressed(KeyCode::KeyP) || keys.just_pressed(KeyCode::KeyE);
    input.dash = keys.just_pressed(KeyCode::ShiftLeft);
    input.special_light = keys.just_pressed(KeyCode::KeyQ);
    input.special_heavy = keys.just_pressed(KeyCode::Space);

    let mut move_direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyJ) || keys.pressed(KeyCode::KeyS) {
        move_direction += Vec2::NEG_Y;
    }
    if keys.pressed(KeyCode::KeyK) || keys.pressed(KeyCode::KeyW) {
        move_direction += Vec2::Y;
    }
    if keys.pressed(KeyCode::KeyF) || keys.pressed(KeyCode::KeyD) {
        move_direction += Vec2::X;
    }
    if keys.pressed(KeyCode::KeyA) {
        move_direction += Vec2::NEG_X;
    }
    input.move_direction = move_direction.normalize_or_zero();

    let mut zoom = 0;
    if keys.just_pressed(KeyCode::Backspace) {
        zoom -= 1;
    }
    if keys.just_pressed(KeyCode::Minus) {
        zoom += 1;
    }

    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                if ev.y > 0.0 {
                    zoom -= 1;
                } else {
                    zoom += 1;
                }
            }
            MouseScrollUnit::Pixel => {
                if ev.y > 0.0 {
                    zoom -= 1;
                } else {
                    zoom += 1;
                }
            }
        };
    }
    input.scroll = zoom;

    input.pause = keys.just_pressed(KeyCode::Escape);

    input.toggle_player_collision_groups = keys.just_pressed(KeyCode::KeyT);

    if input != GamingInput::default() {
        *input_device = InputDevice::Keyboard;
    }
    *gaming_input |= input;
}

fn handle_gamepad_inputs(
    mut gaming_input: ResMut<GamingInput>,
    mut input_device: ResMut<InputDevice>,
    q_gamepads: Query<&Gamepad>,
) {
    let mut input = GamingInput::default();

    for gamepad in &q_gamepads {
        input.light_attack = gamepad.just_pressed(GamepadButton::West);
        input.heavy_attack = gamepad.just_pressed(GamepadButton::North);
        input.parry = gamepad.just_pressed(GamepadButton::LeftTrigger);
        input.dash = gamepad.just_pressed(GamepadButton::LeftTrigger2);
        input.special_light = gamepad.just_pressed(GamepadButton::East);
        input.special_heavy = gamepad.just_pressed(GamepadButton::South);

        input.pause = gamepad.just_pressed(GamepadButton::Start);

        let mut zoom = 0;
        if gamepad.just_pressed(GamepadButton::DPadLeft) {
            zoom -= 1;
        }
        if gamepad.just_pressed(GamepadButton::DPadRight) {
            zoom += 1;
        }
        input.scroll = zoom;

        let left_stick_direction = {
            let (x, y) = (
                gamepad.get(GamepadAxis::LeftStickX).unwrap_or_default(),
                gamepad.get(GamepadAxis::LeftStickY).unwrap_or_default(),
            );
            Vec2::new(x, y).normalize_or_zero()
        };
        input.move_direction = left_stick_direction;
        input.aim_direction = left_stick_direction;

        if input != GamingInput::default() {
            *input_device = InputDevice::Gamepad;
        }
        *gaming_input |= input;
    }
}

pub struct GamingInputPlugin;

impl Plugin for GamingInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (
                fetch_mouse_world_coords,
                update_aim_direction,
                handle_keyboard_inputs,
                handle_gamepad_inputs,
            )
                .chain()
                .run_if(in_state(GameState::Gaming))
                .in_set(InputControllerSystem)
                .after(InputSystem),
        );
    }
}
