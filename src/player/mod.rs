mod input;
mod movement;
mod slash;
mod spawn;

pub use input::{GamingInput, InputControllerSystem};
pub use movement::PlayerMovementSystemSet;
pub use slash::SpawnedSlash;

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            input::InputPlugin,
            spawn::PlayerSpawnPlugin,
            movement::PlayerMovementPlugin,
            slash::PlayerSlashPlugin,
        ));
    }
}

const MOVE_SPEED: f32 = 125.0;

#[derive(Component, Default)]
pub struct Player {
    pub is_over_ui: bool,
}
