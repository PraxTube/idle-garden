mod input;
mod movement;
mod spawn;

pub use input::{GamingInput, InputControllerSystem};

use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            input::InputPlugin,
            spawn::PlayerSpawnPlugin,
            movement::PlayerMovementPlugin,
        ));
    }
}

const MOVE_SPEED: f32 = 200.0;

#[derive(Component)]
pub struct Player;
