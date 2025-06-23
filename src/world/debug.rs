use bevy::prelude::*;

use crate::player::{GamingInput, InputControllerSystem};

/// Indicates whether the game is currently in debug mode.
/// This can be used for just debugging info to the player (developer),
/// or it can also act as a trigger to allow cheats etc.
#[derive(Resource, Default)]
pub struct DebugState {
    pub active: bool,
    pub changed_this_frame: bool,
    pub grid_debug_active: bool,
    pub collision_debug_active: bool,
}

fn reset_changed_this_frame_flag(mut debug_state: ResMut<DebugState>) {
    debug_state.changed_this_frame = false;
}

fn toggle_debug_state(gaming_input: Res<GamingInput>, mut debug_state: ResMut<DebugState>) {
    if gaming_input.toggle_debug {
        debug_state.active = !debug_state.active;
        debug_state.changed_this_frame = true;
    }
}

fn toggle_grid_debug(mut debug_state: ResMut<DebugState>, gaming_input: Res<GamingInput>) {
    if !debug_state.active {
        return;
    }
    if !gaming_input.toggle_debug_grid {
        return;
    }

    debug_state.grid_debug_active = !debug_state.grid_debug_active;
}

fn toggle_collision_debug(mut debug_state: ResMut<DebugState>, gaming_input: Res<GamingInput>) {
    if !debug_state.active {
        return;
    }
    if !gaming_input.toggle_debug_collision {
        return;
    }

    debug_state.collision_debug_active = !debug_state.collision_debug_active;
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>().add_systems(
            PreUpdate,
            (
                reset_changed_this_frame_flag,
                toggle_debug_state,
                toggle_grid_debug,
                toggle_collision_debug,
            )
                .chain()
                .after(InputControllerSystem),
        );
    }
}
