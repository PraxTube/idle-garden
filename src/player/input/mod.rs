mod gaming_input;

use std::ops::BitOrAssign;

use bevy::{
    input::InputSystem,
    prelude::*,
    window::{PrimaryWindow, WindowResized},
};

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((gaming_input::GamingInputPlugin,))
            .init_resource::<GamingInput>()
            .insert_resource(InputDevice::Keyboard)
            .add_systems(
                PreUpdate,
                (
                    reset_inputs.before(InputSystem),
                    check_mouse_movement
                        .in_set(InputControllerSystem)
                        .after(InputSystem)
                        .run_if(resource_exists::<PreviousCursorPosition>),
                ),
            )
            .add_systems(
                Update,
                insert_previous_cursor_position
                    .run_if(not(resource_exists::<PreviousCursorPosition>)),
            );
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, SystemSet)]
pub struct InputControllerSystem;

/// Mouse and Keyboard are treated separately here because there are some cases in which it makes
/// sense to distinguish between them (e.g. UI logic).
///
/// If you just want to check for Mouse or Keyboard input just or it.
#[derive(Resource, PartialEq)]
pub enum InputDevice {
    Mouse,
    Keyboard,
    Gamepad,
}

/// Store cursor position at previous frame.
///
/// We need this because using a `Local<Vec2>` doesn't work because that will trigger a false
/// positive in the very first frame. Also I don't quite like `Local`s to begin with.
#[derive(Resource)]
struct PreviousCursorPosition(Vec2);

#[derive(Resource, Default, Clone, Copy, PartialEq)]
pub struct GamingInput {
    pub toggle_debug: bool,
    pub toggle_grid_debug: bool,
    pub screenshot: bool,

    pub scroll: i32,

    pub move_direction: Vec2,
    pub aim_direction: Vec2,
    pub light_attack: bool,
    pub heavy_attack: bool,
    pub parry: bool,
    pub dash: bool,
    pub special_light: bool,
    pub special_heavy: bool,

    pub pause: bool,

    pub toggle_player_collision_groups: bool,

    mouse_world_coords: Vec2,
}

impl BitOrAssign for GamingInput {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.move_direction == Vec2::ZERO {
            self.move_direction = rhs.move_direction;
        }
        if self.aim_direction == Vec2::ZERO {
            self.aim_direction = rhs.aim_direction;
        }
        if self.mouse_world_coords == Vec2::ZERO {
            self.mouse_world_coords = rhs.mouse_world_coords;
        }
        if self.scroll == 0 {
            self.scroll = rhs.scroll;
        }

        self.toggle_debug |= rhs.toggle_debug;

        self.light_attack |= rhs.light_attack;
        self.heavy_attack |= rhs.heavy_attack;
        self.parry |= rhs.parry;
        self.dash |= rhs.dash;
        self.special_light |= rhs.special_light;
        self.special_heavy |= rhs.special_heavy;
        self.pause |= rhs.pause;
        self.toggle_player_collision_groups |= rhs.toggle_player_collision_groups;
    }
}

fn reset_inputs(mut gaming_input: ResMut<GamingInput>) {
    *gaming_input = GamingInput::default();
}

fn insert_previous_cursor_position(
    mut commands: Commands,
    q_window: Single<&Window, With<PrimaryWindow>>,
) {
    let Some(cursor_position) = q_window.cursor_position() else {
        return;
    };
    commands.insert_resource(PreviousCursorPosition(cursor_position));
}

/// Check for mouse movement, if the mouse movement then switch to mouse and keyboard as input
/// device.
fn check_mouse_movement(
    mut input_device: ResMut<InputDevice>,
    q_window: Single<&Window, With<PrimaryWindow>>,
    mut previous_cursor_position: ResMut<PreviousCursorPosition>,
    mut ev_window_resized: EventReader<WindowResized>,
) {
    let Some(cursor_position) = q_window.cursor_position() else {
        return;
    };

    // When we resize the window (e.g. when toggling fullscreen mode) the cursor position is
    // changed (because relative to the window, it's different now).
    //
    // This is a problem when we use the keyboard or gamepad to toggle fullscreen because there is
    // a chance this will trigger a flase positive and we will switch to mouse input. I say there
    // is a chance because this doesn't trigger always, perhaps some race condition (but I don't
    // really know how because this system runs in PreUpdate).
    //
    // Anyways, all that so say that this fixes the issue.
    if !ev_window_resized.is_empty() && cursor_position != previous_cursor_position.0 {
        ev_window_resized.clear();
        previous_cursor_position.0 = cursor_position;
    }

    if cursor_position != previous_cursor_position.0 {
        previous_cursor_position.0 = cursor_position;
        *input_device = InputDevice::Mouse;
    }
}
