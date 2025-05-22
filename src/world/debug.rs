use bevy::prelude::*;

use crate::player::{GamingInput, InputControllerSystem};

/// Indicates whether the game is currently in debug mode.
/// This can be used for just debugging info to the player (developer),
/// or it can also act as a trigger to allow cheats etc.
#[derive(Resource, Default)]
pub struct DebugState {
    pub active: bool,
}

fn toggle_debug_state(gaming_input: Res<GamingInput>, mut debug_state: ResMut<DebugState>) {
    if gaming_input.toggle_debug {
        debug_state.active = !debug_state.active;
    }
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugState>()
            .add_systems(PreUpdate, toggle_debug_state.after(InputControllerSystem));
    }
}
